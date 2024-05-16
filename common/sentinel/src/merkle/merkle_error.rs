use thiserror::Error;

#[derive(Error, Debug)]
pub enum MerkleError {
    #[error("cannot make proof, target key is not in trie")]
    NoKeyToProve,

    #[error("trie error: {0}")]
    Trie(#[from] eth_trie::TrieError),

    #[error("common error: {0}")]
    Common(#[from] common::CommonError),
}
