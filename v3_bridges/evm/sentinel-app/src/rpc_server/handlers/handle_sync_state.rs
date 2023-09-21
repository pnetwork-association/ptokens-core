use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_sentinel::{
    ConfigT,
    EthRpcMessages,
    LatestBlockNumbers,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::rpc_server::{
    constants::{EthRpcTx, WebSocketTx},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_sync_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        // NOTE: The following will chekc core state for us...
        let latest_block_numbers = match Self::handle_get_latest_block_numbers(websocket_tx, core_cxn).await {
            Ok(WebSocketMessagesEncodable::Success(j)) => Ok(serde_json::from_value::<LatestBlockNumbers>(j)?),
            r => Err(WebSocketMessagesError::UnexpectedResponse(format!("{r:?}"))),
        }?;

        let h_cid = config.host().chain_id();
        let n_cid = config.native().chain_id();
        let h_core_latest_block_num = latest_block_numbers.get_for(&h_cid)?;
        let n_core_latest_block_num = latest_block_numbers.get_for(&n_cid)?;

        let (h_msg, h_rx) = EthRpcMessages::get_latest_block_num_msg(BridgeSide::Host);
        let (n_msg, n_rx) = EthRpcMessages::get_latest_block_num_msg(BridgeSide::Native);
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
            chain_id: EthChainId,
            core_latest_block_num: u64,
            node_latest_block_num: u64,
            delta: u64,
        }

        impl SyncStatus {
            pub fn new(chain_id: EthChainId, core_latest_block_num: u64, node_latest_block_num: u64) -> Self {
                Self {
                    chain_id,
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
            SyncStatus::new(n_cid, n_core_latest_block_num, n_node_latest_block_num),
            SyncStatus::new(h_cid, h_core_latest_block_num, h_node_latest_block_num),
        ]));

        Ok(WebSocketMessagesEncodable::Success(j))
    }
}