use std::str::FromStr;

use chrono::prelude::*;
use eos_chain::{AccountName, BlockTimestamp, TimePoint};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    chains::eos::{
        eos_action_proofs::{EosActionProof, EosActionProofJson, EosActionProofJsons, EosActionProofs},
        eos_block_header::EosBlockHeaderV2,
        eos_extension::EosExtensions,
        eos_producer_schedule::EosProducerScheduleV2,
        eos_state::EosState,
        eos_types::{Checksum256s, EosBlockHeaderJson},
        eos_utils::convert_hex_to_checksum256,
    },
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterial {
    pub block_num: u64,
    pub producer_signature: String,
    pub action_proofs: EosActionProofs,
    pub block_header: EosBlockHeaderV2,
    pub interim_block_ids: Checksum256s,
}

impl EosSubmissionMaterial {
    fn convert_schedule_json_value_to_v2_schedule_json(json_value: &JsonValue) -> Result<EosProducerScheduleV2> {
        EosProducerScheduleV2::from_json(&json_value.to_string())
    }

    fn convert_timestamp_string_to_block_timestamp(timestamp: &str) -> Result<BlockTimestamp> {
        let timestamp_format = "%Y-%m-%dT%H:%M:%S%.3f";
        Ok(BlockTimestamp::from(TimePoint::from_unix_nano_seconds(
            Utc.datetime_from_str(timestamp, timestamp_format)?.timestamp_millis() * 1_000_000,
        )))
    }

    fn parse_eos_action_proof_jsons_to_action_proofs(
        action_proof_jsons: &[EosActionProofJson],
    ) -> Result<EosActionProofs> {
        action_proof_jsons.iter().map(EosActionProof::from_json).collect()
    }

    fn parse_interim_block_ids_from_json(interim_block_ids_json: &[String]) -> Result<Checksum256s> {
        interim_block_ids_json.iter().map(convert_hex_to_checksum256).collect()
    }

    fn parse_schedule_from_block_header_json(
        eos_block_header_json: &EosBlockHeaderJson,
    ) -> Result<Option<EosProducerScheduleV2>> {
        info!("✔ Parsing EOS producer schedule from json...");
        if eos_block_header_json.new_producers.is_some() && eos_block_header_json.new_producer_schedule.is_some() {
            Err("EOS block header found with both old & new producer schedule keys!".into())
        } else if eos_block_header_json.new_producers.is_some() {
            debug!("✔ `new_producers` field in EOS block json!");
            Ok(Some(Self::convert_schedule_json_value_to_v2_schedule_json(
                eos_block_header_json
                    .new_producers
                    .as_ref()
                    .ok_or(NoneError("Could not unwrap `new_producers` field in EOS block json!"))?,
            )?))
        } else if eos_block_header_json.new_producer_schedule.is_some() {
            debug!("✔ `new_producer_schedule` field in EOS block json!");
            Ok(Some(Self::convert_schedule_json_value_to_v2_schedule_json(
                &eos_block_header_json.new_producer_schedule.clone().ok_or(NoneError(
                    "Could not unwrap `new_producer_schedule` field in EOS block json!",
                ))?,
            )?))
        } else {
            debug!("✔ No producers field in EOS block json!");
            Ok(None)
        }
    }

    pub fn parse_eos_block_header_from_json(eos_block_header_json: &EosBlockHeaderJson) -> Result<EosBlockHeaderV2> {
        let extensions = match eos_block_header_json.header_extension {
            None => vec![],
            Some(ref hex_extensions) => EosExtensions::from_hex_strings(hex_extensions)?.to_vec(),
        };
        Ok(EosBlockHeaderV2::new(
            Self::convert_timestamp_string_to_block_timestamp(&eos_block_header_json.timestamp)?,
            AccountName::from_str(&eos_block_header_json.producer)?,
            eos_block_header_json.confirmed,
            convert_hex_to_checksum256(&eos_block_header_json.previous)?,
            convert_hex_to_checksum256(&eos_block_header_json.transaction_mroot)?,
            convert_hex_to_checksum256(&eos_block_header_json.action_mroot)?,
            eos_block_header_json.schedule_version,
            Self::parse_schedule_from_block_header_json(eos_block_header_json)?,
            &extensions,
        ))
    }

