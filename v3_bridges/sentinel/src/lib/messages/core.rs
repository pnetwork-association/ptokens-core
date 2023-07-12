use common::{BridgeSide, CoreType};
use common_eth::EthTransaction;
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde_json::Value as Json;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{CoreState, Responder, SentinelError, UserOp, UserOpList, UserOps};

#[derive(Debug)]
pub enum CoreMessages {
    GetHostConfs(Responder<u64>),
    GetNativeConfs(Responder<u64>),
    GetUserOps(Responder<UserOps>),
    GetGasPrices(Responder<(u64, u64)>),
    GetUserOpList(Responder<UserOpList>),
    GetHostLatestBlockNumber(Responder<u64>),
    GetCancellableUserOps(Responder<UserOps>),
    GetNativeLatestBlockNumber(Responder<u64>),
    GetLatestBlockNumbers(Responder<(u64, u64)>),
    GetCoreState((CoreType, Responder<CoreState>)),
    RemoveUserOp {
        uid: EthHash,
        responder: Responder<Json>,
    },
    GetAddress {
        side: BridgeSide,
        responder: Responder<EthAddress>,
    },
    GetCancellationTx {
        nonce: u64,
        gas_price: u64,
        gas_limit: usize,
        op: Box<UserOp>,
        state_manager: EthAddress,
        responder: Responder<EthTransaction>,
    },
}

impl CoreMessages {
    pub fn get_cancellable_user_ops_msg() -> (Self, Receiver<Result<UserOps, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetCancellableUserOps(tx), rx)
    }

    pub fn get_remove_user_op_msg(uid: EthHash) -> (Self, Receiver<Result<Json, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::RemoveUserOp { uid, responder: tx }, rx)
    }

    #[allow(clippy::type_complexity)]
    pub fn get_gas_prices_msg() -> (Self, Receiver<Result<(u64, u64), SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetGasPrices(tx), rx)
    }

    pub fn get_user_ops_list_msg() -> (Self, Receiver<Result<UserOpList, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetUserOpList(tx), rx)
    }

    pub fn get_user_ops_msg() -> (Self, Receiver<Result<UserOps, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetUserOps(tx), rx)
    }

    pub fn get_latest_block_num_msg(side: &BridgeSide) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        if side.is_native() {
            (Self::GetNativeLatestBlockNumber(tx), rx)
        } else {
            (Self::GetHostLatestBlockNumber(tx), rx)
        }
    }

    pub fn get_core_state_msg(core_type: &CoreType) -> (Self, Receiver<Result<CoreState, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetCoreState((*core_type, tx)), rx)
    }

    pub fn get_confs_msg(side: &BridgeSide) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        if side.is_native() {
            (Self::GetNativeConfs(tx), rx)
        } else {
            (Self::GetHostConfs(tx), rx)
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn get_latest_block_numbers_msg() -> (Self, Receiver<Result<(u64, u64), SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetLatestBlockNumbers(tx), rx)
    }

    pub fn get_cancellation_signature_msg(
        op: UserOp,
        nonce: u64,
        gas_price: u64,
        gas_limit: usize,
        state_manager: EthAddress,
    ) -> (Self, Receiver<Result<EthTransaction, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (
            Self::GetCancellationTx {
                nonce,
                gas_price,
                gas_limit,
                state_manager,
                responder: tx,
                op: Box::new(op),
            },
            rx,
        )
    }

    pub fn get_address_msg(side: BridgeSide) -> (Self, Receiver<Result<EthAddress, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetAddress { responder: tx, side }, rx)
    }
}
