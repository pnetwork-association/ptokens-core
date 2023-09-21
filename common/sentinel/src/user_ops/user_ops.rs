use std::{convert::TryFrom, fmt, str::FromStr};

use common::{BridgeSide, Byte, Bytes};
use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::{
    UserOp,
    UserOpError,
    UserOpUniqueId,
    CANCELLED_USER_OP_TOPIC,
    ENQUEUED_USER_OP_TOPIC,
    EXECUTED_USER_OP_TOPIC,
    WITNESSED_USER_OP_TOPIC,
};
use crate::{get_utc_timestamp, SentinelError, WebSocketMessagesEncodable};

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor, Deref, Serialize, Deserialize)]
pub struct UserOps(Vec<UserOp>);

impl UserOps {
    pub fn get(&self, uid: &UserOpUniqueId) -> Result<UserOp, UserOpError> {
        #[allow(clippy::manual_try_fold)]
        self.iter().fold(Err(UserOpError::NoUserOp(*uid.clone())), |mut r, e| {
            if e.uid == **uid && r.is_err() {
                r = Ok(e.clone());
            }
            r
        })
    }

    pub fn get_tx_cost(&self, gas_limit: usize, gas_price: u64) -> U256 {
        UserOp::get_tx_cost(gas_limit, gas_price) * U256::from(self.len())
    }

    pub fn add(&mut self, other: Self) {
        let a = self.0.clone();
        let b = other.0;
        self.0 = [a, b].concat();
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_sub_mat(
        side: BridgeSide,
        origin_network_id: &[Byte],
        pnetwork_hub: &EthAddress,
        sub_mat: &EthSubmissionMaterial,
    ) -> Result<Self, SentinelError> {
        let block_hash = sub_mat.get_block_hash()?;
        let block_timestamp = sub_mat.get_timestamp().as_secs();
        let witnessed_timestamp = get_utc_timestamp()?;
        let topics = [
            *CANCELLED_USER_OP_TOPIC,
            *ENQUEUED_USER_OP_TOPIC,
            *EXECUTED_USER_OP_TOPIC,
            *WITNESSED_USER_OP_TOPIC,
        ];

        let addresses = [pnetwork_hub];

        let mut user_ops: Vec<UserOp> = vec![];

        for receipt in sub_mat.receipts.iter() {
            let tx_hash = receipt.transaction_hash;
            for log in receipt.logs.iter() {
                if addresses.contains(&&log.address) {
                    for topic in log.topics.iter() {
                        if topics.contains(topic) {
                            let op = UserOp::from_log(
                                side,
                                witnessed_timestamp,
                                block_timestamp,
                                block_hash,
                                tx_hash,
                                origin_network_id,
                                log,
                            )?;
                            user_ops.push(op);
                        };
                    }
                }
            }
        }

        Ok(Self::new(user_ops))
    }

    pub(super) fn get_enqueued_but_not_witnessed(&self) -> Self {
        UserOps::new(
            self.iter()
                .filter(|op| op.is_enqueued() && op.has_not_been_witnessed())
                .cloned()
                .collect::<Vec<UserOp>>(),
        )
    }
}

impl From<Vec<UserOps>> for UserOps {
    fn from(v: Vec<UserOps>) -> Self {
        let mut user_ops: Vec<UserOp> = vec![];
        for ops in v.into_iter() {
            for op in ops.iter() {
                user_ops.push(op.clone())
            }
        }
        Self::new(user_ops)
    }
}

impl fmt::Display for UserOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `UserOps` to string: {e}",),
        }
    }
}

impl FromStr for UserOps {
    type Err = SentinelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

impl TryInto<Bytes> for UserOps {
    type Error = SentinelError;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        Ok(serde_json::to_vec(&self)?)
    }
}

impl TryFrom<&[Byte]> for UserOps {
    type Error = SentinelError;

    fn try_from(b: &[Byte]) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(b)?)
    }
}

impl TryFrom<Bytes> for UserOps {
    type Error = SentinelError;

    fn try_from(b: Bytes) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(&b)?)
    }
}

impl TryFrom<WebSocketMessagesEncodable> for UserOps {
    type Error = SentinelError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}

/*
#[cfg(test)]
mod tests {
    use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256};

    use super::*;
    use crate::{test_utils::get_sample_sub_mat_n, user_ops::UserOpState};

    #[test]
    fn should_get_witnessed_user_operation_from_sub_mat() {
        let side = BridgeSide::Native;
        let sub_mat = get_sample_sub_mat_n(10);
        let sepolia_network_id = hex::decode("e15503e4").unwrap();
        let pnetwork_hub = convert_hex_to_eth_address("b274d81a823c1912c6884e39c2e4e669e04c83f4").unwrap();
        let expected_result = 1;
        let ops = UserOps::from_sub_mat(side, &sepolia_network_id, &pnetwork_hub, &sub_mat).unwrap();
        let result = ops.len();
        assert_eq!(result, expected_result);
        let side = BridgeSide::Native;
        let hash = convert_hex_to_h256("0xf6f24a42e1bfa9ab963786a9d2e146da7a6afad0ed188daa7a88e37bf42db789").unwrap();
        let expected_state = UserOpState::witnessed(side, hash);
        assert_eq!(ops[0].state(), expected_state);
    }

    #[test]
    fn should_get_enqueued_user_operation_from_sub_mat() {
        let side = BridgeSide::Native;
        let sub_mat = get_sample_sub_mat_n(11);
        let sepolia_network_id = hex::decode("e15503e4").unwrap();
        let pnetwork_hub = convert_hex_to_eth_address("0xBcBC92efE0a3C3ca99deBa708CEc92c785AfFB15").unwrap();
        let expected_result = 1;
        let ops = UserOps::from_sub_mat(side, &sepolia_network_id, &pnetwork_hub, &sub_mat).unwrap();
        let result = ops.len();
        assert_eq!(result, expected_result);
        let side = BridgeSide::Native;
        let hash = convert_hex_to_h256("0xc2e677e7e8c73834dc86c237f79f94ad3e4899d6aa7e561a8110a6117d13e8d5").unwrap();
        let expected_state = UserOpState::enqueued(side, hash);
        assert_eq!(ops[0].state(), expected_state);
    }
}
*/
