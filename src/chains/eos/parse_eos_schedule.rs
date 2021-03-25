// TODO Move this mod to the new eos_schedule one and impl on the types etc.
use std::str::FromStr;

use eos_chain::{AccountName as EosAccountName, PublicKey as EosPublicKey};
use serde::{Deserialize, Serialize};

use crate::{
    chains::eos::eos_producer_key::{EosKey, EosKeysAndThreshold, EosProducerKeyV1, EosProducerKeyV2},
    errors::AppError,
    types::Result,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EosProducerScheduleJsonV1 {
    pub version: u32,
    pub producers: Vec<ProducerKeyJsonV1>,
}

impl EosProducerScheduleJsonV1 {
    pub fn from(schedule_string: &str) -> Result<Self> {
        serde_json::from_str::<Self>(schedule_string).map_err(AppError::SerdeJsonError)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProducerKeyJsonV1 {
    pub producer_name: String,
    pub block_signing_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EosProducerScheduleJsonV2 {
    pub version: u32,
    pub producers: Vec<FullProducerKeyJsonV2>,
}

impl EosProducerScheduleJsonV2 {
    pub fn from(schedule_string: &str) -> Result<Self> {
        serde_json::from_str::<Self>(schedule_string).map_err(AppError::SerdeJsonError)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FullProducerKeyJsonV2 {
    pub producer_name: String,
    pub authority: (u8, AuthorityJson),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityJson {
    pub threshold: u32,
    pub keys: Vec<ProducerKeyJsonV2>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProducerKeyJsonV2 {
    weight: u16,
    key: String,
}

pub fn convert_v2_producer_key_jsons_to_v2_producer_keys(
    json: &[FullProducerKeyJsonV2],
) -> Result<Vec<EosProducerKeyV2>> {
    json.iter()
        .map(convert_full_producer_key_json_to_v2_producer_key)
        .collect()
}

fn convert_full_producer_key_json_to_v2_producer_key(json: &FullProducerKeyJsonV2) -> Result<EosProducerKeyV2> {
    Ok(EosProducerKeyV2 {
        producer_name: EosAccountName::from_str(&json.producer_name)?,
        authority: (
            json.authority.0,
            convert_authority_json_to_eos_keys_and_threshold(&json.authority.1)?,
        ),
    })
}

pub fn convert_v1_producer_key_jsons_to_v1_producer_keys(json: &[ProducerKeyJsonV1]) -> Result<Vec<EosProducerKeyV1>> {
    json.iter()
        .map(convert_v1_producer_key_json_to_v1_producer_key)
        .collect()
}

fn convert_v1_producer_key_json_to_v1_producer_key(json: &ProducerKeyJsonV1) -> Result<EosProducerKeyV1> {
    Ok(EosProducerKeyV1::new(
        EosAccountName::from_str(&json.producer_name)?,
        EosPublicKey::from_str(&json.block_signing_key)?,
    ))
}

fn convert_authority_json_to_eos_keys_and_threshold(json: &AuthorityJson) -> Result<EosKeysAndThreshold> {
    Ok(EosKeysAndThreshold {
        threshold: json.threshold,
        keys: convert_keys_json_to_vec_of_eos_keys(&json.keys)?,
    })
}

pub fn convert_keys_json_to_vec_of_eos_keys(keys_json: &[ProducerKeyJsonV2]) -> Result<Vec<EosKey>> {
    keys_json.iter().map(convert_key_json_to_eos_key).collect()
}

pub fn convert_key_json_to_eos_key(key_json: &ProducerKeyJsonV2) -> Result<EosKey> {
    Ok(EosKey {
        weight: key_json.weight,
        key: EosPublicKey::from_str(&key_json.key)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eos::{
        eos_producer_schedule::EosProducerScheduleV1,
        eos_test_utils::{
            get_sample_v1_schedule_json,
            get_sample_v1_schedule_json_string,
            get_sample_v2_schedule_json,
        },
    };

    #[test]
    fn should_parse_v1_schedule_string_to_json() {
        let schedule_string = get_sample_v1_schedule_json_string().unwrap();
        if let Err(e) = EosProducerScheduleJsonV1::from(&schedule_string) {
            panic!("Could not parse EOS schedule json V1: {}", e);
        }
    }

    #[test]
    fn should_convert_v1_schedule_json_to_v1_schedule() {
        let schedule_json = get_sample_v1_schedule_json().unwrap();
        if let Err(e) = EosProducerScheduleV1::from_schedule_json(&schedule_json) {
            panic!("Error converting v1 schedule json to schedule: {}", e);
        }
    }

    #[test]
    fn should_convert_full_producer_key_json_to_producer_key_v2() {
        let producer_key_json = get_sample_v2_schedule_json().unwrap().producers[0].clone();
        if let Err(e) = convert_full_producer_key_json_to_v2_producer_key(&producer_key_json) {
            panic!("Error converting producer key json: {}", e);
        }
    }
}
