use common::BridgeSide;
use common_eth::EthTransaction;
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{Responder, SentinelError};

#[derive(Debug)]
pub enum EthRpcMessages {
    GetNonce((EthAddress, Responder<u64>)),
    PushTx((EthTransaction, Responder<EthHash>)),
    GetLatestBlockNum((BridgeSide, Responder<u64>)),
}

impl EthRpcMessages {
    pub fn get_nonce_msg(a: EthAddress) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetNonce((a, tx)), rx)
    }

    pub fn get_latest_block_num_msg(side: BridgeSide) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetLatestBlockNum((side, tx)), rx)
    }

    pub fn get_push_txs_msg(t: EthTransaction) -> (Self, Receiver<Result<EthHash, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::PushTx((t, tx)), rx)
    }
}
