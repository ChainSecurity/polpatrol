/* Module intended to test random input*/

use substrate_primitives::{Blake2Hasher, H256};
use substrate_sr_primitives::Digest;

use std::fs::File;
use std::io::Read;

use rand::{rngs::StdRng, Rng, SeedableRng};

use codec::{Decode, Encode};

use crate::{
    call_exported_fn, generation, generation::ReadyEx, Block, Header, RuntimeVersion,
    TestExternalities, UncheckedExtrinsicPolkadot,
};

/*
    Black box testing totally random inputs to exposed functions. Expected to fail.

    Input :
    ext: Externalities, the externalities/Environment the wasm blob is to be executed in
    wasmbin: the wasm binary
*/

pub fn test_total_random_inputs(
    seed: u64,
    mut ext: &mut TestExternalities<Blake2Hasher>,
    wasmbin: &[u8],
    mut iterations: u32,
) {
    info!("DEBUG_CS_INSTRUMENTATION_START_RANDOMCALLDATA_INFO");
    let mut functions = Vec::<String>::new();

    //Functions to be tested
    while iterations > 0 {
        functions.push("Core_execute_block".to_string());
        functions.push("BlockBuilder_apply_extrinsic".to_string());
        functions.push("BlockBuilder_finalize_block".to_string());
        functions.push("Core_initialize_block".to_string());
        functions.push("BlockBuilder_inherent_extrinsics".to_string());

        iterations -= 1;
    }
    let mut rng = random_seed_rng(seed);

    while !functions.is_empty() {
        //Randomly select size of calldata & generate random data
        let mut calldata = vec![0; rng.gen_range(0, 256)];
        let mut file = File::open("/dev/urandom").expect("Error getting randomness");
        file.read_exact(&mut calldata)
            .expect("Error reading randomness");
        match call_exported_fn(&mut ext, &wasmbin, &functions.pop().unwrap(), &calldata) {
            Ok(_result) => {
                //very unlikely, random calldata lead to an OK result
                info!(
                    "DEBUG_CS_INSTRUMENTATION_RANDOM_CALLDATA_SUCCESS {:?} calldata: {:?}",
                    _result, calldata
                );
            }
            Err(_e) => {
                // Don't worry - random input, we expect an error to happen, all good.
            }
        }
    }
    info!("DEBUG_CS_INSTRUMENTATION_END_RANDOMCALLDATA_INFO");
}

// Struct to collect Functions infos (name and expected input type)
struct FunctionInfo {
    name: String,
    input: Vec<u8>,
}

/*
    Black box testing random inputs encoded as the proper type for the respective exposed functions. Expected to fail.
    We need to pass the proper version (for now)

    Input :
    ext: Externalities, the externalities/Environment the wasm blob is to be executed in
    wasmbin: the wasm binary
    version: the RuntimeVersion, enocded. Required to build extrinsics
*/
pub fn test_encoded_random_inputs(
    seed: u64,
    mut ext: &mut TestExternalities<Blake2Hasher>,
    wasmbin: &[u8],
    version: Vec<u8>,
    mut iterations: u32,
) {
    info!("DEBUG_CS_INSTRUMENTATION_START_RANDOMCOMPONENTS_INFO");
    let mut rng = random_seed_rng(seed);

    let mut functions = Vec::new();

    while iterations > 0 {
        functions.push(FunctionInfo {
            name: "Core_execute_block".to_string(),
            input: Block::encode(&generate_random_block(
                &mut rng,
                RuntimeVersion::decode(&mut &version[..]).unwrap(),
            )),
        });
        functions.push(FunctionInfo {
            name: "BlockBuilder_apply_extrinsic".to_string(),
            input: UncheckedExtrinsicPolkadot::encode(&generate_random_extrinsic(
                &mut rng,
                RuntimeVersion::decode(&mut &version[..]).unwrap(),
            )),
        });
        functions.push(FunctionInfo {
            name: "Core_initialize_block".to_string(),
            input: Header::encode(&generate_random_header(&mut rng)),
        });

        iterations -= 1;
    }
    while !functions.is_empty() {
        let next = functions.pop().unwrap(); // cannot be empty, q.e.d.
        match call_exported_fn(&mut ext, &wasmbin, &next.name, &next.input) {
            Ok(_result) => {
                //very unlikely, random calldata lead to an OK result
                info!(
                    "DEBUG_CS_INSTRUMENTATION_RANDOM_CALLDATA_SUCCESS {:?}",
                    _result
                );
            }
            Err(_e) => {
                // Don't worry - random input, we expect an error to happen, all good.
            }
        }
    }
    info!("DEBUG_CS_INSTRUMENTATION_END_RANDOMCOMPONENTS_INFO");
}

