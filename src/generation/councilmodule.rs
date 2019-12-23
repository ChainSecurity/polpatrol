// ------ SRML Collective MODULE (Polkadot Council) ------
use polkadot_runtime::Call;
use substrate_sr_collective::Call as CollectiveCall;

use substrate_sr_primitives::traits::{BlakeTwo256, Hash};

use super::*;

pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();

    // Proposal, the call proposed to be executed. `Set_balance` requires root-origin
    let call_proposal = Call::Balances(balances::Call::set_balance(
        Charlie.public().into(),
        1000000,
        1000000,
    ));

    /*
    Extrinsic Description
    Module: Collective
    Function: set_members
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Adds Alice and Bob as members of the Council
    */

    let mut newmembers = Vec::new();
    newmembers.push(Alice.public());
    newmembers.push(Bob.public());

    let ex = PreparedEx {
        function: Call::Sudo(SudoCall::sudo(
            Call::Council(CollectiveCall::set_members(newmembers)).into(),
        )),
        sender: Some(Alice),
        minblock: 0,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Collective
    Function: execute
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice, as member of the council, can directly execute a proposal
    */

    let ex = PreparedEx {
        function: Call::Council(CollectiveCall::execute(call_proposal.clone().into())).into(),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Collective
    Function: propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob submits a proposal, doesn't work as he is not a member
    */

    let num: u32 = 2;

    let ex = PreparedEx {
        function: Call::Council(CollectiveCall::propose(
            num.into(),
            call_proposal.clone().into(),
        ))
        .into(),
        sender: Some(Charlie),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Collective
    Function: propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice submits a proposal
    */

    let num: u32 = 2;

    let ex = PreparedEx {
        function: Call::Council(CollectiveCall::propose(
            num.into(),
            call_proposal.clone().into(),
        ))
        .into(),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Collective
    Function: propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice votes for Bobs proposal, 1/2 required votes
    */

    let proposal_hash = BlakeTwo256::hash_of(&call_proposal.clone());

    let ex = PreparedEx {
        function: Call::Council(CollectiveCall::vote(proposal_hash, 0, true)).into(),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Collective
    Function: propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob votes for Bobs proposal, 2/2 required votes, should be executed
    */

    let proposal_hash = BlakeTwo256::hash_of(&call_proposal.clone());

    let ex = PreparedEx {
        function: Call::Council(CollectiveCall::vote(proposal_hash, 0, true)).into(),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);

    exs
}
