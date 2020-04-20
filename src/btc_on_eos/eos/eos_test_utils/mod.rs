#![cfg(test)]
#![allow(unused_imports)]
use std::{
    path::Path,
    str::FromStr,
    fs::read_to_string,
};
use eos_primitives::{
    ActionName,
    Checksum256,
    AccountName,
    AuthSequence,
    ActionTransfer,
    PermissionLevel,
    Action as EosAction,
    ActionReceipt as EosActionReceipt,
    ProducerSchedule as EosProducerSchedule,
    ProducerScheduleV2 as EosProducerScheduleV2,
};
use crate::{
    errors::AppError,
    types::{
        Bytes,
        Result,
    },
    btc_on_eos::{
        utils::convert_hex_to_checksum256,
        test_utils::get_sample_message_to_sign,
        eos::{
            eos_state::EosState,
            parse_eos_schedule::{
                EosProducerScheduleJson,
                parse_schedule_string_to_json,
                convert_schedule_json_to_schedule_v2,
            },
            parse_submission_material::{
                parse_producer_schedule_from_json_string,
                parse_eos_submission_material_string_to_json,
                parse_eos_submission_material_string_to_struct,
            },
            eos_types::{
                EosAmount,
                EosSignatures,
                EosSignedTransaction,
                EosSignedTransactions,
                EosSubmissionMaterial,
                EosSubmissionMaterialJson,
                ActionProofs,
                ActionProof
            },
            eos_crypto::{
                eos_signature::EosSignature,
                eos_public_key::EosPublicKey,
                eos_private_key::EosPrivateKey,
            },
        },
    },
};

pub const NUM_SAMPLES: usize = 5; // TODO update once all are passing validation!

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_1: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-81784220.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_2: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-80440580.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_3: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-84187467.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_4: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-81772484.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_5: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-10800.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_6: &str =
    "src/btc_on_eos/eos/eos_test_utils/jungle-3-block-8242000.json";

pub const SAMPLE_EOS_ACTIVE_SCHEDULE_PATH_PREFIX: &str =
    "src/btc_on_eos/eos/eos_test_utils/sample-active-schedule-";

pub const EOS_JUNGLE_CHAIN_ID: &str =
    "e70aaab8997e1dfce58fbfac80cbbb8fecec7b99cf982a9444273cbc64c41473";

pub const TEMPORARY_DATABASE_PATH: &str = "src/test_utils/temporary_database";

// Note: Key = provabletokn "active" on Jungle
pub const EOS_SAMPLE_PRIVATE_KEY_WIF: &str =
    "5HzXzUB9sruHL93mf5dVgUJk1A3NMiAAsfu4p6F1hDdktVVErbR";

pub fn get_sample_v2_schedule_json_string() -> Result<String> {
    Ok(
        read_to_string(
            "src/btc_on_eos/eos/eos_test_utils/sample-schedule-v2.0.json"
        )?
    )
}

pub fn get_sample_v2_schedule_json() -> Result<EosProducerScheduleJson> {
    get_sample_v2_schedule_json_string()
        .and_then(|json_string| parse_schedule_string_to_json(&json_string))
}

pub fn get_sample_v2_schedule() -> Result<EosProducerScheduleV2> {
    get_sample_v2_schedule_json()
        .and_then(|json| convert_schedule_json_to_schedule_v2(&json))
}

pub fn get_sample_active_schedule(
    version: u32,
) -> Result<EosProducerSchedule> {
    let path = format!(
        "{}{}.json",
        SAMPLE_EOS_ACTIVE_SCHEDULE_PATH_PREFIX,
        version
    );
    match Path::new(&path).exists() {
        true => parse_producer_schedule_from_json_string(
            &read_to_string(path)?
        ),
        false => Err(AppError::Custom(
            format!("✘ Cannot find sample active schedule json!")
        ))
    }
}

pub fn get_sample_eos_private_key_2() -> EosPrivateKey {
    EosPrivateKey::from_wallet_import_format(
        EOS_SAMPLE_PRIVATE_KEY_WIF
    ).unwrap()
}

pub fn get_sample_eos_submission_material_n(
    n: usize
) -> EosSubmissionMaterial {
    parse_eos_submission_material_string_to_struct(
        &get_sample_eos_submission_material_string_n(n).unwrap()
    ).unwrap()
}

pub fn get_sample_eos_submission_material_json_n(
    n: usize
) -> EosSubmissionMaterialJson {
    parse_eos_submission_material_string_to_json(
        &get_sample_eos_submission_material_string_n(n).unwrap()
    ).unwrap()
}

pub fn get_sample_eos_submission_material_string_n(
    num: usize,
) -> Result<String> {
    let path = match num {
        1 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_1),
        2 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_2),
        3 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_3),
        4 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_4),
        5 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_5),
        6 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_6),
        _ => Err(AppError::Custom(
            format!("Cannot find sample block num: {}", num)
        ))
    }?;
    match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err(AppError::Custom(
            format!("✘ Cannot find sample-eos-block-and-action-json file!")
        ))
    }
}

pub fn get_sample_eos_action() -> EosAction {
    EosAction {
        name: ActionName::from_str("onblock").unwrap(),
        account: AccountName::from_str("eosio").unwrap(),
        authorization: vec![PermissionLevel::from_str("eosio", "active").unwrap()],
        data: hex::decode("e0d2b86b1a3962343021cd2a1eb3e9ad672b00000000000004454f53000000002a3078303236644336413433353631444138413641373735353338623139324133653933366330463239426a01000000000000").unwrap()
    }
    /* NOTE: The data here is serialized from this:
    "data": {
        "sender": "all3manfr3di",
        "receiver": "provabletokn",
        "quantity": "1.1111 EOS",
        "ethereum_sender_str": "0x026dC6A43561DA8A6A775538b192A3e936c0F29B",
        "nonce": 362
    }
    */
}

