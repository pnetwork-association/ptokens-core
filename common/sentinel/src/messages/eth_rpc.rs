use common::Bytes;
use common_eth::{DefaultBlockParameter, EthSubmissionMaterial, EthTransaction};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{Challenge, ChallengeState, NetworkId, Responder, SentinelError, UserOp, UserOpSmartContractState};

#[derive(Debug)]
pub enum EthRpcMessages {
    PushTx((EthTransaction, NetworkId, Responder<EthHash>)),
    GetLatestBlockNum((NetworkId, Responder<u64>)),
    GetNonce((NetworkId, EthAddress, Responder<u64>)),
    EthCall((Bytes, NetworkId, EthAddress, DefaultBlockParameter, Responder<Bytes>)),
    GetGasPrice((NetworkId, Responder<u64>)),
    GetSubMat((NetworkId, u64, Responder<EthSubmissionMaterial>)),
    GetEthBalance((NetworkId, EthAddress, Responder<U256>)),
    GetUserOpState((NetworkId, UserOp, EthAddress, Responder<UserOpSmartContractState>)),
    GetChallengeState((NetworkId, Challenge, EthAddress, Responder<ChallengeState>)),
}

impl EthRpcMessages {
    pub fn get_user_op_state_msg(
        nid: NetworkId,
        o: UserOp,
        a: EthAddress,
    ) -> (Self, Receiver<Result<UserOpSmartContractState, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetUserOpState((nid, o, a, tx)), rx)
    }

    pub fn get_challenge_state_msg(
        nid: NetworkId,
        c: Challenge,
        a: EthAddress,
    ) -> (Self, Receiver<Result<ChallengeState, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetChallengeState((nid, c, a, tx)), rx)
    }

    pub fn get_eth_balance_msg(nid: NetworkId, a: EthAddress) -> (Self, Receiver<Result<U256, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetEthBalance((nid, a, tx)), rx)
    }

    pub fn get_sub_mat_msg(nid: NetworkId, n: u64) -> (Self, Receiver<Result<EthSubmissionMaterial, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetSubMat((nid, n, tx)), rx)
    }

    pub fn get_nonce_msg(nid: NetworkId, a: EthAddress) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetNonce((nid, a, tx)), rx)
    }

    pub fn get_latest_block_num_msg(nid: NetworkId) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetLatestBlockNum((nid, tx)), rx)
    }

    pub fn get_gas_price_msg(nid: NetworkId) -> (Self, Receiver<Result<u64, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::GetLatestBlockNum((nid, tx)), rx)
    }

    pub fn get_push_tx_msg(t: EthTransaction, nid: NetworkId) -> (Self, Receiver<Result<EthHash, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::PushTx((t, nid, tx)), rx)
    }

    pub fn get_eth_call_msg(
        d: Bytes,
        a: EthAddress,
        b: NetworkId,
        p: DefaultBlockParameter,
    ) -> (Self, Receiver<Result<Bytes, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self::EthCall((d, b, a, p, tx)), rx)
    }
}
