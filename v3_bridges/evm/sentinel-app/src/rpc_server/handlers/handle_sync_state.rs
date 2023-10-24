use common_sentinel::{
    call_core,
    EthRpcMessages,
    EthRpcSenders,
    LatestBlockNumbers,
    NetworkId,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
};
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    rpc_server::{RpcCall, STRONGBOX_TIMEOUT_MS},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_sync_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        todo!("this");
        /*
        Self::check_core_is_connected(core_cxn)?;

        let network_ids = config.network_ids();

        // NOTE: The following will check core state for us...
        let core_latest_block_numbers = LatestBlockNumbers::try_from(
            call_core(
                STRONGBOX_TIMEOUT_MS,
                websocket_tx.clone(),
                WebSocketMessagesEncodable::GetLatestBlockNumbers(network_ids),
            )
            .await?,
        )?;

        //let msgs = network_id.iter().map(|id| core_latest_block_numbers.get_for(id)).collect::<Result<Vec<EthRpcMessages>, SentinelError>>()?;

        //let h_core_latest_block_num = core_latest_block_numbers.get_for(&h_nid)?;
        //let n_core_latest_block_num = core_latest_block_numbers.get_for(&n_nid)?;

        let msgs = network_id.iter().map(|id| EthRpcMessages::get_latest_block_num_msg(id)).collect::<Vec<EthRpcMessages>>();
        host_eth_rpc_tx.send(h_msg).await?;
        native_eth_rpc_tx.send(n_msg).await?;
        let h_node_latest_block_num = h_rx.await??;
        let n_node_latest_block_num = n_rx.await??;

        // TODO factor out to own mod in lib probably
        #[derive(Clone, Debug, Serialize, Deserialize, Deref, Constructor)]
        struct SyncState(Vec<SyncStatus>);

        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SyncStatus {
            network_id: NetworkId,
            core_latest_block_num: u64,
            node_latest_block_num: u64,
            delta: u64,
        }

        impl SyncStatus {
            pub fn new(network_id: NetworkId, core_latest_block_num: u64, node_latest_block_num: u64) -> Self {
                Self {
                    network_id,
                    core_latest_block_num,
                    node_latest_block_num,
                    delta: if node_latest_block_num > core_latest_block_num {
                        node_latest_block_num - core_latest_block_num
                    } else {
                        0
                    },
                }
            }
        }

        let j = json!(SyncState::new(vec![
            SyncStatus::new(n_nid, n_core_latest_block_num, n_node_latest_block_num),
            SyncStatus::new(h_nid, h_core_latest_block_num, h_node_latest_block_num),
        ]));

        Ok(WebSocketMessagesEncodable::Success(j))
        */
    }
}
