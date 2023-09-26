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
        // NOTE: We require a max time delta, and at least two chains to check for cancellable user ops
        let n = 3;
        let l = args.len();

        if l < n {
            return Err(WebSocketMessagesError::NotEnoughArgs {
                got: l,
                expected: n,
                args,
            });
        };

        let arg = args[0].clone();
        let max_delta = arg.parse::<u64>().map_err(|_| WebSocketMessagesError::ParseInt(arg))?;

        const DELTA_MIN: u64 = 30;
        const DELTA_MAX: u64 = 30 * 60;

        if !(DELTA_MIN..=DELTA_MAX).contains(&max_delta) {
            return Err(WebSocketMessagesError::MaxDelta {
                got: max_delta,
                min: DELTA_MIN,
                max: DELTA_MAX,
            });
        };

        let mcids = args[1..]
            .iter()
            .map(|arg| {
                MetadataChainId::from_str(arg).map_err(|_| WebSocketMessagesError::UnrecognizedChainId(arg.to_string()))
            })
            .collect::<Result<Vec<MetadataChainId>, WebSocketMessagesError>>()?;

        Ok(Self { max_delta, mcids })
    }
}
