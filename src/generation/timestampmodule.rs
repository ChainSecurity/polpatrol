// ------ SRML TIMESTAMP MODULE ------
use polkadot_runtime::{Call, TimestampCall};

use super::*;

//Function is currently not used. Timestamp already added during block generation, adding a second timestamp is not allowed and would lead to panic
#[allow(dead_code)]
pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();

    /*
    Extrinsic Description
    Module: Timestamp
    Function: set
    Type: Inherent Exstrinsic, ensure_none()

    Description: Sets timestamp. Can only be called once in a block successfully (and this is already done during block generation). Called a 2nd time this will panic.
    */

    let time: u64 = 2000000;

    let ex = PreparedEx {
        function: Call::Timestamp(TimestampCall::set(time)),
        sender: None,
        minblock: 0,
    };

    exs.push(ex);

    exs
}
