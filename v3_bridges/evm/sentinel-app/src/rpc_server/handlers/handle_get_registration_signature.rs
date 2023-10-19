use common_eth::convert_hex_to_eth_address;
use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError};

use crate::{
    rpc_server::{RpcCall, RpcParams, STRONGBOX_TIMEOUT_MS},
    type_aliases::WebSocketTx,
};

// TODO Take whatever other params are required to maybe broadcast this signature too?

impl RpcCall {
    pub(crate) async fn handle_get_registration_signature(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let n = 2;
        let l = params.len();
        if l < n {
            return Err(WebSocketMessagesError::NotEnoughArgs {
                got: l,
                expected: n,
                args: params,
            }
            .into());
        }

        let owner_address = convert_hex_to_eth_address(&params[0])?;
        let nonce = params[1].parse::<u64>()?;

        debug!("owner address: {owner_address}");
        debug!("       inonce: {nonce}");

        let msg = WebSocketMessagesEncodable::GetRegistrationSignature(owner_address, nonce);

        call_core(STRONGBOX_TIMEOUT_MS, websocket_tx.clone(), msg).await
    }
}
