use common::{BridgeSide, Bytes};
use common_eth::{DefaultBlockParameter, EthSubmissionMaterial, EthTransaction};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{Responder, SentinelError, UserOp, UserOpSmartContractState};

#[derive(Debug)]
pub enum EthRpcMessages {
    PushTx((EthTransaction, BridgeSide, Responder<EthHash>)),
    GetLatestBlockNum((BridgeSide, Responder<u64>)),
    GetNonce((BridgeSide, EthAddress, Responder<u64>)),
    EthCall((Bytes, BridgeSide, EthAddress, DefaultBlockParameter, Responder<Bytes>)),
    GetGasPrice((BridgeSide, Responder<u64>)),
    GetSubMat((BridgeSide, u64, Responder<EthSubmissionMaterial>)),
    GetEthBalance((BridgeSide, EthAddress, Responder<U256>)),
    GetUserOpState((BridgeSide, UserOp, EthAddress, Responder<UserOpSmartContractState>)),
}

impl EthRpcMessages {
    pub fn get_user_op_state_msg(
        s: BridgeSide,
        o: UserOp,
        a: EthAddress,
    ) -> (Self, Receiver<Result<UserOpSmartContractState, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetUserOpState((s, o, a, tx)), rx)
    }

    pub fn get_eth_balance_msg(s: BridgeSide, a: EthAddress) -> (Self, Receiver<Result<U256, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetEthBalance((s, a, tx)), rx)
    }

    pub fn get_sub_mat_msg(s: BridgeSide, n: u64) -> (Self, Receiver<Result<EthSubmissionMaterial, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetSubMat((s, n, tx)), rx)
    }

    pub fn get_nonce_msg(s: BridgeSide, a: EthAddress) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetNonce((s, a, tx)), rx)
    }

    pub fn get_latest_block_num_msg(side: BridgeSide) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetLatestBlockNum((side, tx)), rx)
    }

    pub fn get_gas_price_msg(side: BridgeSide) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetLatestBlockNum((side, tx)), rx)
    }

    pub fn get_push_tx_msg(t: EthTransaction, s: BridgeSide) -> (Self, Receiver<Result<EthHash, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::PushTx((t, s, tx)), rx)
    }

    pub fn get_eth_call_msg(
        d: Bytes,
        a: EthAddress,
        b: BridgeSide,
        p: DefaultBlockParameter,
    ) -> (Self, Receiver<Result<Bytes, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::EthCall((d, b, a, p, tx)), rx)
    }
}
