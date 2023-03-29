use std::{convert::TryFrom, fmt, str::FromStr};

use common::{BridgeSide, Byte, Bytes};
use common_eth::{convert_hex_to_h256, EthLog, EthLogExt, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError};

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor, Serialize, Deserialize)]
pub struct UnmatchedUserOps {
    native: UserOperations,
    host: UserOperations,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor, Deref, Serialize, Deserialize)]
pub struct UserOperations(Vec<UserOperation>);

impl UserOperations {
    pub fn add(&mut self, other: Self) {
        let a = self.0.clone();
        let b = other.0;
        self.0 = [a, b].concat();
    }

    pub fn remove_matches(self, other: Self) -> (Self, Self) {
        // TODO remove clones & make this efficient as possible since it's already O(self.len() * other.len())
        let mut self_user_ops: Vec<UserOperation> = vec![];
        let mut other_user_ops = other;

        for self_op in self.iter() {
            let len_before = other_user_ops.len();
            other_user_ops = Self::new(
                other_user_ops
                    .iter()
                    .cloned()
                    .filter(|other_op| self_op != other_op)
                    .collect::<Vec<_>>(),
            );
            let len_after = other_user_ops.len();

            // TODO Check incase > 1 got filtered out? Or should we not care?
            if len_before != len_after {
                debug!("Found a matching user op:\n{}", self_op);
            } else {
                self_user_ops.push(self_op.clone());
            }
        }

        (Self::new(self_user_ops), other_user_ops)
    }
}

impl UserOperations {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_sub_mat(
        side: BridgeSide,
        sub_mat: &EthSubmissionMaterial,
        state_manager: &EthAddress,
    ) -> Result<Self, SentinelError> {
        let block_hash = sub_mat.get_block_hash()?;
        let block_timestamp = sub_mat.get_timestamp().as_secs();
        let witnessed_timestamp = get_utc_timestamp()?;

        let mut logs: Vec<EthLog> = vec![];
        for receipt in sub_mat.receipts.iter() {
            for log in receipt.logs.iter() {
                if !log.topics.is_empty() && &log.address == state_manager && log.topics[0] == *USER_OPERATION_TOPIC {
                    logs.push(log.clone());
                }
            }
        }

        Ok(Self::new(
            logs.iter()
                .map(|l| UserOperation::from_log(side, witnessed_timestamp, block_timestamp, block_hash, l))
                .collect::<Result<Vec<UserOperation>, SentinelError>>()?,
        ))
    }
}

impl From<Vec<UserOperations>> for UserOperations {
    fn from(v: Vec<UserOperations>) -> Self {
        let mut user_ops: Vec<UserOperation> = vec![];
        for ops in v.into_iter() {
            for op in ops.iter() {
                user_ops.push(op.clone())
            }
        }
        Self::new(user_ops)
    }
}

#[cfg(test)]
impl UserOperation {
    pub fn set_destination_account(&mut self, s: String) {
        self.destination_account = s;
    }
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

#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize)]
pub struct UserOperation {
    block_hash: EthHash,
    block_timestamp: u64,
    bridge_side: BridgeSide,
    witnessed_timestamp: u64,
    user_operation: UserOp,
}

impl PartialEq for UserOperation {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We only care about the equality of the user operation from the log itself.
        self.user_operation == other.user_operation
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserOp {
    nonce: U256,
    destination_account: String,
    destination_network_id: MetadataChainId,
    underlying_asset_name: String,
    underlying_asset_symbol: String,
    underlying_asset_token_address: EthAddress,
    underlying_asset_chain_id: U256,
    asset_token_address: EthAddress,
    asset_amount: U256,
    user_data: Bytes,
    options_mask: Bytes,
}

impl TryFrom<&EthLog> for UserOp {
    type Error = SentinelError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        debug!("Decoding `UserOperation` from `EthLog`...");

        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::String,
                EthAbiParamType::String,
                EthAbiParamType::Address,
                EthAbiParamType::Uint(256),
                EthAbiParamType::Address,
                EthAbiParamType::Uint(256),
                EthAbiParamType::Bytes,
                EthAbiParamType::FixedBytes(32),
            ],
            &l.get_data(),
        )?;

        let nonce = Self::get_u256_from_token(&tokens[0])?;
        let destination_account = Self::get_string_from_token(&tokens[1])?;
        let destination_network_id = Self::get_metadata_chain_id_from_token(&tokens[2])?;
        let underlying_asset_name = Self::get_string_from_token(&tokens[3])?;
        let underlying_asset_symbol = Self::get_string_from_token(&tokens[4])?;
        let underlying_asset_token_address = Self::get_address_from_token(&tokens[5])?;
        let underlying_asset_chain_id = Self::get_u256_from_token(&tokens[6])?;
        let asset_token_address = Self::get_address_from_token(&tokens[7])?;
        let asset_amount = Self::get_u256_from_token(&tokens[8])?;
        let user_data = Self::get_bytes_from_token(&tokens[9])?;
        let options_mask = Self::get_bytes_from_token(&tokens[10])?;

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
            underlying_asset_chain_id,
            underlying_asset_token_address,
        })
    }
}

impl UserOperation {
    fn from_log(
        bridge_side: BridgeSide,
        witnessed_timestamp: u64,
        block_timestamp: u64,
        block_hash: EthHash,
        l: &EthLog,
    ) -> Result<Self, SentinelError> {
        Ok(Self {
            bridge_side,
            block_hash,
            block_timestamp,
            witnessed_timestamp,
            user_operation: UserOp::try_from(l)?,
        })
    }
}

impl UserOp {
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

    #[allow(unused)]
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

impl fmt::Display for UserOperations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `UserOperations` to string: {e}",),
        }
    }
}

impl fmt::Display for UserOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `UserOperation` to string: {e}",),
        }
    }
}

impl FromStr for UserOperations {
    type Err = SentinelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

impl TryInto<Bytes> for UserOperations {
    type Error = SentinelError;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        Ok(serde_json::to_vec(&self)?)
    }
}

impl TryFrom<&[Byte]> for UserOperations {
    type Error = SentinelError;

    fn try_from(b: &[Byte]) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(b)?)
    }
}

impl TryFrom<Bytes> for UserOperations {
    type Error = SentinelError;

    fn try_from(b: Bytes) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(&b)?)
    }
}
