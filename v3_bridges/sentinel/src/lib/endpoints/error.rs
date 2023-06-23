use common::BridgeSide;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EndpointError {
    #[error("could not get block {0}")]
    NoBlock(u64),

    #[error("could not get latest block")]
    NoLatestBlock,

    #[error("could not make rpc call: {0}")]
    Call(jsonrpsee::core::Error),

    #[error("could not push tx to endpoint: {0}")]
    PushTx(jsonrpsee::core::Error),

    #[error("ws client has disconnected whilst {0}")]
    WsClientDisconnected(String),

    #[error("endpoint timed out whilst {0}")]
    TimeOut(String),

    #[error("max number of {0} endpoint rotations of {1} reached")]
    MaxRotations(BridgeSide, usize),
}
