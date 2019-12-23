// ------ Polkadot FinalityTracker MODULE ------

use substrate_sr_finality_tracker::Call as FinalityCall;

use polkadot_runtime::Call;

use super::*;

pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();
    /*
    Extrinsic Description
    Module: FinalityTracker
    Function: final_hint
    Type: Inherent Exstrinsic, ensure_none()

    Description: Testing final_hint, argument < block extrinsic is executed, should ensure no panic
     */

    let bnum: u32 = 5;

    let ex = PreparedEx {
        function: Call::FinalityTracker(FinalityCall::final_hint(bnum)),
        sender: None,
        minblock: 6,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: FinalityTracker
    Function: final_hint
    Type: Inherent Exstrinsic, ensure_none()

    Description: Testing final_hint, argument > block extrinsic is executed, will panic (thus disabled)
     */

    /*let bnum: u32 = 5;

    let ex = PreparedEx {
        function: Call::FinalityTracker(FinalityCall::final_hint(bnum)),
        sender: None,
        minblock: 6,
    };

    exs.push(ex);*/
    exs
}
