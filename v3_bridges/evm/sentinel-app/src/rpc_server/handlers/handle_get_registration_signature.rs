use std::str::FromStr;

use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::Address as EthAddress;

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

// NOTE: A registration signature is a signature from the sentinel's TEE-protected signing key over
// the address which owns the sentinel plus a nonce to stop signature reuse.
impl RpcCalls {
    pub(crate) async fn handle_get_registration_signature(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        const MIN_NUM_PARAMS: usize = 2;
        let checked_params = Self::check_params(params, MIN_NUM_PARAMS)?;

        let owner_address = EthAddress::from_str(&checked_params[0])?;
        let nonce = checked_params[1].parse::<u64>()?;
        let sig = checked_params.get(MIN_NUM_PARAMS);

        debug!("owner address: {owner_address}");
        debug!("        nonce: {nonce}");

        let msg = WebSocketMessagesEncodable::GetRegistrationSignature(owner_address, nonce, sig.into());

        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }
}
