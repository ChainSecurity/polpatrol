//Polkadot Runtime Checker

//Strategies of includes: Use Polkadot imports wherever possible, only include from substrate directly where Polkadot also imports them from Substrate

#[macro_use]
extern crate log;

use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

use substrate_executor::WasmExecutor;
use substrate_inherents::{InherentData, InherentDataProviders};
use substrate_primitives::{traits::Externalities, Blake2Hasher, OpaqueMetadata, H256};
use substrate_service::ChainSpec;
//use substrate_sr_metadata::{DecodeDifferent, RuntimeMetadataPrefixed};
use substrate_sr_primitives::{weights::GetDispatchInfo, *};
use substrate_state_machine::TestExternalities as CoreTestExternalities;
use substrate_trie::trie_types::Layout;
use substrate_version::RuntimeVersion;

use polkadot_primitives::parachain::AttestedCandidate;
use polkadot_runtime::{
    GenesisConfig, UncheckedExtrinsic as UncheckedExtrinsicPolkadot, NEW_HEADS_IDENTIFIER,
};

use trie_db::TrieConfiguration;

use codec::*;

use structopt::StructOpt;

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use std::mem;

// Our module used to craft extrinsics for testing
pub mod generation;
//Our module to craft random inputs
mod random;

use generation::ReadyEx;

type TestExternalities<H> = CoreTestExternalities<H, u64>;
/// An index to a block, defined as in Polkadot
pub type BlockNumber = u32;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, traits::BlakeTwo256>;
/// Alias to 512-bit hash when used in the context of a signature on the relay chain.
/// Equipped with logic for possibly "unsigned" messages.
pub type Signature = AnySignature;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsicPolkadot>;

/*
    Calls exposed function of the given wasm blob

    Input :
    ext: Externalities, the externalities/Environment the wasm blob is to be executed in
    wasmbin: the wasm binary
    fn_name: the name of the function to call
    calldata: data passed to the function call

    Returns: A Result, either a vector with the encoded result or an Error
*/
fn call_exported_fn(
    ext: &mut TestExternalities<Blake2Hasher>,
    wasmbin: &[u8],
    fn_name: &str,
    calldata: &[u8],
) -> Result<Vec<u8>, substrate_executor::error::Error> {
    let now = Instant::now();
    //Heap Pages set to 1024, default value see: https://github.com/paritytech/substrate/blob/master/core/executor/src/native_executor.rs#L31
    info!("DEBUG_CS_INSTRUMENTATION_WASM_EXEC_START_INFO {}", fn_name);
    let ret = WasmExecutor::new().call(ext, 1024, &wasmbin, fn_name, calldata);
    let elapsed = now.elapsed();
    info!(
        "DEBUG_CS_INSTRUMENTATION_WASM_EXEC_END_INFO {} {:?}",
        fn_name, elapsed
    );
    debug!("wasm execution took: {:?} seconds", elapsed.as_secs());
    match &ret {
        Ok(_result) => {}
        Err(_e) => {
            //TODO: log it for the statistics
            // error detected -> already caught by the Executor -> log it for statistics
            info!("DEBUG_CS_INSTRUMENTATION_WASM_EXEC_ERROR {:?}", _e);
            info!("Investigate the affected extrinsic, remove it & rerun PolPatrol.");
        }
    }
    return ret;
}

/*
    Crafts and returns a vector of UncheckedExtrinsics for the inherents

    Input :
    ext: Externalities, the environment the wasm blob is executed in
    wasbmin: the wasm binary

    Returns: A Vector of Unchecked Extrinsics
*/
fn craft_inherents(
    ext: &mut TestExternalities<Blake2Hasher>,
    wasmbin: &[u8],
) -> Vec<UncheckedExtrinsicPolkadot> {
    //Build inherents -- TODO: Infer from the Runtime's Metadata which it needs
    //1. Create new inherent provider
    let inherent_data_providers = InherentDataProviders::new();
    //2. Create & Register providers,. For now this is only the Timestamp module
    let timestamp_provider = substrate_sr_timestamp::InherentDataProvider;
    &inherent_data_providers.register_provider(timestamp_provider);

    //3. Export inherent data & encode them in SCALE
    let mut rawcalldata = inherent_data_providers
        .create_inherent_data()
        .expect("create inherent data failed");

    //Adding empty parachain heads inherent -> that means no parachain attached
    //TODO: To support parachain connected to the relay chain under test, attach the correct heads here!

    let candidates = Vec::<AttestedCandidate>::new();

    /*let candidate = AttestedCandidate {
        validity_votes: vec![],
        validator_indices: BitVec::new(), //use bitvec::vec::BitVec;
        candidate: CandidateReceipt {
            parachain_index: 0.into(),
            collator: Default::default(),
            signature: Default::default(),
            head_data: HeadData(vec![]),
            egress_queue_roots: vec![],
            fees: 0,
            block_data_hash: Default::default(),
            upward_messages: vec![],
        },
    };

    candidates.push(candidate);*/

    rawcalldata
        .put_data(NEW_HEADS_IDENTIFIER, &candidates)
        .expect("Adding the parachainheads to the rawcalldata for the inherents failed");
    let calldata = &InherentData::encode(&rawcalldata);
    let ret = call_exported_fn(ext, &wasmbin, "BlockBuilder_inherent_extrinsics", calldata)
        .expect("wasm call to BlockBuilder_inherent_extrinsics failed");
    debug!("{:?}", ret);

    let inherents = Vec::<UncheckedExtrinsicPolkadot>::decode(&mut &ret[..]).expect("Decoding the vector of unchecked extrinsics returned by BlockBuilder_inherent_extrinsics failed");
    info!("created Inherent: {:?}", inherents);
    inherents
}

