use derive_more::Constructor;
use eth_trie::Trie;
use serde::{Deserialize, Serialize};

use super::{MerkleError, MerkleTree};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct MerkleProof(Vec<Vec<u8>>);

// FIXME consider using Bytes
impl TryFrom<(&mut MerkleTree, &[u8])> for MerkleProof {
    type Error = MerkleError;

    fn try_from((merkle_tree, target_tx_receipt): (&mut MerkleTree, &[u8])) -> Result<Self, Self::Error> {
        // NOTE: Proof format contains all encoded nodes on the path to the value at key. The
        // value itself is also included in the last node.  We don't have to care about the
        // case where there's no value for the key since we've handled it above.
        // Docs here: https://github.com/carver/eth-trie.rs/blob/94ad815505c4a1dce97d6f30a052446ce3b2abfb/src/trie.rs#L34
        Ok(Self::new(merkle_tree.get_proof(target_tx_receipt)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_merkle_proof() {
        todo!("write this test");
    }
}
