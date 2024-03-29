use std::{convert::TryFrom, fmt, str::FromStr};

use common::{Byte, Bytes};
use common_eth::EthSubmissionMaterial;
use common_network_ids::NetworkId;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use super::{
    UserOp,
    UserOpError,
    UserOpUniqueId,
    CANCELLED_USER_OP_TOPIC,
    ENQUEUED_USER_OP_TOPIC,
    EXECUTED_USER_OP_TOPIC,
    WITNESSED_USER_OP_TOPIC,
};
use crate::{get_utc_timestamp, SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError};

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
        origin_nid: &NetworkId,
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
                                witnessed_timestamp,
                                block_timestamp,
                                block_hash,
                                tx_hash,
                                origin_nid,
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
        match m {
            WebSocketMessagesEncodable::Success(json) => Ok(serde_json::from_value(json)?),
            other => Err(WebSocketMessagesError::CannotConvert {
                from: format!("{other}"),
                to: "UserOps".to_string(),
            }
            .into()),
        }
    }
}
