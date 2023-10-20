use std::{convert::TryFrom, fmt};

use common::{BridgeSide, Byte, Bytes, MIN_DATA_SENSITIVITY_LEVEL};
use common_eth::EthLog;
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethabi::{encode as eth_abi_encode, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{UserOpError, UserOpFlag, UserOpLog, UserOpState};
use crate::{DbKey, DbUtilsT, NetworkId, SentinelError};

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

#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize, Getters)]
pub struct UserOp {
    #[getter(skip)]
    pub(super) uid: EthHash,
    pub(super) tx_hash: EthHash,
    pub(super) asset_amount: u64,
    pub(super) state: UserOpState,
    pub(super) block_hash: EthHash,
    pub(super) block_timestamp: u64,
    pub(super) user_op_log: UserOpLog,
    pub(super) bridge_side: BridgeSide,
    pub(super) witnessed_timestamp: u64,
    pub(super) origin_network_id: NetworkId,
    pub(super) previous_states: Vec<UserOpState>,
}

impl PartialEq for UserOp {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We only care about the equality of the user operation from the log itself.
        self.user_op_log == other.user_op_log
    }
}

impl UserOp {
    pub fn origin_mcid(&self) -> Result<MetadataChainId, UserOpError> {
        Ok(MetadataChainId::try_from(self.origin_network_id())?)
    }

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

    pub fn origin_nid(&self) -> Result<NetworkId, UserOpError> {
        match self.state {
            UserOpState::Witnessed(nid, ..) => Ok(nid),
            _ => Err(UserOpError::NotWitnessed(Box::new(self.clone()))),
        }
    }

    pub fn destination_nid(&self) -> Result<NetworkId, UserOpError> {
        match self.state {
            UserOpState::Enqueued(nid, ..) | UserOpState::Executed(nid, ..) | UserOpState::Cancelled(nid, ..) => {
                Ok(nid)
            },
            _ => Err(UserOpError::DestinationUnknown(Box::new(self.clone()))),
        }
    }

    pub fn from_log(
        bridge_side: BridgeSide,
        witnessed_timestamp: u64,
        block_timestamp: u64,
        block_hash: EthHash,
        tx_hash: EthHash,
        origin_network_id: &NetworkId,
        log: &EthLog,
    ) -> Result<Self, UserOpError> {
        let mut user_op_log = UserOpLog::try_from(log)?;

        let asset_amount = user_op_log.asset_amount.as_u64();
        // NOTE: A witnessed user op needs these fields from the block it was witnessed in. All
        // other states will include the full log, with these fields already included.
        user_op_log.maybe_update_fields(block_hash, tx_hash, *origin_network_id);

        let mut op = Self {
            tx_hash,
            block_hash,
            bridge_side,
            user_op_log,
            asset_amount,
            block_timestamp,
            witnessed_timestamp,
            uid: EthHash::zero(),
            previous_states: vec![],
            origin_network_id: *origin_network_id,
            state: UserOpState::try_from_log(*origin_network_id, tx_hash, log, block_timestamp)?,
        };

        let uid = op.uid()?;
        op.uid = uid;

        Ok(op)
    }
}

