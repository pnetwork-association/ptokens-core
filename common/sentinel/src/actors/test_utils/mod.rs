#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common_eth::{EthLog, EthSubmissionMaterial};

pub fn get_sample_actors_propagated_log() -> EthLog {
    EthSubmissionMaterial::from_str(
        &read_to_string("src/actors/test_utils/polygon-block-48520980-with-actors-propagated-event.json").unwrap(),
    )
    .unwrap()
    .receipts[0]
        .logs[0]
        .clone()
}
