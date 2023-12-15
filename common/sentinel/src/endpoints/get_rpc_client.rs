use std::result::Result;

use jsonrpsee::ws_client::{WsClient, WsClientBuilder};

use crate::SentinelError;

const MAX_BODY_SIZE_BYTES: u32 = 50_000_000; // NOTE: some BSC blocks exceed 10mb!

pub async fn get_rpc_client(url: &str) -> Result<WsClient, SentinelError> {
    debug!("getting RPC client using URL '{url}'...");
    Ok(WsClientBuilder::default()
        .max_request_size(MAX_BODY_SIZE_BYTES)
        .max_response_size(MAX_BODY_SIZE_BYTES)
        .build(&url)
        .await?)
}
