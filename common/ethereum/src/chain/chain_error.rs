use std::fmt;

use common_metadata::{MetadataChainId, MetadataChainIdError};
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Error, Clone, Serialize, Deserialize)]
pub enum ChainError {
    #[error("cannot reset chain, got mcid: {got}, expected mcid: {expected}")]
    CannotReset {
        got: MetadataChainId,
        expected: MetadataChainId,
    },

    #[error("block num {0} not in chain (oldest: {1}, latest {2})")]
    BlockNumNotInChain(u64, u64, u64),

    #[error("no canon block candidate found")]
    NoCanonBlockCandidates,

    #[error("too many canon block candidates found ({0})")]
    TooManyCanonBlockCandidates(usize),

    #[error("expected but failed to get block data from chain at index {0}")]
    ExpectedChainBlockDataAtIndex(usize),

    #[error("invalid receipts for block number '{0}', hash '{1}' on chain '{2}'")]
    InvalidReceipts(MetadataChainId, EthHash, u64),

    #[error("invalid block for block number '{0}', hash '{1}' on chain '{2}'")]
    InvalidBlock(MetadataChainId, EthHash, u64),

    #[error("{0}")]
    MetadataChainIdError(#[from] MetadataChainIdError),

    #[error("chain already initialized for id: {0}")]
    AlreadyInitialized(MetadataChainId),

    #[error("chain not initialized for id: {0}")]
    NotInitialized(MetadataChainId),

    #[error("expected a block when getting from the chain vecdeque")]
    ExpectedABlock,

    #[error("serde json (in chain) error: {0}")]
    SerdeJson(String),

    #[error("block {num} already in db with hash: {hash} for chain id: {mcid}")]
    BlockAlreadyInDb {
        num: u64,
        mcid: MetadataChainId,
        hash: EthHash,
    },

    #[error("failed to insert into db: {0}")]
    DbInsert(String),

    #[error("failed to delete from db: {0}")]
    DbDelete(String),

    #[error("failed to get from db: {0}")]
    DbGet(String),

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

    #[error("no chain block data in chain vecdeque @ index: {0}")]
    NoChainBlockData(u64),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Constructor, Getters)]
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

impl From<serde_json::Error> for ChainError {
    fn from(e: serde_json::Error) -> ChainError {
        ChainError::SerdeJson(format!("{e}"))
    }
}
