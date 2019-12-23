// ------ Polkadot Democracy MODULE ------
// Note: Basic tests only, TODO: add more exhaustive (governance-) testing, in connection with the Council module
use substrate_sr_democracy::{Call as DemocracyCall, Conviction, Vote};

use polkadot_runtime::Call;

use super::*;

pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();

    // Proposal, the call that should be made. `Set_balance` requires root-origin
    let call_proposal = Call::Balances(balances::Call::set_balance(
        Bob.public().into(),
        1000000,
        1000000,
    ));

    /*
    Extrinsic Description
    Module: Democracy
    Function: propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob submits a proposal
    */

    let amount: u128 = 50000;

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::propose(call_proposal.clone().into(), amount)),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: vote
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob votes
    */

    let bobvote = Vote {
        aye: true,
        conviction: Conviction::None,
    };

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::vote(0, bobvote)),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);

    /* Dave has no funds, fails . . .
    /*
    Extrinsic Description
    Module: Democracy
    Function: vote
    Type: Signed Extrinsic, ensure_signed()

    Description: Dave votes against Bob's proposal
    */

    let davevote = Vote {
        aye: false,
        conviction: Conviction::None,
    };

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::vote(0, davevote)),
        sender: Some(Dave),
        minblock: 3,
    };

    exs.push(ex);
    */

    /*
    Extrinsic Description
    Module: Democracy
    Function: vote
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice votes against Bob's proposal
    */

    let alicevote = Vote {
        aye: false,
        conviction: Conviction::None,
    };

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::vote(0, alicevote)),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: second
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice seconds Bob's proposal
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::second(0)),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: second
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice seconds a non-existing proposal
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::second(42)),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: emergency_canel
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice attempts to schedule an emergency cancellation of an none existing proposal
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::emergency_cancel(42)),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: external_propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob submits an external proposal
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::external_propose(
            call_proposal.clone().into(),
        )),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: external_propose_majority
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob submits an external majority proposal
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::external_propose_majority(
            call_proposal.clone().into(),
        )),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: external_propose_default
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob submits an external default proposal
    */
    
    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::external_propose_default(
            call_proposal.clone().into(),
        )),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: fast_track
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice attempts to fast-track a proposal, should fail
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::fast_track(H256::default(), 5, 5)),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: veto_external
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice attempts veto_external, should fail
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::veto_external(H256::default())),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: cancel_referendum
    Type: Signed Extrinsic, ensure_signed()

    Description: bob attempts to cancel his referendum
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::cancel_referendum(0)),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: cancel_queued
    Type: Signed Extrinsic, ensure_signed()

    Description: bob attempts to cancel his referendum
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::cancel_queued(15, 0, 0)),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    // ------- Tests for Proxy Functionality  -------

    /*
    Extrinsic Description
    Module: Democracy
    Function: set_proxy
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob sets Alice as proxy
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::set_proxy(Alice.into())),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: resign_proxy
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice resign's as Bob's proxy
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::resign_proxy()),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: set_proxy
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice sets Bob as proxy
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::set_proxy(Bob.into())),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob submits a proposal
    */

    let amount: u128 = 50000;

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::propose(call_proposal.clone().into(), amount)),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: vote
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob votes
    */

    let bobvote = Vote {
        aye: true,
        conviction: Conviction::None,
    };

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::vote(0, bobvote)),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: vote
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob acting as proxy for Alice, votes
    */

    let bobvote = Vote {
        aye: true,
        conviction: Conviction::None,
    };

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::proxy_vote(0, bobvote)),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: remove_proxy
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice removes Bob as proxy
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::remove_proxy(Bob.into())),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    // ------- Tests for Delegate Functionality  -------

    /*
    Extrinsic Description
    Module: Democracy
    Function: delegate
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice delegates her vote to Bob
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::delegate(Bob.into(), Conviction::None)),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: propose
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob submits a proposal
    */

    let amount: u128 = 50000;

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::propose(call_proposal.clone().into(), amount)),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: vote
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob votes + Alices delegated votes
    */

    let bobvote = Vote {
        aye: true,
        conviction: Conviction::None,
    };

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::vote(0, bobvote)),
        sender: Some(Bob),
        minblock: 1,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Democracy
    Function: undelegate
    Type: Signed Extrinsic, ensure_signed()

    Description: Alice undelegates her vote
    */

    let ex = PreparedEx {
        function: Call::Democracy(DemocracyCall::undelegate()),
        sender: Some(Alice),
        minblock: 1,
    };

    exs.push(ex);

    exs
}
