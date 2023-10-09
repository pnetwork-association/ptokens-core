use common_metadata::MetadataChainId;
use common_sentinel::{
    BroadcastChannelMessages,
    BroadcasterMessages,
    EthRpcMessages,
    StatusPublisherMessages,
    WebSocketMessages,
};
use tokio::sync::{
    broadcast::{Receiver as MpmcRx, Sender as MpmcTx},
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
};

pub(crate) type CoreCxnStatus = bool;
pub(crate) type Mcids = Vec<MetadataChainId>;
pub(crate) type EthRpcTx = MpscTx<EthRpcMessages>;
pub(crate) type WebSocketRx = MpscRx<WebSocketMessages>;
pub(crate) type WebSocketTx = MpscTx<WebSocketMessages>;
pub(crate) type BroadcasterTx = MpscTx<BroadcasterMessages>;
pub(crate) type StatusPublisherTx = MpscTx<StatusPublisherMessages>;
pub(crate) type StatusPublisherRx = MpscRx<StatusPublisherMessages>;
pub(crate) type BroadcastChannelTx = MpmcTx<BroadcastChannelMessages>;
pub(crate) type BroadcastChannelRx = MpmcRx<BroadcastChannelMessages>;
