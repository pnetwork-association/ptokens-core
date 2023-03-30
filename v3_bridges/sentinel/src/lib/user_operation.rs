use std::{convert::TryFrom, fmt, str::FromStr};

use common::{BridgeSide, Byte, Bytes};
use common_eth::{encode_fxn_call, EthLog, EthLogExt, EthSubmissionMaterial};
use derive_more::{Constructor, Deref};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use ethers_core::abi::{self, Token};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{get_utc_timestamp, SentinelError, USER_OPERATION_TOPIC};

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
        origin_network_id: &[Byte],
    ) -> Result<Self, SentinelError> {
        let block_hash = sub_mat.get_block_hash()?;
        let block_timestamp = sub_mat.get_timestamp().as_secs();
        let witnessed_timestamp = get_utc_timestamp()?;

        let mut user_ops: Vec<UserOperation> = vec![];

        for receipt in sub_mat.receipts.iter() {
            let tx_hash = receipt.transaction_hash;
            for log in receipt.logs.iter() {
                if !log.topics.is_empty() && &log.address == state_manager {
                    for topic in log.topics.iter() {
                        if topic == &*USER_OPERATION_TOPIC {
                            let op = UserOperation::from_log(
                                side,
                                witnessed_timestamp,
                                block_timestamp,
                                block_hash,
                                tx_hash,
                                origin_network_id,
                                log,
                            )?;
                            user_ops.push(op);
                        }
                    }
                }
            }
        }

        Ok(Self::new(user_ops))
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
        self.user_operation.destination_account = s;
    }
}

#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize)]
pub struct UserOperation {
    tx_hash: EthHash,
    block_hash: EthHash,
    block_timestamp: u64,
    bridge_side: BridgeSide,
    origin_network_id: Bytes,
    witnessed_timestamp: u64,
    user_operation: UserOp, // NOTE This remains separate since we can parse it entirely from the log
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
    destination_network_id: Bytes,
    underlying_asset_name: String,
    underlying_asset_symbol: String,
    underlying_asset_decimals: U256,
    underlying_asset_token_address: EthAddress,
    underlying_asset_network_id: Bytes, // TODO make a type for this!
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

impl UserOperation {
    pub fn to_cancel_fxn_data(&self) -> Result<Bytes, SentinelError> {
        const CANCEL_FXN_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"}],\"internalType\":\"structTest.Operation\",\"name\":\"op\",\"type\":\"tuple\"}],\"name\":\"protocolCancelOperation\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

        Ok(encode_fxn_call(CANCEL_FXN_ABI, "protocolCancelOperation", &[
            EthAbiToken::Tuple(vec![
                EthAbiToken::FixedBytes(self.block_hash.as_bytes().to_vec()),
                EthAbiToken::FixedBytes(self.tx_hash.as_bytes().to_vec()),
                EthAbiToken::FixedBytes(self.user_operation.options_mask.clone()),
                EthAbiToken::Uint(self.user_operation.nonce),
                EthAbiToken::Uint(self.user_operation.underlying_asset_decimals),
                EthAbiToken::Uint(self.user_operation.asset_amount),
                EthAbiToken::Address(self.user_operation.underlying_asset_token_address),
                EthAbiToken::FixedBytes(self.origin_network_id.clone()),
                EthAbiToken::FixedBytes(self.user_operation.destination_network_id.clone()),
                EthAbiToken::FixedBytes(self.user_operation.underlying_asset_network_id.clone()),
                EthAbiToken::String(self.user_operation.destination_account.clone()),
                EthAbiToken::String(self.user_operation.underlying_asset_name.clone()),
                EthAbiToken::String(self.user_operation.underlying_asset_symbol.clone()),
                EthAbiToken::Bytes(self.user_operation.user_data.clone()),
            ]),
        ])?)
    }

    fn from_log(
        bridge_side: BridgeSide,
        witnessed_timestamp: u64,
        block_timestamp: u64,
        block_hash: EthHash,
        tx_hash: EthHash,
        origin_network_id: &[Byte],
        log: &EthLog,
    ) -> Result<Self, SentinelError> {
        Ok(Self {
            tx_hash,
            bridge_side,
            block_hash,
            block_timestamp,
            witnessed_timestamp,
            user_operation: UserOp::try_from(log)?,
            origin_network_id: origin_network_id.to_vec(),
        })
    }
}

impl UserOperation {
    pub fn to_uid(&self) -> Result<EthHash, SentinelError> {
        let mut hasher = Keccak::v256();
        let input = self.abi_encode_packed()?;
        let mut output = [0u8; 32];
        hasher.update(&input);
        hasher.finalize(&mut output);
        Ok(EthHash::from_slice(&output))
    }

    fn abi_encode_packed(&self) -> Result<Bytes, SentinelError> {
        Ok(abi::encode_packed(&[
            Token::FixedBytes(self.block_hash.as_bytes().to_vec()),
            Token::FixedBytes(self.tx_hash.as_bytes().to_vec()),
            Token::FixedBytes(self.origin_network_id.clone()),
            Token::Uint(Self::convert_u256_type(self.user_operation.nonce)),
            Token::String(self.user_operation.destination_account.clone()),
            Token::FixedBytes(self.user_operation.destination_network_id.clone()),
            Token::String(self.user_operation.underlying_asset_name.clone()),
            Token::String(self.user_operation.underlying_asset_symbol.clone()),
            Token::Uint(Self::convert_u256_type(self.user_operation.underlying_asset_decimals)),
            Token::Address(Self::convert_address_type(
                self.user_operation.underlying_asset_token_address,
            )),
            Token::FixedBytes(self.user_operation.underlying_asset_network_id.clone()),
            Token::Uint(Self::convert_u256_type(self.user_operation.asset_amount)),
            Token::Bytes(self.user_operation.user_data.clone()),
            Token::FixedBytes(self.user_operation.options_mask.clone()),
        ])?)
    }
}

impl UserOperation {
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
}

impl UserOp {
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

#[cfg(test)]
mod tests {
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::test_utils::get_sample_sub_mat_n;

    #[test]
    fn should_encode_fxn_data_for_user_op() {
        let op = UserOperation::default();
        let result = op.to_cancel_fxn_data();
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_user_operation_from_sub_mat() {
        let side = BridgeSide::Native;
        let sub_mat = get_sample_sub_mat_n(11);
        let sepolia_network_id = hex::decode("e15503e4").unwrap();
        let state_manager = convert_hex_to_eth_address("b274d81a823c1912c6884e39c2e4e669e04c83f4").unwrap();
        let expected_result = 1;
        let ops = UserOperations::from_sub_mat(side, &sub_mat, &state_manager, &sepolia_network_id).unwrap();
        let result = ops.len();
        assert_eq!(result, expected_result);
    }
}
