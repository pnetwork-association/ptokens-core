use ethabi::Token as EthAbiToken;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChallengesError {
    #[error("cannot get challenge status from: '{0}'")]
    CannotGetChallengeStatusFrom(String),

    #[error("challenges network id error: {0}")]
    NetworkId(#[from] crate::NetworkIdError),

    #[error("challenges app error: {0}")]
    AppError(#[from] common::AppError),

    #[error("challenges metadata chain id error: {0}")]
    MetadataChainId(#[from] common_metadata::MetadataChainIdError),

    #[error("challenges actors error: {0}")]
    Actors(#[from] crate::ActorsError),

    #[error("cannot parse challenge from log, there are no topics")]
    NoTopics,

    #[error("cannot parse challenge from log, wrong topic")]
    WrongTopic,

    #[error("challenges eth abi error: {0}")]
    EthAbi(#[from] ethabi::Error),

    #[error("wrong number of tokens to parse challenged - got {got}, expected {expected}")]
    IncorrectNumTokens { got: usize, expected: usize },

    #[error("wrong `EthAbiToken` when parsing challenge, got: {got} expected: {expected}")]
    WrongToken { got: EthAbiToken, expected: String },
}
