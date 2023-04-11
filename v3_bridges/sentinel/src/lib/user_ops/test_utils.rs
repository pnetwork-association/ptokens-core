#![cfg(test)]
use common::BridgeSide;
use common_eth::convert_hex_to_eth_address;
use ethereum_types::Address as EthAddress;

use super::{UserOp, UserOps};
use crate::test_utils::get_sample_sub_mat_n;

#[allow(unused)]
pub fn get_sample_witnessed_user_op() -> UserOp {
    let side = BridgeSide::Native;
    let sub_mat = get_sample_sub_mat_n(10);
    let sepolia_network_id = hex::decode("e15503e4").unwrap();
    let state_manager = convert_hex_to_eth_address("b274d81a823c1912c6884e39c2e4e669e04c83f4").unwrap();
    let router = EthAddress::random();
    let _expected_result = 1;
    UserOps::from_sub_mat(side, &router, &sepolia_network_id, &state_manager, &sub_mat).unwrap()[0].clone()
}

pub fn get_sample_enqueued_user_op() -> UserOp {
    let side = BridgeSide::Native;
    let sub_mat = get_sample_sub_mat_n(11);
    let sepolia_network_id = hex::decode("e15503e4").unwrap();
    let state_manager = convert_hex_to_eth_address("0xBcBC92efE0a3C3ca99deBa708CEc92c785AfFB15").unwrap();
    let router = EthAddress::random();
    UserOps::from_sub_mat(side, &router, &sepolia_network_id, &state_manager, &sub_mat).unwrap()[0].clone()
}
