use std::convert::TryFrom;

use common::Bytes;
use common_eth::{EthLog, EthLogExt};
use derive_getters::{Dissolve, Getters};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::{
    user_ops::{UserOp, UserOpError},
    NetworkId,
};

// NOTE: See example here: https://bscscan.com/tx/0x1b245f033511dd60a5d50094c92c3d023ae81cb5d261a8824adff8429debf756

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Getters, Dissolve)]
pub struct UserSendLog {
    pub(crate) nonce: U256,
    pub(crate) origin_account: String,
    pub(crate) destination_account: String,
    pub(crate) destination_network_id: NetworkId,
    pub(crate) underlying_asset_name: String,
    pub(crate) underlying_asset_symbol: String,
    pub(crate) underlying_asset_decimals: U256,
    pub(crate) underlying_asset_token_address: EthAddress,
    pub(crate) underlying_asset_network_id: NetworkId,
    pub(crate) asset_token_address: EthAddress,
    pub(crate) asset_amount: U256,
    pub(crate) user_data_protocol_fee_asset_amount: U256,
    pub(crate) network_fee_asset_amount: U256,
    pub(crate) forward_network_fee_asset_amount: U256,
    pub(crate) forward_destination_network_id: NetworkId,
    pub(crate) origin_network_id: NetworkId,
    pub(crate) user_data: Bytes,
    pub(crate) options_mask: EthHash,
    pub(crate) is_for_protocol: bool,
}

impl TryFrom<&EthLog> for UserSendLog {
    type Error = UserOpError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        debug!("decoding `UserSendLog` from `EthLog`...");
        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
                EthAbiParamType::String,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::String,
                EthAbiParamType::String,
                EthAbiParamType::Uint(256),
                EthAbiParamType::Address,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::Address,
                EthAbiParamType::Uint(256),
                EthAbiParamType::Uint(256),
                EthAbiParamType::Uint(256),
                EthAbiParamType::Uint(256),
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::Bytes,
                EthAbiParamType::FixedBytes(32),
                EthAbiParamType::Bool,
            ],
            &l.get_data(),
        )?;

        UserOp::check_num_tokens(&tokens, 19, "UserSendLog")?;

        let nonce = UserOp::get_u256_from_token(&tokens[0])?;
        let origin_account = UserOp::get_string_from_token(&tokens[1])?;
        let destination_account = UserOp::get_string_from_token(&tokens[2])?;
        let destination_network_id = NetworkId::try_from(UserOp::get_fixed_bytes_from_token(&tokens[3])?)?;
        let underlying_asset_name = UserOp::get_string_from_token(&tokens[4])?;
        let underlying_asset_symbol = UserOp::get_string_from_token(&tokens[5])?;
        let underlying_asset_decimals = UserOp::get_u256_from_token(&tokens[6])?;
        let underlying_asset_token_address = UserOp::get_address_from_token(&tokens[7])?;
        let underlying_asset_network_id = NetworkId::try_from(UserOp::get_fixed_bytes_from_token(&tokens[8])?)?;
        let asset_token_address = UserOp::get_address_from_token(&tokens[9])?;
        let asset_amount = UserOp::get_u256_from_token(&tokens[10])?;
        let user_data_protocol_fee_asset_amount = UserOp::get_u256_from_token(&tokens[11])?;
        let network_fee_asset_amount = UserOp::get_u256_from_token(&tokens[12])?;
        let forward_network_fee_asset_amount = UserOp::get_u256_from_token(&tokens[13])?;
        let forward_destination_network_id = NetworkId::try_from(UserOp::get_fixed_bytes_from_token(&tokens[14])?)?;
        let origin_network_id = NetworkId::try_from(UserOp::get_fixed_bytes_from_token(&tokens[15])?)?;
        let user_data = UserOp::get_bytes_from_token(&tokens[16])?;
        let options_mask = UserOp::get_eth_hash_from_token(&tokens[17])?;
        let is_for_protocol = UserOp::get_bool_from_token(&tokens[18])?;

        Ok(Self {
            nonce,
            user_data,
            asset_amount,
            options_mask,
            origin_account,
            is_for_protocol,
            origin_network_id,
            asset_token_address,
            destination_account,
            underlying_asset_name,
            destination_network_id,
            underlying_asset_symbol,
            network_fee_asset_amount,
            underlying_asset_decimals,
            underlying_asset_network_id,
            underlying_asset_token_address,
            forward_destination_network_id,
            forward_network_fee_asset_amount,
            user_data_protocol_fee_asset_amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_ops::test_utils::get_sample_log_with_user_send;

    #[test]
    fn should_parse_user_send_log_correctly() {
        let l = get_sample_log_with_user_send();
        let r = UserSendLog::try_from(&l);
        assert!(r.is_ok());
    }
}
