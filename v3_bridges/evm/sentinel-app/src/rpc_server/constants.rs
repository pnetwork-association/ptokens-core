use common_sentinel::{BroadcastChannelMessages, EthRpcMessages, WebSocketMessages};
use tokio::sync::{
    broadcast::{Receiver as MpmcRx, Sender as MpmcTx},
    mpsc::Sender as MpscTx,
};

pub(crate) type RpcId = Option<u64>;
pub(crate) type CoreCxnStatus = bool;
pub(crate) type RpcParams = Vec<String>;
pub(crate) type EthRpcTx = MpscTx<EthRpcMessages>;
pub(crate) type WebSocketTx = MpscTx<WebSocketMessages>;
pub(crate) type BroadcastChannelTx = MpmcTx<BroadcastChannelMessages>;
pub(crate) type BroadcastChannelRx = MpmcRx<BroadcastChannelMessages>;
pub(crate) const STRONGBOX_TIMEOUT_MS: u64 = 30000; // FIXME make configurable
