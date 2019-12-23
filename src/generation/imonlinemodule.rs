// ------ Polkadot ImOnline MODULE ------

use substrate_sr_im_online::{Call as ImOnlineCall, Heartbeat};

use polkadot_runtime::Call;

use super::*;

pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();
    /*
    Extrinsic Description
    Module: ImOnline
    Function: heartbeat
    Type: Inherent Exstrinsic, ensure_none()

    Description: Testing final_hint
     */

    let block_number: u32 = 42;
    let session_index: u32 = 0;
    let authority_index: u32 = 1;
    let network_state = OpaqueNetworkState {
        peer_id: OpaquePeerId(vec![1]),
        external_addresses: vec![],
    };

    let heartbeat = Heartbeat {
        block_number,
        network_state,
        session_index,
        authority_index,
    };

    let signature = Bob.sign(&heartbeat.encode()).into();

    let ex = PreparedEx {
        function: Call::ImOnline(ImOnlineCall::heartbeat(heartbeat, signature)),
        sender: None,
        minblock: 3,
    };

    exs.push(ex);
    exs
}
