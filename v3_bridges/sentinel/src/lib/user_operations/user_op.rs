use std::{convert::TryFrom, fmt};

use common::{BridgeSide, Byte, Bytes};
use common_eth::{encode_fxn_call, EthLog, EthLogExt};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use ethers_core::abi::{self, Token};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{SentinelError, UserOpState};

#[cfg(test)]
impl UserOperation {
    pub fn set_destination_account(&mut self, s: String) {
        self.user_operation.destination_account = s;
    }
}

#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize)]
pub struct UserOperation {
    tx_hash: EthHash,
    state: UserOpState,
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

    pub fn from_log(
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
            state: UserOpState::Witnessed(bridge_side, tx_hash),
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

impl fmt::Display for UserOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `UserOperation` to string: {e}",),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_encode_fxn_data_for_user_op() {
        let op = UserOperation::default();
        let result = op.to_cancel_fxn_data();
        assert!(result.is_ok());
    }
}
