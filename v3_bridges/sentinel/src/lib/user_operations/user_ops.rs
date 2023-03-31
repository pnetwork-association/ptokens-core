use std::{convert::TryFrom, fmt, str::FromStr};

use common::{BridgeSide, Byte, Bytes};
use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError, UserOperation, USER_OPERATION_TOPIC};

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

impl fmt::Display for UserOperations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `UserOperations` to string: {e}",),
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
