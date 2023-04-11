use std::{convert::TryFrom, fmt};

use common::{BridgeSide, Byte, Bytes, MIN_DATA_SENSITIVITY_LEVEL};
use common_eth::{encode_fxn_call, EthLog};
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use ethers_core::abi::{self, Token};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use super::{UserOpError, UserOpFlag, UserOpLog, UserOpState};
use crate::{DbKey, DbUtilsT, SentinelError};

impl DbUtilsT for UserOp {
    fn key(&self) -> Result<DbKey, SentinelError> {
        Ok(self.to_uid().into())
    }

    fn sensitivity() -> Option<Byte> {
        MIN_DATA_SENSITIVITY_LEVEL
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[cfg(test)]
impl UserOp {
    pub fn set_destination_account(&mut self, s: String) {
        self.user_op_log.destination_account = s;
    }
}

#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize)]
pub struct UserOp {
    tx_hash: EthHash,
    state: UserOpState,
    block_hash: EthHash,
    block_timestamp: u64,
    user_op_log: UserOpLog,
    bridge_side: BridgeSide,
    origin_network_id: Bytes,
    witnessed_timestamp: u64,
    previous_states: Vec<UserOpState>,
}

impl PartialEq for UserOp {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We only care about the equality of the user operation from the log itself.
        self.user_op_log == other.user_op_log
    }
}

impl UserOp {
    pub fn to_flag(&self) -> UserOpFlag {
        self.into()
    }

    pub fn to_cancel_fxn_data(&self) -> Result<Bytes, SentinelError> {
        const CANCEL_FXN_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"}],\"internalType\":\"structTest.Operation\",\"name\":\"op\",\"type\":\"tuple\"}],\"name\":\"protocolCancelOperation\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

        Ok(encode_fxn_call(CANCEL_FXN_ABI, "protocolCancelOperation", &[
            EthAbiToken::Tuple(vec![
                EthAbiToken::FixedBytes(self.block_hash.as_bytes().to_vec()),
                EthAbiToken::FixedBytes(self.tx_hash.as_bytes().to_vec()),
                EthAbiToken::FixedBytes(self.user_op_log.options_mask.as_bytes().to_vec()),
                EthAbiToken::Uint(self.user_op_log.nonce),
                EthAbiToken::Uint(self.user_op_log.underlying_asset_decimals),
                EthAbiToken::Uint(self.user_op_log.amount),
                EthAbiToken::Address(self.user_op_log.underlying_asset_token_address),
                EthAbiToken::FixedBytes(self.origin_network_id.clone()),
                EthAbiToken::FixedBytes(self.user_op_log.destination_network_id.clone()),
                EthAbiToken::FixedBytes(self.user_op_log.underlying_asset_network_id.clone()),
                EthAbiToken::String(self.user_op_log.destination_account.clone()),
                EthAbiToken::String(self.user_op_log.underlying_asset_name.clone()),
                EthAbiToken::String(self.user_op_log.underlying_asset_symbol.clone()),
                EthAbiToken::Bytes(self.user_op_log.user_data.clone()),
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
    ) -> Result<Self, UserOpError> {
        Ok(Self {
            tx_hash,
            block_hash,
            bridge_side,
            block_timestamp,
            witnessed_timestamp,
            previous_states: vec![],
            user_op_log: UserOpLog::try_from(log)?,
            origin_network_id: origin_network_id.to_vec(),
            state: UserOpState::try_from_log(bridge_side, tx_hash, log)?,
        })
    }
}

impl UserOp {
    pub fn state(&self) -> UserOpState {
        self.state
    }

    pub fn to_uid(&self) -> EthHash {
        let mut hasher = Keccak::v256();
        let input = self.abi_encode();
        let mut output = [0u8; 32];
        hasher.update(&input);
        hasher.finalize(&mut output);
        EthHash::from_slice(&output)
    }

    fn abi_encode(&self) -> Bytes {
        abi::encode(&[
            Token::FixedBytes(self.block_hash.as_bytes().to_vec()),
            Token::FixedBytes(self.tx_hash.as_bytes().to_vec()),
            Token::FixedBytes(self.origin_network_id.clone()),
            Token::Uint(Self::convert_u256_type(self.user_op_log.nonce)),
            Token::String(self.user_op_log.destination_account.clone()),
            Token::FixedBytes(self.user_op_log.destination_network_id.clone()),
            Token::String(self.user_op_log.underlying_asset_name.clone()),
            Token::String(self.user_op_log.underlying_asset_symbol.clone()),
            Token::Uint(Self::convert_u256_type(self.user_op_log.underlying_asset_decimals)),
            Token::Address(Self::convert_address_type(
                self.user_op_log.underlying_asset_token_address,
            )),
            Token::FixedBytes(self.user_op_log.underlying_asset_network_id.clone()),
            Token::Uint(Self::convert_u256_type(self.user_op_log.amount)),
            Token::Bytes(self.user_op_log.user_data.clone()),
            Token::FixedBytes(self.user_op_log.options_mask.as_bytes().to_vec()),
        ])
    }
}

impl UserOp {
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

impl fmt::Display for UserOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `UserOp` to string: {e}",),
        }
    }
}

#[cfg(test)]
mod tests {
    use common_eth::convert_hex_to_h256;

    use super::*;
    use crate::{get_utc_timestamp, test_utils::get_sample_sub_mat_n};

    #[test]
    fn should_encode_fxn_data_for_user_op() {
        let op = UserOp::default();
        let result = op.to_cancel_fxn_data();
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_user_op_correctly_from_log() {
        let sub_mat = get_sample_sub_mat_n(11);
        let bridge_side = BridgeSide::Native;
        let witnessed_timestamp = get_utc_timestamp().unwrap();
        let block_timestamp = sub_mat.get_timestamp().as_secs() as u64;
        let block_hash = sub_mat.block.unwrap().hash.clone();
        let receipt = sub_mat.receipts[1].clone();
        let tx_hash = receipt.transaction_hash.clone();
        let origin_network_id = hex::decode("01020304").unwrap();
        let log = receipt.logs[0].clone();
        let result = UserOp::from_log(
            bridge_side,
            witnessed_timestamp,
            block_timestamp,
            block_hash,
            tx_hash,
            &origin_network_id,
            &log,
        )
        .unwrap();
        let expected_state = UserOpState::Enqueued(bridge_side, tx_hash);
        assert_eq!(result.state, expected_state);
    }
}