pub fn get_sample_eos_action_receipt() -> EosActionReceipt {
    EosActionReceipt {
        recipient: AccountName::from_str("provabletokn").unwrap(),
        act_digest: convert_hex_to_checksum256(
            &"4f72e85ee91bb26bf223f0ad1e08e8ac11a143b4eb1ac9854e4e726e85cc9b51"
                .to_string()
        ).unwrap(),
        global_sequence: 499094015,
        recv_sequence: 2046,
        auth_sequence: vec![
            AuthSequence::new(
                "provabletokn",
                2216
            ).unwrap(),
        ],
        code_sequence: 80,
        abi_sequence: 48,
    }
}

pub fn get_sample_eos_private_key_wif() -> &'static str {
    "5HrBLKfeEdqH9KLMv1daHLVjrXV3DGVERAkN5cdSSc58bzqqfT4"
}

pub fn get_jungle_provable_tokn_private_key() -> EosPrivateKey {
    EosPrivateKey::from_wallet_import_format(
        "5HzXzUB9sruHL93mf5dVgUJk1A3NMiAAsfu4p6F1hDdktVVErbR"
    ).unwrap()
}

pub fn get_sample_eos_private_key_str() -> &'static str {
    "5K8ufCfDxaFXqkRdeGmLywEh32F3MZf67E8hFFvQoH3imDwQ9Ea"
}

pub fn get_sample_eos_public_key_str() -> &'static str {
    "EOS5vMQQqeG6ixyaLSvQacyZe9bH1kmMeYY296fFdc3d3317MdV2f"
}

pub fn get_sample_eos_private_key() -> EosPrivateKey {
    EosPrivateKey::from_wallet_import_format(
        get_sample_eos_private_key_str()
    ).unwrap()
}

pub fn get_sample_eos_public_key() -> EosPublicKey {
    EosPublicKey::from(&get_sample_eos_private_key())
}

pub fn get_sample_eos_public_key_bytes() -> Bytes {
    get_sample_eos_public_key()
        .to_bytes()
}

pub fn get_sample_eos_signature() -> EosSignature {
    get_sample_eos_private_key()
        .sign_message_bytes(&get_sample_message_to_sign().as_bytes())
        .unwrap()
}

pub fn get_sample_eos_signatures() -> EosSignedTransactions {
    let mut signed_txs: EosSignedTransactions = Vec::new();
    signed_txs.push(EosSignedTransaction::new(
        "signature 1".to_string(),
        "transaction 1".to_string(),
        "recipientttt1".to_string(),
        "1.0000 EOS".to_string(),
    ));
    signed_txs.push(EosSignedTransaction::new(
        "signature 2".to_string(),
        "transaction 2".to_string(),
        "recipientttt2".to_string(),
        "2.0000 EOS".to_string(),
    ));
    signed_txs.push(EosSignedTransaction::new(
        "signature 3".to_string(),
        "transaction 3".to_string(),
        "recipientttt3".to_string(),
        "3.0000 EOS".to_string(),
    ));
    signed_txs
}

fn get_sample_action_receipts() -> Vec<EosActionReceipt> {
    vec![
        EosActionReceipt::new(
            "eosio",
            "3b434aa9331f5e2a0e7a0060d576fa6688406667100bdf3458104dede44ec4e9",
            62826453,
            12,
            503081363,
            10,
            vec![AuthSequence::new(
                "eosio",
                61285932
            ).unwrap()],
        ).unwrap(),
        EosActionReceipt::new(
            "pokerpokerts",
            "3d380413463e8716ef9c1f8c853dfab0c70f209cce75cae9a5b74e4e678a68a0",
            241512,
            4,
            503081364,
            30,
            vec![AuthSequence::new(
                "pokerpokerts",
                241552
            ).unwrap()],
        ).unwrap(),
        EosActionReceipt::new(
            "oracleoracle",
            "065527f0429dfa9bb79575ec5270b20f714fb9e61a9ce6ba9c86b2e69a773f82",
            531231,
            2,
            503081365,
            2,
            vec![AuthSequence::new(
                "feeder111112",
                152730
            ).unwrap()],
        ).unwrap(),
        EosActionReceipt::new(
            "dvmh1tbb1him",
            "18e42aa86473509cf620764ca606136b037e1a8ee6fb8efaa8fa657c7fa2fffc",
            805647,
            2,
            503081366,
            1,
            vec![AuthSequence::new(
                "dvmh1tbb1him",
                805667
            ).unwrap()],
        ).unwrap(),
    ]
}

pub fn get_sample_action_digests() -> Vec<Bytes> {
    get_sample_action_receipts()
        .into_iter()
        .map(|receipt| receipt.to_digest())
        .collect()
}

fn get_sample_action_proofs_n(
    n: usize
) -> ActionProofs {
    get_sample_eos_submission_material_n(n).action_proofs
}

pub fn get_sample_action_proof_n(
    n: usize
) -> ActionProof {
    get_sample_action_proofs_n(n)[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_sample_eos_blocks_n() {
        let max = 1;
        for i in 1..max {
            get_sample_eos_submission_material_json_n(i);
        }
    }
}
