use std::str::FromStr;

use common_sentinel::{call_core, DebugSignature, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::Address as EthAddress;

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_add_debug_signers(
        params: RpcParams,
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 2)?;
        let mut signers = vec![];

        // NOTE: We expect the input to be an array of strings that are zero or more names and addresses,
        // possibly followed by a debug signature.
        for chunk in checked_params[..checked_params.len() - (checked_params.len() % 2)].chunks(2) {
            signers.push((chunk[0].clone(), EthAddress::from_str(&chunk[1])?));
        }

        // NOTE: If no debug signers are present the signature check for the first addition(s) is
        // omitted.
        let sig = if checked_params.len() % 2 == 0 {
            DebugSignature::new(None)
        } else {
            DebugSignature::new(checked_params.last().cloned())
        };

        let msg = WebSocketMessagesEncodable::AddDebugSigners(signers, sig);

        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }
}
