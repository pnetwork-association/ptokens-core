use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignedEventError {
    #[error("merkle proof error: {0}")]
    MerkleProof(#[from] crate::MerkleError),

    #[error("common error: {0}")]
    Common(#[from] common::CommonError),

    #[error("network id error: {0}")]
    NetworkId(#[from] common_network_ids::NetworkIdError),
}
