use std::collections::HashMap;

use derive_more::{Constructor, Deref, DerefMut};
use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
};

use crate::{EthRpcMessages, NetworkId, SentinelConfigError, MAX_CHANNEL_CAPACITY};

#[derive(Debug, Clone, Constructor, Deref)]
pub struct EthRpcSenders(HashMap<NetworkId, Sender<EthRpcMessages>>);

impl EthRpcSenders {
    pub fn sender(&self, nid: &NetworkId) -> Result<Sender<EthRpcMessages>, SentinelConfigError> {
        self.get(nid)
            .cloned()
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }
}

impl From<&EthRpcChannels> for EthRpcSenders {
    fn from(cs: &EthRpcChannels) -> Self {
        let mut r: HashMap<NetworkId, Sender<EthRpcMessages>> = HashMap::new();
        for (k, v) in cs.iter() {
            r.insert(*k, v.0.clone());
        }
        Self::new(r)
    }
}

#[derive(Debug, Constructor, DerefMut, Deref)]
pub struct EthRpcChannels(HashMap<NetworkId, (Sender<EthRpcMessages>, Receiver<EthRpcMessages>)>);

impl From<Vec<NetworkId>> for EthRpcChannels {
    fn from(nids: Vec<NetworkId>) -> Self {
        let mut r: HashMap<NetworkId, (Sender<EthRpcMessages>, Receiver<EthRpcMessages>)> = HashMap::new();
        for id in nids.into_iter() {
            r.insert(id, mpsc::channel(MAX_CHANNEL_CAPACITY));
        }
        Self::new(r)
    }
}

impl EthRpcChannels {
    pub fn to_receivers(mut self) -> Vec<(NetworkId, Receiver<EthRpcMessages>)> {
        let mut r: Vec<(NetworkId, Receiver<EthRpcMessages>)> = vec![];
        for (id, v) in self.drain() {
            r.push((id, v.1))
        }
        r
    }
}
