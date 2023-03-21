use common::{BridgeSide, CoreType};
use ethereum_types::Address as EthAddress;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{CoreState, Responder, SentinelError};

#[derive(Debug)]
pub enum EthRpcMessages {
    GetLatestBlockNum(Responder<u64>),
    GetNonce((EthAddress, Responder<u64>)),
}

impl EthRpcMessages {
    pub fn get_nonce_msg(a: EthAddress) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetNonce((a, tx)), rx)
    }

    pub fn get_latest_block_num_msg() -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetLatestBlockNum(tx), rx)
    }
}
