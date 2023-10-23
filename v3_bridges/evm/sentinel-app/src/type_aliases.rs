use common_sentinel::{
    BroadcastChannelMessages,
    ChallengeResponderMessages,
    EthRpcMessages,
    StatusPublisherMessages,
    UserOpCancellerMessages,
    WebSocketMessages,
};
use tokio::sync::{
    broadcast::{Receiver as MpmcRx, Sender as MpmcTx},
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
};

pub(crate) type CoreCxnStatus = bool;
pub(crate) type EthRpcTx = MpscTx<EthRpcMessages>;
pub(crate) type WebSocketRx = MpscRx<WebSocketMessages>;
pub(crate) type WebSocketTx = MpscTx<WebSocketMessages>;
pub(crate) type UserOpCancellerTx = MpscTx<UserOpCancellerMessages>;
pub(crate) type UserOpCancellerRx = MpscRx<UserOpCancellerMessages>;
pub(crate) type StatusPublisherTx = MpscTx<StatusPublisherMessages>;
pub(crate) type StatusPublisherRx = MpscRx<StatusPublisherMessages>;
pub(crate) type BroadcastChannelTx = MpmcTx<BroadcastChannelMessages>;
pub(crate) type BroadcastChannelRx = MpmcRx<BroadcastChannelMessages>;
pub(crate) type ChallengeResponderTx = MpscTx<ChallengeResponderMessages>;
pub(crate) type ChallengeResponderRx = MpscRx<ChallengeResponderMessages>;