/*
    Removes and returns the next extrinsics from the pool

    Input : pool where the extrinsics will be taken from

    Returns: A Vector of Unchecked Extrinsics
*/
fn fetch_next_extrinsics(
    pool: &mut Vec<ReadyEx>,
    nextblock: u32,
    num: u32,
) -> Vec<UncheckedExtrinsicPolkadot> {
    //amount of extrinsics to be included in a block
    let mut exs: Vec<UncheckedExtrinsicPolkadot> = Vec::new();
    let mut amount = num;

    while amount > 0 && !pool.is_empty() {
        amount -= 1;
        let candidate = pool
            .pop()
            .expect("Attempted to pop from the extrinsics pool but pool is empty!");
        //if min block for next extrinsic has not been reached, push on stack & break
        if candidate.minblock >= nextblock {
            pool.push(candidate);
            break;
        }
        exs.push(candidate.extrinsic);
    }

    exs
}

/// Configure cmd line arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Amount of extrinsics to include in a block (taken from the pool, except the required inherents)
    #[structopt(short, long, default_value = "3")]
    num: u32,

    /// The relay chain runtime wasm blob to be inspected
    #[structopt(
        short = "f",
        long,
        name = "runtime",
        parse(from_os_str),
        default_value = "./res/polkadot_runtime.compact_no_onlystakingandclaims_release.wasm"
    )]
    runtimefile: PathBuf,

    /// Genesis file containing the initial state
    #[structopt(
        short,
        long,
        name = "genesis",
        parse(from_os_str),
        default_value = "./res/kusama_modified_for_polpatrol.json"
    )]
    genesisfile: PathBuf,

    /// Amount of random black box testing, type 1: randomly created calldata
    #[structopt(short = "a", long, default_value = "0")]
    random_type_one: u32,

    /// Amount of random black box testing, type 2: randomly created input types, properly encoded
    #[structopt(short = "b", long, default_value = "0")]
    random_type_two: u32,

    /// u64 to create the seed from for the random generator
    #[structopt(short = "s", long, default_value = "0")]
    seed: u64,

    /// Switch to raw output
    #[structopt(short = "r", long)]
    raw_output: bool,

    /// verbose output flag -> generates more output to standard output and standard error output
    #[structopt(short = "v", long)]
    verbose: bool,
}

#[derive(Default, Debug)]
struct Measurements {
    avg_total_time: u128,      // for counting average time
    avg_amount_of_calls: u128, // number of times it was called 
    maximum_time: u128,        // longest seen execution time
    avg_total_mem: u128,       // for counting average memory
    maximum_mem: u128,         // highest seen memory consumption
    weight: u128,               // extrinsics weight (only for extrinsics)
    // Number of TAG occurences per category
    counter_per_cat: HashMap<String, u32>,
    // Maximum number of TAG occurences per category
    max_per_cat: HashMap<String, u32>,
    // Current number of TAG occurences per category
    current_per_cat: HashMap<String, u32>
}

impl Measurements {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug)]
struct PolPatrolLoggerMutData {
    counter_per_category: HashMap<String, u32>,
    // Current Extrinsic going on
    current_extrinsic: String,
    // Current Runtime Entry going on
    current_fn: String,
    // Whether current log outputs belong to block generation/execution 
    in_block: bool,
    // Block stats
    block_stats: HashMap<u32, BlockStats>,
    // Peak memory usage in Bytes for runtime entry
    used_mem_fn: u128,
    // Peak memory usage in Bytes for extrinsic 
    used_mem_extrinsic: u128,
    // Measurements for Runtime Entries
    measurements_per_fn_name: HashMap<String, Measurements>,
    // Time when the current extrinsic started to execute
    current_extrinsic_time: u128, 
    // Weight of currently executing extrinsic
    current_extrinsic_weight: u128,
    // Time when the current runtime entry started to execute
    current_fn_time: u128, 
    // Has counters for the number of calls to environment functions
    // Also has counters for number of calls aggregated by category -> see categories
    measurements_per_extrinsic: HashMap<String, Measurements>,
}

impl PolPatrolLoggerMutData {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug)]
struct BlockStats{
    bn: u32,      // Block number
    weight: u128, // Total Block weight
    length: u128, // size of SCALE encoded block in bytes
}

impl BlockStats{
    fn new() -> Self {
        Default::default()
    }
}

struct PolPatrolLogger {
    start: std::time::Instant,
    data: Arc<Mutex<PolPatrolLoggerMutData>>,
    raw_output_flag: bool,
    verbose_output_flag: bool,
    // Regular expressions for extrinsics
    none_extrinsic: Regex,
    some_extrinsic: Regex,
    sudo_extrinsic: Regex,
    categories: [String; 5],
}

impl PolPatrolLogger {

    fn increment_tag(&self, data: &mut PolPatrolLoggerMutData, for_extrinsic: bool, category_name: String) {
        let map: &mut HashMap<String, Measurements>;
        let index: &mut String;
        if for_extrinsic {
            map = &mut data.measurements_per_extrinsic;
            index = &mut data.current_extrinsic;
        } else {
            map = &mut data.measurements_per_fn_name;
            index = &mut data.current_fn;
        }



        if !index.is_empty() {
            let measurements = map
                .entry(index.to_string())
                .or_insert(Measurements::new());
            let counter = measurements.current_per_cat.entry(category_name).or_insert(0);
            *counter += 1;
        }
    }


