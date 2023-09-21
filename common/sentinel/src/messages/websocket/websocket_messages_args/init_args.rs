use std::str::FromStr;

use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::EthSubmissionMaterial;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::WebSocketMessagesError;

pub type Confirmations = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct WebSocketMessagesInitArgs {
    native_validate: bool,
    native_chain_id: EthChainId,
    native_confirmations: Confirmations,
    native_block: Option<EthSubmissionMaterial>,
    host_validate: bool,
    host_chain_id: EthChainId,
    host_confirmations: Confirmations,
    host_block: Option<EthSubmissionMaterial>,
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

// NOTE: Because these args are passed in via an RPC call
impl TryFrom<Vec<String>> for WebSocketMessagesInitArgs {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };

        let expected_num_args = 6;
        if args.len() != expected_num_args {
            return Err(WebSocketMessagesError::NotEnoughArgs {
                got: args.len(),
                expected: expected_num_args,
                args,
            });
        }

        Ok(Self {
            native_validate: matches!(args[0].as_ref(), "true"),
            native_chain_id: EthChainId::from_str(&args[1])
                .map_err(|_| WebSocketMessagesError::UnrecognizedEthChainId(args[3].clone()))?,
            native_confirmations: args[2]
                .parse::<Confirmations>()
                .map_err(|_| WebSocketMessagesError::ParseInt(args[4].clone()))?,
            native_block: None,

            host_validate: matches!(args[3].as_ref(), "true"),
            host_chain_id: EthChainId::from_str(&args[4])
                .map_err(|_| WebSocketMessagesError::UnrecognizedEthChainId(args[5].clone()))?,
            host_confirmations: args[5]
                .parse::<Confirmations>()
                .map_err(|_| WebSocketMessagesError::ParseInt(args[7].clone()))?,
            host_block: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::WebSocketMessagesEncodable;

    #[test]
    fn should_get_init_message_from_string_of_args() {
        let args = vec!["init", "true", "EthereumMainnet", "10", "true", "BscMainnet", "100"];
        let r = WebSocketMessagesEncodable::try_from(args);
        assert!(r.is_ok());
    }
}