/*
    Generates a random block

    Input :
    rng: a StdRng random generator
    version: the RuntimeVersion, enocded. Required to build the extrinsics

    Returns:
    A randomly generated block
*/
fn generate_random_block(rng: &mut StdRng, version: RuntimeVersion) -> Block {
    Block {
        header: generate_random_header(rng),
        extrinsics: generate_vector_of_random_extrinsics(rng, version),
    }
}

/*
    Generates a random extrinsics

    Input :
    &rng: a StdRng random generator
    version: the RuntimeVersion, enocded. Required to build the extrinsics
    Returns:
    A randomly generated UncheckedExtrinsic
*/
fn generate_random_extrinsic(
    rng: &mut StdRng,
    version: RuntimeVersion,
) -> UncheckedExtrinsicPolkadot {
    //Unable to craft proper "random" Extrinsics - for using our pool of ex and grabbing one
    let mut pool: Vec<ReadyEx> = generation::craft_testcases(H256::random_using(rng), version);
    let mut rand = rng.gen_range(0, pool.len() - 1); // len -1 so there is always one to pop for the return statement at the very end

    while rand > 0 {
        pool.pop().unwrap();
        rand -= 1;
    }
    pool.pop().unwrap().extrinsic
}

/*
    Generates a vector containing a random amount of random extrinsics

    Input :
    rng: a StdRng random generator
    version: the RuntimeVersion, enocded. Required to build the extrinsics
    Returns:
    A vector of random unchecked extrinsics
*/
fn generate_vector_of_random_extrinsics(
    rng: &mut StdRng,
    version: RuntimeVersion,
) -> Vec<UncheckedExtrinsicPolkadot> {
    //iterate a random amount of times on generate_random_extrinsic(), craft vector
    let mut amount: u8 = rng.gen(); // 0-255
    let mut pool = Vec::new();

    while amount > 0 {
        pool.push(generate_random_extrinsic(rng, version.clone()));
        amount -= 1;
    }
    pool
}

/*
    Generates a random header

    Input:
    rng: a StdRng random generator

    Returns:
    A block header containing random data
*/
fn generate_random_header(rng: &mut StdRng) -> Header {
    //craft random block header
    let bn: u32 = rng.gen();

    Header {
        parent_hash: H256::random_using(rng),
        number: bn,
        state_root: H256::random_using(rng),
        /*extrinsics_root: Layout::<Blake2Hasher>::ordered_trie_root(
            messages.iter().map(Encode::encode),
        ),*/ //Could be when creating random block, create proper extrinsics root over the random extrinsics
        extrinsics_root: H256::random_using(rng),
        digest: Digest {
            logs: vec![].into(),
        }, // empty logs, can be randomized in the future
    }
}

fn random_seed_rng(seed: u64) -> StdRng {
    // if seed == 0 this equals to the default seed:
    //let default_seed = <StdRng as SeedableRng>::Seed::default();
    info!(
        "Random generator has been initialized, seed generated from following u64 passed as cmd line parameter: {}",
         seed
    );
    StdRng::seed_from_u64(seed)
}
