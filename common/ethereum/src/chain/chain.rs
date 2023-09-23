use std::collections::VecDeque;

use common::DatabaseInterface;
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};

use crate::{
    chain::chain_error::{ChainError, NoParentError},
    EthSubmissionMaterial as EthSubMat,
};

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Serialize, Deserialize)]
struct BlockData {
    hash: EthHash,
    parent_hash: EthHash,
}

impl TryFrom<&EthSubMat> for BlockData {
    type Error = ChainError;

    fn try_from(m: &EthSubMat) -> Result<Self, Self::Error> {
        Ok(Self::new(Chain::block_hash(m)?, Chain::parent_hash(m)?))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chain {
    offset: u64,
    hub: EthAddress,
    tail_length: u64,
    confirmations: u64,
    linker_hash: EthHash,
    chain: VecDeque<Vec<BlockData>>, // Bounded vecdeque?
    chain_id: MetadataChainId,
}

// TODO strip receipts (if any) after validating (if any)

#[derive(Debug, Clone, Deref, Constructor)]
struct InsertionIndex(u64);

impl From<u64> for InsertionIndex {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl Chain {
    fn block_num(m: &EthSubMat) -> Result<u64, ChainError> {
        m.get_block_number().map(|n| n.as_u64()).map_err(|e| {
            error!("{e}");
            ChainError::NoBlockNumber
        })
    }

    fn parent_hash(m: &EthSubMat) -> Result<EthHash, ChainError> {
        m.get_parent_hash().map_err(|e| {
            error!("{e}");
            ChainError::NoParentHash
        })
    }

    fn block_hash(m: &EthSubMat) -> Result<EthHash, ChainError> {
        m.get_block_hash().map_err(|e| {
            error!("{e}");
            ChainError::NoHash
        })
    }

    fn new(
        hub: EthAddress,
        tail_length: u64,
        confirmations: u64,
        sub_mat: EthSubMat,
        chain_id: MetadataChainId,
    ) -> Result<Self, ChainError> {
        let n = Self::block_num(&sub_mat)?;
        let h = Self::block_hash(&sub_mat)?;
        Ok(Self {
            hub,
            chain_id,
            offset: n,
            tail_length,
            confirmations,
            linker_hash: EthHash::zero(),
            chain: VecDeque::from([vec![BlockData::try_from(&sub_mat)?]]),
        })
    }

    fn save<D>(self, db: D) -> Result<(), ChainError> {
        todo!("save self under key which is the bytes of the metadata chain id");
        Ok(())
    }

    fn to_db_key(m: &EthSubMat) -> EthHash {
        todo!("hash block hash with chain id")
    }

    fn latest_block_num(&self) -> u64 {
        self.offset
    }

    fn chain_len(&self) -> u64 {
        self.chain.len() as u64
    }

    fn check_for_parent(&self, sub_mat: &EthSubMat) -> Result<InsertionIndex, ChainError> {
        let submat_block_num = Self::block_num(sub_mat)?;
        let oldest_block_num = self.offset - self.chain_len();
        let latest_block_num = self.offset;
        let block_data: BlockData = sub_mat.try_into()?;
        let cid = self.chain_id;
        let msg = format!(
            "cid: {}, submitted block num: {}, latest block num: {}, oldest block num: {}",
            cid, submat_block_num, latest_block_num, oldest_block_num,
        );
        debug!("{msg}");

        if submat_block_num > latest_block_num + 1 {
            debug!("no parent, block too far ahead");
            return Err(ChainError::NoParent(NoParentError::new(
                submat_block_num,
                format!("too far ahead: {msg}"),
                cid,
            )));
        }

        if submat_block_num <= oldest_block_num {
            debug!("no parent, block too far behind");
            return Err(ChainError::NoParent(NoParentError::new(
                submat_block_num,
                format!("too far behind: {msg}"),
                cid,
            )));
        }

        let parent_index = if submat_block_num == latest_block_num + 1 {
            0
        } else {
            let own_index = latest_block_num - submat_block_num;
            let parent_index = own_index + 1;
            debug!("submission material's own index: {own_index}, parent_index {parent_index}");
            parent_index
        };

        let insertion_index = if parent_index == 0 { 0 } else { parent_index + 1 };

        let parent_hash = Self::parent_hash(sub_mat)?;

        let error = ChainError::NoParent(NoParentError::new(
            submat_block_num,
            format!("no parent exists in chain for block num {submat_block_num} on chain {cid}"),
            cid,
        ));

        let potential_parents = self.chain.get(parent_index as usize).ok_or_else(|| {
            error!("{error}");
            error.clone()
        })?;

        if !potential_parents.contains(&block_data) {
            Err(error)
        } else {
            Ok(insertion_index.into())
        }
    }

    fn add<D: DatabaseInterface>(&mut self, db: &D, m: EthSubMat) {
        todo!()
        // [ ] check for parent
        // [ ] get index to insert
        // [ ] check block num is in bounds w/r/t vec index
        // [ ] if all fine:
        //  [ ] get block key and add to db via it
        //  [ ] insert hash into vec wherever it belongs
        // [ ] linker hash!
        // [ ] remove elements after tail
        // [ ] truncate the dec
        // [ ] update offset if front of queue
    }
}
