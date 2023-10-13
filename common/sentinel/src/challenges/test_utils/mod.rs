#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common_eth::{EthLog, EthSubmissionMaterial};

pub fn get_sample_sub_mat_with_challenge_pending_event() -> EthSubmissionMaterial {
    let s =
        read_to_string("src/challenges/test_utils/polygon-block-48644355-with-challenge-pending-event.json").unwrap();
    EthSubmissionMaterial::from_str(&s).unwrap()
}

pub fn get_sample_log_with_challenge_pending_event() -> EthLog {
    let m = get_sample_sub_mat_with_challenge_pending_event();
    m.receipts[2].logs[1].clone()
}
