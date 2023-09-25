use common::BridgeSide;
use common_sentinel::{
    ConfigT,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesInitArgs,
};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{EthRpcTx, RpcParams, WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_init(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let mut args = WebSocketMessagesInitArgs::try_from(params)?;
        // NOTE: Alas we're still stuck with host and native here so we need to figure out which
        // syncer to call.
        let mcid = *args.mcid();
        let eth_chain_id_from_args = mcid.to_eth_chain_id()?;

        let h_eth_chain_id_from_config = config.host().chain_id();
        let n_eth_chain_id_from_config = config.native().chain_id();

        let use_native = if eth_chain_id_from_args == n_eth_chain_id_from_config {
            Result::<bool, SentinelError>::Ok(true)
        } else if eth_chain_id_from_args == h_eth_chain_id_from_config {
            Result::<bool, SentinelError>::Ok(false)
        } else {
            Err(WebSocketMessagesError::Unsupported(mcid).into())
        }?;

        // NOTE: Now we can get the latest block number for the correct chain from the ETH RPC
        let (latest_block_num_msg, latest_block_num_responder) =
            EthRpcMessages::get_latest_block_num_msg(if use_native {
                BridgeSide::Native
            } else {
                BridgeSide::Host
            });
        if use_native {
            native_eth_rpc_tx.send(latest_block_num_msg).await?;
        } else {
            host_eth_rpc_tx.send(latest_block_num_msg).await?;
        };
        let latest_block_num = latest_block_num_responder.await??;

        // NOTE: Get use that number to get the latest submission material
        let (latest_block_msg, latest_block_responder) = EthRpcMessages::get_sub_mat_msg(
            if use_native {
                BridgeSide::Native
            } else {
                BridgeSide::Host
            },
            latest_block_num,
        );
        if use_native {
            native_eth_rpc_tx.send(latest_block_msg).await?;
        } else {
            host_eth_rpc_tx.send(latest_block_msg).await?;
        }
        let sub_mat = latest_block_responder.await??;

        // NOTE: Now we need to add the sub mat to the args to send to strongbox
        args.add_sub_mat(sub_mat);
        let final_msg = WebSocketMessagesEncodable::Initialize(Box::new(args));
        //
        // NOTE: Now we send out msg to the websocket loop
        let (msg, rx) = WebSocketMessages::new(final_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "initializing core";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
