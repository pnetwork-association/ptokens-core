use std::collections::VecDeque;

use common::{crypto_utils::keccak_hash_bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};

use crate::{
    chain::{ChainDbUtils, ChainError, NoParentError},
    EthSubmissionMaterial as EthSubMat,
};

// TODO init (call new and save it in the db)
// TODO get canon block receipts
// TODO take flag for validation on insert fxn. Only validate if flag is true

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Serialize, Deserialize, Getters)]
pub struct BlockData {
    hash: EthHash,
    parent_hash: EthHash,
}

impl TryFrom<&EthSubMat> for BlockData {
    type Error = ChainError;

    fn try_from(m: &EthSubMat) -> Result<Self, Self::Error> {
        Ok(Self::new(Chain::block_hash(m)?, Chain::parent_hash(m)?))
    }
}

#[derive(Clone, Debug, Constructor, Deref)]
struct DbKey(EthHash);

impl DbKey {
    fn to_vec(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    fn from(mcid: &MetadataChainId, hash: EthHash) -> Result<Self, ChainError> {
        // NOTE: We hash the block hash with the chain ID to get a unique key for the db.
        let mcid_bytes = mcid.to_bytes().map_err(|e| {
            error!("{e}");
            ChainError::CouldNotGetChainIdBytes(*mcid)
        })?;
        let hash_bytes = DbKey(hash).to_vec();
        Ok(Self(keccak_hash_bytes(&[mcid_bytes, hash_bytes].concat())))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct Chain {
    offset: u64,
    hub: EthAddress,
    tail_length: u64,
    confirmations: u64,
    linker_hash: EthHash,
    chain: VecDeque<Vec<BlockData>>, // Bounded vecdeque?
    chain_id: MetadataChainId,
}

#[derive(Debug, Clone, Deref, Constructor)]
struct ParentIndex(u64);

impl From<u64> for ParentIndex {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl ParentIndex {
    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn as_usize(&self) -> usize {
        self.0 as usize
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

    fn latest_block_num(&self) -> u64 {
        self.offset
    }

    fn chain_len(&self) -> u64 {
        self.chain.len() as u64
    }

    fn check_for_parent(&self, sub_mat: &EthSubMat) -> Result<ParentIndex, ChainError> {
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

        let parent_index: ParentIndex = if submat_block_num == latest_block_num + 1 {
            0.into()
        } else {
            let own_index = latest_block_num - submat_block_num;
            let parent_index = own_index + 1;
            debug!("submission material's own index: {own_index}, parent_index {parent_index}");
            parent_index.into()
        };

        let no_parent_error = NoParentError::new(
            submat_block_num,
            format!("no parent exists in chain for block num {submat_block_num} on chain {cid}"),
            cid,
        );

        let potential_parents = self.chain.get(parent_index.as_usize()).ok_or_else(|| {
            error!("{no_parent_error}");
            ChainError::NoParent(no_parent_error.clone())
        })?;

        if !potential_parents.contains(&block_data) {
            Err(ChainError::NoParent(no_parent_error))
        } else {
            Ok(parent_index)
        }
    }

    fn insert<D: DatabaseInterface>(
        &mut self,
        db_utils: &ChainDbUtils<D>,
        parent_index: ParentIndex,
        sub_mat: EthSubMat,
    ) -> Result<(), ChainError> {
        let block_data = BlockData::try_from(&sub_mat)?;

        // NOTE: First we update our chain data...
        if parent_index.is_zero() {
            // NOTE: Block can't already exist in db!
            self.chain.push_front(vec![block_data]);
            Ok(())
        } else {
            let insertion_index = parent_index.as_usize() - 1;
            match self.chain.get_mut(insertion_index) {
                None => Err(ChainError::FailedToInsert(insertion_index)),
                Some(existing_block_data) => {
                    if existing_block_data.contains(&block_data) {
                        Err(ChainError::BlockAlreadyInDb(self.chain_id, *block_data.hash()))
                    } else {
                        existing_block_data.push(block_data);
                        Ok(())
                    }
                },
            }
        }?;

        // NOTE: Now we prune receipts we don't care about
        let pruned_sub_mat = sub_mat.remove_receipts_if_no_logs_from_addresses(&[self.hub]);
        let sub_mat_bytes = serde_json::to_vec(&pruned_sub_mat)?;

        // NOTE: Now we save the block itself in the db...
        let db_key = self.sub_mat_to_db_key(&pruned_sub_mat)?;
        db_utils
            .db()
            .put(db_key.to_vec(), sub_mat_bytes, MIN_DATA_SENSITIVITY_LEVEL)
            .map_err(|e| {
                error!("{e}");
                ChainError::DbInsert(format!("{e}"))
            })?;

        // NOTE: Now we prune any excess off the end of the chain that we don't need any more.
        let total_allowable_length = self.confirmations + self.tail_length;
        if self.chain_len() > total_allowable_length {
            let excess_length = self.chain_len() - total_allowable_length;
            let mut block_data_to_delete: Vec<Vec<BlockData>> = vec![];
            for _ in 0..excess_length {
                let data = self.chain.pop_back().ok_or_else(|| ChainError::ExpectedABlock)?;
                block_data_to_delete.push(data);
            }
            // NOTE: Now we must remove those saved blocks from the db
            block_data_to_delete.iter().flatten().try_for_each(|data| {
                let key = DbKey::from(&self.chain_id, *data.hash())?;
                db_utils.db().delete(key.to_vec()).map_err(|e| {
                    error!("{e}");
                    ChainError::DbDelete(format!("{e}"))
                })?;
                Ok::<(), ChainError>(())
            })?;
        };

        // TODO update the linker hash <- is this meaningless though? What if there are forked blocks there?

        Ok(())
    }

    fn sub_mat_to_db_key(&self, sub_mat: &EthSubMat) -> Result<DbKey, ChainError> {
        let block_num = Self::block_num(sub_mat)?;
        let block_hash = Self::block_hash(sub_mat)?;
        let db_key = DbKey::from(&self.chain_id, block_hash)?;
        debug!("db key for block num: {block_num}: 0x{}", hex::encode(*db_key));
        Ok(db_key)
    }

    fn db_key(&self) -> Result<DbKey, ChainError> {
        Self::db_key_from_chain_id(self.chain_id())
    }

    fn db_key_from_chain_id(mcid: &MetadataChainId) -> Result<DbKey, ChainError> {
        mcid.to_bytes()
            .map(|bs| DbKey(EthHash::from_slice(&bs[..])))
            .map_err(|e| {
                error!("{e}");
                ChainError::CouldNotGetChainIdBytes(*mcid)
            })
    }

    fn save<D: DatabaseInterface>(self, db_utils: ChainDbUtils<D>) -> Result<(), ChainError> {
        let key = self.db_key()?;
        let value = serde_json::to_vec(&self)?;
        db_utils
            .db()
            .put(key.to_vec(), value, MIN_DATA_SENSITIVITY_LEVEL)
            .map_err(|e| {
                error!("{e}");
                ChainError::DbInsert(format!("{e}"))
            })
    }

    fn get<D: DatabaseInterface>(db_utils: &ChainDbUtils<D>, mcid: MetadataChainId) -> Result<Self, ChainError> {
        let key = Self::db_key_from_chain_id(&mcid)?;
        db_utils
            .db()
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bs| Ok(serde_json::from_slice(&bs)?))
            .map_err(|e| {
                error!("error getting chain for chain id '{mcid}': {e}");
                ChainError::NotInitialized(mcid)
            })
    }

    fn get_canonical_sub_mat() -> Result<Option<EthSubMat>, ChainError> {
        todo!("walk back the chain confs number of times to get this");
    }
}
