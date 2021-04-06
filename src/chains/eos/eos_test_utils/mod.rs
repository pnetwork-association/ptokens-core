#![cfg(test)]
use std::{fs::read_to_string, path::Path};

use bitcoin::hashes::{sha256, Hash as HashTrait};
use eos_chain::{NumBytes, Write};
use secp256k1::Message as Secp256k1Message;
use serde::{Deserialize, Serialize};

use crate::{
    chains::eos::{
        core_initialization::eos_init_utils::EosInitJson,
        eos_action_proofs::{EosActionProof, EosActionProofs},
        eos_action_receipt::{AuthSequence, EosActionReceipt},
        eos_block_header::EosBlockHeaderV2,
        eos_chain_id::EosChainId,
        eos_crypto::{eos_private_key::EosPrivateKey, eos_public_key::EosPublicKey, eos_signature::EosSignature},
        eos_merkle_utils::Incremerkle,
        eos_producer_schedule::{
            EosProducerScheduleJsonV1,
            EosProducerScheduleJsonV2,
            EosProducerScheduleV1,
            EosProducerScheduleV2,
        },
        eos_submission_material::{EosSubmissionMaterial, EosSubmissionMaterialJson},
        eos_types::{Checksum256s, EosBlockHeaderJson},
        eos_utils::convert_hex_to_checksum256,
        protocol_features::WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH,
    },
    errors::AppError,
    test_utils::get_sample_message_to_sign_bytes,
    types::{Byte, Bytes, Result},
};

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_1: &str = "src/chains/eos/eos_test_utils/eos-block-81784220.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_2: &str = "src/chains/eos/eos_test_utils/eos-block-80440580.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_3: &str = "src/chains/eos/eos_test_utils/eos-block-84187467.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_4: &str = "src/chains/eos/eos_test_utils/eos-block-81772484.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_5: &str = "src/chains/eos/eos_test_utils/eos-block-10800.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_6: &str = "src/chains/eos/eos_test_utils/jungle-3-block-8242000.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_7: &str = "src/chains/eos/eos_test_utils/eos-block-10700626.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_8: &str =
    "src/chains/eos/eos_test_utils/eos-mainnet-block-with-schedule-1714.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_9: &str =
    "src/chains/eos/eos_test_utils/eos-j3-block-with-schedule.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_10: &str =
    "src/chains/eos/eos_test_utils/mainnet-submission-material-with-perc20-redeem.json";

pub const SAMPLE_J3_INIT_BLOCK_JSON_PATH_1: &str = "src/chains/eos/eos_test_utils/jungle-3-init-block-10857380.json";

pub const SAMPLE_J3_INIT_BLOCK_JSON_PATH_2: &str = "src/chains/eos/eos_test_utils/jungle-3-init-block-11879805.json";

pub const SAMPLE_J3_INIT_BLOCK_JSON_PATH_3: &str = "src/chains/eos/eos_test_utils/jungle-3-init-block-11379805.json";

pub const SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_1: &str =
    "src/chains/eos/eos_test_utils/mainnet-init-block-125292121.json";

pub const SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_2: &str =
    "src/chains/eos/eos_test_utils/mainnet-init-block-125293807.json";

pub const SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_3: &str =
    "src/chains/eos/eos_test_utils/mainnet-init-block-125293952.json";

pub const SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_4: &str =
    "src/chains/eos/eos_test_utils/mainnet-init-block-125293952_with_eos_eth_token_dictionary.json";

pub const SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_JUNGLE_3_JSON_1: &str =
    "src/chains/eos/eos_test_utils/eos-init-and-subsequent-blocks-jungle-3-1.json";

pub const SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_MAINNET_JSON_1: &str =
    "src/chains/eos/eos_test_utils/eos-init-and-subsequent-blocks-mainnet-1.json";

pub const EOS_JUNGLE_CHAIN_ID: EosChainId = EosChainId::EosJungleTestnet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosInitAndSubsequentBlocksJson {
    pub init_block: EosInitJson,
    pub subsequent_blocks: Vec<EosSubmissionMaterialJson>,
}

