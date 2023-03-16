#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use common::AppError;
use common::{traits::DatabaseInterface, types::Result};
use common_safe_addresses::{SAFE_ETH_ADDRESS, SAFE_EVM_ADDRESS};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{eth_constants::ETH_TAIL_LENGTH, eth_database_utils::EthDbUtilsExt};

macro_rules! make_enclave_state_struct {
    ($name:ident, $prefix:ident) => {
        paste! {
            #[cfg(test)]
            impl FromStr for $name {
                type Err = AppError;

                fn from_str(s: &str) -> Result<Self> {
                    Ok(serde_json::from_str(&s)?)
                }
            }

            #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
            pub struct $name {
                [<$prefix:lower _gas_price>]: u64,
                [<$prefix:lower _chain_id>]: u64,
                [<$prefix:lower _address>]: String,
                [<$prefix:lower _tail_length>]: u64,
                any_sender_nonce: u64,
                [<$prefix:lower _account_nonce>]: u64,
                [<$prefix:lower _linker_hash>]: String,
                [<$prefix:lower _safe_address>]: String,
                [<$prefix:lower _tail_block_hash>]: String,
                [<$prefix:lower _canon_to_tip_length>]: u64,
                [<$prefix:lower _tail_block_number>]: usize,
                [<$prefix:lower _canon_block_hash>]: String,
                [<$prefix:lower _anchor_block_hash>]: String,
                [<$prefix:lower _latest_block_hash>]: String,
                [<$prefix:lower _canon_block_number>]: usize,
                [<$prefix:lower _anchor_block_number>]: usize,
                [<$prefix:lower _latest_block_number>]: usize,
                [<$prefix:lower _core_is_validating>]: bool,
                smart_contract_address: String,
                router_contract_address: String,
                erc777_proxy_contract_address: String,
            }

            impl $name {
                pub fn new<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
                    db_utils: &E,
                    contract_address: &EthAddress,
                    router_address: Option<EthAddress>,
                ) -> Result<Self> {
                    info!("✔ Getting {} enclave state...", if db_utils.get_is_for_eth() { "ETH" } else { "EVM "});
                    let [<$prefix:lower _tail_block>] = db_utils.get_eth_tail_block_from_db()?;
                    let [<$prefix:lower _canon_block>] = db_utils.get_eth_canon_block_from_db()?;
                    let [<$prefix:lower _anchor_block>] = db_utils.get_eth_anchor_block_from_db()?;
                    let [<$prefix:lower _latest_block>] = db_utils.get_eth_latest_block_from_db()?;
                    let safe_address = if stringify!($prefix:lower) == "native" || stringify!($prefix:lower) == "eth" {
                        hex::encode(SAFE_ETH_ADDRESS.as_bytes())
                    } else {
                        hex::encode(SAFE_EVM_ADDRESS.as_bytes())
                    };
                    Ok(Self {
                        [<$prefix:lower _safe_address>]:
                            safe_address,
                        [<$prefix:lower _tail_length>]:
                            ETH_TAIL_LENGTH,
                        any_sender_nonce:
                            db_utils.get_any_sender_nonce_from_db()?,
                        [<$prefix:lower _gas_price>]:
                            db_utils.get_eth_gas_price_from_db()?,
                        smart_contract_address:
                            hex::encode(contract_address.as_bytes()),
                        [<$prefix:lower _chain_id>]:
                            db_utils.get_eth_chain_id_from_db()?.to_u64(),
                        [<$prefix:lower _account_nonce>]:
                            db_utils.get_eth_account_nonce_from_db()?,
                        [<$prefix:lower _canon_to_tip_length>]:
                            db_utils.get_eth_canon_to_tip_length_from_db()?,
                        [<$prefix:lower _tail_block_number>]:
                            [<$prefix:lower _tail_block>].get_block_number()?.as_usize(),
                        [<$prefix:lower _canon_block_number>]:
                            [<$prefix:lower _canon_block>].get_block_number()?.as_usize(),
                        [<$prefix _anchor_block_number>]:
                            [<$prefix:lower _anchor_block>].get_block_number()?.as_usize(),
                        [<$prefix:lower _latest_block_number>]:
                            [<$prefix:lower _latest_block>].get_block_number()?.as_usize(),
                        [<$prefix:lower _address>]:
                            hex::encode(db_utils.get_public_eth_address_from_db()?.as_bytes()),
                        [<$prefix:lower _tail_block_hash>]:
                            hex::encode([<$prefix:lower _tail_block>].get_block_hash()?.as_bytes()),
                        erc777_proxy_contract_address:
                            hex::encode(db_utils.get_erc777_proxy_contract_address_from_db()?),
                        [<$prefix:lower _canon_block_hash>]:
                            hex::encode([<$prefix:lower _canon_block>].get_block_hash()?.as_bytes()),
                        [<$prefix:lower _linker_hash>]:
                            hex::encode(db_utils.get_linker_hash_or_genesis_hash()?.as_bytes()),
                        [<$prefix:lower _anchor_block_hash>]:
                            hex::encode([<$prefix _anchor_block>].get_block_hash()?.as_bytes()),
                        [<$prefix:lower _latest_block_hash>]:
                            hex::encode([<$prefix _latest_block>].get_block_hash()?.as_bytes()),
                        [<$prefix:lower _core_is_validating>]: !cfg!(feature="non-validating"),
                        router_contract_address: match router_address {
                            Some(address) => hex::encode(address.as_bytes()),
                            None => hex::encode(EthAddress::zero().as_bytes()),
                        }
                    })
                }
            }
        }
    };
}

