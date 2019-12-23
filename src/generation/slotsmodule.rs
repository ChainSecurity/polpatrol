// ------ Polkadot Slots MODULE ------
use polkadot_runtime::{slots, Call};

use hex_literal::hex;

use super::*;

pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();
    //Loading demo parachain module to be used:
    let mut file = File::open("./res/adder.compact.wasm").expect("Error opening wasm file");
    let mut code = Vec::new();
    file.read_to_end(&mut code).expect("Error reading file");

    // Initial head data for the adder module - fetched from it's collator binary
    let initial_head: Vec<u8> = hex![00000000000000000000000000000000000000000000000000000000000000000000000000000000011b4d03dd8c01f1049143cf9c4c817e4b167f1d1b83e5c6f0f10d89ba1e7bce].to_vec();

    /*
    Extrinsic Description
    Module: Slots
    Function: new_auction
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Attempting to create a new Auction should fail as it requires root_origin
     */
    /*
    let duration: u32 = 42;
    let leaseduration: u32 = 42;

    let ex = PreparedEx {
        function: Call::Slots(slots::Call::new_auction(duration, leaseduration)),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);
    */
    /*
    Extrinsic Description
    Module: Slots
    Function: new_auction
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Creating a new auction, using the Sudo module to dispatch this call
     */

    let duration: u32 = 5;
    let leaseduration: u32 = 10;

    let ex = PreparedEx {
        function: Call::Sudo(SudoCall::sudo(
            Call::Slots(slots::Call::new_auction(duration, leaseduration)).into(),
        ))
        .into(),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Slots
    Function: bid
    Type: Signed Extrinsic, ensure_signed()

    Description: bidding for an open auction

    */

    let sub: u32 = 0;
    let auction_index: u32 = 1;
    let first_slot: u32 = 10; // auction window must be active, see bn ex new_auction + duration (minimum)
    let last_slot: u32 = 11; // auction window must be active, see bn ex new_auction + duration + leaseduration (maximum)
    let amount: u128 = 400000;

    let ex = PreparedEx {
        function: Call::Slots(slots::Call::bid(
            sub,
            auction_index,
            first_slot,
            last_slot,
            amount,
        )),
        sender: Some(Bob),
        minblock: 4,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Slots
    Function: bid_renew
    Type: Signed Extrinsic, ensure_signed()

    Description: Renewing bidding for an open auction

    */

    /*
    let auction_index: u32 = 1;
    let first_slot: u32 = 10; // auction window must be active, see bn ex new_auction + duration (minimum)
    let last_slot: u32 = 11; // auction window must be active, see bn ex new_auction + duration + leaseduration (maximum)
    let amount: u128 = 5;

    let ex = PreparedEx {
        function: Call::Slots(slots::Call::bid_renew(
            auction_index,
            first_slot,
            last_slot,
            amount,
        )),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Slots
    Function: set_offboarding
    Type: Signed Extrinsic, ensure_signed()

    Description: Offboarding a parachain

    */

    //TODO: assign proper values!
    let parachainaccount = Bob.public();

    let ex = PreparedEx {
        function: Call::Slots(slots::Call::set_offboarding(parachainaccount.into())),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);
    */

    /*
    Extrinsic Description
    Module: Slots
    Function: fix_deploy_data
    Type: Signed Extrinsic, ensure_signed()

    Description: After a successful bid, setting the deploy information to deploy a new parachain

    */

    //TODO: fix assigned value, currently fails due to "parachain not in onboarding"
    let sub: u32 = 1;
    let para_id: u32 = 1;

    //Calculate hash of the code
    let code_hash = Blake2Hasher::hash(&code);

    let ex = PreparedEx {
        function: Call::Slots(slots::Call::fix_deploy_data(
            sub,
            para_id.into(),
            code_hash,
            initial_head.clone(),
        )),
        sender: Some(Bob),
        minblock: 5,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Slots
    Function: elaborate_deploy_data
    Type: Extrinsic. No authentication, relies on hash collision is not possible

    Description: After a successful bid, deploying the code of the parachain

    */

    //TODO: assign proper values!
    let para_id: u32 = 0;

    let ex = PreparedEx {
        function: Call::Slots(slots::Call::elaborate_deploy_data(para_id.into(), code)),
        sender: Some(Bob),
        minblock: 20,
    };

    exs.push(ex);
    exs
}
