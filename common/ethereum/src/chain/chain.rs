#![allow(unused)] // FIXME rm once it's in and working and we know we won't need the unused fxns
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Constructor, Serialize, Deserialize, Deref)]
pub struct BlockDatas(Vec<BlockData>);

impl BlockDatas {
    fn get_parent_hashes(&self) -> Vec<EthHash> {
        self.iter().map(|d| d.parent_hash()).cloned().collect()
    }

    fn empty() -> Self {
        Self::new(vec![])
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Constructor, Serialize, Deserialize, Getters)]
pub struct BlockData {
    number: u64,
    hash: EthHash,
    parent_hash: EthHash,
}

impl TryFrom<&EthSubMat> for BlockData {
    type Error = ChainError;

    fn try_from(m: &EthSubMat) -> Result<Self, Self::Error> {
        Ok(Self::new(
            Chain::block_num(m)?,
            Chain::block_hash(m)?,
            Chain::parent_hash(m)?,
        ))
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
    chain_id: MetadataChainId,
    chain: VecDeque<Vec<BlockData>>, // TODO use the `BlockDatas` struct above
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
    pub fn get_latest_block_data(&self) -> Option<&Vec<BlockData>> {
        self.chain.get(0)
    }

    pub fn get_tail_block_data(&self) -> Option<&Vec<BlockData>> {
        self.chain.back()
    }

    pub fn get_canon_block_data(&self) -> Option<&Vec<BlockData>> {
        self.chain.get(*self.confirmations() as usize - 1)
    }

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

    // TODO factor out the pruning and teh saving of the block since it's used in the insert fxn
    // too.
    pub fn init<D: DatabaseInterface>(
        db_utils: &ChainDbUtils<D>,
        hub: EthAddress,
        tail_length: u64,
        confirmations: u64,
        sub_mat: EthSubMat,
        mcid: MetadataChainId,
        validate: bool,
    ) -> Result<(), ChainError> {
        debug!("initializing chain for mcid: {mcid}");

        // NOTE: First lets see if this chain has already been initialized
        if Self::get(db_utils, mcid).is_ok() {
            return Err(ChainError::AlreadyInitialized(mcid));
        };

        // NOTE: Now lets validate the block & receipts if we're required to
        Self::validate(&mcid, &sub_mat, validate)?;

        // NOTE: Now we can create the chain structure
        let c = Self::new(hub, tail_length, confirmations, sub_mat.clone(), mcid)?;

        // NOTE: Now we can prune the sub mat's receipts...
        let pruned_sub_mat = sub_mat.remove_receipts_if_no_logs_from_addresses(&[hub]);

        // NOTE: Now we save the block itself in the db...
        let sub_mat_bytes = serde_json::to_vec(&pruned_sub_mat)?;
        let db_key = c.sub_mat_to_db_key(&pruned_sub_mat)?;
        db_utils
            .db()
            .put(db_key.to_vec(), sub_mat_bytes, MIN_DATA_SENSITIVITY_LEVEL)
            .map_err(|e| {
                error!("{e}");
                ChainError::DbInsert(format!("{e}"))
            })?;

        // NOTE: And finally, save it in the db
        c.save_in_db(db_utils)
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
        let parent_hash = Self::parent_hash(sub_mat)?;
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

        if !potential_parents.iter().any(|bd| bd.hash() == &parent_hash) {
            Err(ChainError::NoParent(no_parent_error))
        } else {
            Ok(parent_index)
        }
    }

    fn validate(mcid: &MetadataChainId, sub_mat: &EthSubMat, validate: bool) -> Result<(), ChainError> {
        if validate {
            let n = Self::block_num(sub_mat)?;
            let h = Self::block_hash(sub_mat)?;
            let cid = mcid.to_eth_chain_id()?;
            debug!("validating block {n} & receipts for chain id {cid}...");
            if let Err(e) = sub_mat.block_is_valid(&mcid.to_eth_chain_id()?) {
                error!("invalid block: {e}");
                return Err(ChainError::InvalidBlock(*mcid, h, n));
            }

            // NOTE Receipts may have bene prefiltered outside of the TEE
            if !sub_mat.receipts.is_empty() {
                if let Err(e) = sub_mat.receipts_are_valid() {
                    error!("invalid receipts: {e}");
                    return Err(ChainError::InvalidReceipts(*mcid, h, n));
                }
            }
            Ok(())
        } else {
            warn!("not validating sub mat for chain ID: {mcid}");
            Ok(())
        }
    }

    fn get_block<D: DatabaseInterface>(
        &self,
        db_utils: &ChainDbUtils<D>,
        requested: u64,
    ) -> Result<Vec<EthSubMat>, ChainError> {
        let oldest = self.offset() - self.chain_len();
        let latest = *self.offset();

        if requested > latest || requested < oldest {
            return Err(ChainError::BlockNumNotInChain(requested, oldest, latest));
        }

        let idx = (latest - requested) as usize;
        let block_data = self
            .chain
            .get(idx)
            .ok_or_else(|| ChainError::ExpectedBlockDataAtIndex(idx))?;
        let db_keys = block_data
            .iter()
            .map(|d| DbKey::from(self.chain_id(), *d.hash()))
            .collect::<Result<Vec<DbKey>, ChainError>>()?;

        let mut r: Vec<EthSubMat> = vec![];
        for key in db_keys {
            let bytes = db_utils
                .db()
                .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
                .map_err(|e| {
                    error!("{e}");
                    ChainError::DbGet(format!("{e}"))
                })?;
            let sub_mat = serde_json::from_slice::<EthSubMat>(&bytes)?;
            r.push(sub_mat);
        }
        Ok(r)
    }

    pub fn insert<D: DatabaseInterface>(
        &mut self,
        db_utils: &ChainDbUtils<D>,
        sub_mat: EthSubMat,
        validate: bool,
    ) -> Result<(), ChainError> {
        let parent_index = self.check_for_parent(&sub_mat)?;

        let mcid = *self.chain_id();
        // NOTE: First lets validate the sub mat if we're required to
        Self::validate(&mcid, &sub_mat, validate)?;

        let block_data = BlockData::try_from(&sub_mat)?;

        // NOTE: Next we update our chain data...
        if parent_index.is_zero() {
            // NOTE: Block can't already exist in db!
            self.chain.push_front(vec![block_data]);
            self.offset += 1;
            Ok(())
        } else {
            let insertion_index = parent_index.as_usize() - 1;
            match self.chain.get_mut(insertion_index) {
                None => Err(ChainError::FailedToInsert(insertion_index)),
                Some(existing_block_data) => {
                    if existing_block_data.contains(&block_data) {
                        Err(ChainError::BlockAlreadyInDb(mcid, *block_data.hash()))
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
                let key = DbKey::from(&mcid, *data.hash())?;
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
        // NOTE: We store the chain under a hash of it's chain ID as bytes.
        mcid.to_bytes()
            .map(|bs| DbKey(keccak_hash_bytes(&bs[..])))
            .map_err(|e| {
                error!("{e}");
                ChainError::CouldNotGetChainIdBytes(*mcid)
            })
    }

    fn save_in_db<D: DatabaseInterface>(self, db_utils: &ChainDbUtils<D>) -> Result<(), ChainError> {
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

    pub fn get<D: DatabaseInterface>(db_utils: &ChainDbUtils<D>, mcid: MetadataChainId) -> Result<Self, ChainError> {
        let key = Self::db_key_from_chain_id(&mcid)?;
        db_utils
            .db()
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bs| Ok(serde_json::from_slice(&bs)?))
            .map_err(|e| {
                warn!("error getting chain for chain id '{mcid}': {e}");
                ChainError::NotInitialized(mcid)
            })
    }

    fn get_canonical_block_hash(&self) -> Result<Option<EthHash>, ChainError> {
        debug!("getting canonical block data...");

        let length = self.chain.len();
        let confs = *self.confirmations() as usize;

        if length < confs {
            warn!("chain is to short to have a canon block - length: {length}, confirmations: {confs}");
            return Ok(None);
        };

        let mut hashes: Vec<EthHash> = vec![];
        for i in 0..confs {
            let mut data = self
                .chain
                .get(i)
                .ok_or_else(|| ChainError::ExpectedBlockDataAtIndex(i))?
                .to_vec();

            if i > 0 {
                // NOTE We only filter the data by the existing parents _after_ the first iteration,
                // because on the first step there will never be any
                data = data
                    .iter()
                    .filter(|d| hashes.contains(d.hash()))
                    .cloned()
                    .collect::<Vec<BlockData>>();
            }
            if i < confs - 1 {
                // NOTE: IE, any except the _last_ iteration. Here we get all the parent hashes,
                // then sort and deduplicate that list.
                hashes = data.iter().map(|d| d.parent_hash()).cloned().collect();
                hashes.sort_unstable();
                hashes.dedup();
            } else {
                // NOTE: The last iteration. So here instead we get the list of _block_ hashes, not
                // parent hashes. These are our candidates for the canon block. Hopefully at this
                // point there is only one!
                hashes = data.iter().map(|d| d.hash()).cloned().collect();
            }
        }

        if hashes.is_empty() {
            // NOTE We've already checked if the chain is too short, so this is a legit error
            Err(ChainError::NoCanonBlockCandidates)
        } else if hashes.len() > 1 {
            // NOTE: This _can_ happen, but if so it means we've encountered a fork longer than our
            // set confirmations which is a problem needed external help (IE a discussion on what
            // to increase the number of confirmations to, or a discussion as to which fork to see
            // as canonical etc)
            Err(ChainError::TooManyCanonBlockCandidates(hashes.len()))
        } else {
            Ok(Some(hashes[0]))
        }
    }

    pub fn get_canonical_sub_mat<D: DatabaseInterface>(
        &self,
        db_utils: &ChainDbUtils<D>,
    ) -> Result<Option<EthSubMat>, ChainError> {
        if let Ok(Some(hash)) = self.get_canonical_block_hash() {
            let key = DbKey::from(self.chain_id(), hash)?;
            let sub_mat = db_utils
                .db()
                .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
                .and_then(|bytes| Ok(serde_json::from_slice::<EthSubMat>(&bytes)?))
                .map_err(|e| {
                    error!("{e}");
                    ChainError::DbGet(format!("{e}"))
                })?;
            Ok(Some(sub_mat))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{chain::ChainState, test_utils::get_sequential_eth_blocks_and_receipts};

    #[test]
    fn should_init_core_correctly() {
        let sub_mat = get_sequential_eth_blocks_and_receipts()[0].clone();
        let mcid = MetadataChainId::EthereumMainnet;
        let validate = true;
        let hub = EthAddress::zero();
        let db = get_test_database();
        let db_utils = ChainDbUtils::new(&db);
        let confirmations = 3;
        let tail_length = 2;

        Chain::init(&db_utils, hub, tail_length, confirmations, sub_mat, mcid, validate).unwrap();
    }

    #[test]
    fn should_get_not_initted_error() {
        let db = get_test_database();
        let db_utils = ChainDbUtils::new(&db);
        let mcid = MetadataChainId::EthereumMainnet;
        match Chain::get(&db_utils, mcid) {
            Ok(_) => panic!("should not have succeeded"),
            Err(ChainError::NotInitialized(id)) => assert_eq!(id, mcid),
            Err(e) => panic!("wrong error received {e}"),
        }
    }

    #[test]
    fn should_get_already_initted_error() {
        let sub_mat = get_sequential_eth_blocks_and_receipts()[0].clone();
        let mcid = MetadataChainId::EthereumMainnet;
        let validate = true;
        let hub = EthAddress::zero();
        let db = get_test_database();
        let db_utils = ChainDbUtils::new(&db);
        let confirmations = 3;
        let tail_length = 2;

        Chain::init(
            &db_utils,
            hub,
            tail_length,
            confirmations,
            sub_mat.clone(),
            mcid,
            validate,
        )
        .unwrap();

        match Chain::init(&db_utils, hub, tail_length, confirmations, sub_mat, mcid, validate) {
            Ok(_) => panic!("should not have succeeded"),
            Err(ChainError::AlreadyInitialized(id)) => assert_eq!(id, mcid),
            Err(e) => panic!("wrong error received {e}"),
        }
    }

    #[test]
    fn should_manage_chain_correctly() {
        use simple_logger; // FIXME rm
        simple_logger::init_with_level(log::Level::Debug).unwrap();

        let sub_mats = get_sequential_eth_blocks_and_receipts();
        let mcid = MetadataChainId::EthereumMainnet;
        let validate = true;
        let sub_mat = sub_mats[0].clone();
        let hub = EthAddress::zero();
        let db = get_test_database();
        let db_utils = ChainDbUtils::new(&db);
        let confirmations = 3;
        let tail_length = 2;

        Chain::init(&db_utils, hub, tail_length, confirmations, sub_mat, mcid, validate).unwrap();

        let mut chain = Chain::get(&db_utils, mcid).unwrap();
        assert_eq!(chain.chain_len(), 1);
        assert!(matches!(chain.get_canonical_sub_mat(&db_utils), Ok(None)));
        assert_eq!(
            chain.get_tail_block_data().unwrap()[0].number,
            sub_mats[0].get_block_number().unwrap().as_u64()
        );

        chain.insert(&db_utils, sub_mats[1].clone(), validate).unwrap();
        assert_eq!(chain.chain_len(), 2);
        assert!(matches!(chain.get_canonical_sub_mat(&db_utils), Ok(None)));
        assert_eq!(
            chain.get_tail_block_data().unwrap()[0].number,
            sub_mats[0].get_block_number().unwrap().as_u64()
        );

        chain.insert(&db_utils, sub_mats[2].clone(), validate).unwrap();
        assert_eq!(chain.chain_len(), 3);
        assert_eq!(
            chain.get_tail_block_data().unwrap()[0].number,
            sub_mats[0].get_block_number().unwrap().as_u64()
        );
        assert_eq!(
            chain
                .get_canonical_sub_mat(&db_utils)
                .unwrap()
                .unwrap()
                .get_block_number()
                .unwrap(),
            sub_mats[0].get_block_number().unwrap()
        );

        chain.insert(&db_utils, sub_mats[3].clone(), validate).unwrap();
        assert_eq!(chain.chain_len(), 4);
        assert_eq!(
            chain.get_tail_block_data().unwrap()[0].number,
            sub_mats[0].get_block_number().unwrap().as_u64()
        );
        assert_eq!(
            chain
                .get_canonical_sub_mat(&db_utils)
                .unwrap()
                .unwrap()
                .get_block_number()
                .unwrap(),
            sub_mats[1].get_block_number().unwrap()
        );

        chain.insert(&db_utils, sub_mats[4].clone(), validate).unwrap();
        assert_eq!(chain.chain_len(), 5);
        assert_eq!(
            chain.get_tail_block_data().unwrap()[0].number,
            sub_mats[0].get_block_number().unwrap().as_u64()
        );
        assert_eq!(
            chain
                .get_canonical_sub_mat(&db_utils)
                .unwrap()
                .unwrap()
                .get_block_number()
                .unwrap(),
            sub_mats[2].get_block_number().unwrap()
        );

        chain.insert(&db_utils, sub_mats[5].clone(), validate).unwrap();
        assert_eq!(chain.chain_len(), 5);
        assert_eq!(
            chain.get_tail_block_data().unwrap()[0].number,
            sub_mats[1].get_block_number().unwrap().as_u64()
        );
        assert_eq!(
            chain
                .get_canonical_sub_mat(&db_utils)
                .unwrap()
                .unwrap()
                .get_block_number()
                .unwrap(),
            sub_mats[3].get_block_number().unwrap()
        );

        chain.insert(&db_utils, sub_mats[6].clone(), validate).unwrap();
        assert_eq!(chain.chain_len(), 5);
        assert_eq!(
            chain.get_tail_block_data().unwrap()[0].number,
            sub_mats[2].get_block_number().unwrap().as_u64()
        );
        assert_eq!(
            chain
                .get_canonical_sub_mat(&db_utils)
                .unwrap()
                .unwrap()
                .get_block_number()
                .unwrap(),
            sub_mats[4].get_block_number().unwrap()
        );

        // NOTE: Test submitting a block that's too far ahead
        match chain.insert(&db_utils, sub_mats[10].clone(), validate) {
            Ok(_) => panic!("should not have succeeded"),
            Err(ChainError::NoParent(e)) => {
                assert_eq!(e.cid(), &mcid);
                assert_eq!(e.block_num(), &8065760);
                assert!(e.message().contains("too far ahead"));
            },
            Err(e) => panic!("wrong error received {e}"),
        };

        // NOTE: Test submitting a block that's too far behind
        match chain.insert(&db_utils, sub_mats[0].clone(), validate) {
            Ok(_) => panic!("should not have succeeded"),
            Err(ChainError::NoParent(e)) => {
                assert_eq!(e.cid(), &mcid);
                assert_eq!(e.block_num(), &8065750);
                assert!(e.message().contains("too far behind"));
            },
            Err(e) => panic!("wrong error received {e}"),
        };

        // NOTE: Test submitting a block that's already extant
        match chain.insert(&db_utils, sub_mats[4].clone(), validate) {
            Ok(_) => panic!("should not have succeeded"),
            Err(ChainError::BlockAlreadyInDb(id, hash)) => {
                let expected_hash = Chain::block_hash(&sub_mats[4]).unwrap();
                assert_eq!(id, mcid);
                assert_eq!(hash, expected_hash);
            },
            Err(e) => panic!("wrong error received {e}"),
        };

        // NOTE: assert that all the blocks in the chain exist in the db, and they they've been
        // pruned of receipts (Note we clone the chain, so we don't drain our _actual_ chain!)
        let block_datas = chain.chain.clone().drain(0..).flatten().collect::<Vec<BlockData>>();
        let block_nums = block_datas.iter().map(|d| *d.number()).collect::<Vec<u64>>();
        for block_num in block_nums {
            let sub_mats = chain.get_block(&db_utils, block_num).unwrap();
            assert_eq!(sub_mats.len(), 1); // NOTE There should be no forks here.
            assert!(sub_mats[0].get_receipts().is_empty());
        }

        // NOTE: Now assert that blocks that re no longer in our chain were deleted.
        let expected_deleted_block_hashes = vec![
            Chain::block_hash(&sub_mats[0]).unwrap(),
            Chain::block_hash(&sub_mats[1]).unwrap(),
        ];
        let expected_deleted_db_keys = expected_deleted_block_hashes
            .iter()
            .map(|h| DbKey::from(&mcid, *h).unwrap())
            .collect::<Vec<DbKey>>();
        for key in expected_deleted_db_keys {
            match db.get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
                Ok(_) => panic!("should not have succeeded"),
                Err(common::AppError::Custom(e)) => assert_eq!(&e, "Cannot find item in database!"),
                Err(e) => panic!("wrong error received: {e}"),
            }
        }
    }
}
