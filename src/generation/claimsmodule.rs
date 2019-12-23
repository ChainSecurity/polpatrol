// ------ Polkadot Claim MODULE ------

use polkadot_runtime::{claims, claims::EcdsaSignature, Call};

use hex_literal::hex;

use super::*;

pub fn craft_extrinsics() -> Vec<PreparedEx> {
    let mut exs = Vec::new();

    /*
    Extrinsic Description
    Module: Claims
    Function: claim
    Type: Inherent Exstrinsic, ensure_none()

    Description: Successful claim, see https://polkascan.io/pre/kusama-cc2/extrinsic/524876-3
    */

    /*let ex = PreparedEx {
        function: Call::Claims(claims::Call::claim("FniPsTzC44LYdbdydDQTCGHme9MTiqJ9RdUFGCgSwmY5dZb".into(), "dac8ae8adebf72fe659f2e031945dcfb1f11ac53b555ce6b9a25723a792923904efacb214d03ae60f15ff5181fac6a3d0c5330a8c10e1db5e04f2d0079acdffb1b".into())),
        sender: None,
    };

    exs.push(ex);
    */

    /*
    Extrinsic Description
    Module: Claims
    Function: claim
    Type: Inherent Exstrinsic, ensure_none()

    Description: Unsuccessful, random claim
    */
    let sig = hex!["444023e89b67e67c0562ed0305d252a5dd12b2af5ac51d6d3cb69a0b486bc4b3191401802dc29d26d586221f7256cd3329fe82174bdf659baea149a40e1c495d1c"];
    let sig = EcdsaSignature(sig);

    let ex = PreparedEx {
        //function: Call::Claims(claims::Call::claim("FniPsTzC44LYdbdydDQTCGHme9MTiqJ9RdUFGCgSwmY5dZb".into(), "dac8ae8adebf72fe659f2e031945dcfb1f11ac53b555ce6b9a25723a792923904efacb214d03ae60f15ff5181fac6a3d0c5330a8c10e1db5e04f2d0079acdffb1b".into())),
        function: Call::Claims(claims::Call::claim(Alice.public().into(), sig)),
        sender: None,
        minblock: 0,
    };

    exs.push(ex);

    /*
    Extrinsic Description
    Module: Claims
    Function: claim
    Type: Inherent Exstrinsic, ensure_none()

    Description: Unsuccessful claim, see https://polkascan.io/pre/kusama-cc2/extrinsic/512701-3
    */

    /*let ex = PreparedEx{
        function: Call::Claims(claims::Call::claim("GyaghoLgXu4FbsqSkKM2YdXXPXsXAdTa1K3gVwPGHjuihXR".into(), "126a2ffc38612d9282dfc68ef22952e9ba2facc5ca0be60e1ecde6087ecf327a3a8a8024e76f8095181a73416bcbf7891795d15b6622d7ab9621eac3b5097e1200".into())),
        sender: None,
        minblock: 3,
    };

    exs.push(ex);*/
    exs
}
