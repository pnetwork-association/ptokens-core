#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256, EthSubmissionMaterial};

use crate::{NetworkId, ProtocolId, UserOp, UserOps};

fn get_arbitrum_protocol_queue_operation_sub_mat() -> EthSubmissionMaterial {
    let path = "src/lib/eth_rpc_calls/test_utils/host-sub-mat-num-105419318.json";
    EthSubmissionMaterial::from_str(&read_to_string(path).unwrap()).unwrap()
}

pub fn get_arbitrum_protocol_queue_user_op() -> UserOp {
    let side = BridgeSide::Host;
    let protocol_id = ProtocolId::Ethereum;
    let origin_chain_id = EthChainId::XDaiMainnet;
    let origin_network_id = NetworkId::new(origin_chain_id.to_u64(), protocol_id)
        .to_bytes_4()
        .unwrap()
        .to_vec();
    let pnetwork_hub = convert_hex_to_eth_address("0xf84552a4B276B47718b8E25E8151eF749D64C4A6").unwrap();
    let ops = UserOps::from_sub_mat(
        side,
        &origin_network_id,
        &pnetwork_hub,
        &get_arbitrum_protocol_queue_operation_sub_mat(),
    )
    .unwrap();
    assert!(ops.len() == 1);
    let op = ops[0].clone();
    let uid = op.uid().unwrap();
    let expected_uid =
        convert_hex_to_h256("0x1b4d34d28d49cf03dccda141fc65118f11fb08afd28e2d2ac8520558185a71f3").unwrap();
    assert_eq!(uid, expected_uid);
    op
}