    fn complete_measurement(&self, data: &mut PolPatrolLoggerMutData, elapsed_ns: u128, for_extrinsic: bool){
        let map: &mut HashMap<String, Measurements>;
        let index: &mut String;
        let time: &mut u128;
        let used_mem: &mut u128;
        if for_extrinsic {
            map = &mut data.measurements_per_extrinsic;
            index = &mut data.current_extrinsic;
            time = &mut data.current_extrinsic_time;
            used_mem = &mut data.used_mem_extrinsic;
        } else {
            map = &mut data.measurements_per_fn_name;
            index = &mut data.current_fn;
            time = &mut data.current_fn_time;
            used_mem = &mut data.used_mem_fn;
        }

        if index.is_empty() {
            return
        }

        let measurements = map 
            .entry(index.to_string())
            .or_insert(Measurements::new());
        let time_for_fn_name = elapsed_ns - *time;
        measurements.avg_total_time += time_for_fn_name;
        measurements.avg_total_mem += *used_mem;
        measurements.avg_amount_of_calls += 1;
        if measurements.maximum_time < time_for_fn_name {
            measurements.maximum_time = time_for_fn_name;
        }
        if measurements.maximum_mem < *used_mem{
            measurements.maximum_mem = *used_mem;
        }
        // Process the environment calls
        // Loop through seen calls 
        for (category, current) in measurements.current_per_cat.iter_mut() {
            // Update total counter
            let counter = measurements.counter_per_cat.entry(category.to_string()).or_insert(0);                            
            *counter += *current;
            // Update max counter
            let max = measurements.max_per_cat.entry(category.to_string()).or_insert(0);
            if current > max {
                *max += *current;
            }
        }
        if for_extrinsic {
            // Remember the weight
            measurements.weight = data.current_extrinsic_weight;
        }
        // Clear current values
        measurements.current_per_cat.clear();    
        *index = "".to_string();
        *time = 0;
        *used_mem = 0;
    }

    fn print_extrinsic_table(&self, extrinsic: &String, measurements_per_extrinsic_map: &HashMap<String, Measurements>) {
        let m = measurements_per_extrinsic_map.get(extrinsic).unwrap();
        println!("[++] {} has been called {} times.", extrinsic, m.avg_amount_of_calls);
        println!("{}", std::iter::repeat("=").take(65).collect::<String>());
        println!(
            "{:38} | {:11} | {:10}",
            "Environment Function", "Mean #Calls", "Max #Calls"
        );
        println!("{}", std::iter::repeat("-").take(65).collect::<String>());
        let mut sorted_tmp: Vec<_> = m.counter_per_cat.iter().collect();
        sorted_tmp.sort();
        for (environment_fn, counter) in sorted_tmp {
            if !self.categories.contains(environment_fn){
                let max_counter = m.max_per_cat.get(environment_fn).unwrap();
                println!("{:38} | {:>11.2} | {:>10}", 
                    environment_fn, 
                    *counter as f64 / m.avg_amount_of_calls as f64,
                    max_counter
                );
            }
        }
        println!("{}", std::iter::repeat("=").take(65).collect::<String>());
        println!("");

    }

}

impl log::Log for PolPatrolLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut data = self.data.lock().unwrap();
            let elapsed_ns: u128 = self.start.elapsed().as_nanos();
            let args: String = record.args().to_string().to_owned();
            let v: Vec<&str> = args.split(' ').collect();
            // Ingnore tool-based operations
            if !data.in_block {
                if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_START_BLOCK_INFO") {
                    data.in_block = true;
                }
            }else if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_END_BLOCK_INFO") {
                data.in_block = false;
                let bn = v[1].to_string().parse().unwrap();
                let bn_weight = v[3].to_string();
                let bn_length = v[5].to_string();

                let stats = data.block_stats.entry(bn).or_insert(BlockStats::new());
                stats.bn = bn;
                stats.weight = bn_weight.parse().unwrap();
                stats.length = bn_length.parse().unwrap();

            }else if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_WASM_EXEC_START_INFO") {
                let fn_name = v[1].to_string();
                data.current_fn_time = elapsed_ns;
                data.current_fn = fn_name;
            }else if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_WASM_EXEC_END_INFO") {
                // End of a Runtime Entry
                let fn_name = v[1].to_string();
                assert_eq!(fn_name, data.current_fn);

                self.complete_measurement(&mut data, elapsed_ns, false);


            }else if v.len() >= 3 && v[2].contains("TAG_") {
                let category_name = v[2].get(4..).unwrap();
                let environment_fn = v[3];

                // Count for global category overview
                {
                let borrowed_counters_map = &mut data.counter_per_category;
                let counter = borrowed_counters_map
                    .entry(category_name.to_string())
                    .or_insert(0);
                *counter += 1;
                }
                // Register this category tag in case a runtime entry is going on
                self.increment_tag(&mut data, true, category_name.to_string());
                // Register this category tag in case an extrinsic is going on
                self.increment_tag(&mut data, false, category_name.to_string());
                // Register this environment function in case a runtime entry is going on
                self.increment_tag(&mut data, true, environment_fn.to_string());
                // Register this environment function in case an extrinsic is going on
                self.increment_tag(&mut data, false, environment_fn.to_string());
            }else if v.len() >= 5 && v[2].contains("used_mem") {
                let new_peak = v[4].parse::<u128>().unwrap();
                if !data.current_fn.is_empty() {
                    if new_peak > data.used_mem_fn {
                        data.used_mem_fn = new_peak;
                    }
                }
                if !data.current_extrinsic.is_empty() {
                    if new_peak > data.used_mem_extrinsic {
                        data.used_mem_extrinsic = new_peak;
                    }
                }
            }else if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_EXTRINSIC_END_INFO") {

                self.complete_measurement(&mut data, elapsed_ns, true);

            } else if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_EXTRINSIC_START_INFO"){
                // Find the beginning of a new extrinsic
                let mut key = String::from("");
                if self.none_extrinsic.is_match(args.as_str()) {
                    let caps = self.none_extrinsic.captures(args.as_str()).unwrap();
                    key = caps.get(1).unwrap().as_str().replace("(", "::");
                } else if self.some_extrinsic.is_match(args.as_str()) {
                    let caps = self.some_extrinsic.captures(args.as_str()).unwrap();
                    key = caps.get(1).unwrap().as_str().replace("(", "::");
                    if key == "Sudo::sudo" {
                        let sudo_caps = self.sudo_extrinsic.captures(args.as_str()).unwrap();
                        key = sudo_caps.get(1).unwrap().as_str().replace("(", "::");
                    }
                }
                if key != "" {
                    data.current_extrinsic = key;
                    data.current_extrinsic_weight = v[v.len() - 1].parse::<u128>().unwrap();
                    data.current_extrinsic_time = elapsed_ns;
                }

