// ------ SRML BALANCES MODULE ------

use substrate_sr_sudo::Call as SudoCall;

use polkadot_runtime::{BalancesCall, Call};

use super::*;

pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();

    /*
    Extrinsic Description
    Module: Balances
    Function: set_balances
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Sets the initial Balance for Bob
    */

    let amount: u128 = 200000000000;

    let ex = PreparedEx {
        function: Call::Sudo(SudoCall::sudo(
            Call::from(BalancesCall::set_balance(
                Bob.public().into(),
                amount,
                amount,
            ))
            .into(),
        )),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: transfer
    Type: Signed Extrinsic, ensure_signed()

    Description: A normal transfer, Bob transfers parts of his fund to Dave
    */

    let amount: u128 = 100000000000;

    let ex = PreparedEx {
        function: Call::Balances(BalancesCall::transfer(Dave.public().into(), amount)),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: transfer
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob attempts to transfer more than his current Balance to Dave, must fail
    */

    let amount: u128 = 300000000000;

    let ex = PreparedEx {
        function: Call::Balances(BalancesCall::transfer(Dave.public().into(), amount)),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: transfer
    Type: Signed Extrinsic, ensure_signed()

    Description: Dave transfers some of his funds to Alice
    */

    let amount: u128 = 50000000000;

    let ex = PreparedEx {
        function: Call::Balances(BalancesCall::transfer(Alice.public().into(), amount)),
        sender: Some(Dave),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: force_transfer
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Force a possible transfer from Dave to Bob
    */

    let amount: u128 = 20000000000;

    let ex = PreparedEx {
        function: Call::Sudo(SudoCall::sudo(
            Call::from(BalancesCall::force_transfer(
                Dave.public().into(),
                Bob.public().into(),
                amount,
            ))
            .into(),
        )),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: force_transfer
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Attempts to force an impossible transfer (insufficient funds) from Dave to Bob, must fail
    */

    let amount: u128 = 9990000000000;

    let ex = PreparedEx {
        function: Call::Sudo(SudoCall::sudo(
            Call::from(BalancesCall::force_transfer(
                Dave.public().into(),
                Bob.public().into(),
                amount,
            ))
            .into(),
        )),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: force_transfer
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Attempts to force an impossible transfer from Eve (an account with no balance / no contact with the balances module before)
    */

    let amount: u128 = 20000000000;

    let ex = PreparedEx {
        function: Call::Sudo(SudoCall::sudo(
            Call::from(BalancesCall::force_transfer(
                Eve.public().into(),
                Bob.public().into(),
                amount,
            ))
            .into(),
        )),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: force_transfer
    Type: Signed Extrinsic

    Description: Dave attempts a call to foce_transfer, must fail as force_transfer requires root_origin
    */

    let amount: u128 = 50000000000;

    let ex = PreparedEx {
        function: Call::Balances(BalancesCall::force_transfer(
            Alice.public().into(),
            Bob.public().into(),
            amount,
        )),
        sender: Some(Dave),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: set_balance
    Type: Signed Extrinsic, ensure_signed()

    Description: Bob attempts a call to set_balance, must fail as set_balance requires root_origin
    */

    let amount: u128 = 50000000000;

    let ex = PreparedEx {
        function: Call::Balances(BalancesCall::set_balance(
            Alice.public().into(),
            amount,
            amount,
        )),
        sender: Some(Bob),
        minblock: 3,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Balances
    Function: set_balances
    Type: Requires root origin, ensure_root(). Signed Extrinsic to Sudo module

    Description: Sets Balance for Bobs account, needed for further tests
    */

    let amount: u128 = 20000000000000;

    let ex = PreparedEx {
        function: Call::Sudo(SudoCall::sudo(
            Call::from(BalancesCall::set_balance(
                Bob.public().into(),
                amount,
                amount,
            ))
            .into(),
        )),
        sender: Some(Alice),
        minblock: 3,
    };

    exs.push(ex);

    exs
}
