use std::{convert::TryFrom, fmt};

use common::{BridgeSide, Byte, Bytes, MIN_DATA_SENSITIVITY_LEVEL};
use common_eth::EthLog;
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use ethers_core::abi::{self, Token};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tiny_keccak::{Hasher, Keccak};

use super::{UserOpError, UserOpFlag, UserOpLog, UserOpState};
use crate::{DbKey, DbUtilsT, SentinelError};

impl DbUtilsT for UserOp {
    fn key(&self) -> Result<DbKey, SentinelError> {
        Ok(self.uid()?.into())
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

#[serde_as]
#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize)]
pub struct UserOp {
    pub(super) uid: EthHash,
    pub(super) tx_hash: EthHash,
    pub(super) state: UserOpState,
    pub(super) block_hash: EthHash,
    pub(super) block_timestamp: u64,
    pub(super) user_op_log: UserOpLog,
    pub(super) bridge_side: BridgeSide,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) origin_network_id: Bytes,
    pub(super) witnessed_timestamp: u64,
    pub(super) previous_states: Vec<UserOpState>,
}

impl PartialEq for UserOp {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We only care about the equality of the user operation from the log itself.
        self.user_op_log == other.user_op_log
    }
}

impl UserOp {
    pub fn enqueued_timestamp(&self) -> Result<u64, UserOpError> {
        let e = UserOpError::HasNotBeenEnqueued;

        if self.has_not_been_enqueued() {
            return Err(e);
        };

        let enqueued_state = if self.state.is_enqueued() {
            self.state
        } else {
            let x = self
                .previous_states
                .iter()
                .filter(|state| state.is_enqueued())
                .cloned()
                .collect::<Vec<UserOpState>>();
            if x.is_empty() {
                return Err(e);
            } else {
                x[0]
            }
        };

        Ok(enqueued_state.timestamp())
    }

    pub fn side(&self) -> BridgeSide {
        self.bridge_side
    }

    pub fn to_flag(&self) -> UserOpFlag {
        self.into()
    }

    pub fn origin_side(&self) -> BridgeSide {
        // NOTE: This depends entirely on which side of the bridge sees the user op first, since
        // that's not guaranteed to be either or.
        match self.state {
            UserOpState::Witnessed(side, ..) => side,
            _ => self.state.side().opposite(),
        }
    }

    pub fn destination_side(&self) -> BridgeSide {
        self.origin_side().opposite()
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
        let mut user_op_log = UserOpLog::try_from(log)?;

        // NOTE: A witnessed user op needs these fields from the block it was witnessed in. All
        // other states will include the full log, with these fields already included.
        user_op_log.maybe_update_fields(block_hash, tx_hash, origin_network_id.to_vec());

        let mut op = Self {
            tx_hash,
            block_hash,
            bridge_side,
            user_op_log,
            block_timestamp,
            witnessed_timestamp,
            uid: EthHash::zero(),
            previous_states: vec![],
            origin_network_id: origin_network_id.to_vec(),
            state: UserOpState::try_from_log(bridge_side, tx_hash, log, block_timestamp)?,
        };

        let uid = op.uid()?;
        op.uid = uid;

        Ok(op)
    }
}

impl UserOp {
    pub fn has_been_enqueued(&self) -> bool {
        self.state.is_enqueued() || self.previous_states.iter().any(|state| state.is_enqueued())
    }

    pub fn has_not_been_enqueued(&self) -> bool {
        !self.has_been_enqueued()
    }

    pub fn has_been_witnessed(&self) -> bool {
        self.state.is_witnessed() || self.previous_states.iter().any(|state| state.is_witnessed())
    }

    pub fn has_not_been_witnessed(&self) -> bool {
        !self.has_been_witnessed()
    }

    pub fn is_enqueued(&self) -> bool {
        self.state.is_enqueued()
    }

    pub fn maybe_update_state(&mut self, other: Self) -> Result<(), UserOpError> {
        let self_state = self.state();
        let other_state = other.state();

        if self.uid()? != other.uid()? {
            return Err(UserOpError::UidMismatch {
                a: self.uid()?,
                b: other.uid()?,
            });
        };

        if self_state >= other_state {
            if !self.previous_states.contains(&other_state) {
                info!("previous state ({other_state}) not seen before, saving it but not updating self");
                self.previous_states.push(other_state);
            } else {
                info!("previous state ({other_state}) seen before, doing nothing");
            }
        } else {
            info!("state more advanced, updating self from {self_state} to {other_state}");
            self.previous_states.push(self_state);
            self.state = other_state;
        };

        Ok(())
    }

    pub fn state(&self) -> UserOpState {
        self.state
    }

