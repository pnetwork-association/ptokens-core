#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common::BridgeSide;
use common_eth::{convert_hex_to_eth_address, EthSubmissionMaterial};

use super::{UserOp, UserOps};

pub fn get_sample_sub_mat_n(n: usize) -> EthSubmissionMaterial {
    let suffix = match n {
        _ => "bsc-block-31915441-with-user-send.json",
    };
    let prefix = "src/lib/user_ops/test_utils/";
    let path = format!("{prefix}{suffix}");
    EthSubmissionMaterial::from_str(&read_to_string(path).unwrap()).unwrap()
}

fn get_sample_submission_material_with_user_send() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(1)
}

pub fn get_sample_witnessed_user_op() -> UserOp {
    let side = BridgeSide::Native;
    let sub_mat = get_sample_sub_mat_n(10);
    let sepolia_network_id = hex::decode("e15503e4").unwrap();
    let pnetwork_hub = convert_hex_to_eth_address("b274d81a823c1912c6884e39c2e4e669e04c83f4").unwrap();
    let _expected_result = 1;
    let op = UserOps::from_sub_mat(side, &sepolia_network_id, &pnetwork_hub, &sub_mat).unwrap()[0].clone();
    assert!(op.state().is_witnessed());
    op
}

pub fn get_sample_enqueued_user_op() -> UserOp {
    let side = BridgeSide::Native;
    let sub_mat = get_sample_sub_mat_n(11);
    let sepolia_network_id = hex::decode("e15503e4").unwrap();
    let pnetwork_hub = convert_hex_to_eth_address("0xBcBC92efE0a3C3ca99deBa708CEc92c785AfFB15").unwrap();
    let op = UserOps::from_sub_mat(side, &sepolia_network_id, &pnetwork_hub, &sub_mat).unwrap()[0].clone();
    assert!(op.state().is_enqueued());
    op
}

pub fn get_sample_cancelled_user_op() -> UserOp {
    let side = BridgeSide::Native;
    let sub_mat = get_sample_sub_mat_n(14);
    let sepolia_network_id = hex::decode("e15503e4").unwrap();
    let pnetwork_hub = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
    let op = UserOps::from_sub_mat(side, &sepolia_network_id, &pnetwork_hub, &sub_mat).unwrap()[0].clone();
    assert!(op.state().is_cancelled());
    op
}

pub fn get_sample_executed_user_op() -> UserOp {
    let side = BridgeSide::Native;
    let sub_mat = get_sample_sub_mat_n(15);
    let sepolia_network_id = hex::decode("e15503e4").unwrap();
    let pnetwork_hub = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
    let op = UserOps::from_sub_mat(side, &sepolia_network_id, &pnetwork_hub, &sub_mat).unwrap()[0].clone();
    assert!(op.state().is_executed());
    op
}
