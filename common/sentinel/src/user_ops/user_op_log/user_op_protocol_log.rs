use std::convert::TryFrom;

use common::Bytes;
use common_eth::{EthLog, EthLogExt};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::{
    user_ops::{UserOp, UserOpError},
    NetworkId,
};
// NOTE: A protocol cancellation log also includes information pertaining to the actor who
// performed the cancellation. The actor's address & type are indexed, and so appear in the topic
// list, meaning the rest of the log is parsable by the same logic below as other protocol logs.
// The sentinel has no need for this extra information and so we ignore it here.

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserOpProtocolLog {
    pub(super) origin_block_hash: EthHash,
    pub(super) origin_transaction_hash: EthHash,
    pub(super) options_mask: EthHash,
    pub(super) nonce: U256,
    pub(super) underlying_asset_decimals: U256,
    pub(super) asset_amount: U256,
    pub(super) protocol_fee_asset_amount: U256,
    pub(super) network_fee_asset_amount: U256,
    pub(super) forward_network_fee_asset_amount: U256,
    pub(super) underlying_asset_token_address: EthAddress,
    pub(super) origin_network_id: NetworkId,
    pub(super) destination_network_id: NetworkId,
    pub(super) forward_destination_network_id: NetworkId,
    pub(super) underlying_asset_network_id: NetworkId,
    pub(super) origin_account: String,
    pub(super) destination_account: String,
    pub(super) underlying_asset_name: String,
    pub(super) underlying_asset_symbol: String,
    pub(super) user_data: Bytes,
    pub(super) is_for_protocol: bool,
}

impl TryFrom<&EthLog> for UserOpProtocolLog {
    type Error = UserOpError;

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
                    EthAbiParamType::Uint(256),
                    EthAbiParamType::Uint(256),
                    EthAbiParamType::Uint(256),
                    EthAbiParamType::Address,
                    EthAbiParamType::FixedBytes(4),
                    EthAbiParamType::FixedBytes(4),
                    EthAbiParamType::FixedBytes(4),
                    EthAbiParamType::FixedBytes(4),
                    EthAbiParamType::String,
                    EthAbiParamType::String,
                    EthAbiParamType::String,
                    EthAbiParamType::String,
                    EthAbiParamType::Bytes,
                    EthAbiParamType::Bool,
                ]),
            ],
            &l.get_data(),
        )?;

        let tokens = UserOp::get_tuple_from_token(&tuple_of_tokens[0])?;
        UserOp::check_num_tokens(&tokens, 20, "UserOpProtocolLog")?;

        let origin_block_hash = UserOp::get_eth_hash_from_token(&tokens[0])?;
        let origin_transaction_hash = UserOp::get_eth_hash_from_token(&tokens[1])?;
        let options_mask = UserOp::get_eth_hash_from_token(&tokens[2])?;
        let nonce = UserOp::get_u256_from_token(&tokens[3])?;
        let underlying_asset_decimals = UserOp::get_u256_from_token(&tokens[4])?;
        let asset_amount = UserOp::get_u256_from_token(&tokens[5])?;
        let protocol_fee_asset_amount = UserOp::get_u256_from_token(&tokens[6])?;
        let network_fee_asset_amount = UserOp::get_u256_from_token(&tokens[7])?;
        let forward_network_fee_asset_amount = UserOp::get_u256_from_token(&tokens[8])?;
        let underlying_asset_token_address = UserOp::get_address_from_token(&tokens[9])?;
        let origin_network_id = NetworkId::try_from(&UserOp::get_fixed_bytes_from_token(&tokens[10])?)?;
        let destination_network_id = NetworkId::try_from(&UserOp::get_fixed_bytes_from_token(&tokens[11])?)?;
        let forward_destination_network_id = NetworkId::try_from(&UserOp::get_fixed_bytes_from_token(&tokens[12])?)?;
        let underlying_asset_network_id = NetworkId::try_from(&UserOp::get_fixed_bytes_from_token(&tokens[13])?)?;
        let origin_account = UserOp::get_string_from_token(&tokens[14])?;
        let destination_account = UserOp::get_string_from_token(&tokens[15])?;
        let underlying_asset_name = UserOp::get_string_from_token(&tokens[16])?;
        let underlying_asset_symbol = UserOp::get_string_from_token(&tokens[17])?;
        let user_data = UserOp::get_bytes_from_token(&tokens[18])?;
        let is_for_protocol = UserOp::get_bool_from_token(&tokens[19])?;

        Ok(Self {
            nonce,
            user_data,
            options_mask,
            asset_amount,
            origin_account,
            is_for_protocol,
            origin_block_hash,
            origin_network_id,
            destination_account,
            underlying_asset_name,
            destination_network_id,
            underlying_asset_symbol,
            origin_transaction_hash,
            network_fee_asset_amount,
            protocol_fee_asset_amount,
            underlying_asset_decimals,
            underlying_asset_network_id,
            underlying_asset_token_address,
            forward_destination_network_id,
            forward_network_fee_asset_amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_ops::test_utils::{get_sample_log_with_protocol_queue, get_sub_mat_with_protocol_cancellation_log};

    #[test]
    fn should_parse_protocol_log_correctly() {
        let l = get_sample_log_with_protocol_queue();
        let r = UserOpProtocolLog::try_from(&l);
        assert!(r.is_ok());
    }

    #[test]
    fn should_parse_protocol_log_from_cancellation_log_correctly() {
        let l = get_sub_mat_with_protocol_cancellation_log();
        let r = UserOpProtocolLog::try_from(&l);
        assert!(r.is_ok());
    }
}