make_enclave_state_struct!(EthEnclaveState, eth);
make_enclave_state_struct!(EvmEnclaveState, evm);
make_enclave_state_struct!(NativeCoreState, native);
make_enclave_state_struct!(HostCoreState, host);

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;
    use common_chain_ids::EthChainId;

    use super::*;
    use crate::{test_utils::get_sample_eth_submission_material_n, EthDbUtils, EvmDbUtils};

    fn test_enclave_state<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) {
        let submission_material = get_sample_eth_submission_material_n(0).unwrap();
        let block_hash = submission_material.block.as_ref().unwrap().hash;
        let any_sender_nonce = 666;
        let eth_account_nonce = 555;
        let eth_canon_to_tip_length = 20;
        let gas_price = 1337;
        let chain_id = EthChainId::Mainnet;
        let eth_address = EthAddress::from_slice(&hex::decode("1aD91ee08f21bE3dE0BA2ba6918E714dA6B45836").unwrap());
        db_utils.put_eth_canon_block_in_db(&submission_material).unwrap();
        db_utils.put_eth_tail_block_hash_in_db(&block_hash).unwrap();
        db_utils.put_eth_anchor_block_hash_in_db(&block_hash).unwrap();
        db_utils.put_eth_latest_block_hash_in_db(&block_hash).unwrap();
        db_utils.put_eth_gas_price_in_db(gas_price).unwrap();
        db_utils.put_eth_chain_id_in_db(&chain_id).unwrap();
        db_utils.put_public_eth_address_in_db(&eth_address).unwrap();
        db_utils.put_any_sender_nonce_in_db(any_sender_nonce).unwrap();
        db_utils.put_eth_account_nonce_in_db(eth_account_nonce).unwrap();
        db_utils
            .put_eth_canon_to_tip_length_in_db(eth_canon_to_tip_length)
            .unwrap();
        let router_address = None;
        if db_utils.get_is_for_eth() {
            let expected_result_str = format!("{{\"eth_gas_price\":1337,\"eth_chain_id\":1,\"eth_address\":\"1ad91ee08f21be3de0ba2ba6918e714da6b45836\",\"eth_tail_length\":{ETH_TAIL_LENGTH},\"any_sender_nonce\":666,\"eth_account_nonce\":555,\"eth_linker_hash\":\"7eb2e65416dd107602495454d1ed094ae475cff2f3bfb2e2ae68a1c52bc0d66f\",\"eth_safe_address\":\"71a440ee9fa7f99fb9a697e96ec7839b8a1643b8\",\"eth_tail_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"eth_canon_to_tip_length\":20,\"eth_tail_block_number\":8503804,\"eth_canon_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"eth_anchor_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"eth_latest_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"eth_canon_block_number\":8503804,\"eth_anchor_block_number\":8503804,\"eth_latest_block_number\":8503804,\"eth_core_is_validating\":true,\"smart_contract_address\":\"1ad91ee08f21be3de0ba2ba6918e714da6b45836\",\"router_contract_address\":\"0000000000000000000000000000000000000000\",\"erc777_proxy_contract_address\":\"0000000000000000000000000000000000000000\"}}");
            let mut expected_result = EthEnclaveState::from_str(&expected_result_str).unwrap();
            expected_result.eth_core_is_validating = !cfg!(feature = "non-validating");
            let result = EthEnclaveState::new(db_utils, &eth_address, router_address).unwrap();
            assert_eq!(result, expected_result);
        } else {
            let expected_result_str = format!("{{\"evm_gas_price\":1337,\"evm_chain_id\":1,\"evm_address\":\"1ad91ee08f21be3de0ba2ba6918e714da6b45836\",\"evm_tail_length\":{ETH_TAIL_LENGTH},\"any_sender_nonce\":666,\"evm_account_nonce\":555,\"evm_linker_hash\":\"7eb2e65416dd107602495454d1ed094ae475cff2f3bfb2e2ae68a1c52bc0d66f\",\"evm_safe_address\":\"71a440ee9fa7f99fb9a697e96ec7839b8a1643b8\",\"evm_tail_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"evm_canon_to_tip_length\":20,\"evm_tail_block_number\":8503804,\"evm_canon_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"evm_anchor_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"evm_latest_block_hash\":\"b626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0\",\"evm_canon_block_number\":8503804,\"evm_anchor_block_number\":8503804,\"evm_latest_block_number\":8503804,\"evm_core_is_validating\":true,\"smart_contract_address\":\"1ad91ee08f21be3de0ba2ba6918e714da6b45836\",\"router_contract_address\":\"0000000000000000000000000000000000000000\",\"erc777_proxy_contract_address\":\"0000000000000000000000000000000000000000\"}}");
            let mut expected_result = EvmEnclaveState::from_str(&expected_result_str).unwrap();
            expected_result.evm_core_is_validating = !cfg!(feature = "non-validating");
            let result = EvmEnclaveState::new(db_utils, &eth_address, router_address).unwrap();
            assert_eq!(result, expected_result);
        }
    }

    #[test]
    fn should_test_eth_enclave_state() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        test_enclave_state(&db_utils);
    }

    #[test]
    fn should_test_evm_enclave_state() {
        let db = get_test_database();
        let db_utils = EvmDbUtils::new(&db);
        test_enclave_state(&db_utils);
    }
}
