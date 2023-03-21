use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct BlockAlreadyInDbError {
    pub block_num: u64,
    pub message: String,
    pub bridge_side: crate::BridgeSide,
}

impl BlockAlreadyInDbError {
    pub fn to_json(&self) -> crate::types::Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl std::fmt::Display for BlockAlreadyInDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.to_json() {
            Ok(j) => write!(f, "{j}"),
            Err(e) => write!(f, "error parsing block already in db error to json: {e}"),
        }
    }
}