/*
            } else if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_START_RANDOMCALLDATA_INFO") {
                let mut s = self.current_extrinsic.lock().unwrap();
                *s = "Random Calldata".to_string();
            } else if v.len() >= 2 && v[0].contains("DEBUG_CS_INSTRUMENTATION_START_RANDOMCOMPONENTS_INFO") {
                let mut s = self.current_extrinsic.lock().unwrap();
                *s = "Random Components".to_string();
*/
            }
            if self.verbose_output_flag {
                eprintln!(
                    ">>> {:.9}=={}nanoseconds {}:{} -- {}",
                    elapsed_ns as f64 / 1e9,
                    elapsed_ns,
                    record.level(),
                    record.target(),
                    args
                );
            }
        }
    }

    fn flush(&self) {
        let data = self.data.lock().unwrap();

        // Checking hardcaps
        const BLOCKEXECTIMELIMIT: u128  = 2000000000; // 2 seconds (in ns)
        const BLOCKLENGTHLIMIT: u128 = 5000000; // 5 MB (in Bytes)
        const BLOCKMEMORYLIMIT: u128 = 10000000000; // 10 GB (in Bytes)
        //const BLOCKSTATEINCREASELIMIT: u128 = 1000; // 1MB (in KB)

        // Initially none has been violated
        let mut maxblockexectimeok = true;
        let mut maxblocklengthok = true;
        let mut maxblockmemoryok = true;
        //let mut maxblockstateincreaseok = true;

        // Check Block Exec Time Limit
        let max_exec_time: u128;
        {
            let m = data.measurements_per_fn_name.get("Core_execute_block").unwrap();
            max_exec_time = m.maximum_time;
            if m.maximum_time > BLOCKEXECTIMELIMIT{
                maxblockexectimeok = false;
            }
        }

        // Block Memory Limit
        // iterate through all extrinsics, take max. memory
        let mut max_mem = 0;
        for (_extrinsic, m) in &data.measurements_per_fn_name {
            if max_mem < m.maximum_mem{
                max_mem = m.maximum_mem;
            }
        }

        if max_mem > BLOCKMEMORYLIMIT{
            maxblockmemoryok = false;
        }

        // Block Length Limit
        // iterate through all block stats, take max length 
        let mut max_blength = 0;
        for (_bn, stats) in &data.block_stats{
            if max_blength < stats.length{
                max_blength = stats.length;
            }
        }

        if max_blength > BLOCKLENGTHLIMIT{
            maxblocklengthok = false;
        }

        println!("[+] Information on executed tests");
        println!("[++] Created and executed blocks: {:>4}", data.block_stats.len());
        println!("[++] Number of Unique Extrinsics: {:>4}", data.measurements_per_extrinsic.len());
        let mut total_extrinsics = 0;
        for (_, m) in &data.measurements_per_extrinsic {
            total_extrinsics += m.avg_amount_of_calls;
        }
        println!("[++] Number of Total Extrinsics:  {:>4}", total_extrinsics);
        println!("");


        println!("[+] Property checks for hardcaps");

        if maxblockexectimeok{
            println!("[++] Maximum block execution time: {:>7.3}s , Never exceeded limit of {:>10.3}s  - OK", max_exec_time as f64 / 1e9, BLOCKEXECTIMELIMIT as f64 / 1e9);
        }else{
            println!("[!!] ########## CAUTION, maximum block execution time has been exceeded! ######### Max allowed: {}, Actual: {} (Units: Nanoseconds)", BLOCKEXECTIMELIMIT, max_exec_time);
        }

        if maxblockmemoryok{
            println!("[++] Maximum block memory:         {:>7.3}MB, Never exceeded limit of {:>10.3}MB - OK", max_mem as f64 / 1e6, BLOCKMEMORYLIMIT as f64 / 1e6);
        }else{
            println!("[!!] ########## CAUTION, maximum block memory allowance has been exceeded! ######### Max allowed: {}, Actual: {} (Units: Bytes)", BLOCKMEMORYLIMIT, max_mem);
        }

        if maxblocklengthok{
            println!("[++] Maximum block length:         {:>7.3}MB, Never exceeded limit of {:>10.3}MB - OK - (measured as bytes of a block, scale-encoded)", max_blength as f64 / 1e6, BLOCKMEMORYLIMIT as f64 / 1e6);
        }else{
            println!("[!!] ########## CAUTION, maximum block length (bytes of a block, scale-encoded) has been exceeded! ######### Max allowed: {}, Actual: {} (Units: Bytes)", BLOCKMEMORYLIMIT, max_blength);
        }

        println!("");




        if self.raw_output_flag {
            println!("########## RAW OUTPUT FLAG TURNED ON! #########");
            println!("------------------------------------------------------------------");
            println!("category,total_calls");
            let mut sorted_tmp: Vec<_> = data.counter_per_category.iter().collect();
            sorted_tmp.sort();
            for (category, counter) in sorted_tmp {
                println!("{},{}", category, counter);
            }

            println!("------------------------------------------------------------------");
            println!("extrinsic,category,total_calls");
            let mut sorted_tmp: Vec<_> = data.measurements_per_extrinsic.iter().collect();
            // Sort by keys
            sorted_tmp.sort_by(|a, b| a.0.cmp(b.0));
            for (extrinsic, m) in sorted_tmp {
                let mut sorted_tmp2: Vec<_> = m.counter_per_cat.iter().collect();
                sorted_tmp2.sort();
                for (category, counter) in sorted_tmp2 {
                    println!("{},{},{}", extrinsic, category, counter);
                }
            }

            println!("------------------------------------------------------------------");
            println!("entry_name,num_calls,mean_time,max_time,mean_mem,max_mem,mean_num_storage_calls,max_num_storage_calls");
            let mut sorted_tmp: Vec<_> = data.measurements_per_fn_name.iter().collect();
            sorted_tmp.sort_by(|a, b| a.0.cmp(b.0));
            for (fn_name, m) in sorted_tmp {
                println!(
                    "{},{},{},{},{},{},{},{}",
                    fn_name,
                    m.avg_amount_of_calls,
                    m.avg_total_time as f64 / m.avg_amount_of_calls as f64 / 1e9,
                    m.maximum_time as f64 / 1e9,
                    m.avg_total_mem as f64 / m.avg_amount_of_calls as f64 / 1e6,
                    m.maximum_mem as f64 / 1e6,
                    *(m.counter_per_cat.get(&"Storage".to_string()).unwrap_or(&0)) as f64 / m.avg_amount_of_calls as f64,
                    m.max_per_cat.get(&"Storage".to_string()).unwrap_or(&0)
                );
            }

            println!("------------------------------------------------------------------");
            println!("extrinsic,num_calls,mean_time,max_time,mean_mem,max_mem,mean_num_storage_calls,max_num_storage_calls,weight");
            let mut sorted_tmp: Vec<_> = data.measurements_per_extrinsic.iter().collect();
            sorted_tmp.sort_by(|a, b| a.0.cmp(b.0));
            for (extrinsic, m) in sorted_tmp {
                println!(
                    "{},{},{},{},{},{},{},{},{}",
                    extrinsic,
                    m.avg_amount_of_calls,
                    m.avg_total_time as f64 / m.avg_amount_of_calls as f64 / 1e9,
                    m.maximum_time as f64 / 1e9,
                    m.avg_total_mem as f64 / m.avg_amount_of_calls as f64 / 1e6,
                    m.maximum_mem as f64 / 1e6,
                    *(m.counter_per_cat.get(&"Storage".to_string()).unwrap_or(&0)) as f64 / m.avg_amount_of_calls as f64,
                    m.max_per_cat.get(&"Storage".to_string()).unwrap_or(&0),
                    m.weight
                );
            }
        // Pretty Printing instead of raw output
        } else {
            println!("[+] Printing Statistics about Calls to the Environment");
            println!("{}", std::iter::repeat("=").take(51).collect::<String>());
            println!("{:32} | {:16}", "Category", "#Calls");
            println!("{}", std::iter::repeat("-").take(51).collect::<String>());
            let mut sorted_tmp: Vec<_> = data.counter_per_category.iter().collect();
            sorted_tmp.sort();
            for (category, counter) in sorted_tmp {
                println!("{:32} | {:>16}", category, counter);
            }
            println!("{}", std::iter::repeat("=").take(51).collect::<String>());
            println!("");

            println!("[+] Printing Statistics about Calls to the Environment split by categories and applied extrensics");
            println!("{}", std::iter::repeat("=").take(76).collect::<String>());
            println!("{:38} | {:16} | {:16}", "Extrinsic", "Category", "Number of calls");
            println!("{}", std::iter::repeat("-").take(76).collect::<String>());
            let mut sorted_tmp: Vec<_> = data.measurements_per_extrinsic.iter().collect();
            // Sort by keys
            sorted_tmp.sort_by(|a, b| a.0.cmp(b.0));
            for (extrinsic, m) in sorted_tmp {
                let mut sorted_tmp2: Vec<_> = m.counter_per_cat.iter().collect();
                sorted_tmp2.sort();
                for (category, counter) in sorted_tmp2 {
                    if self.categories.contains(category){
                        println!("{:38} | {:16} | {2:>16}", extrinsic, category, counter);
                    }
                }
            }
            println!("{}", std::iter::repeat("=").take(76).collect::<String>());
            println!("");

            println!("[+] Printing Statistics about Calls to the Runtime Entries");
            println!("[++] Storage refers to the number of storage-related Environment calls");
            println!("{}", std::iter::repeat("=").take(143).collect::<String>());
            println!(
                "{:38} | {:7} | {:14} | {:13} | {:13} | {:12} | {:13} | {:12}",
                "Entry Name", "# Calls", "Mean time (s)", "Max Time (s)", "Mean Mem (MB)", "Max Mem (MB)", "Mean #Storage", "Max #Storage"
            );
            println!("{}", std::iter::repeat("-").take(143).collect::<String>());
            let mut sorted_tmp: Vec<_> = data.measurements_per_fn_name.iter().collect();
            sorted_tmp.sort_by(|a, b| a.0.cmp(b.0));
            for (fn_name, m) in sorted_tmp {
                println!(
                    "{:38} | {:>7} | {2:>14.8} | {3:>13.8} | {4:>13.3} | {5:>12.3} | {6:>13.2} | {7:>12.2}",
                    fn_name,
                    m.avg_amount_of_calls,
                    m.avg_total_time as f64 / m.avg_amount_of_calls as f64 / 1e9,
                    m.maximum_time as f64 / 1e9,
                    m.avg_total_mem as f64 / m.avg_amount_of_calls as f64 / 1e6,
                    m.maximum_mem as f64 / 1e6,
                    *(m.counter_per_cat.get(&"Storage".to_string()).unwrap_or(&0)) as f64 / m.avg_amount_of_calls as f64,
                    m.max_per_cat.get(&"Storage".to_string()).unwrap_or(&0),
                );
            }
            println!("{}", std::iter::repeat("=").take(143).collect::<String>());
            println!("");

            let mut slowest_extrinsic_time = 0;
            let mut slowest_extrinsic = String::from("");
            let mut cheapest_extrinsic_price = 0.0;
            let mut cheapest_extrinsic = String::from("");
            println!("[+] Printing Statistics about applied Extrinsics");
            println!("[++] Storage refers to the number of storage-related Environment calls");
            println!("{}", std::iter::repeat("=").take(154).collect::<String>());
            println!(
                "{:38} | {:7} | {:14} | {:13} | {:13} | {:12} | {:13} | {:12} | {:8}",
                "Entry Name", "# Calls", "Mean time (s)", "Max Time (s)", "Mean Mem (MB)", "Max Mem (MB)", "Mean #Storage", "Max #Storage", "Weight"
            );
            println!("{}", std::iter::repeat("-").take(154).collect::<String>());
            let mut sorted_tmp: Vec<_> = data.measurements_per_extrinsic.iter().collect();
            sorted_tmp.sort_by(|a, b| a.0.cmp(b.0));
            for (extrinsic, m) in sorted_tmp {
                println!(
                    "{:38} | {:>7} | {2:>14.8} | {3:>13.8} | {4:>13.3} | {5:>12.3} | {6:>13.2} | {7:>12.2} | {8:>8}",
                    extrinsic,
                    m.avg_amount_of_calls,
                    m.avg_total_time as f64 / m.avg_amount_of_calls as f64 / 1e9,
                    m.maximum_time as f64 / 1e9,
                    m.avg_total_mem as f64 / m.avg_amount_of_calls as f64 / 1e6,
                    m.maximum_mem as f64 / 1e6,
                    *(m.counter_per_cat.get(&"Storage".to_string()).unwrap_or(&0)) as f64 / m.avg_amount_of_calls as f64,
                    m.max_per_cat.get(&"Storage".to_string()).unwrap_or(&0),
                    m.weight
                );
                // Find the slowest extrinsic
                if m.maximum_time > slowest_extrinsic_time {
                    slowest_extrinsic_time = m.maximum_time;
                    slowest_extrinsic = extrinsic.to_string();
                }
                // Find the cheapest extrinsic
                let price = m.weight as f64 / m.maximum_time as f64;
                if (m.weight > 0 && price < cheapest_extrinsic_price) || cheapest_extrinsic_price == 0.0 {
                    cheapest_extrinsic_price = price;
                    cheapest_extrinsic = extrinsic.to_string();
                }
            }
            println!("{}", std::iter::repeat("=").take(154).collect::<String>());
            println!("");

            println!("[+] Evaluation on applied Extrinsics");
            println!("[++] The cheapest extrinsic (ratio of weight over time) was: {}", cheapest_extrinsic);
            println!("[++] The extrinsic with the highest execution time was: {}", slowest_extrinsic);
            println!("");


            if self.verbose_output_flag {
                let mut sorted_tmp: Vec<_> = data.measurements_per_extrinsic.iter().collect();
                sorted_tmp.sort_by(|a, b| a.0.cmp(b.0));
                for (extrinsic, _) in sorted_tmp{
                println!("[++] Printing Statistics about Extrinsic: {}", extrinsic);
                    self.print_extrinsic_table(&extrinsic, &data.measurements_per_extrinsic);
                }
            } else {
                println!("[+] Printing Statistics about slowest Extrinsics: {}", slowest_extrinsic);
                println!("[++] Showing precise information about Environment calls");
                self.print_extrinsic_table(&slowest_extrinsic, &data.measurements_per_extrinsic);

                if slowest_extrinsic != cheapest_extrinsic {
                    println!("[+] Printing Statistics about cheapest Extrinsics: {}", cheapest_extrinsic);
                    println!("[++] Showing precise information about Environment calls");
                    self.print_extrinsic_table(&cheapest_extrinsic, &data.measurements_per_extrinsic);

                }
            }
        }

    }
}


