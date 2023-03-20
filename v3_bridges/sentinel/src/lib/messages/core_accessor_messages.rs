use common::{BridgeSide, CoreType};
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{CoreState, Responder, SentinelError};

#[derive(Debug)]
pub enum CoreAccessorMessages {
    GetHostConfs(Responder<u64>),
    GetNativeConfs(Responder<u64>),
    GetHostLatestBlockNumber(Responder<u64>),
    GetNativeLatestBlockNumber(Responder<u64>),
    GetCoreState((CoreType, Responder<CoreState>)),
}

impl CoreAccessorMessages {
    pub fn get_latest_block_num_msg(side: &BridgeSide) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        if side.is_native() {
            (Self::GetNativeLatestBlockNumber(resp_tx), resp_rx)
        } else {
            (Self::GetHostLatestBlockNumber(resp_tx), resp_rx)
        }
    }

    pub fn get_core_state_msg(core_type: &CoreType) -> (Self, Receiver<Result<CoreState, SentinelError>>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        (Self::GetCoreState((*core_type, resp_tx)), resp_rx)
    }

    pub fn get_confs_msg(side: &BridgeSide) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        if side.is_native() {
            (Self::GetNativeConfs(resp_tx), resp_rx)
        } else {
            (Self::GetHostConfs(resp_tx), resp_rx)
        }
    }
}
