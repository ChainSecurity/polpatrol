/* Module intended to aggregate testcases of the different modules */

use substrate_keyring::{AccountKeyring, AccountKeyring::*};

use substrate_primitives::crypto::*;
use substrate_primitives::{
    blake2_256,
    offchain::{OpaqueNetworkState, OpaquePeerId},
    Blake2Hasher, H256,
};

use substrate_sr_balances as balances;
use substrate_sr_primitives::generic::Era;
use substrate_sr_sudo::Call as SudoCall;
use substrate_sr_system as system;

use substrate_version::RuntimeVersion;

use polkadot_runtime::Runtime;
use polkadot_runtime::{Call, OnlyStakingAndClaims, SignedExtra, UncheckedExtrinsic};

use codec::*;
use hash_db::Hasher;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

mod balancesmodule;
mod claimsmodule;
mod democracymodule;
mod finalitytrackermodule;
mod imonlinemodule;
mod slotsmodule;
mod timestampmodule;
mod councilmodule;

// Struct to collect prepared test cases, function call and sender
pub struct PreparedEx {
    function: Call,
    sender: Option<AccountKeyring>,
    minblock: u32,
}

// Struct to collect test cases ready to be applied
pub struct ReadyEx {
    pub extrinsic: UncheckedExtrinsic,
    pub minblock: u32,
}

/*
    Crafts and returns the extras needed for an unchecked extrinsic

    Input:
    sender: AccountKeyring of the sender
    nonces: Hashmap with accounts & used nonces

    Returns: A signed Extra
*/
fn craft_extras(sender: &AccountKeyring, nonces: &mut HashMap<AccountKeyring, u32>) -> SignedExtra {
    let sender_nonce = match nonces.get(sender) {
        // Account already in Hashmap
        Some(x) => *x,
        None => 0,
    };

    //increase sender's nonce (Caution TODO: handle this correctly when ex fails)
    nonces.insert(*sender, sender_nonce + 1);

    let check_version = system::CheckVersion::<Runtime>::new();
    let check_genesis = system::CheckGenesis::<Runtime>::new();
    let check_era = system::CheckEra::<Runtime>::from(Era::Immortal);
    let check_nonce = system::CheckNonce::<Runtime>::from(sender_nonce);
    let check_weight = system::CheckWeight::<Runtime>::new(); //Caution, weights not implemented correctly
    let take_fees = balances::TakeFees::<Runtime>::from(0); // Cautions fees not implemented correctly

    (
        OnlyStakingAndClaims,
        check_version,
        check_genesis,
        check_era,
        check_nonce,
        check_weight,
        take_fees,
    )
}

/*
    Crafts and returns a vector of UncheckedExtrinsics (a.k.a. test cases)

    Input: Hash of the genesis block, runtime version. Both needed to craft the UncheckedExtrinsic correctly

    Returns: A Vector of Unchecked Extrinsics
*/
pub fn craft_testcases(genesis_hash: H256, version: RuntimeVersion) -> Vec<ReadyEx> {
    /*Todo: All modules we should create test cases for can "register" here
    Then we iterate through the list
    Finally we assemble them in a vector
    */

    //Sudo key
    debug!("\nAlice's public key: {}", Alice.public().to_ss58check());

    //Normal accounts
    debug!("Bob's public key: {}", Bob.public().to_ss58check());
    debug!("Charlie's public key: {}", Charlie.public().to_ss58check());
    debug!("Dave's public key: {}", Dave.public().to_ss58check());
    debug!("Eve's public key: {}\n", Eve.public().to_ss58check());

    //---------- Craft these ex (use the different modules)
    let mut exs = Vec::new();

    exs.append(&mut balancesmodule::craft_extrinsics());
    exs.append(&mut claimsmodule::craft_extrinsics());
    exs.append(&mut imonlinemodule::craft_extrinsics());
    exs.append(&mut slotsmodule::craft_extrinsics());
    exs.append(&mut finalitytrackermodule::craft_extrinsics());
    exs.append(&mut democracymodule::craft_extrinsics());
    exs.append(&mut councilmodule::craft_extrinsics());
    //exs.append(&mut timestampmodule::craft_extrinsics());

    // ----- Finalize them, create the signature
    let mut uexs = Vec::new();
    let mut nonces = HashMap::new();

    for ex in exs.iter() {
        //Unchecked Extrinsic: Differentiate between signed extrinsic and inherents
        match ex.sender {
            Some(sender) => {
                //signed extrinsic
                let extra = craft_extras(&sender, &mut nonces);
                let raw_payload = (
                    ex.function.clone(),
                    extra.clone(),
                    version.spec_version,
                    genesis_hash,
                    genesis_hash,
                );
                let signature = raw_payload.using_encoded(|payload| {
                    if payload.len() > 256 {
                        sender.sign(&blake2_256(payload)[..])
                    } else {
                        sender.sign(payload)
                    }
                });

                let uex = UncheckedExtrinsic::new_signed(
                    ex.function.clone(),
                    sender.public().into(),
                    signature.into(),
                    extra,
                );

                uexs.push(ReadyEx {
                    extrinsic: uex,
                    minblock: ex.minblock,
                });
            }
            None => {
                //inherent
                let uex = UncheckedExtrinsic {
                    signature: None,
                    function: ex.function.clone(),
                };
                uexs.push(ReadyEx {
                    extrinsic: uex,
                    minblock: ex.minblock,
                });
            }
        }
    }
    //return the vector of extrinsics ready to be applied
    uexs
}