impl EosInitAndSubsequentBlocksJson {
    pub fn is_msig_enabled(&self) -> bool {
        match self.init_block.maybe_protocol_features_to_enable {
            None => false,
            Some(ref features) => features.contains(&hex::encode(WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH)),
        }
    }

    pub fn from_json_string(json_string: &str) -> Result<Self> {
        Ok(serde_json::from_str(&json_string)?)
    }

    pub fn total_num_blocks(&self) -> usize {
        self.num_subsequent_blocks() + 1
    }

    pub fn num_subsequent_blocks(&self) -> usize {
        self.subsequent_blocks.len()
    }

    fn check_n(&self, n: usize) -> Result<()> {
        match n >= 1 && n <= self.total_num_blocks() {
            true => Ok(()),
            false => Err(format!("✘ Not enough blocks to get block num {}!", n).into()),
        }
    }

    fn get_incremerkle_for_initial_block(&self) -> Result<Incremerkle> {
        Ok(Incremerkle::new(
            self.init_block.block.block_num - 1,
            self.init_block
                .blockroot_merkle
                .iter()
                .map(convert_hex_to_checksum256)
                .collect::<Result<Checksum256s>>()?,
        ))
    }

    pub fn get_active_schedule(&self) -> Result<EosProducerScheduleV2> {
        EosProducerScheduleV2::from_schedule_json(&self.init_block.active_schedule)
    }

    pub fn get_block_json_n(&self, n: usize) -> Result<EosBlockHeaderJson> {
        self.check_n(n)?;
        Ok(if n == 1 {
            self.init_block.block.clone()
        } else {
            self.subsequent_blocks[n - 2].block_header.clone()
        })
    }

    pub fn get_block_n(&self, n: usize) -> Result<EosBlockHeaderV2> {
        EosSubmissionMaterial::parse_eos_block_header_from_json(&self.get_block_json_n(n)?)
    }

    pub fn get_producer_signature_for_block_n(&self, n: usize) -> Result<String> {
        self.check_n(n)
            .and_then(|_| self.get_block_json_n(n))
            .map(|block_json| block_json.producer_signature)
    }

    pub fn get_incremerkle_for_block_n(&self, n: usize) -> Result<Incremerkle> {
        self.check_n(n).and_then(|_| {
            let mut incremerkle = self.get_incremerkle_for_initial_block()?;
            match n == 1 {
                true => Ok(incremerkle),
                _ => {
                    vec![0; n - 1]
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let mut block_ids = vec![];
                            self.subsequent_blocks[i].interim_block_ids.iter().for_each(|id| {
                                block_ids.push(id.clone());
                            });
                            block_ids
                        })
                        .flatten()
                        .map(convert_hex_to_checksum256)
                        .for_each(|checksum| {
                            incremerkle.append(checksum.unwrap()).unwrap();
                        });
                    Ok(incremerkle)
                },
            }
        })
    }

    pub fn get_block_mroot_for_block_n(&self, n: usize) -> Result<Bytes> {
        self.get_incremerkle_for_block_n(n)
            .map(|incremerkle| incremerkle.get_root().to_bytes().to_vec())
    }
}

pub fn get_init_and_subsequent_blocks_json_n(num: usize) -> Result<EosInitAndSubsequentBlocksJson> {
    let path = match num {
        1 => Ok(SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_JUNGLE_3_JSON_1),
        2 => Ok(SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_MAINNET_JSON_1),
        _ => Err(AppError::Custom(format!("Cannot find sample block num: {}", num))),
    }?;
    if let Ok(contents) = read_to_string(path) {
        EosInitAndSubsequentBlocksJson::from_json_string(&contents)
    } else {
        Err(format!("✘ Can't find sample init block json file @ path: {}", path).into())
    }
}

pub const NUM_J3_INIT_SAMPLES: usize = 3;

