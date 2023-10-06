use common_sentinel::{
    BroadcastChannelMessages,
    BroadcasterMessages,
    EthRpcMessages,
    StatusMessages,
    WebSocketMessages,
};
use tokio::sync::{
    broadcast::{Receiver as MpmcRx, Sender as MpmcTx},
    mpsc::Sender as MpscTx,
};

pub(crate) type CoreCxnStatus = bool;
pub(crate) type StatusTx = MpscTx<StatusMessages>;
pub(crate) type EthRpcTx = MpscTx<EthRpcMessages>;
pub(crate) type WebSocketTx = MpscTx<WebSocketMessages>;
pub(crate) type BroadcasterTx = MpscTx<BroadcasterMessages>;
pub(crate) type BroadcastChannelTx = MpmcTx<BroadcastChannelMessages>;
pub(crate) type BroadcastChannelRx = MpmcRx<BroadcastChannelMessages>;
