use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::{NetworkId, WebSocketMessagesError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Dissolve, Constructor)]
pub struct WebSocketMessagesGetCancellableUserOpArgs {
    max_delta: u64,
    network_ids: Vec<NetworkId>,
}

// NOTE: Because these args are passed in via an RPC call
impl TryFrom<Vec<String>> for WebSocketMessagesGetCancellableUserOpArgs {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        // NOTE: We require a max time delta, and at least one chain to check for user op cancellability
        let n = 2;
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

        let network_ids = args[1..]
            .iter()
            .map(|arg| {
                NetworkId::try_from(arg).map_err(|_| WebSocketMessagesError::UnrecognizedNetworkId(arg.to_string()))
            })
            .collect::<Result<Vec<NetworkId>, WebSocketMessagesError>>()?;

        Ok(Self { max_delta, network_ids })
    }
}
