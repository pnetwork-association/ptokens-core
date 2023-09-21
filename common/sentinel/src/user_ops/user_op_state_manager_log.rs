use std::convert::TryFrom;

use common::Bytes;
use common_eth::{EthLog, EthLogExt};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use super::UserOp;
use crate::SentinelError;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserOpStateManagerLog {
    pub(super) origin_block_hash: EthHash,
    pub(super) origin_transaction_hash: EthHash,
    pub(super) options_mask: EthHash,
    pub(super) nonce: U256,
    pub(super) underlying_asset_decimals: U256,
    pub(super) amount: U256,
    pub(super) underlying_asset_token_address: EthAddress,
    pub(super) origin_network_id: Bytes,           // TODO use type for this!
    pub(super) destination_network_id: Bytes,      // TODO use type for this!
    pub(super) underlying_asset_network_id: Bytes, // TODO use type for this!
    pub(super) destination_account: String,
    pub(super) underlying_asset_name: String,
    pub(super) underlying_asset_symbol: String,
    pub(super) user_data: Bytes,
}

impl TryFrom<&EthLog> for UserOpStateManagerLog {
    type Error = SentinelError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        debug!("Decoding `StateManagerUserOp` from `EthLog`...");

        let tuple_of_tokens = eth_abi_decode(
            &[
                // NOTE Because the log contains a struct, which gets encoded as a tuple
                EthAbiParamType::Tuple(vec![
                    EthAbiParamType::FixedBytes(32),
                    EthAbiParamType::FixedBytes(32),
                    EthAbiParamType::FixedBytes(32),
                    EthAbiParamType::Uint(256),
                    EthAbiParamType::Uint(256),
                    EthAbiParamType::Uint(256),
                    EthAbiParamType::Address,
                    EthAbiParamType::FixedBytes(4),
                    EthAbiParamType::FixedBytes(4),
                    EthAbiParamType::FixedBytes(4),
                    EthAbiParamType::String,
                    EthAbiParamType::String,
                    EthAbiParamType::String,
                    EthAbiParamType::Bytes,
                ]),
            ],
            &l.get_data(),
        )?;

        let tokens = UserOp::get_tuple_from_token(&tuple_of_tokens[0])?;

        let origin_block_hash = UserOp::get_eth_hash_from_token(&tokens[0])?;
        let origin_transaction_hash = UserOp::get_eth_hash_from_token(&tokens[1])?;
        let options_mask = UserOp::get_eth_hash_from_token(&tokens[2])?;
        let nonce = UserOp::get_u256_from_token(&tokens[3])?;
        let underlying_asset_decimals = UserOp::get_u256_from_token(&tokens[4])?;
        let amount = UserOp::get_u256_from_token(&tokens[5])?;
        let underlying_asset_token_address = UserOp::get_address_from_token(&tokens[6])?;
        let origin_network_id = UserOp::get_fixed_bytes_from_token(&tokens[7])?;
        let destination_network_id = UserOp::get_fixed_bytes_from_token(&tokens[8])?;
        let underlying_asset_network_id = UserOp::get_fixed_bytes_from_token(&tokens[9])?;
        let destination_account = UserOp::get_string_from_token(&tokens[10])?;
        let underlying_asset_name = UserOp::get_string_from_token(&tokens[11])?;
        let underlying_asset_symbol = UserOp::get_string_from_token(&tokens[12])?;
        let user_data = UserOp::get_bytes_from_token(&tokens[13])?;

        Ok(Self {
            origin_block_hash,
            origin_transaction_hash,
            options_mask,
            nonce,
            underlying_asset_decimals,
            amount,
            underlying_asset_token_address,
            origin_network_id,
            destination_network_id,
            underlying_asset_network_id,
            destination_account,
            underlying_asset_name,
            underlying_asset_symbol,
            user_data,
        })
    }
}

/*
#[cfg(test)]
mod tests {
    use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256};

    use super::*;
    use crate::{
        test_utils::get_sample_sub_mat_n,
        user_ops::{CANCELLED_USER_OP_TOPIC, ENQUEUED_USER_OP_TOPIC, EXECUTED_USER_OP_TOPIC},
    };

    fn get_sample_enqueued_log() -> EthLog {
        get_sample_sub_mat_n(11).receipts[1].logs[0].clone()
    }

    fn get_expected_user_op_log_from_state_manager() -> UserOpStateManagerLog {
        UserOpStateManagerLog {
            origin_block_hash: convert_hex_to_h256(
                "0x81803894d2305fd729ac0b90a4262a85c4d11b70b8bea98c40ee68bf56c8a1c2",
            )
            .unwrap(),
            origin_transaction_hash: convert_hex_to_h256(
                "0xeb5cbe8387d5e9e247ea886459bcd0e599732e1a4e02a38b235cd93cac96bf30",
            )
            .unwrap(),
            options_mask: EthHash::zero(),
            nonce: U256::from(42),
            underlying_asset_decimals: U256::from(4),
            amount: U256::from(1337),
            underlying_asset_token_address: convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD")
                .unwrap(),
            origin_network_id: hex::decode("01020304").unwrap(),
            destination_network_id: hex::decode("04030201").unwrap(),
            underlying_asset_network_id: hex::decode("01030307").unwrap(),
            destination_account: "0xDAFEA492D9c6733ae3d56b7Ed1ADB60692c98Bc5".to_string(),
            underlying_asset_name: "some token".to_string(),
            underlying_asset_symbol: "STK".to_string(),
            user_data: hex::decode("c0ffee").unwrap(),
        }
    }

    #[test]
    fn should_parse_user_op_log_from_state_manager_enqueued_event_correctly() {
        let log = get_sample_enqueued_log();
        assert_eq!(log.topics[0], *ENQUEUED_USER_OP_TOPIC);
        let result = UserOpStateManagerLog::try_from(&log).unwrap();
        assert_eq!(result, get_expected_user_op_log_from_state_manager());
    }

    #[test]
    fn should_parse_user_op_log_from_state_manager_executed_event_correctly() {
        let log = get_sample_sub_mat_n(12).receipts[8].logs[0].clone();
        assert_eq!(log.topics[0], *EXECUTED_USER_OP_TOPIC);
        let result = UserOpStateManagerLog::try_from(&log).unwrap();
        assert_eq!(result, get_expected_user_op_log_from_state_manager());
    }

    #[test]
    fn should_parse_user_op_log_from_state_manager_cancelled_event_correctly() {
        let log = get_sample_sub_mat_n(13).receipts[14].logs[0].clone();
        assert_eq!(log.topics[0], *CANCELLED_USER_OP_TOPIC);
        let result = UserOpStateManagerLog::try_from(&log).unwrap();
        assert_eq!(result, get_expected_user_op_log_from_state_manager());
    }
}
*/
