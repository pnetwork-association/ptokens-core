#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common_eth::{EthLog, EthSubmissionMaterial};

pub fn get_sample_sub_mat_n(n: usize) -> EthSubmissionMaterial {
    let suffix = match n {
        1 => "bsc-block-31915441-with-user-send-event.json",
        2 => "goerli-block-9734264-with-protocol-queue-event.json",
        _ => "bsc-block-31966799-with-user-send-event.json",
    };
    let prefix = "src/user_ops/test_utils/";
    let path = format!("{prefix}{suffix}");
    EthSubmissionMaterial::from_str(&read_to_string(path).unwrap()).unwrap()
}

pub fn get_sample_submission_material_with_user_send() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(1)
}

pub fn get_sample_submission_material_with_protocol_queue() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(2)
}

pub fn get_sample_log_with_user_send() -> EthLog {
    get_sample_submission_material_with_user_send().receipts[56].logs[4].clone()
}

pub fn get_sample_submission_material_with_user_send_2() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(3)
}

pub fn get_sample_log_with_protocol_queue() -> EthLog {
    get_sample_submission_material_with_protocol_queue().receipts[3].logs[0].clone()
}
