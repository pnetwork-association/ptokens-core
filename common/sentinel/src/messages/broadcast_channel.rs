use common_chain_ids::EthChainId;

#[derive(Debug, Clone)]
pub enum SyncerBroadcastChannelMessages {
    Stop,
    Start,
}

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    Syncer(EthChainId, SyncerBroadcastChannelMessages),
}
