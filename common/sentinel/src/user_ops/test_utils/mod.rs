#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common_eth::{EthLog, EthSubmissionMaterial};

pub fn get_sample_sub_mat_n(n: usize) -> EthSubmissionMaterial {
    let suffix = match n {
        2 => "goerli-block-9734264-with-protocol-queue-event.json",
        4 => "polygon-block-48041305-with-enqueu-user-op-event.json",
        5 => "polygon-block-49178593-with-cancellation-log.json",
        6 => "bsc-block-34114822-with-user-send-event.json",
        _ => "bsc-block-34461744-with-protocol-queue.json",
    };
    let prefix = "src/user_ops/test_utils/";
    let path = format!("{prefix}{suffix}");
    EthSubmissionMaterial::from_str(&read_to_string(path).unwrap()).unwrap()
}

pub fn get_sample_submission_material_with_user_send() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(6)
}

pub fn get_sample_submission_material_with_protocol_queue() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(2)
}

pub fn get_sample_log_with_user_send() -> EthLog {
    get_sample_submission_material_with_user_send().receipts[85].logs[5].clone()
}

pub fn get_sample_log_with_protocol_queue() -> EthLog {
    get_sample_submission_material_with_protocol_queue().receipts[3].logs[0].clone()
}

pub fn get_sample_submission_material_with_protocol_queue_2() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(7)
}
pub fn get_sample_log_with_protocol_queue_2() -> EthLog {
    get_sample_submission_material_with_protocol_queue_2().receipts[18].logs[0].clone()
}

pub fn get_sub_mat_with_enqueued_user_op() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(4)
}

pub(crate) fn get_log_with_protocol_cancellation_log() -> EthLog {
    get_sub_mat_with_protocol_cancellation_log().receipts[60].logs[0].clone()
}

pub fn get_sub_mat_with_protocol_cancellation_log() -> EthSubmissionMaterial {
    get_sample_sub_mat_n(5)
}
