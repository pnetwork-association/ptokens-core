use std::sync::Arc;

use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref, DerefMut};
use eth_trie::{EthTrie, MemoryDB, Trie};

use super::MerkleError;

#[derive(Debug, Constructor, Deref, DerefMut)]
pub struct MerkleTree(EthTrie<MemoryDB>);

impl TryFrom<&EthSubmissionMaterial> for MerkleTree {
    type Error = MerkleError;

    fn try_from(sub_mat: &EthSubmissionMaterial) -> Result<Self, Self::Error> {
        // NOTE: https://github.com/carver/eth-trie.rs/blob/94ad815505c4a1dce97d6f30a052446ce3b2abfb/src/db.rs#L52
        let db = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(db);

        for receipt in sub_mat.receipts.iter() {
            let (k, v) = receipt.get_rlp_encoded_index_and_rlp_encoded_receipt_tuple()?;
            trie.insert(&k, &v)?;
        }

        Ok(Self::new(trie))
    }
}
