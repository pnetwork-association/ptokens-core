use common_sentinel::{
    call_core,
    EthRpcMessages,
    EthRpcSenders,
    LatestBlockInfos,
    NetworkId,
    Responder,
    SentinelConfig,
    SentinelError,
    SyncState,
    WebSocketMessagesEncodable,
};
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    rpc_server::{RpcCall, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_sync_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let network_ids = config.network_ids();

        // NOTE: The following will check core state for us...
        let core_latest_block_infos = LatestBlockInfos::try_from(
            call_core(
                STRONGBOX_TIMEOUT,
                websocket_tx.clone(),
                WebSocketMessagesEncodable::GetLatestBlockInfos(network_ids.clone()),
            )
            .await?,
        )?;

        let core_latest_block_numbers = network_ids
            .iter()
            .map(|id| core_latest_block_infos.get_for(id).map(|info| *info.block_number()))
            .collect::<Result<Vec<u64>, SentinelError>>()?;

        let mut rpc_latest_block_nums = vec![];

        for id in network_ids.iter() {
            let (msg, rx) = EthRpcMessages::get_latest_block_num_msg(*id);
            let sender = eth_rpc_senders.sender(id)?;
            sender.send(msg).await?;
            let n = rx.await??;
            rpc_latest_block_nums.push(n);
        }

        let mut state = SyncState::from((network_ids, core_latest_block_numbers, rpc_latest_block_nums));

        Ok(WebSocketMessagesEncodable::Success(json!(state)))
    }
}
