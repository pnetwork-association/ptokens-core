use std::fmt;

use common_metadata::MetadataChainId;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ChainError {
    #[error("no block number in eth submission material")]
    NoBlockNumber,

    #[error("no parent hash in eth submission material")]
    NoParentHash,

    #[error("no hash in eth submission materiall")]
    NoHash,

    #[error("{0}")]
    NoParent(NoParentError),

    #[error("no block data in chain vecdeque @ index: {0}")]
    NoBlockData(u64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Constructor)]
pub struct NoParentError {
    block_num: u64,
    message: String,
    cid: MetadataChainId,
}

impl fmt::Display for NoParentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", json!(self))
    }
}