    pub fn uid(&self) -> Result<EthHash, UserOpError> {
        let mut hasher = Keccak::v256();
        let input = self.abi_encode()?;
        let mut output = [0u8; 32];
        hasher.update(&input);
        hasher.finalize(&mut output);
        Ok(EthHash::from_slice(&output))
    }

    pub fn uid_hex(&self) -> Result<String, UserOpError> {
        self.uid().map(|uid| format!("0x{}", hex::encode(uid.as_bytes())))
    }

    fn abi_encode(&self) -> Result<Bytes, UserOpError> {
        Ok(abi::encode(&[
            Token::FixedBytes(self.user_op_log.origin_block_hash()?.as_bytes().to_vec()),
            Token::FixedBytes(self.user_op_log.origin_transaction_hash()?.as_bytes().to_vec()),
            Token::FixedBytes(self.user_op_log.origin_network_id()?),
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
        ]))
    }

    pub(super) fn convert_address_type(t: EthAddress) -> ethers_core::types::Address {
        // NOTE: Sigh. The ethabi crate re-exports the ethereum_types which we use elsewhere, so
        // that's annoying.
        let s = t.as_bytes();
        ethers_core::types::Address::from_slice(s)
    }

    pub(super) fn convert_u256_type(t: U256) -> ethers_core::types::U256 {
        // NOTE: Sigh. The ethabi crate re-exports the ethereum_types which we use elsewhere, so
        // that's annoying.
        let mut r = [0u8; 32];
        t.to_big_endian(&mut r);
        ethers_core::types::U256::from_big_endian(&r)
    }

    pub(super) fn get_tuple_from_token(t: &EthAbiToken) -> Result<Vec<EthAbiToken>, SentinelError> {
        match t {
            EthAbiToken::Tuple(v) => Ok(v.to_vec()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to tuple token"))),
        }
    }

    pub(super) fn get_address_from_token(t: &EthAbiToken) -> Result<EthAddress, SentinelError> {
        match t {
            EthAbiToken::Address(t) => Ok(EthAddress::from_slice(t.as_bytes())),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to ETH address!"))),
        }
    }

    pub(super) fn get_string_from_token(t: &EthAbiToken) -> Result<String, SentinelError> {
        match t {
            EthAbiToken::String(ref t) => Ok(t.clone()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to string!"))),
        }
    }

    pub(super) fn get_bytes_from_token(t: &EthAbiToken) -> Result<Bytes, SentinelError> {
        match t {
            EthAbiToken::Bytes(b) => Ok(b.clone()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to bytes:"))),
        }
    }

    pub(super) fn get_fixed_bytes_from_token(t: &EthAbiToken) -> Result<Bytes, SentinelError> {
        match t {
            EthAbiToken::FixedBytes(b) => Ok(b.to_vec()),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to bytes:"))),
        }
    }

    pub(super) fn get_eth_hash_from_token(t: &EthAbiToken) -> Result<EthHash, SentinelError> {
        match t {
            EthAbiToken::FixedBytes(ref b) => Ok(EthHash::from_slice(b)),
            _ => Err(SentinelError::Custom(format!("Cannot convert `{t}` to EthHash!"))),
        }
    }

    pub(super) fn get_u256_from_token(t: &EthAbiToken) -> Result<U256, SentinelError> {
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
    use crate::{
        get_utc_timestamp,
        test_utils::get_sample_sub_mat_n,
        user_ops::test_utils::{get_sample_enqueued_user_op, get_sample_witnessed_user_op},
    };

    #[test]
    fn should_get_user_op_correctly_from_log() {
        let sub_mat = get_sample_sub_mat_n(11);
        let bridge_side = BridgeSide::Native;
        let witnessed_timestamp = get_utc_timestamp().unwrap();
        let block_timestamp = sub_mat.get_timestamp().as_secs();
        let block_hash = sub_mat.block.unwrap().hash;
        let receipt = sub_mat.receipts[1].clone();
        let tx_hash = receipt.transaction_hash;
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
        let expected_state = UserOpState::Enqueued(bridge_side, tx_hash, 1);
        assert_eq!(result.state, expected_state);
    }

    #[test]
    fn should_get_enqueued_user_op_uid() {
        let user_op = get_sample_enqueued_user_op();
        let expected_uid =
            convert_hex_to_h256("be0a969cf68c8a51804458b2d841df79e2c7fa2f0e94b72b2859c5f8d660083d").unwrap();
        let uid = user_op.uid().unwrap();
        assert_eq!(uid, expected_uid);
    }

    #[test]
    fn should_get_witnessed_user_op_uid() {
        let user_op = get_sample_witnessed_user_op();
        let expected_uid =
            convert_hex_to_h256("68387313a7d1eacdbc7b8e8f6125bc4c6efaece93eee4d93ce6c1324ecb85c8c").unwrap();
        let uid = user_op.uid().unwrap();
        assert_eq!(uid, expected_uid);
    }
}
