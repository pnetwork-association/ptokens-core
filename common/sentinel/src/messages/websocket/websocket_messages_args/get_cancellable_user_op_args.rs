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
        // NOTE: We require two chains to check for cancellable user ops.

        let n = 2;
        let l = args.len();
        if l < n {
            return Err(WebSocketMessagesError::InsufficientMcids { got: l, expected: n });
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
