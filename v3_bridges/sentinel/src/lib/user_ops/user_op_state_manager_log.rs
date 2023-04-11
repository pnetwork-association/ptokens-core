use std::convert::TryFrom;

use common::Bytes;
use common_eth::{EthLog, EthLogExt};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use ethers_core::abi::{self, Token};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::SentinelError;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserOpRouterLogFromStateManager {
    origin_block_hash: EthHash,
    origin_transaction_hash: EthHash,
    options_mask: EthHash,
    nonce: U256,
    underlying_asset_decimals: U256,
    amount: U256,
    underlying_asset_token_address: EthAddress,
    origin_network_id: Bytes,           // TODO use type for this!
    destination_network_id: Bytes,      // TODO use type for this!
    underlying_asset_network_id: Bytes, // TODO use type for this!
    destination_account: String,
    underlying_asset_name: String,
    underlying_asset_symbol: String,
    user_data: Bytes,
}

impl TryFrom<&EthLog> for UserOpRouterLogFromStateManager {
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

        let tokens = Self::get_tuple_from_token(&tuple_of_tokens[0])?;

        let origin_block_hash = Self::get_eth_hash_from_token(&tokens[0])?;
        let origin_transaction_hash = Self::get_eth_hash_from_token(&tokens[1])?;
        let options_mask = Self::get_eth_hash_from_token(&tokens[2])?;
        let nonce = Self::get_u256_from_token(&tokens[3])?;
        let underlying_asset_decimals = Self::get_u256_from_token(&tokens[4])?;
        let amount = Self::get_u256_from_token(&tokens[5])?;
        let underlying_asset_token_address = Self::get_address_from_token(&tokens[6])?;
        let origin_network_id = Self::get_fixed_bytes_from_token(&tokens[7])?;
        let destination_network_id = Self::get_fixed_bytes_from_token(&tokens[8])?;
        let underlying_asset_network_id = Self::get_fixed_bytes_from_token(&tokens[9])?;
        let destination_account = Self::get_string_from_token(&tokens[10])?;
        let underlying_asset_name = Self::get_string_from_token(&tokens[11])?;
        let underlying_asset_symbol = Self::get_string_from_token(&tokens[12])?;
        let user_data = Self::get_bytes_from_token(&tokens[13])?;

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

// FIXME rm repetition in this!
impl UserOpRouterLogFromStateManager {
    pub fn to_uid(&self) -> Result<EthHash, SentinelError> {
        let mut hasher = Keccak::v256();
        let input = self.abi_encode_packed()?;
        let mut output = [0u8; 32];
        hasher.update(&input);
        hasher.finalize(&mut output);
        Ok(EthHash::from_slice(&output))
    }

    // TODO Question for Alessandro: should this be encoded _packed_?
    fn abi_encode_packed(&self) -> Result<Bytes, SentinelError> {
        Ok(abi::encode(&[
            Token::FixedBytes(self.origin_block_hash.as_bytes().to_vec()),
            Token::FixedBytes(self.origin_transaction_hash.as_bytes().to_vec()),
            Token::FixedBytes(self.origin_network_id.clone()),
            Token::Uint(Self::convert_u256_type(self.nonce)),
            Token::String(self.destination_account.clone()),
            Token::FixedBytes(self.destination_network_id.clone()),
            Token::String(self.underlying_asset_name.clone()),
            Token::String(self.underlying_asset_symbol.clone()),
            Token::Uint(Self::convert_u256_type(self.underlying_asset_decimals)),
            Token::Address(Self::convert_address_type(self.underlying_asset_token_address)),
            Token::FixedBytes(self.underlying_asset_network_id.clone()),
            Token::Uint(Self::convert_u256_type(self.amount)),
            Token::Bytes(self.user_data.clone()),
            Token::FixedBytes(self.options_mask.as_bytes().to_vec()),
        ]))
    }

    fn convert_u256_type(t: U256) -> ethers_core::types::U256 {
        // NOTE: Sigh. The ethabi crate re-exports the ethereum_types which we use elsewhere, so
        // that's annoying.
        let mut r = [0u8; 32];
        t.to_big_endian(&mut r);
        ethers_core::types::U256::from_big_endian(&r)
    }

    fn convert_address_type(t: EthAddress) -> ethers_core::types::Address {
        // NOTE: Sigh. The ethabi crate re-exports the ethereum_types which we use elsewhere, so
        // that's annoying.
        let s = t.as_bytes();
        ethers_core::types::Address::from_slice(s)
    }

