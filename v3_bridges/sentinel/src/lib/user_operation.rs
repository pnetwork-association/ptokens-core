use std::convert::TryFrom;

use common::Bytes;
use common_eth::{convert_hex_to_h256, EthLog, EthLogExt};
use common_metadata::MetadataChainId;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::SentinelError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UserOperation {
    block_hash: EthHash,
    nonce: U256,
    destination_account: String,
    destination_network_id: MetadataChainId,
    underlying_asset_token_address: EthAddress,
    underlying_asset_name: String,
    underlying_asset_symbol: String,
    underlying_asset_chain_id: MetadataChainId,
    asset_token_address: EthAddress,
    asset_amount: U256,
    user_data: Bytes,
    options_mask: Bytes,
}

// NOTE: Originally we worked w/ > 1 topic, hence using a macro - bit overkill now.
macro_rules! get_topics {
    ($($name:ident => $hex:expr),* $(,)?) => {
        $(
            lazy_static! {
                pub static ref $name: EthHash = convert_hex_to_h256(&$hex)
                    .expect(&format!("Converting from hex shouldn't fail for {}", stringify!($name)));
            }
        )*
    }
}

get_topics!(
    USER_OPERATION_TOPIC => "375102e6250006aa44e53e96d29b6a719df98a1c40b28c133e684ef40e52b989",
);

impl TryFrom<EthLog> for UserOperation {
    type Error = SentinelError;
    fn try_from(l: EthLog) -> Result<Self, Self::Error> {
        Self::try_from(&l)
    }
}

impl TryFrom<&EthLog> for UserOperation {
    type Error = SentinelError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        debug!("Decoding `UserOperation` from `EthLog`...");
        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::FixedBytes(32),
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::Address,
                EthAbiParamType::String,
                EthAbiParamType::String,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::Address,
                EthAbiParamType::Uint(256),
                EthAbiParamType::Bytes,
                EthAbiParamType::FixedBytes(32),
            ],
            &l.get_data(),
        )?;

        let block_hash = Self::get_eth_hash_from_token(&tokens[0])?;
        let nonce = Self::get_u256_from_token(&tokens[1])?;
        let destination_account = Self::get_string_from_token(&tokens[2])?;
        let destination_network_id = Self::get_metadata_chain_id_from_token(&tokens[3])?;
        let underlying_asset_token_address = Self::get_address_from_token(&tokens[4])?;
        let underlying_asset_name = Self::get_string_from_token(&tokens[5])?;
        let underlying_asset_symbol = Self::get_string_from_token(&tokens[6])?;
        let underlying_asset_chain_id = Self::get_metadata_chain_id_from_token(&tokens[7])?;
        let asset_token_address = Self::get_address_from_token(&tokens[8])?;
        let asset_amount = Self::get_u256_from_token(&tokens[9])?;
        let user_data = Self::get_bytes_from_token(&tokens[10])?;
        let options_mask = Self::get_bytes_from_token(&tokens[11])?;

        Ok(Self {
            block_hash,
            nonce,
            destination_account,
            destination_network_id,
            underlying_asset_token_address,
            underlying_asset_name,
            underlying_asset_symbol,
            underlying_asset_chain_id,
            asset_token_address,
            asset_amount,
            user_data,
            options_mask,
        })
    }
}

impl UserOperation {
    fn uid() -> EthHash {
        todo!("uid conversion from user operation")
    }

    fn get_address_from_token(t: &EthAbiToken) -> Result<EthAddress, SentinelError> {
        match t {
            EthAbiToken::Address(t) => Ok(EthAddress::from_slice(t.as_bytes())),
            _ => Err(SentinelError::Custom("Cannot convert `{t}` to ETH address!".into())),
        }
    }

    fn get_string_from_token(t: &EthAbiToken) -> Result<String, SentinelError> {
        match t {
            EthAbiToken::String(ref t) => Ok(t.clone()),
            _ => Err(SentinelError::Custom("Cannot convert `{t}` to string!".into())),
        }
    }

    fn get_bytes_from_token(t: &EthAbiToken) -> Result<Bytes, SentinelError> {
        match t {
            EthAbiToken::Bytes(b) => Ok(b.clone()),
            _ => Err(SentinelError::Custom("Cannot convert `{t}` to bytes!".into())),
        }
    }

    fn get_eth_hash_from_token(t: &EthAbiToken) -> Result<EthHash, SentinelError> {
        match t {
            EthAbiToken::FixedBytes(ref b) => Ok(EthHash::from_slice(b)),
            _ => Err(SentinelError::Custom("Cannot convert `{t}` to EthHash!".into())),
        }
    }

    fn get_u256_from_token(t: &EthAbiToken) -> Result<U256, SentinelError> {
        match t {
            EthAbiToken::Uint(u) => {
                let mut b = [0u8; 32];
                u.to_big_endian(&mut b);
                Ok(U256::from_big_endian(&b))
            },
            _ => Err(SentinelError::Custom("Cannot convert `{t}` to U256!".into())),
        }
    }

    fn get_metadata_chain_id_from_token(t: &EthAbiToken) -> Result<MetadataChainId, SentinelError> {
        Self::get_bytes_from_token(t).and_then(|ref bs| Ok(MetadataChainId::from_bytes(bs)?))
    }
}
