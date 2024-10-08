#![cfg(all(test, not(feature = "spring_1-0")))]
use std::fs::read_to_string;

use common::dictionaries::eos_eth::{
    EosEthTokenDictionary,
    EosEthTokenDictionaryEntry,
    EosEthTokenDictionaryEntryJson,
};

pub fn get_init_block() -> String {
    read_to_string("src/test_utils/multi_incremerkle_submission/init-block.json").unwrap()
}

pub fn get_submission_block() -> String {
    read_to_string("src/test_utils/multi_incremerkle_submission/submission-material.json").unwrap()
}

pub fn get_incremekle_update_block() -> String {
    read_to_string("src/test_utils/multi_incremerkle_submission/incremerkle-update-block.json").unwrap()
}

pub fn get_sample_dictionary() -> EosEthTokenDictionary {
    EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_json(
        &EosEthTokenDictionaryEntryJson {
            eth_token_decimals: 18,
            eos_token_decimals: 4,
            eth_symbol: "PWOMBAT".to_string(),
            eos_symbol: "PWOMBAT".to_string(),
            eth_address: "0xb048a1f2d0c839002ee7f7bdc2049c2142f264d6".to_string(),
            eos_address: "wmbt.ptokens".to_string(),
            eth_fee_basis_points: None,
            eos_fee_basis_points: None,
            accrued_fees: None,
            last_withdrawal: None,
        },
    )
    .unwrap()])
}