    fn from_json(json: &EosSubmissionMaterialJson) -> Result<Self> {
        Ok(Self {
            block_num: json.block_header.block_num,
            producer_signature: json.block_header.producer_signature.clone(),
            block_header: Self::parse_eos_block_header_from_json(&json.block_header)?,
            interim_block_ids: Self::parse_interim_block_ids_from_json(&json.interim_block_ids)?,
            action_proofs: Self::parse_eos_action_proof_jsons_to_action_proofs(&json.action_proofs)?,
        })
    }

    pub fn from_str(s: &str) -> Result<Self> {
        info!("✔ Parsing EOS submission material...");
        EosSubmissionMaterialJson::from_str(s)
            .and_then(|json| Self::from_json(&json))
            .map(|submission_material| {
                info!(
                    "✔ EOS submission material parsed! Block number: {}",
                    submission_material.block_num
                );
                submission_material
            })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterialJson {
    pub interim_block_ids: Vec<String>,
    pub action_proofs: EosActionProofJsons,
    pub block_header: EosBlockHeaderJson,
}

impl EosSubmissionMaterialJson {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn parse_submission_material_and_add_to_state<'a, D: DatabaseInterface>(
    submission_material: &str,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    EosSubmissionMaterial::from_str(submission_material).and_then(|material| state.add_submission_material(material))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eos::eos_test_utils::{get_sample_eos_submission_material_string_n, serialize_block_header_v2},
        errors::AppError,
    };

    #[test]
    fn should_parse_eos_submission_material_string_to_json() {
        let submission_material_string = get_sample_eos_submission_material_string_n(2).unwrap();
        let result = EosSubmissionMaterialJson::from_str(&submission_material_string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_timestamp_string_to_block_timestamp() {
        let expected_result: u32 = 1192621811;
        let eos_time_stamp_string = "2018-11-23T17:55:05.500";
        let result = EosSubmissionMaterial::convert_timestamp_string_to_block_timestamp(eos_time_stamp_string)
            .unwrap()
            .as_u32();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_eos_block_header() {
        let expected_id =
            convert_hex_to_checksum256(&"04cb6d0413d124ea2df08183d579967e3e47c9853c40126f06110bb20e9330d4".to_string())
                .unwrap();
        let submission_material_string = get_sample_eos_submission_material_string_n(2).unwrap();
        let json = EosSubmissionMaterialJson::from_str(&submission_material_string).unwrap();
        let result = EosSubmissionMaterial::parse_eos_block_header_from_json(&json.block_header).unwrap();
        let id = result.id().unwrap();
        assert_eq!(id, expected_id);
    }

    #[test]
    fn should_parse_eos_submission_material_string_to_struct() {
        let submission_material_string = get_sample_eos_submission_material_string_n(2).unwrap();
        let json = EosSubmissionMaterialJson::from_str(&submission_material_string).unwrap();
        let result = EosSubmissionMaterial::from_json(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn should_parse_block_header_from_json_2() {
        // NOTE: This block === https://jungle.bloks.io/block/10800
        // NOTE: Blocks herein chosen because of repo here:
        // https://github.com/KyberNetwork/bridge_eth_smart_contracts/tree/master/test
        // Which has producer keys etc as test vectors.
        let block_id = "00002a304f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda".to_string();
        let expected_block_id = convert_hex_to_checksum256(&block_id).unwrap();
        let json = EosBlockHeaderJson {
            block_id,
            confirmed: 0,
            producer: "funnyhamster".to_string(),
            previous: "00002a2fb72da881babc192b80bab59c289e2db1b4318160a4c0ab5e50618f57".to_string(),
            block_num: 1337,
            timestamp: "2018-11-23T17:55:05.500".to_string(),
            action_mroot: "33cfa41c93d0d37dd162d1341114122d76446ec6ce5ff6686205fa15f2fe6d46".to_string(),
            schedule_version: 2,
            transaction_mroot: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            producer_signature:
                "SIG_K1_KX9Y5xYQrBYtpdKm4njsMerfzoPU6qbiW3G3RmbmbSyZ5sjE2J1U4PHC1vQ8arZQrBKqwW1adLPwYDzqt3v137GLp1ZWse"
                    .to_string(), // Ignored
            header_extension: None,
            new_producer_schedule: None,
            new_producers: None,
        };
        let result = EosSubmissionMaterial::parse_eos_block_header_from_json(&json).unwrap();
        let expected_serialized = "f3f615477055c6d2343fa75e000000002a2fb72da881babc192b80bab59c289e2db1b4318160a4c0ab5e50618f57000000000000000000000000000000000000000000000000000000000000000033cfa41c93d0d37dd162d1341114122d76446ec6ce5ff6686205fa15f2fe6d46020000000000";
        let result_serialized = hex::encode(serialize_block_header_v2(&result).unwrap());
        assert_eq!(result.id().unwrap(), expected_block_id);
        assert_eq!(result_serialized, expected_serialized);
    }

    #[test]
    fn should_parse_block_header_from_json_3() {
        // NOTE: This block === https://jungle.bloks.io/block/10801
        let block_id = "00002a31c3261813a1e737a5b821a1f318f731ff12c5dd9cc14dc2a1c661fce6".to_string();
        let expected_block_id = convert_hex_to_checksum256(&block_id).unwrap();
        let json = EosBlockHeaderJson {
            block_id,
            confirmed: 240,
            producer: "gorillapower".to_string(),
            previous: "00002a304f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda".to_string(),
            block_num: 1337,
            timestamp: "2018-11-23T17:55:06.000".to_string(),
            action_mroot: "ff146c3b50187542da35111cc9057031d1d5a6961110725cc4409e0895de572b".to_string(),
            schedule_version: 2,
            transaction_mroot: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            producer_signature:
                "SIG_K1_KAYaAyqWGxo38cxuNexehkqQEghJY5iekGj56A1v7c8Qs61v4rLgH3cFdqpQ6rLzeNcAb1xZVXsNfayiHuQKzbyC2Kr36Y"
                    .to_string(),
            header_extension: None,
            new_producer_schedule: None,
            new_producers: None,
        };
        let result = EosSubmissionMaterial::parse_eos_block_header_from_json(&json).unwrap();
        let expected_serialized = "f4f615477015a7d5c4e82e65f00000002a304f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda0000000000000000000000000000000000000000000000000000000000000000ff146c3b50187542da35111cc9057031d1d5a6961110725cc4409e0895de572b020000000000";
        let result_serialized = hex::encode(serialize_block_header_v2(&result).unwrap());
        assert_eq!(result.id().unwrap(), expected_block_id);
        assert_eq!(result_serialized, expected_serialized);
    }

    #[test]
    fn should_parse_block_header_from_json_4() {
        // NOTE: This block === https://jungle.bloks.io/block/75230993
        let block_id = "047bef11966be96d0898f76a951637367e83eb13de5f8a9e3770c5c8a32e736f".to_string();
        let expected_block_id = convert_hex_to_checksum256(&block_id).unwrap();
        let json = EosBlockHeaderJson {
            block_id,
            confirmed: 0,
            producer: "jungleswedeo".to_string(),
            previous: "047bef1059cd1da401e09bda1617bc2b58c6dfdb11d7f05db14c55f442d036ad".to_string(),
            block_num: 1337,
            timestamp: "2020-02-11T09:17:41.500".to_string(),
            action_mroot: "74ef05af4a312a8f010e3e442f3097dc33ec4b22738504ab38d8e30724f24d4b".to_string(),
            schedule_version: 379,
            transaction_mroot: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            producer_signature:
                "SIG_K1_K8S9NPR8Xv8hyi7EWT6fjy4iBYtt3F6PPxv5S5H2a9rucP8YxtZUmxeyxxsxg6HHNeNQ4JJTRKCzdqdN3drRFWDi9KJduL"
                    .to_string(),
            header_extension: None,
            new_producer_schedule: None,
            new_producers: None,
        };
        let result = EosSubmissionMaterial::parse_eos_block_header_from_json(&json).unwrap();
        let expected_serialized = "6b5baa4b4055521cabc8a67e0000047bef1059cd1da401e09bda1617bc2b58c6dfdb11d7f05db14c55f442d036ad000000000000000000000000000000000000000000000000000000000000000074ef05af4a312a8f010e3e442f3097dc33ec4b22738504ab38d8e30724f24d4b7b0100000000";
        let result_serialized = hex::encode(serialize_block_header_v2(&result).unwrap());
        assert_eq!(result.id().unwrap(), expected_block_id);
        assert_eq!(result_serialized, expected_serialized);
    }

    #[test]
    fn should_parse_submisson_material_with_action_proofs() {
        let submission_material_str = get_sample_eos_submission_material_string_n(2).unwrap();
        let result = EosSubmissionMaterial::from_str(&submission_material_str);
        assert!(result.is_ok());
    }

    #[test]
    fn should_parse_j3_block_with_new_producers_schedule_field_correctly() {
        let block_str = "{\"timestamp\":\"2020-06-11T02:45:18.000\",\"producer\":\"eosarabianet\",\"confirmed\":240,\"previous\":\"01280aa8aac7c41385233583b461d36f958c3a99b7cb4e8e076165317922a124\",\"transaction_mroot\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"action_mroot\":\"008940fda9fc47b4239d2b40484b3405e5fef28d117c5c0cf4d56a97448598a9\",\"schedule_version\":37,\"new_producers\":null,\"new_producer_schedule\":{\"version\":38,\"producers\":[{\"producer_name\":\"atticlabjbpn\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7pfLMz45bKTVqVMfnxktqi6RYjDV46C82Q5eE8NZHM9Nnsai6T\",\"weight\":1}]}]},{\"producer_name\":\"batinthedark\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6dwoM8XGMQn49LokUcLiony7JDkbHrsFDvh5svLvPDkXtvM7oR\",\"weight\":1}]}]},{\"producer_name\":\"bighornsheep\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5xfwWr4UumKm4PqUGnyCrFWYo6j5cLioNGg5yf4GgcTp2WcYxf\",\"weight\":1}]}]},{\"producer_name\":\"bigpolarbear\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6oZi9WjXUcLionUtSiKRa4iwCW5cT6oTzoWZdENXq1p2pq53Nv\",\"weight\":1}]}]},{\"producer_name\":\"clevermonkey\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5mp5wmRyL5RH2JUeEh3eoZxkJ2ZZJ9PVd1BcLioNuq4PRCZYxQ\",\"weight\":1}]}]},{\"producer_name\":\"eosarabianet\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6nrJJGhoZPShQ2T4se2RqxRh5rD2LUvqBK6r5y5VVN9x1oTBwa\",\"weight\":1}]}]},{\"producer_name\":\"eosbarcelona\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8N1MhQpFQR3YABzVp4woPBywQnS5BunJtHv8jxtNQGrGEiTBhD\",\"weight\":1}]}]},{\"producer_name\":\"eosdacserval\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH\",\"weight\":1}]}]},{\"producer_name\":\"eosnationftw\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6Fat9KYfu22yxWJuwjXeWKhCnFxj4GaCQJ7pwjLwpU8XxVzjyi\",\"weight\":1}]}]},{\"producer_name\":\"eosphereiobp\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5P7EBrzje2ZPjYfRNe9aFGvrXiXj2j9xQy3Pj4Jxh3z5P81uGr\",\"weight\":1}]}]},{\"producer_name\":\"funnyhamster\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7A9BoRetjpKtE3sqA6HRykRJ955MjQ5XdRmCLionVte2uERL8h\",\"weight\":1}]}]},{\"producer_name\":\"gorillapower\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8X5NCx1Xqa1xgQgBa9s6EK7M1SjGaDreAcLion4kDVLsjhQr9n\",\"weight\":1}]}]},{\"producer_name\":\"hippopotamus\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7qDcxm8YtAZUA3t9kxNGuzpCLioNnzpTRigi5Dwsfnszckobwc\",\"weight\":1}]}]},{\"producer_name\":\"hungryolddog\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6tw3AqqVUsCbchYRmxkPLqGct3vC63cEzKgVzLFcLionoY8YLQ\",\"weight\":1}]}]},{\"producer_name\":\"iliketurtles\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6itYvNZwhqS7cLion3xp3rLJNJAvKKegxeS7guvbBxG1XX5uwz\",\"weight\":1}]}]},{\"producer_name\":\"ivote4eosusa\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8WHzxnaVoXek6mwU7BJiBbyugeqZfb2y2SKa7mVUv8atLfbcjK\",\"weight\":1}]}]},{\"producer_name\":\"jumpingfrogs\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7oVWG413cLioNG7RU5Kv7NrPZovAdRSP6GZEG4LFUDWkgwNXHW\",\"weight\":1}]}]},{\"producer_name\":\"junglesweden\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5D1YP3nYVQvE8NPPM5a9wnqVaD54mJAHEuH9vJuNG1E2UsgbY2\",\"weight\":1}]}]},{\"producer_name\":\"lioninjungle\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7ueKyvQJpBLVjuNgLedAgJakw3bLyd4GBx1N4jXswpBhJif5mV\",\"weight\":1}]}]},{\"producer_name\":\"ohtigertiger\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7tigERwXDRuHsok212UDToxFS1joUhAxzvDUhRof8NjuvwtoHX\",\"weight\":1}]}]},{\"producer_name\":\"tokenika4tst\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6wkp1PpqQUgEA6UtgW21Zo3o1XcQeLXzcLLgKcPJhTz2aSF6fz\",\"weight\":1}]}]}]},\"producer_signature\":\"SIG_K1_KVD7iAWRSCD49MhnRXzneoHHf2sot11jbs3JayWgUkR7CYPJjGj9SHay6Dtqc4KzoQUd1VRAXA8VHmNBG66XYszVRtN5Ec\",\"transactions\":[],\"id\":\"01280aa9fe99add0e000a9d668c154948df20a3bd010dba773e5bd97943336c0\",\"block_num\":19401385,\"ref_block_prefix\":3601400032,\"block_id\":\"01280aa9fe99add0e000a9d668c154948df20a3bd010dba773e5bd97943336c0\"}".to_string();
        let block_json: EosBlockHeaderJson = serde_json::from_str(&block_str).unwrap();
        if let Err(e) = EosSubmissionMaterial::parse_eos_block_header_from_json(&block_json) {
            panic!("Error converting J3 json with new producers to block header: {}", e)
        }
    }

    #[test]
    fn should_parse_mainnet_block_with_new_producers_field_correctly() {
        let block_str = "{\"timestamp\":\"2020-06-15T11:22:18.000\",\"producer\":\"eosflytomars\",\"confirmed\":240,\"previous\":\"0784fef202972a60796f5f6c52f18f6e2fc2b5f4d846d6fd8478ffe53a0c833f\",\"transaction_mroot\":\"4d36272088b12b603b7ba4ddb60ee6954edc2100202235ef9734aaa7f84b2412\",\"action_mroot\":\"7476848441ea1372af67fca4a31909f8625edeeee541d890917c66e322a2a0b6\",\"schedule_version\":1720,\"new_producers\":{\"version\":1721,\"producers\":[{\"producer_name\":\"atticlabeosb\",\"block_signing_key\":\"EOS7PfA3A4UdfMu2wKbuXdbHn8EWAxbMnFoFWui4X2zsr2oPwdQJP\"},{\"producer_name\":\"big.one\",\"block_signing_key\":\"EOS8MpYyXwn3DLqk9Y9XTHYcd6wGGijNqJefFoQEwEoXTq1awZ42w\"},{\"producer_name\":\"bitfinexeos1\",\"block_signing_key\":\"EOS4tkw7LgtURT3dvG3kQ4D1sg3aAtPDymmoatpuFkQMc7wzZdKxc\"},{\"producer_name\":\"blockpooleos\",\"block_signing_key\":\"EOS61FDJz3GC42GhaPSsmKh7SxuesyZhjm7hBwBKqN52v1HukEqBu\"},{\"producer_name\":\"eoscannonchn\",\"block_signing_key\":\"EOS73cTi9V7PNg4ujW5QzoTfRSdhH44MPiUJkUV6m3oGwj7RX7kML\"},{\"producer_name\":\"eosdotwikibp\",\"block_signing_key\":\"EOS7RsdDs8k8GDAdZrETnTjoGwiqAwwdNyxeH8q6fmHgpHjPPnyco\"},{\"producer_name\":\"eoseouldotio\",\"block_signing_key\":\"EOS6SSA4gYCSZ3q9NWpxGsYDv5MWjSwKseyq25RRZexwj8EM6YHDa\"},{\"producer_name\":\"eosflytomars\",\"block_signing_key\":\"EOS6Agpfp38bTyRjJDmB4Qb1EpQSq7wnEAsALXgXE7KFSzKjokkFD\"},{\"producer_name\":\"eoshuobipool\",\"block_signing_key\":\"EOS5XKswW26cR5VQeDGwgNb5aixv1AMcKkdDNrC59KzNSBfnH6TR7\"},{\"producer_name\":\"eosinfstones\",\"block_signing_key\":\"EOS6CSvGzNhNxVYbcnWSuheNcfzjGeGBY9trR4YAJ4Yvakq4oCh6y\"},{\"producer_name\":\"eosiomeetone\",\"block_signing_key\":\"EOS5gS4ZtanRS2Jx4vpjAQaVNci3v65iZiGCgMr9DNwu67x2pt8Qd\"},{\"producer_name\":\"eosiosg11111\",\"block_signing_key\":\"EOS7zVBQMhV7dZ5zRQwBgDmmbFCHA6YcmwW6Dq5CePGpqLR1ZsVAc\"},{\"producer_name\":\"eoslaomaocom\",\"block_signing_key\":\"EOS8QgURqo875qu3a8vgZ58qBeu2cTehe9zAWRfpdCXAQipicu1Fi\"},{\"producer_name\":\"eosnationftw\",\"block_signing_key\":\"EOS8L12yBrtx7mpewHmjwgJeNb2aLaeQdoDgMW82dzDSu17ec2XNL\"},{\"producer_name\":\"eosrapidprod\",\"block_signing_key\":\"EOS8QEFsgUWj7BscQNkiremtpSoRkzwDqmCPpKKCHYXGNaqxXFQ4h\"},{\"producer_name\":\"hashfineosio\",\"block_signing_key\":\"EOS7jSfvStvbKDmGvQdtrQsCyNkWczXfvh6CHmBVmeypJyHsUrMqj\"},{\"producer_name\":\"helloeoscnbp\",\"block_signing_key\":\"EOS79cHpaEittzgJWgj79tdRhgzLEWy8wXmmQ3fL7kkDjmYYiGNet\"},{\"producer_name\":\"newdex.bp\",\"block_signing_key\":\"EOS688SnH8tQ7NiyhamiCzWXAGPDLF9S7K8ga79UBHKFgjS1MhqhB\"},{\"producer_name\":\"okcapitalbp1\",\"block_signing_key\":\"EOS6NqWZ1i9KSNoeBiby6Nmf1seAbEfhvrDoCbwSi1hV4cuqqnYRP\"},{\"producer_name\":\"starteosiobp\",\"block_signing_key\":\"EOS4wZZXm994byKANLuwHD6tV3R3Mu3ktc41aSVXCBaGnXJZJ4pwF\"},{\"producer_name\":\"whaleex.com\",\"block_signing_key\":\"EOS88EGcFghfQJER1mDaEe4kDJ7MGDoPmXQfA7q2QMTLLqiYP1UQR\"}]},\"producer_signature\":\"SIG_K1_KfqhgnM8BmBUYHW6qZGGwDaTvmGmZfe44bdummNvLa8e5jMWToCUvKZ8QeAG7jBKjAw2bSQhRRNFeUZ1Zv3A51kVA3Ly2R\",\"transactions\":[],\"block_id\":\"0784fef384c51b170036b5e08a4e56c00a31c18406ac3a75b6494f812680868a\",\"block_num\":126156531,\"ref_block_prefix\":3769972224}".to_string();
        let block_json: EosBlockHeaderJson = serde_json::from_str(&block_str).unwrap();
        let result = EosSubmissionMaterial::parse_eos_block_header_from_json(&block_json);
        assert!(result.is_ok());
    }

    #[test]
    fn should_error_when_parsing_schedule_from_block_header_json_if_old_and_new_schedules_present() {
        let block_str = "{\"timestamp\":\"2020-06-11T02:45:18.000\",\"producer\":\"eosarabianet\",\"confirmed\":240,\"previous\":\"01280aa8aac7c41385233583b461d36f958c3a99b7cb4e8e076165317922a124\",\"transaction_mroot\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"action_mroot\":\"008940fda9fc47b4239d2b40484b3405e5fef28d117c5c0cf4d56a97448598a9\",\"schedule_version\":37,\"new_producers\":{},\"new_producer_schedule\":{\"version\":38,\"producers\":[{\"producer_name\":\"atticlabjbpn\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7pfLMz45bKTVqVMfnxktqi6RYjDV46C82Q5eE8NZHM9Nnsai6T\",\"weight\":1}]}]},{\"producer_name\":\"batinthedark\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6dwoM8XGMQn49LokUcLiony7JDkbHrsFDvh5svLvPDkXtvM7oR\",\"weight\":1}]}]},{\"producer_name\":\"bighornsheep\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5xfwWr4UumKm4PqUGnyCrFWYo6j5cLioNGg5yf4GgcTp2WcYxf\",\"weight\":1}]}]},{\"producer_name\":\"bigpolarbear\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6oZi9WjXUcLionUtSiKRa4iwCW5cT6oTzoWZdENXq1p2pq53Nv\",\"weight\":1}]}]},{\"producer_name\":\"clevermonkey\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5mp5wmRyL5RH2JUeEh3eoZxkJ2ZZJ9PVd1BcLioNuq4PRCZYxQ\",\"weight\":1}]}]},{\"producer_name\":\"eosarabianet\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6nrJJGhoZPShQ2T4se2RqxRh5rD2LUvqBK6r5y5VVN9x1oTBwa\",\"weight\":1}]}]},{\"producer_name\":\"eosbarcelona\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8N1MhQpFQR3YABzVp4woPBywQnS5BunJtHv8jxtNQGrGEiTBhD\",\"weight\":1}]}]},{\"producer_name\":\"eosdacserval\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH\",\"weight\":1}]}]},{\"producer_name\":\"eosnationftw\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6Fat9KYfu22yxWJuwjXeWKhCnFxj4GaCQJ7pwjLwpU8XxVzjyi\",\"weight\":1}]}]},{\"producer_name\":\"eosphereiobp\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5P7EBrzje2ZPjYfRNe9aFGvrXiXj2j9xQy3Pj4Jxh3z5P81uGr\",\"weight\":1}]}]},{\"producer_name\":\"funnyhamster\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7A9BoRetjpKtE3sqA6HRykRJ955MjQ5XdRmCLionVte2uERL8h\",\"weight\":1}]}]},{\"producer_name\":\"gorillapower\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8X5NCx1Xqa1xgQgBa9s6EK7M1SjGaDreAcLion4kDVLsjhQr9n\",\"weight\":1}]}]},{\"producer_name\":\"hippopotamus\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7qDcxm8YtAZUA3t9kxNGuzpCLioNnzpTRigi5Dwsfnszckobwc\",\"weight\":1}]}]},{\"producer_name\":\"hungryolddog\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6tw3AqqVUsCbchYRmxkPLqGct3vC63cEzKgVzLFcLionoY8YLQ\",\"weight\":1}]}]},{\"producer_name\":\"iliketurtles\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6itYvNZwhqS7cLion3xp3rLJNJAvKKegxeS7guvbBxG1XX5uwz\",\"weight\":1}]}]},{\"producer_name\":\"ivote4eosusa\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS8WHzxnaVoXek6mwU7BJiBbyugeqZfb2y2SKa7mVUv8atLfbcjK\",\"weight\":1}]}]},{\"producer_name\":\"jumpingfrogs\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7oVWG413cLioNG7RU5Kv7NrPZovAdRSP6GZEG4LFUDWkgwNXHW\",\"weight\":1}]}]},{\"producer_name\":\"junglesweden\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS5D1YP3nYVQvE8NPPM5a9wnqVaD54mJAHEuH9vJuNG1E2UsgbY2\",\"weight\":1}]}]},{\"producer_name\":\"lioninjungle\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7ueKyvQJpBLVjuNgLedAgJakw3bLyd4GBx1N4jXswpBhJif5mV\",\"weight\":1}]}]},{\"producer_name\":\"ohtigertiger\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS7tigERwXDRuHsok212UDToxFS1joUhAxzvDUhRof8NjuvwtoHX\",\"weight\":1}]}]},{\"producer_name\":\"tokenika4tst\",\"authority\":[0,{\"threshold\":1,\"keys\":[{\"key\":\"EOS6wkp1PpqQUgEA6UtgW21Zo3o1XcQeLXzcLLgKcPJhTz2aSF6fz\",\"weight\":1}]}]}]},\"producer_signature\":\"SIG_K1_KVD7iAWRSCD49MhnRXzneoHHf2sot11jbs3JayWgUkR7CYPJjGj9SHay6Dtqc4KzoQUd1VRAXA8VHmNBG66XYszVRtN5Ec\",\"transactions\":[],\"id\":\"01280aa9fe99add0e000a9d668c154948df20a3bd010dba773e5bd97943336c0\",\"block_num\":19401385,\"ref_block_prefix\":3601400032,\"block_id\":\"01280aa9fe99add0e000a9d668c154948df20a3bd010dba773e5bd97943336c0\"}".to_string();
        let block_json: EosBlockHeaderJson = serde_json::from_str(&block_str).unwrap();
        let expected_error = "EOS block header found with both old & new producer schedule keys!";
        match EosSubmissionMaterial::parse_eos_block_header_from_json(&block_json) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
