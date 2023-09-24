use std::fmt;

use common_metadata::MetadataChainId;
use derive_more::Constructor;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("chain not initialized for id: {0}")]
    NotInitialized(MetadataChainId),

    #[error("expected a block when getting from the chain vecdeque")]
    ExpectedABlock,

    #[error("serde json (in chain) error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("block already in db with hash: {1} for chain id: {0}")]
    BlockAlreadyInDb(MetadataChainId, EthHash),

    #[error("failed to insert into db: {0}")]
    DbInsert(String),

    #[error("failed to delete from db: {0}")]
    DbDelete(String),

    #[error("could not get bytes for chain id: {0}")]
    CouldNotGetChainIdBytes(MetadataChainId),

    #[error("failed to insert @ index {0}")]
    FailedToInsert(usize),

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
