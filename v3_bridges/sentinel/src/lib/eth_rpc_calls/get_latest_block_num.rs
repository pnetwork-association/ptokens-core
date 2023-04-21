use std::result::Result;

use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::{constants::HEX_RADIX, endpoints::EndpointError, Endpoints, SentinelError};

const GET_LATEST_BLOCK_NUM_RPC_CMD: &str = "eth_blockNumber";

pub async fn get_latest_block_num(endpoints: &Endpoints) -> Result<u64, SentinelError> {
    debug!("Getting latest block number via endpoint...");
    let client = endpoints.get_web_socket().await?;
    let res: jsonrpsee::core::RpcResult<String> = client.request(GET_LATEST_BLOCK_NUM_RPC_CMD, rpc_params![]).await;
    match res {
        Err(_) => Err(SentinelError::Endpoint(EndpointError::NoLatestBlock)),
        Ok(ref s) => Ok(u64::from_str_radix(&s.replace("0x", ""), HEX_RADIX)?),
    }
}

#[cfg(test)]
mod tests {
    use common::BridgeSide;
    use tungstenite::accept;
    use warp::{Filter, Rejection};

    use super::*;
    use crate::test_utils::get_test_endpoints;

    #[tokio::test]
    async fn should_get_latest_block_num() {
        let endpoints = get_test_endpoints().await;
        let result = get_latest_block_num(&endpoints).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    use warp::ws::WebSocket;

    async fn client_connection(ws: WebSocket) {
        println!("establishing client connection... {:?}", ws);
    }

    async fn ws_handler(ws: warp::ws::Ws) -> Result<impl warp::Reply, Rejection> {
        println!("ws_handler");
        Ok(ws.on_upgrade(move |socket| client_connection(socket)))
    }

    async fn run_server() {
        let ws_route = warp::path("ws").and(warp::ws()).and_then(ws_handler);
        let routes = ws_route.with(warp::cors().allow_any_origin());
        warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
    }

    #[tokio::test]
    async fn should_x() {
        let url = "ws://127.0.0.1:8000/";
        let endpoints = Endpoints::new(false, 3, BridgeSide::Native, vec![url.to_string()]);

        let server_thread = tokio::spawn(run_server());
        /*
        std::thread::spawn(|| async {
            let ws_route = warp::path("ws")
                .and(warp::ws())
                .and_then(ws_handler);
            let routes = ws_route.with(warp::cors().allow_any_origin());
            warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
        });
        */

        let _r = tokio::join!(server_thread);
        let _res = get_latest_block_num(&endpoints).await.unwrap();
    }
}