pub fn get_j3_init_json_n(num: usize) -> Result<EosInitJson> {
    let path = match num {
        1 => Ok(SAMPLE_J3_INIT_BLOCK_JSON_PATH_1),
        2 => Ok(SAMPLE_J3_INIT_BLOCK_JSON_PATH_2),
        3 => Ok(SAMPLE_J3_INIT_BLOCK_JSON_PATH_3),
        _ => Err(AppError::Custom(format!("Cannot find sample block num: {}", num))),
    }?;
    if let Ok(contents) = read_to_string(path) {
        EosInitJson::from_json_string(&contents)
    } else {
        Err(format!("✘ Can't find sample init block json file @ path: {}", path).into())
    }
}

pub const NUM_MAINNET_INIT_SAMPLES: usize = 2;

pub fn get_mainnet_init_json_n(num: usize) -> Result<EosInitJson> {
    let path = match num {
        1 => Ok(SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_1),
        2 => Ok(SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_2),
        3 => Ok(SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_3),
        4 => Ok(SAMPLE_MAINNET_INIT_BLOCK_JSON_PATH_4),
        _ => Err(AppError::Custom(format!("Cannot find sample block num: {}", num))),
    }?;
    if let Ok(contents) = read_to_string(path) {
        EosInitJson::from_json_string(&contents)
    } else {
        Err(format!("✘ Can't find sample init block json file @ path: {}", path).into())
    }
}

pub fn get_sample_mainnet_init_json_with_eos_eth_token_dictionary() -> Result<EosInitJson> {
    get_mainnet_init_json_n(4)
}

pub fn sha256_hash_message_bytes(message_bytes: &[Byte]) -> Result<Secp256k1Message> {
    Ok(Secp256k1Message::from_slice(&sha256::Hash::hash(message_bytes))?)
}

pub fn get_sample_v1_schedule_json_string() -> Result<String> {
    Ok(read_to_string(
        "src/chains/eos/eos_test_utils/sample-schedule-389-v1.json",
    )?)
}

pub fn get_sample_v2_schedule_json_string() -> Result<String> {
    Ok(read_to_string(
        "src/chains/eos/eos_test_utils/sample-schedule-28-v2.json",
    )?)
}

pub fn get_sample_mainnet_schedule_1713() -> Result<EosProducerScheduleV2> {
    EosProducerScheduleJsonV1::from(&read_to_string(
        "src/chains/eos/eos_test_utils/sample-schedule-1713-v1.json",
    )?)
    .and_then(|v1_json| EosProducerScheduleV1::from_schedule_json(&v1_json))
    .map(|v1_schedule| EosProducerScheduleV2::from(v1_schedule))
}

pub fn get_sample_j3_schedule_37() -> Result<EosProducerScheduleV2> {
    EosProducerScheduleJsonV1::from(&read_to_string(
        "src/chains/eos/eos_test_utils/sample-j3-schedule-37.json",
    )?)
    .and_then(|v1_json| EosProducerScheduleV1::from_schedule_json(&v1_json))
    .map(|v1_schedule| EosProducerScheduleV2::from(v1_schedule))
}

pub fn get_sample_v1_schedule_json() -> Result<EosProducerScheduleJsonV1> {
    get_sample_v1_schedule_json_string().and_then(|json_string| EosProducerScheduleJsonV1::from(&json_string))
}

pub fn get_sample_v1_schedule() -> Result<EosProducerScheduleV1> {
    get_sample_v1_schedule_json().and_then(|json| EosProducerScheduleV1::from_schedule_json(&json))
}

pub fn get_sample_v2_schedule_json() -> Result<EosProducerScheduleJsonV2> {
    get_sample_v2_schedule_json_string().and_then(|json_string| EosProducerScheduleJsonV2::from(&json_string))
}

pub fn get_sample_v2_schedule() -> Result<EosProducerScheduleV2> {
    get_sample_v2_schedule_json().and_then(|json| EosProducerScheduleV2::from_schedule_json(&json))
}

