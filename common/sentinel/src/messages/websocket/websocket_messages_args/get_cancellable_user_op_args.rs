use std::str::FromStr;

use common_metadata::MetadataChainId;
use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::WebSocketMessagesError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Dissolve, Constructor)]
pub struct WebSocketMessagesGetCancellableUserOpArgs {
    max_delta: u64,
    mcids: Vec<MetadataChainId>,
}

// NOTE: Because these args are passed in via an RPC call
impl TryFrom<Vec<String>> for WebSocketMessagesGetCancellableUserOpArgs {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        // NOTE: Example: ["EthereumMainnet", "latest", "10", "false"]

        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };

        let arg = args[0].clone();
        let max_delta = arg.parse::<u64>().map_err(|_| WebSocketMessagesError::ParseInt(arg))?;

        let mcids = args[1..]
            .iter()
            .map(|arg| {
                MetadataChainId::from_str(arg).map_err(|_| WebSocketMessagesError::UnrecognizedChainId(arg.to_string()))
            })
            .collect::<Result<Vec<MetadataChainId>, WebSocketMessagesError>>()?;

        Ok(Self { max_delta, mcids })
    }
}
