#![cfg(test)]
use std::{env, fs::read_to_string, str::FromStr};

use common_eth::EthSubmissionMaterial;
use dotenv::dotenv;
use jsonrpsee::ws_client::WsClient;

use crate::{check_endpoint, get_rpc_client, Batch, SentinelError};

const ENV_VAR: &str = "TEST_ENDPOINT";

pub async fn get_test_ws_client() -> WsClient {
    dotenv().ok();
    let time_limit = 5000; // NOTE: 5s
    let url = env::var(ENV_VAR)
        .map_err(|_| SentinelError::Custom("Please set env var '{ENV_VAR}' to a working endpoint!".into()))
        .unwrap();
    let ws_client = get_rpc_client(&url).await.unwrap();
    check_endpoint(&ws_client, time_limit).await.unwrap();
    ws_client
}

pub fn get_sample_sub_mat_n(n: usize) -> EthSubmissionMaterial {
    let suffix = match n {
        1 => "host-sub-mat-num-16776500.json",
        2 => "host-sub-mat-num-16776501.json",
        3 => "host-sub-mat-num-16776502.json",
        4 => "host-sub-mat-num-16776503.json",
        5 => "host-sub-mat-num-16776504.json",
        6 => "host-sub-mat-num-16776505.json",
        7 => "host-sub-mat-num-16776506.json",
        8 => "host-sub-mat-num-16776507.json",
        _ => "host-sub-mat-num-16776508.json",
    };
    let prefix = "src/lib/test_utils/";
    let path = format!("{prefix}{suffix}");
    EthSubmissionMaterial::from_str(&read_to_string(path).unwrap()).unwrap()
}

pub fn get_sample_batch() -> Batch {
    let mut batch = Batch::default();
    (1..10).for_each(|i| batch.push(get_sample_sub_mat_n(i)));
    batch
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn should_get_test_ws_client() {
        get_test_ws_client().await;
    }

    #[test]
    fn should_get_sample_sub_mat_n() {
        let n = 1;
        get_sample_sub_mat_n(n);
    }

    #[test]
    fn should_get_sample_batch() {
        get_sample_batch();
    }
}
