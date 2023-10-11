use common_metadata::MetadataChainId;
use ethabi::Token as EthAbiToken;
use ethereum_types::H256 as EthHash;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActorsError {
    #[error("cannot get actors from sub mat - too many logs from block {block_hash} on {chain}")]
    TooManyLogs { chain: MetadataChainId, block_hash: String },

    #[error("cannot create inclusion proof for idx {idx}, there are only {num_leaves} leaves")]
    CannotCreateInclusionProof { idx: usize, num_leaves: usize },

    #[error("ethabi error: {0}")]
    EthAbi(#[from] ethabi::Error),

    #[error("wrong topic for actors propagated event: {topic}")]
    WrongTopic { topic: EthHash },

    #[error("cannot create actors inlcusion proof from {0}")]
    CannotCreateProofFrom(String),

    #[error("actors propagated event has wrong number of topics - got {got}, expected {expected}")]
    WrongNumberOfTopics { got: usize, expected: usize },

    #[error("actors serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("found {num_actors} actor addresses but {num_types} actor types")]
    ActorAddressesAndTypesMismatch { num_actors: usize, num_types: usize },

    #[error("wrong EthAbiToken, got: {got} expected: {expected}")]
    WrongToken { got: EthAbiToken, expected: String },

    #[error("cannot get actor type from number: {0}")]
    CannotGetActorType(u64),

    #[error("cannot get actor type from string: '{0}'")]
    CannotDetermineActorType(String),

    #[error("hex error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid hash size in proof - got {got}, expected {expected} in element {element}")]
    InvalidHashSizeInProof {
        got: usize,
        expected: usize,
        element: String,
    },
}