pub fn get_sample_eos_submission_material_n(n: usize) -> EosSubmissionMaterial {
    EosSubmissionMaterial::from_str(&get_sample_eos_submission_material_string_n(n).unwrap()).unwrap()
}

pub fn get_sample_eos_submission_material_json_n(n: usize) -> EosSubmissionMaterialJson {
    EosSubmissionMaterialJson::from_str(&get_sample_eos_submission_material_string_n(n).unwrap()).unwrap()
}

pub fn get_sample_eos_submission_material_string_n(num: usize) -> Result<String> {
    let path = match num {
        1 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_1),
        2 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_2),
        3 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_3),
        4 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_4),
        5 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_5),
        6 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_6),
        7 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_7),
        8 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_8),
        9 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_9),
        10 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_10),
        _ => Err(AppError::Custom(format!("Cannot find sample block num: {}", num))),
    }?;
    match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err("✘ Cannot find sample-eos-block-and-action-json file!".into()),
    }
}

pub fn get_sample_eos_private_key_str() -> &'static str {
    "5K8ufCfDxaFXqkRdeGmLywEh32F3MZf67E8hFFvQoH3imDwQ9Ea"
}

pub fn get_sample_eos_public_key_str() -> &'static str {
    "EOS5vMQQqeG6ixyaLSvQacyZe9bH1kmMeYY296fFdc3d3317MdV2f"
}

pub fn get_sample_eos_private_key() -> EosPrivateKey {
    EosPrivateKey::from_wallet_import_format(get_sample_eos_private_key_str()).unwrap()
}

pub fn get_sample_eos_public_key() -> EosPublicKey {
    get_sample_eos_private_key().to_public_key()
}

pub fn get_sample_eos_public_key_bytes() -> Bytes {
    get_sample_eos_public_key().to_bytes()
}

pub fn get_sample_eos_signature() -> EosSignature {
    get_sample_eos_private_key()
        .sign_message_bytes(&get_sample_message_to_sign_bytes())
        .unwrap()
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
            vec![AuthSequence::new("eosio", 61285932).unwrap()],
        )
        .unwrap(),
        EosActionReceipt::new(
            "pokerpokerts",
            "3d380413463e8716ef9c1f8c853dfab0c70f209cce75cae9a5b74e4e678a68a0",
            241512,
            4,
            503081364,
            30,
            vec![AuthSequence::new("pokerpokerts", 241552).unwrap()],
        )
        .unwrap(),
        EosActionReceipt::new(
            "oracleoracle",
            "065527f0429dfa9bb79575ec5270b20f714fb9e61a9ce6ba9c86b2e69a773f82",
            531231,
            2,
            503081365,
            2,
            vec![AuthSequence::new("feeder111112", 152730).unwrap()],
        )
        .unwrap(),
        EosActionReceipt::new(
            "dvmh1tbb1him",
            "18e42aa86473509cf620764ca606136b037e1a8ee6fb8efaa8fa657c7fa2fffc",
            805647,
            2,
            503081366,
            1,
            vec![AuthSequence::new("dvmh1tbb1him", 805667).unwrap()],
        )
        .unwrap(),
    ]
}

pub fn get_sample_action_digests() -> Result<Vec<Bytes>> {
    get_sample_action_receipts()
        .into_iter()
        .map(|receipt| receipt.to_digest())
        .collect()
}

fn get_sample_action_proofs_n(n: usize) -> EosActionProofs {
    get_sample_eos_submission_material_n(n).action_proofs
}

pub fn get_sample_action_proof_n(n: usize) -> EosActionProof {
    get_sample_action_proofs_n(n)[0].clone()
}

pub fn serialize_block_header_v2(header: &EosBlockHeaderV2) -> Result<Bytes> {
    let mut data = vec![0u8; header.num_bytes()];
    header.write(&mut data, &mut 0)?;
    Ok(data)
}
