use std::convert::TryFrom;

use common::Bytes;
use common_eth::{EthLog, EthLogExt};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::SentinelError;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserOpLog {
    pub(super) nonce: U256,
    pub(super) destination_account: String,
    pub(super) destination_network_id: Bytes,
    pub(super) underlying_asset_name: String,
    pub(super) underlying_asset_symbol: String,
    pub(super) underlying_asset_decimals: U256,
    pub(super) underlying_asset_token_address: EthAddress,
    pub(super) underlying_asset_network_id: Bytes, // TODO make a type for this!
    pub(super) asset_token_address: EthAddress,
    pub(super) asset_amount: U256,
    pub(super) user_data: Bytes,
    pub(super) options_mask: Bytes,
}

impl TryFrom<&EthLog> for UserOpLog {
    type Error = SentinelError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        debug!("Decoding `UserOp` from `EthLog`...");

        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::String,
                EthAbiParamType::String,
                EthAbiParamType::Uint(256),
                EthAbiParamType::Address,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::Address,
                EthAbiParamType::Uint(256),
                EthAbiParamType::Bytes,
                EthAbiParamType::FixedBytes(32),
            ],
            &l.get_data(),
        )?;

        let nonce = Self::get_u256_from_token(&tokens[0])?;
        let destination_account = Self::get_string_from_token(&tokens[1])?;
        let destination_network_id = Self::get_fixed_bytes_from_token(&tokens[2])?;
        let underlying_asset_name = Self::get_string_from_token(&tokens[3])?;
        let underlying_asset_symbol = Self::get_string_from_token(&tokens[4])?;
        let underlying_asset_decimals = Self::get_u256_from_token(&tokens[5])?;
        let underlying_asset_token_address = Self::get_address_from_token(&tokens[6])?;
        let underlying_asset_network_id = Self::get_fixed_bytes_from_token(&tokens[7])?;
        let asset_token_address = Self::get_address_from_token(&tokens[8])?;
        let asset_amount = Self::get_u256_from_token(&tokens[9])?;
        let user_data = Self::get_bytes_from_token(&tokens[10])?;
        let options_mask = Self::get_fixed_bytes_from_token(&tokens[11])?;

        Ok(Self {
            nonce,
            user_data,
            asset_amount,
            options_mask,
            asset_token_address,
            destination_account,
            underlying_asset_name,
            destination_network_id,
            underlying_asset_symbol,
            underlying_asset_decimals,
            underlying_asset_network_id,
            underlying_asset_token_address,
        })
    }
}

impl UserOpLog {
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
