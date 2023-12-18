use std::{convert::TryFrom, fmt};

use common::{Byte, Bytes, MIN_DATA_SENSITIVITY_LEVEL};
use common_eth::EthLog;
use derive_getters::Getters;
use ethabi::{encode as eth_abi_encode, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use sha2::{Digest, Sha256};

use super::{UserOpError, UserOpFlag, UserOpLog, UserOpState, UserOpVersion};
use crate::{ActorType, DbKey, DbUtilsT, NetworkId, SentinelError};

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
#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize, Getters)]
pub struct UserOp {
    #[getter(skip)]
    pub(super) uid: EthHash,
    pub(super) tx_hash: EthHash,
    pub(super) asset_amount: U256,
    pub(super) state: UserOpState,
    pub(super) block_hash: EthHash,
    pub(super) block_timestamp: u64,
    pub(super) version: UserOpVersion,
    pub(super) user_op_log: UserOpLog,
    pub(super) witnessed_timestamp: u64,
    #[serde_as(as = "DisplayFromStr")]
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
    #[cfg(test)]
    #[allow(unused)]
    pub fn to_tuple_string(&self) -> Result<String, SentinelError> {
        let ss = vec![
            format!(
                "0x{}",
                hex::encode(&self.user_op_log.origin_block_hash()?.as_bytes().to_vec())
            ),
            format!(
                "0x{}",
                hex::encode(&self.user_op_log.origin_transaction_hash()?.as_bytes().to_vec())
            ),
            format!("0x{}", hex::encode(&self.user_op_log.options_mask.as_bytes().to_vec())),
            self.user_op_log.nonce.to_string(),
            self.user_op_log.underlying_asset_decimals.to_string(),
            self.user_op_log.asset_amount.to_string(),
            self.user_op_log.user_data_protocol_fee_asset_amount.to_string(),
            self.user_op_log.network_fee_asset_amount.to_string(),
            self.user_op_log.forward_network_fee_asset_amount.to_string(),
            format!("0x{}", hex::encode(&self.user_op_log.underlying_asset_token_address)),
            format!(
                "0x{}",
                hex::encode(&self.user_op_log.origin_network_id.to_bytes_4()?.to_vec())
            ),
            format!(
                "0x{}",
                hex::encode(&self.user_op_log.destination_network_id.to_bytes_4()?.to_vec())
            ),
            format!(
                "0x{}",
                hex::encode(&self.user_op_log.forward_destination_network_id.to_bytes_4()?.to_vec())
            ),
            format!(
                "0x{}",
                hex::encode(&self.user_op_log.underlying_asset_network_id.to_bytes_4()?.to_vec())
            ),
            self.user_op_log.origin_account.to_string(),
            self.user_op_log.destination_account.to_string(),
            self.user_op_log.underlying_asset_name.to_string(),
            self.user_op_log.underlying_asset_symbol.to_string(),
            format!("0x{}", hex::encode(&self.user_op_log.user_data.clone())),
            self.user_op_log.is_for_protocol.to_string(),
        ];
        Ok("".to_string())
    }

    pub fn enqueued_block_timestamp(&self) -> Result<u64, UserOpError> {
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

        enqueued_state.block_timestamp()
    }

    fn has_been_executed(&self) -> bool {
        if self.state.is_executed() {
            true
        } else {
            self.previous_states.iter().any(|s| s.is_executed())
        }
    }

    pub fn has_not_been_executed(&self) -> bool {
        !self.has_been_executed()
    }

    pub fn to_flag(&self) -> UserOpFlag {
        self.into()
    }

    pub fn destination_network_id(&self) -> NetworkId {
        *self.user_op_log().destination_network_id()
    }

    pub fn from_log(
        witnessed_timestamp: u64,
        block_timestamp: u64,
        block_hash: EthHash,
        tx_hash: EthHash,
        origin_network_id: &NetworkId,
        log: &EthLog,
    ) -> Result<Self, UserOpError> {
        let mut user_op_log = UserOpLog::try_from(log)?;

        debug!("parsed user op log: {user_op_log:?}");

        // NOTE: A witnessed user op needs these fields from the block it was witnessed in. All
        // other states will include the full log, with these fields already included.
        user_op_log.maybe_update_fields(block_hash, tx_hash);

        let mut op = Self {
            tx_hash,
            block_hash,
            block_timestamp,
            witnessed_timestamp,
            uid: EthHash::zero(),
            previous_states: vec![],
            version: UserOpVersion::latest(),
            asset_amount: user_op_log.asset_amount,
            origin_network_id: *user_op_log.origin_network_id(),
            state: UserOpState::try_from_log(*origin_network_id, tx_hash, block_timestamp, log)?,
            user_op_log,
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

    pub fn has_been_cancelled(&self) -> bool {
        // NOTE: For a user op to have been cancelled, it needs to have had a call to cancel the
        // user op from at least two different actor types in the pnetwork protocol.
        let mut cancelled_states = vec![];
        if self.state.is_cancelled() {
            cancelled_states.push(self.state);
        };
        cancelled_states.append(
            &mut self
                .previous_states
                .iter()
                .filter(|s| s.is_cancelled())
                .cloned()
                .collect::<Vec<_>>(),
        );
        let mut n = cancelled_states.len();

        debug!("user op has {n} cancelled states {:?}", cancelled_states);
        if n < 2 {
            // NOTE In this case there can never have been two different actor types who've called for a
            // cancellation.
            return false;
        };

        let mut actor_types = cancelled_states
            .iter()
            .filter_map(|s| s.actor_type())
            .collect::<Vec<ActorType>>();
        debug!("actor types before sorting & deduplicating: {:?}", actor_types);
        actor_types.sort_unstable();
        actor_types.dedup();
        n = actor_types.len();
        debug!("{n} different actors types have called to cancel this user op");

        let has_been_cancelled = n >= 2;
        if has_been_cancelled {
            debug!("this user op is ineligible for cancellation");
        } else {
            debug!("this user op is eligible for cancellation");
        }
        has_been_cancelled
    }

    pub fn has_not_been_cancelled(&self) -> bool {
        !self.has_been_cancelled()
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
            EthAbiToken::Uint(self.user_op_log.user_data_protocol_fee_asset_amount),
            EthAbiToken::Uint(self.user_op_log.network_fee_asset_amount),
            EthAbiToken::Uint(self.user_op_log.forward_network_fee_asset_amount),
            EthAbiToken::Address(self.user_op_log.underlying_asset_token_address),
            EthAbiToken::FixedBytes(self.user_op_log.origin_network_id.to_bytes_4()?.to_vec()),
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
    use std::str::FromStr;

    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::user_ops::{
        test_utils::{
            get_sample_submission_material_with_protocol_queue_2,
            get_sample_submission_material_with_user_send,
            get_sub_mat_with_protocol_cancellation_log,
        },
        UserOpUniqueId,
        UserOps,
    };

    #[test]
    fn should_get_user_op_from_user_send() {
        let origin_network_id = NetworkId::try_from("binance").unwrap();
        let pnetwork_hub = convert_hex_to_eth_address("0x26b9EF42c92c41667A8688e61C44818Ca620986F").unwrap();
        let sub_mat = get_sample_submission_material_with_user_send();
        let ops = UserOps::from_sub_mat(&origin_network_id, &pnetwork_hub, &sub_mat).unwrap();
        assert_eq!(ops.len(), 1);
        let op = ops[0].clone();
        let bytes = hex::encode(op.abi_encode().unwrap());
        let expected_bytes = "000000000000000000000000000000000000000000000000000000000000002036302fa87ff33c74b5af176332fa2f761af281f48e7c04e4cbfb3171728913b41b245f033511dd60a5d50094c92c3d023ae81cb5d261a8824adff8429debf756000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000193310000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000174876e800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064000000000000000000000000daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af925aca268b00000000000000000000000000000000000000000000000000000000d41b1c5b00000000000000000000000000000000000000000000000000000000d41b1c5b000000000000000000000000000000000000000000000000000000005aca268b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000340000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786134313635376266323235663865633765323031306338396333663038343137323934383236346400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786134313635376266323235463845633745323031304338396333463038343137323934383236344400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e704e6574776f726b20546f6b656e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003504e5400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(bytes, expected_bytes);
        let uid = op.uid_hex().unwrap();
        let expected_uid = "0xc333ca6de1882261d3b4b00584cc6af7d140664c5d5cb2cb300342459c49bf12";
        assert_eq!(uid, expected_uid);
    }

    #[test]
    fn should_get_op_from_cancellation_log() {
        let sub_mat = get_sub_mat_with_protocol_cancellation_log();
        let origin_network_id = NetworkId::try_from("polygon").unwrap();
        let pnetwork_hub = convert_hex_to_eth_address("0xf28910cc8f21e9314eD50627c11De36bC0B7338F").unwrap();
        let ops = UserOps::from_sub_mat(&origin_network_id, &pnetwork_hub, &sub_mat).unwrap();
        assert_eq!(ops.len(), 1);
        let op = ops[0].clone();
        let uid = op.uid_hex().unwrap();
        let expected_uid = "0x50d0d882be1781e777469cd07322c84fd4652d7ee3cbd323bb3539164a3708e9";
        assert_eq!(uid, expected_uid);
        let r = op.to_tuple_string();
        assert!(r.is_ok());
    }

    #[test]
    fn should_parse_user_send_correctly() {
        let origin_network_id = NetworkId::try_from("bsc").unwrap();
        let pnetwork_hub = EthAddress::from_str("0x26b9ef42c92c41667a8688e61c44818ca620986f").unwrap();
        let sub_mat = get_sample_submission_material_with_user_send();
        let ops = UserOps::from_sub_mat(&origin_network_id, &pnetwork_hub, &sub_mat).unwrap();
        assert_eq!(ops.len(), 1);
        let expected_uid =
            UserOpUniqueId::from_str("0xc333ca6de1882261d3b4b00584cc6af7d140664c5d5cb2cb300342459c49bf12").unwrap();
        let uid = ops[0].uid().unwrap();
        assert_eq!(uid, *expected_uid);
    }

    #[test]
    fn should_get_protocol_queue_operation_from_sub_mat_correctly() {
        let sub_mat = get_sample_submission_material_with_protocol_queue_2();
        let origin_network_id = NetworkId::try_from("bsc").unwrap();
        let pnetwork_hub = EthAddress::from_str("0x26b9EF42c92c41667A8688e61C44818Ca620986F").unwrap();
        let ops = UserOps::from_sub_mat(&origin_network_id, &pnetwork_hub, &sub_mat).unwrap();
        assert_eq!(ops.len(), 1);
        let expected_uid =
            UserOpUniqueId::from_str("0x6aaa705f2df60c3da5fce7ebda54152d39b0c7395b14e595372c06a18c269b53").unwrap();
        let uid = ops[0].uid().unwrap();
        assert_eq!(uid, *expected_uid);
    }
}