pub fn init_simple_logger(raw_output_flag: bool, verbose_output_flag: bool) -> Result<(), SetLoggerError> {
    let logger = PolPatrolLogger {
        start: Instant::now(),
        data: Arc::new(Mutex::new(PolPatrolLoggerMutData::new())),
        verbose_output_flag,
        raw_output_flag,
        none_extrinsic: Regex::new(r"DEBUG_CS_INSTRUMENTATION_EXTRINSIC_START_INFO UncheckedExtrinsic\(None, ([A-z]*\([A-z]*)").unwrap(),
        some_extrinsic: Regex::new(r"DEBUG_CS_INSTRUMENTATION_EXTRINSIC_START_INFO UncheckedExtrinsic\(Some[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*, ([A-z]*\([A-z]*)").unwrap(),
        sudo_extrinsic: Regex::new(r"DEBUG_CS_INSTRUMENTATION_EXTRINSIC_START_INFO UncheckedExtrinsic\(Some[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*,[A-z0-9() .<>]*, [A-z]*\([A-z]*\(([A-z]*\([A-z]*)").unwrap(),
        categories: ["Storage".to_string(), "Memory".to_string(), "Hash".to_string(), "Crypto".to_string(), "Other".to_string()],
    };
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}

fn main() {
    //parse cmd line arguments
    let opt = Opt::from_args();

    init_simple_logger(opt.raw_output, opt.verbose).unwrap();

    println!("\n\n##### PolPatrol - Polkadot Runtime Checker #####\n------------------------------------------------\n\n");
    println!("\nAs part of the execution you may see some outputs from Substrate and Polkadot code, as it is being tested. PolPatrol intentionally triggers error conditions within that code to test the corresponding behavior. Consequently, error messages may appear.\n\n");

    // Load the runtime wasm blob to be inspected
    let mut file = File::open(opt.runtimefile.clone()).expect("Error opening wasm runtime");
    let mut wasmbin = Vec::new();
    file.read_to_end(&mut wasmbin).expect("Error reading file");
    info!("Loaded runtime to inspect: {:?}\n", file);

    //TODO: Do some basic checks on the exported functions:
    //https://webassembly.org/docs/binary-encoding/#export-section

    //Set up the Externalities, the execution environment the wasm blob is executed in. This includes the initial state of the storage.

    let spec = ChainSpec::<GenesisConfig>::from_json_file(opt.genesisfile)
        .expect("Error opening genesis file");

    /*
        Externalities define the execution environment the wasm blob is executed in, this includes the state of the storage
        Thus we need  2x execution environments: 1x to create a new block and 1x to execute the block (otherwise we would do the "same" storage changes twice, which doesn't work)
        As neither the COPY nor CLONE trait is implemented for these types, we have to manually set up 2x
        TODO: to be cleaned up, but works for now

        3rd used for random blackbox testing
    */

    let mut ext_build_block = TestExternalities::<Blake2Hasher>::new(
        spec.build_storage()
            .expect("Unable to build storage using the genesis"),
    );
    let mut ext_execute_block = TestExternalities::<Blake2Hasher>::new(
        spec.build_storage()
            .expect("Unable to build storage using the genesis"),
    );
    let mut ext_random = TestExternalities::<Blake2Hasher>::new(
        spec.build_storage()
            .expect("Unable to build storage using the genesis"),
    );

    // Use as parent_hash for Block 0
    let genesis_hash = H256::zero();
    let initial_bn = 1;

    //printing storage root
    info!(
        "Storage has been initialized, current storage root hash: {}",
        ext_build_block.storage_root()
    );

    //Fetch version of runtime - we need it later to craft a proper signature for the extrinsics

    let calldata: &[u8] = &Vec::new(); // empty calldata
    let returnedversion =
        call_exported_fn(&mut ext_build_block, &wasmbin, "Core_version", calldata)
            .expect("wasm call to Core_version failed");
    let version =
        RuntimeVersion::decode(&mut &returnedversion[..]).expect("Unable to decode version");
    info!("\nThe version of the loaded runtime is: {:?}", version);

    //Fetching metadata

    let calldata: &[u8] = &Vec::new(); // empty calldata
    let returnedmetadata = call_exported_fn(
        &mut ext_build_block,
        &wasmbin,
        "Metadata_metadata",
        calldata,
    )
    .expect("wasm call to Metadata_metadata failed");
    // This decoding doesn't work, Error("Error decoding field RuntimeMetadataPrefixed.1")
    // What exact "Metadata" datatype does the runtime use before it converts it into OpaqueMetadata (which is just a vec![u8])?
    // (hidden inside a macro . . .)
    //let modulescalls = DecodeDifferent::<OpaqueMetadata, RuntimeMetadataPrefixed>::decode(&mut &returnedmetadata[..]).unwrap();
    //info!("\nAvailable calls/modules: {}", modulescalls);
    let _modulescalls = OpaqueMetadata::decode(&mut &returnedmetadata[..])
        .expect("Unable to decode OpaqueMetadata");

    // Part 1 test random calldata
    random::test_total_random_inputs(opt.seed, &mut ext_random, &wasmbin, opt.random_type_one);
    random::test_encoded_random_inputs(
        opt.seed,
        &mut ext_random,
        &wasmbin,
        returnedversion,
        opt.random_type_two,
    );

    // Part 2 proper block building

    //Crafting pool of extrinsics here

    //We need to select the extrinsics we include in the block here to correctly calculate & set the extrinsic_root in the header!
    let mut pool: Vec<ReadyEx> = generation::craft_testcases(genesis_hash, version);
    //We push and pop extrinsics -> need to reverse the order otherwise the nonces are the wrong way round
    pool.reverse();
    info!(
        "Created extrinsics pool, contains {:?} extrinsics",
        pool.len()
    );

    //While the pool is not empty, craft the next block with a set of ex's

    let mut bn = initial_bn;
    let mut parent_hash = genesis_hash;

    while !pool.is_empty() {
        info!("DEBUG_CS_INSTRUMENTATION_START_BLOCK_INFO {}", bn);

        //Decide on next messages's to include

        let mut messages: Vec<UncheckedExtrinsicPolkadot> = Vec::new();
        messages.append(&mut fetch_next_extrinsics(&mut pool, bn, opt.num));

        // 1st craft the inherents
        let mut inherents = craft_inherents(&mut ext_build_block, &wasmbin);

        //inherents must be first . . .
        inherents.append(&mut messages);
        messages = inherents;

        //craft initial block header
        let mut header = Header {
            parent_hash: parent_hash.into(), //ok, see comment where genesis_hash is defined
            number: bn, // Polkadot Blocks start on 0, but number 0 leads to a crash here thus starting at 1 . . .
            state_root: ext_build_block.storage_root(), // ok
            extrinsics_root: Layout::<Blake2Hasher>::ordered_trie_root(
                messages.iter().map(Encode::encode),
            ),
            digest: Digest {
                logs: vec![].into(),
            }, // empty logs, o.k.
        };

        debug!(" \n Header: {:?} \n", header);

        //Initialize the Runtime with this Block header
        // encode this block header with the SCALE codec before passing it as calldata
        let calldata = &Header::encode(&mut header);
        // Pass the Block header to the runtime, set's the runtime environment to the correct block number / storage root etc. (that's  a guess! Not documented, not confirmed)
        let _ = call_exported_fn(
            &mut ext_build_block,
            &wasmbin,
            "Core_initialize_block",
            calldata,
        )
        .expect("wasm call to Core_initialize_block failed");

        //--------------------------------------------------------
        // Adding signed extrinsics - aka transactions

        let mut blockweight = 0;

        for ex in messages.iter() {
            //we pass the encoded UncheckedExtrinsic
            let exweight = ex.function.get_dispatch_info().weight;
            info!(
                "DEBUG_CS_INSTRUMENTATION_EXTRINSIC_START_INFO {:?}, {}",
                ex, exweight
            );
            blockweight += exweight;
            let calldata = &UncheckedExtrinsicPolkadot::encode(&ex);
            let res = call_exported_fn(
                &mut ext_build_block,
                &wasmbin,
                "BlockBuilder_apply_extrinsic",
                calldata,
            )
            .expect("wasm call to BlockBuilder_apply_extrinsic failed");
            let apply_res =
                ApplyResult::decode(&mut &res[..]).expect("Decoding the ApplyResult failed");
            debug!("Decoded result of apply extrinsic is {:?}", apply_res);
            info!(
                "DEBUG_CS_INSTRUMENTATION_EXTRINSIC_END_INFO {:?}",
                apply_res
            );
        }

        //--------------------------------------------------------

        //Finalizing the block after all extrinsics have been applied
        let calldata: &[u8] = &Vec::new();
        let returnedheader = call_exported_fn(
            &mut ext_build_block,
            &wasmbin,
            "BlockBuilder_finalize_block",
            calldata,
        )
        .expect("Unable to decode returned data from wasm");
        let header =
            Header::decode(&mut &returnedheader[..]).expect("Decoding the returned header failed");
        let headercpy = header.clone();
        debug!("Header before core execute block: {:?}", headercpy);
        // Craft block using the header & the Extrinsic
        let next_block = Block {
            header: header,
            extrinsics: messages,
        };

        // we pass the encoded block
        let calldata = &Block::encode(&mut &next_block);
        let _res = call_exported_fn(
            &mut ext_execute_block,
            &wasmbin,
            "Core_execute_block",
            calldata,
        );
        
        info!("DEBUG_CS_INSTRUMENTATION_END_BLOCK_INFO {} TOTAL_WEIGHT {} LENGTH {}", bn, blockweight, calldata.len()*mem::size_of::<u8>());

        assert_eq!(ext_execute_block.storage_root(), ext_build_block.storage_root(), "Storage root mismatch, storages of ext_execute_block and ext_build_block do not match! Most likely this is due to an extrinsic that did apply successfully during block generation. Please use -v to identify the extrinsic in question and make sure this extrinsic is expected to fail. Please remove the extrinsic in case it behaved as expected and rerun.");

        //printing storage root . . .
        debug!(
            "Current storage root hash after execute block of ext_execute_block: {}",
            ext_execute_block.storage_root()
        );

        debug!(
            "Current storage root hash after execute block of ext_build_block: {}",
            ext_build_block.storage_root()
        );

        info!("Hash of the new Block: {:?}", headercpy.hash());
        //Update hash and bn for next iteration of the loop
        parent_hash = headercpy.hash();
        bn += 1;
    }

    /* Some Inherents, e.g. set() of the timestamp module or set_heads() of the parachain module can only be applied once per block. To craft proper blocks successfully, these Inherents with valid parameters are needed.
    Consequently, in the execution above these cannot be tested with arbitrary inputs. To test these modules with arbitrary inputs, "empty blocks" could be generated here: Add all required inherents expect the one to be tested
    correctly to the block + add the one to be tested with arbitrary input.
    */

    println!("\n\n##### PolPatrol Results #####\n-----------------------------\n");
    println!("For Runtime: {:?}\n", opt.runtimefile);

    log::logger().flush();

    info!("[+] Executed successfully");
}