    fn get_tuple_from_token(t: &EthAbiToken) -> Result<Vec<EthAbiToken>, SentinelError> {
        match t {
            EthAbiToken::Tuple(v) => Ok(v.to_vec()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to tuple token"))),
        }
    }

    fn get_address_from_token(t: &EthAbiToken) -> Result<EthAddress, SentinelError> {
        match t {
            EthAbiToken::Address(t) => Ok(EthAddress::from_slice(t.as_bytes())),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to ETH address!"))),
        }
    }

    fn get_string_from_token(t: &EthAbiToken) -> Result<String, SentinelError> {
        match t {
            EthAbiToken::String(ref t) => Ok(t.clone()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to string!"))),
        }
    }

    fn get_bytes_from_token(t: &EthAbiToken) -> Result<Bytes, SentinelError> {
        match t {
            EthAbiToken::Bytes(b) => Ok(b.clone()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to bytes:"))),
        }
    }

    fn get_fixed_bytes_from_token(t: &EthAbiToken) -> Result<Bytes, SentinelError> {
        match t {
            EthAbiToken::FixedBytes(b) => Ok(b.to_vec()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to bytes:"))),
        }
    }

    #[allow(unused)]
    fn get_eth_hash_from_token(t: &EthAbiToken) -> Result<EthHash, SentinelError> {
        match t {
            EthAbiToken::FixedBytes(ref b) => Ok(EthHash::from_slice(b)),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to EthHash!"))),
        }
    }

    fn get_u256_from_token(t: &EthAbiToken) -> Result<U256, SentinelError> {
        match t {
            EthAbiToken::Uint(u) => {
                let mut b = [0u8; 32];
                u.to_big_endian(&mut b);
                Ok(U256::from_big_endian(&b))
            },
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to U256!"))),
        }
    }
}

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

    fn get_expected_user_op_log_from_state_manager() -> UserOpRouterLogFromStateManager {
        UserOpRouterLogFromStateManager {
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
        let result = UserOpRouterLogFromStateManager::try_from(&log).unwrap();
        assert_eq!(result, get_expected_user_op_log_from_state_manager());
    }

    #[test]
    fn should_parse_user_op_log_from_state_manager_executed_event_correctly() {
        let log = get_sample_sub_mat_n(12).receipts[8].logs[0].clone();
        assert_eq!(log.topics[0], *EXECUTED_USER_OP_TOPIC);
        let result = UserOpRouterLogFromStateManager::try_from(&log).unwrap();
        assert_eq!(result, get_expected_user_op_log_from_state_manager());
    }

    #[test]
    fn should_parse_user_op_log_from_state_manager_cancelled_event_correctly() {
        let log = get_sample_sub_mat_n(13).receipts[14].logs[0].clone();
        assert_eq!(log.topics[0], *CANCELLED_USER_OP_TOPIC);
        let result = UserOpRouterLogFromStateManager::try_from(&log).unwrap();
        assert_eq!(result, get_expected_user_op_log_from_state_manager());
    }

    #[test]
    fn should_get_abi_encoded_data_correctly_from_state_manager_log() {
        let user_op_log = UserOpRouterLogFromStateManager::try_from(&get_sample_enqueued_log()).unwrap();
        let expected_result = hex::decode("81803894d2305fd729ac0b90a4262a85c4d11b70b8bea98c40ee68bf56c8a1c2eb5cbe8387d5e9e247ea886459bcd0e599732e1a4e02a38b235cd93cac96bf300102030400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000001c0040302010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000000400000000000000000000000089ab32156e46f46d02ade3fecbe5fc4243b9aaed0103030700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000053900000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30784441464541343932443963363733336165336435366237456431414442363036393263393842633500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a736f6d6520746f6b656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000353544b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c0ffee0000000000000000000000000000000000000000000000000000000000").unwrap();
        let result = user_op_log.abi_encode_packed().unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_uid_correctly_from_state_manager_log() {
        let user_op_log = UserOpRouterLogFromStateManager::try_from(&get_sample_enqueued_log()).unwrap();
        let expected_result =
            convert_hex_to_h256("be0a969cf68c8a51804458b2d841df79e2c7fa2f0e94b72b2859c5f8d660083d").unwrap();
        let result = user_op_log.to_uid().unwrap();
        assert_eq!(result, expected_result);
    }
}
