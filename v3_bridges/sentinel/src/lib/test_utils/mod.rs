#![cfg(test)]
use std::{env, fs::read_to_string, str::FromStr};

use common::BridgeSide;
use common_eth::EthSubmissionMaterial;
use dotenv::dotenv;
use jsonrpsee::ws_client::WsClient;

use crate::{get_rpc_client, Batch, Endpoints, SentinelError};

const ENV_VAR: &str = "TEST_ENDPOINT";

pub async fn get_test_ws_client() -> WsClient {
    dotenv().ok();
    let url = env::var(ENV_VAR)
        .map_err(|_| SentinelError::Custom("Please set env var '{ENV_VAR}' to a working endpoint!".into()))
        .unwrap();

    get_rpc_client(&url).await.unwrap()
}

pub async fn get_test_endpoints() -> Endpoints {
    dotenv().ok();
    let is_native = true;
    let sleep_time = 500;
    let url = env::var(ENV_VAR)
        .map_err(|_| SentinelError::Custom("Please set env var '{ENV_VAR}' to a working endpoint!".into()))
        .unwrap();
    let urls = vec![url];
    Endpoints::new(is_native, sleep_time, BridgeSide::Native, urls)
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
        9 => "host-sub-mat-num-16776508.json",
        10 => "sepolia-native-sub-mat-num-3193051.json",
        11 => "sepolia-native-sub-mat-num-3256520.json",
        12 => "sepolia-native-sub-mat-num-3264438.json",
        13 => "sepolia-native-sub-mat-num-3264465.json",
        14 => "sepolia-native-sub-mat-num-3270755.json",
        _ => "sepolia-native-sub-mat-num-3270818.json",
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
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_test_ws_client() {
        get_test_ws_client().await;
    }

    #[test]
    fn should_get_sample_sub_mat_n() {
        let n = 11;
        get_sample_sub_mat_n(n);
    }

    #[test]
    fn should_get_sample_batch() {
        get_sample_batch();
    }
}
