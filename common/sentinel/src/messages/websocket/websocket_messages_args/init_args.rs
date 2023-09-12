use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::EthSubmissionMaterial;
use derive_getters::Getters;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::WebSocketMessagesError;

pub type Confirmations = u64;

#[derive(Debug, Clone, PartialEq, Constructor, Serialize, Deserialize, Getters)]
pub struct WebSocketMessagesInitArgs {
    host_validate: bool,
    native_validate: bool,
    host_chain_id: EthChainId,
    host_confirmations: Confirmations,
    native_chain_id: EthChainId,
    native_confirmations: Confirmations,
    host_block: Option<EthSubmissionMaterial>,
    native_block: Option<EthSubmissionMaterial>,
}

impl WebSocketMessagesInitArgs {
    fn name(&self) -> String {
        "WebSocketMessagesInitArgs".into()
    }

    pub fn add_host_block(&mut self, m: EthSubmissionMaterial) {
        self.host_block = Some(m);
    }

    pub fn add_native_block(&mut self, m: EthSubmissionMaterial) {
        self.native_block = Some(m);
    }

    pub fn to_host_sub_mat(&self) -> Result<EthSubmissionMaterial, WebSocketMessagesError> {
        match self.host_block {
            Some(ref b) => Ok(b.clone()),
            None => Err(WebSocketMessagesError::NoBlock {
                side: BridgeSide::Host,
                struct_name: self.name(),
            }),
        }
    }

    pub fn to_native_sub_mat(&self) -> Result<EthSubmissionMaterial, WebSocketMessagesError> {
        match self.native_block {
            Some(ref b) => Ok(b.clone()),
            None => Err(WebSocketMessagesError::NoBlock {
                side: BridgeSide::Native,
                struct_name: self.name(),
            }),
        }
    }
}