impl UserOp {
    pub(super) fn check_num_tokens(tokens: &[EthAbiToken], n: usize, location: &str) -> Result<(), UserOpError> {
        let l = tokens.len();
        if l != n {
            Err(UserOpError::NotEnoughTokens {
                got: l,
                expected: n,
                location: location.into(),
            })
        } else {
            Ok(())
        }
    }

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
            if !self.previous_states.contains(other_state) {
                info!("previous state ({other_state}) not seen before, saving it but not updating self");
                self.previous_states.push(*other_state);
            } else {
                info!("previous state ({other_state}) seen before, doing nothing");
            }
        } else {
            info!("state more advanced, updating self from {self_state} to {other_state}");
            self.previous_states.push(*self_state);
            self.state = *other_state;
        };

        Ok(())
    }

    pub fn uid(&self) -> Result<EthHash, UserOpError> {
        let mut hasher = Sha256::new();
        let input = self.abi_encode()?;
        hasher.update(&input);
        Ok(EthHash::from_slice(&hasher.finalize()))
    }

    pub fn uid_hex(&self) -> Result<String, UserOpError> {
        self.uid().map(|uid| format!("0x{}", hex::encode(uid.as_bytes())))
    }

    pub(super) fn to_eth_abi_token(&self) -> Result<EthAbiToken, UserOpError> {
        Ok(EthAbiToken::Tuple(vec![
            EthAbiToken::FixedBytes(self.user_op_log.origin_block_hash()?.as_bytes().to_vec()),
            EthAbiToken::FixedBytes(self.user_op_log.origin_transaction_hash()?.as_bytes().to_vec()),
            EthAbiToken::FixedBytes(self.user_op_log.options_mask.as_bytes().to_vec()),
            EthAbiToken::Uint(self.user_op_log.nonce),
            EthAbiToken::Uint(self.user_op_log.underlying_asset_decimals),
            EthAbiToken::Uint(self.user_op_log.asset_amount),
            EthAbiToken::Uint(self.user_op_log.protocol_fee_asset_amount),
            EthAbiToken::Uint(self.user_op_log.network_fee_asset_amount),
            EthAbiToken::Uint(self.user_op_log.forward_network_fee_asset_amount),
            EthAbiToken::Address(self.user_op_log.underlying_asset_token_address),
            EthAbiToken::FixedBytes(self.user_op_log.origin_network_id()?.to_bytes_4()?.to_vec()),
            EthAbiToken::FixedBytes(self.user_op_log.destination_network_id.to_bytes_4()?.to_vec()),
            EthAbiToken::FixedBytes(self.user_op_log.forward_destination_network_id.to_bytes_4()?.to_vec()),
            EthAbiToken::FixedBytes(self.user_op_log.underlying_asset_network_id.to_bytes_4()?.to_vec()),
            EthAbiToken::String(self.user_op_log.origin_account.clone()),
            EthAbiToken::String(self.user_op_log.destination_account.clone()),
            EthAbiToken::String(self.user_op_log.underlying_asset_name.clone()),
            EthAbiToken::String(self.user_op_log.underlying_asset_symbol.clone()),
            EthAbiToken::Bytes(self.user_op_log.user_data.clone()),
            EthAbiToken::Bool(self.user_op_log.is_for_protocol),
        ]))
    }

    fn abi_encode(&self) -> Result<Bytes, UserOpError> {
        Ok(eth_abi_encode(&[self.to_eth_abi_token()?]))
    }

    pub(super) fn get_tuple_from_token(t: &EthAbiToken) -> Result<Vec<EthAbiToken>, UserOpError> {
        match t {
            EthAbiToken::Tuple(v) => Ok(v.to_vec()),
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "tuple token".into(),
            }),
        }
    }

    pub(super) fn get_address_from_token(t: &EthAbiToken) -> Result<EthAddress, UserOpError> {
        match t {
            EthAbiToken::Address(t) => Ok(EthAddress::from_slice(t.as_bytes())),
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "ETH address".into(),
            }),
        }
    }

    pub(super) fn get_string_from_token(t: &EthAbiToken) -> Result<String, UserOpError> {
        match t {
            EthAbiToken::String(ref t) => Ok(t.clone()),
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "string".into(),
            }),
        }
    }

    pub(super) fn get_bytes_from_token(t: &EthAbiToken) -> Result<Bytes, UserOpError> {
        match t {
            EthAbiToken::Bytes(b) => Ok(b.clone()),
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "bytes".into(),
            }),
        }
    }

    pub(super) fn get_fixed_bytes_from_token(t: &EthAbiToken) -> Result<Bytes, UserOpError> {
        match t {
            EthAbiToken::FixedBytes(b) => Ok(b.to_vec()),
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "fixed bytes".into(),
            }),
        }
    }

    pub(super) fn get_eth_hash_from_token(t: &EthAbiToken) -> Result<EthHash, UserOpError> {
        match t {
            EthAbiToken::FixedBytes(ref b) => Ok(EthHash::from_slice(b)),
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "EthHash".into(),
            }),
        }
    }

    pub(super) fn get_u256_from_token(t: &EthAbiToken) -> Result<U256, UserOpError> {
        match t {
            EthAbiToken::Uint(u) => {
                let mut b = [0u8; 32];
                u.to_big_endian(&mut b);
                Ok(U256::from_big_endian(&b))
            },
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "U256".into(),
            }),
        }
    }

    pub(super) fn get_bool_from_token(t: &EthAbiToken) -> Result<bool, UserOpError> {
        match t {
            EthAbiToken::Bool(b) => Ok(*b),
            _ => Err(UserOpError::CannotConvertEthAbiToken {
                from: t.clone(),
                to: "U256".into(),
            }),
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
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::user_ops::{
        test_utils::{get_sample_submission_material_with_user_send, get_sample_submission_material_with_user_send_2},
        UserOps,
    };

    #[test]
    fn should_get_user_op_from_user_send() {
        let side = BridgeSide::Native;
        let origin_network_id = NetworkId::try_from("binance").unwrap();
        let pnetwork_hub = convert_hex_to_eth_address("0x22BeC08c2241Ef915ed72bd876F4e4Bc4336d055").unwrap();
        let sub_mat = get_sample_submission_material_with_user_send();
        let ops = UserOps::from_sub_mat(side, &origin_network_id, &pnetwork_hub, &sub_mat).unwrap();
        assert_eq!(ops.len(), 1);
        println!("{}", ops[0]);
        let op = ops[0].clone();
        let bytes = hex::encode(op.abi_encode().unwrap());
        let expected_bytes = "0000000000000000000000000000000000000000000000000000000000000020a64cb6297de82bf16aca0b38760991cc0eec7b3ca5ae2e93d8754902eb927744eadd7dcd6beae94fceac5322937fb9994ee32e25cc12a54959eadc0d5b36e7ca0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001d2d700000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000002710000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064000000000000000000000000daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af925aca268b00000000000000000000000000000000000000000000000000000000f9b459a1000000000000000000000000000000000000000000000000000000005aca268b000000000000000000000000000000000000000000000000000000005aca268b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000340000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786464623566343533353132336461613561653334336332343030366634303735616261663566376200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786444623566343533353132334441613561453334336332343030364634303735614241463546374200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e704e6574776f726b20546f6b656e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003504e5400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(bytes, expected_bytes);
        let uid = op.uid_hex().unwrap();
        let expected_uid = "0x724820e71f874192a9c41ea55b36a2a4f3b98be0b8cfb617295cdf21010bfeb3";
        assert_eq!(uid, expected_uid);
    }

    #[test]
    fn should_get_user_op_from_user_send_2() {
        let sub_mat = get_sample_submission_material_with_user_send_2();
        let side = BridgeSide::Native;
        let origin_network_id = NetworkId::try_from("binance").unwrap();
        let pnetwork_hub = convert_hex_to_eth_address("0x02878021ba5472F7F1e2bfb223ee6cf4b1eadA07").unwrap();
        let ops = UserOps::from_sub_mat(side, &origin_network_id, &pnetwork_hub, &sub_mat).unwrap();
        assert_eq!(ops.len(), 1);
        let op = ops[0].clone();
        let uid = op.uid_hex().unwrap();
        let expected_uid = "0xd9feb6e60cd73c396cbaeb3e5fa55c774c03a274c54f5bc53a62a59855ec7cc4";
        assert_eq!(uid, expected_uid);
    }
}
