use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result, AppError};
use common_eth::{
    check_for_parent_of_eth_block_in_state,
    maybe_add_eth_block_and_receipts_to_db_and_return_state,
    maybe_increment_int_account_nonce_and_return_eth_state,
    maybe_remove_old_eth_tail_block_and_return_state,
    maybe_remove_receipts_from_eth_canon_block_and_return_state,
    maybe_update_eth_canon_block_hash_and_return_state,
    maybe_update_eth_linker_hash_and_return_state,
    maybe_update_eth_tail_block_hash_and_return_state,
    maybe_update_latest_eth_block_hash_and_return_state,
    parse_eth_submission_material_json_and_put_in_state,
    validate_eth_block_in_state,
    validate_receipts_in_state,
    EthState,
    EthSubmissionMaterialJson,
    EthSubmissionMaterialJsons,
};

use crate::eth::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::{
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_vault_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
    },
    filter_submission_material::filter_submission_material_for_peg_in_events_in_state,
    filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
    filter_zero_value_tx_infos::filter_out_zero_value_evm_tx_infos_from_state,
    get_eth_output_json::{get_eth_output_json, EthOutput, EthOutputs},
    parse_tx_info::maybe_parse_tx_info_from_canon_block_and_add_to_state,
    sign_txs::maybe_sign_int_txs_and_add_to_eth_state,
};

fn submit_eth_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<EthOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_eth_block_in_state)
        .and_then(|state| state.get_eth_evm_token_dictionary_and_add_to_state())
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(filter_out_zero_value_evm_tx_infos_from_state)
        .and_then(maybe_account_for_fees)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(maybe_sign_int_txs_and_add_to_eth_state)
        .and_then(maybe_increment_int_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(get_eth_output_json)
}

/// # Submit ETH Block to Core
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_eth_block_to_core<D: DatabaseInterface>(db: &D, block: &str) -> Result<String> {
    info!("✔ Submitting INT block to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJson::from_str(block))
        .and_then(|json| submit_eth_block(db, &json))
        .and_then(|output| {
            db.end_transaction()?;
            Ok(output.to_string())
        })
}

/// # Submit ETH Blocks to Core
///
/// Submit multiple ETH blocks to the core.
pub fn submit_eth_blocks_to_core<D: DatabaseInterface>(db: &D, blocks: &str) -> Result<String> {
    info!("✔ Batch submitting INT blocks to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJsons::from_str(blocks))
        .and_then(|jsons| {
            let mut outputs = vec![];

            for json in jsons.iter() {
                match submit_eth_block(db, json) {
                    Ok(o) => {
                        outputs.push(o);
                        continue;
                    },
                    Err(AppError::BlockAlreadyInDbError(e)) => {
                        warn!("block already in db error: {e}");
                        info!("moving on to next block in batch!");
                        continue;
                    },
                    Err(e) => return Err(e),
                }
            }

            Ok(outputs)
        })
        .map(EthOutputs::new)
        .and_then(|outputs| {
            db.end_transaction()?;
            Ok(outputs.to_output().to_string())
        })
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use common::{dictionaries::eth_evm::EthEvmTokenDictionary, test_utils::get_test_database};
    use common_chain_ids::EthChainId;
    use common_eth::{
        convert_hex_to_eth_address,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        initialize_evm_core_with_no_contract_tx,
        parse_eth_submission_material_and_put_in_state,
        EthDbUtilsExt,
        EthPrivateKey,
        EvmDbUtils,
        VaultUsingCores,
    };
    use common_eth_debug::reset_eth_chain;
    use serde_json::json;

    use super::*;
    use crate::{
        eth::get_eth_output_json::EthOutput,
        test_utils::{
            get_sample_eth_init_block_json_string,
            get_sample_goerli_peg_in_init_block_json_string,
            get_sample_goerli_peg_in_submission_string,
            get_sample_goerli_token_dictionary_entry,
            get_sample_int_init_block_json_string,
            get_sample_peg_in_1_submission_string,
            get_sample_sepolia_init_block_json_string,
            get_sample_sepolia_peg_in_submission_string,
            get_sample_sepolia_token_dictionary_entry,
            get_sample_token_dictionary_entry,
        },
    };
    #[test]
    fn should_submit_eth_block_successfully() {
        let eth_chain_id = EthChainId::Rinkeby;
        let db = get_test_database();
        let router_address = convert_hex_to_eth_address("0x0e1c8524b1D1891B201ffC7BB58a82c96f8Fc4F6").unwrap();
        let vault_address = convert_hex_to_eth_address("0x866e3fC7043EFb8ff3A994F7d59F53fe045d4d7A").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        let eth_init_block = get_sample_eth_init_block_json_string();
        let int_init_block = get_sample_int_init_block_json_string();
        // NOTE: Initialize the ETH side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &eth_init_block,
            &eth_chain_id,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::Erc20OnInt,
            true, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Initialize the INT side of the core...
        initialize_evm_core_with_no_contract_tx(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("8549cf9b30276305de31fa7533938e7ce366d12a").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("d22ecd05f55019604c5484bdb55d6c78c631cd7a05cc31781900ce356186617e").unwrap(),
        )
        .unwrap();
        let db_utils = EvmDbUtils::new(&db);
        // NOTE: Overwrite the nonce since the test sample used the 3rd nonce...
        let evm_nonce = 2;
        db_utils.put_eth_account_nonce_in_db(evm_nonce).unwrap();
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), evm_nonce);
        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address,);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key,);
        let is_for_eth = true;
        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry(), &db)
            .unwrap();
        // NOTE: Bring the chain up to the block prior to the block containing a peg-in...
        reset_eth_chain(
            parse_eth_submission_material_and_put_in_state(
                &read_to_string("src/test_utils/eth-before-peg-in-1-block.json").unwrap(),
                EthState::init(&db),
            )
            .unwrap(),
            confirmations,
            is_for_eth,
        )
        .unwrap();
        let submission_string = get_sample_peg_in_1_submission_string();
        // NOTE: Finally, submit the block containting the peg in....
        let output = submit_eth_block_to_core(&db, &submission_string).unwrap();
        let expected_result_json = json!({
            "eth_latest_block_number": 9750222,
            "int_signed_transactions": [
                {
                    "_id": "perc20-on-int-int-2",
                    "broadcast": false,
                    "int_tx_hash": "0x48ced47886e05c775d39506bc39da2e0324cfd14eb4649f8e9a19856040389f7",
                    "int_tx_amount": "1336",
                    "int_tx_recipient": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                    "witnessed_timestamp": 1638537255,
                    "host_token_address": "0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                    "originating_tx_hash": "0xf691d432fe940b2ecef70b6c9069ae124af9d160d761252d7ca570f5cd443dd4",
                    "originating_address": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                    "native_token_address": "0xc63ab9437f5589e2c67e04c00a98506b43127645",
                    "int_signed_tx": "f9036b028504a817c8008306ddd094a83446f219baec0b6fd6b3031c5a49a54543045b80b90304dcdc7dd00000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60000000000000000000000000000000000000000000000000000000000000538000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002e000000000000000000000000000000000000000000000000000000000000002400300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000f343680000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001400069c3220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003c0ffee0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786665646665323631366562333636316362386665643237383266356630636339316435396463616300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a307866656466653236313665623336363163623866656432373832663566306363393164353964636163000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0ecf943fb39a073c5453c981fd3c3a2651857959990bb9b0e40dc6f15b3a3eab7a067b8fa3ff4154694c904251caa716096c2ecac9b3ae874d789b231c854f51726",
                    "any_sender_nonce": null,
                    "int_account_nonce": 2,
                    "int_latest_block_number": 11544277,
                    "broadcast_tx_hash": null,
                    "broadcast_timestamp": null,
                    "any_sender_tx": null,
                    "destination_chain_id": "0x0069c322",
                }
            ]
        });
        let expected_result = EthOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = EthOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_submit_goerli_block_successfully() {
        let eth_chain_id = EthChainId::Goerli;
        let db = get_test_database();
        let router_address = convert_hex_to_eth_address("0x0e1c8524b1D1891B201ffC7BB58a82c96f8Fc4F6").unwrap();
        let vault_address = convert_hex_to_eth_address("0x711C50B31eE0B9e8ed4D434819AC20b4fBBb5532").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        let goerli_init_block = get_sample_goerli_peg_in_init_block_json_string();
        let int_init_block = get_sample_int_init_block_json_string();
        // NOTE: Initialize the ETH side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &goerli_init_block,
            &eth_chain_id,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::Erc20OnInt,
            true, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Initialize the INT side of the core...
        initialize_evm_core_with_no_contract_tx(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("8549cf9b30276305de31fa7533938e7ce366d12a").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("d22ecd05f55019604c5484bdb55d6c78c631cd7a05cc31781900ce356186617e").unwrap(),
        )
        .unwrap();
        let db_utils = EvmDbUtils::new(&db);

        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address,);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key,);

        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_goerli_token_dictionary_entry(), &db)
            .unwrap();

        let submission_string = get_sample_goerli_peg_in_submission_string();
        get_sample_goerli_peg_in_submission_string();
        // NOTE: Finally, submit the block containting the peg in....
        let output = submit_eth_block_to_core(&db, &submission_string).unwrap();
        let expected_result_json = json!({
            "eth_latest_block_number": 7463642,
            "int_signed_transactions": [{
                "_id":"perc20-on-int-int-0",
                "broadcast":false,
                "int_tx_hash":"0x4d23e1bdba422eee5443b36ca60c3e73698f1521ef80eca9bd66db299e74b7cb",
                "int_tx_amount":"1336",
                "int_tx_recipient":"0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC",
                "witnessed_timestamp":1661344865,
                "host_token_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "originating_tx_hash":"0xe9c9e2297cb5979904fb68c9593ff2cd2a07572e35dce0cfa03dcc0f041da50d",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address":"0x5eb802abe474290aacc1ef2786431e1ff6c03191",
                "destination_chain_id":"0x00e4b170",
                "int_signed_tx":"f9036b808504a817c8008306ddd094a83446f219baec0b6fd6b3031c5a49a54543045b80b90304dcdc7dd00000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60000000000000000000000000000000000000000000000000000000000000538000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002e000000000000000000000000000000000000000000000000000000000000002400300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000b4f6c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014000e4b1700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003c0ffee0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786665646665323631366562333636316362386665643237383266356630636339316435396463616300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a3078666564666532363136656233363631636238666564323738326635663063633931643539646361630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029a067949d973c114bcb70e075ed1a34d0423f5c584f69d67aa79eaa6b60ef57c777a04a0f587c0bda107a5e2cf53cccf442b60413d4be85bde7b6d069f08274fa2746",
                "any_sender_nonce":null,
                "int_account_nonce":0,
                "int_latest_block_number":11544277,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null,
                "any_sender_tx":null,
            }]
        });
        let expected_result = EthOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = EthOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_submit_sepolia_block_successfully() {
        let db = get_test_database();
        let router_address = convert_hex_to_eth_address("0x0e1c8524b1D1891B201ffC7BB58a82c96f8Fc4F6").unwrap();
        let vault_address = convert_hex_to_eth_address("0x97B8CAA7aCe2daA7995bF679f5dA30aF187897DE").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        let int_init_block = get_sample_int_init_block_json_string();
        // NOTE: Initialize the ETH side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_sepolia_init_block_json_string(),
            &EthChainId::Sepolia,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::Erc20OnInt,
            true, // NOTE is_native
        )
        .unwrap();
        // NOTE: Initialize the INT side of the core...
        initialize_evm_core_with_no_contract_tx(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE is_native
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("8549cf9b30276305de31fa7533938e7ce366d12a").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("d22ecd05f55019604c5484bdb55d6c78c631cd7a05cc31781900ce356186617e").unwrap(),
        )
        .unwrap();
        let db_utils = EvmDbUtils::new(&db);

        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address,);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key,);

        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_sepolia_token_dictionary_entry(), &db)
            .unwrap();

        // NOTE: Finally, submit the block containting the peg in....
        let output = submit_eth_block_to_core(&db, &get_sample_sepolia_peg_in_submission_string()).unwrap();
        let expected_result_json = json!({
            "eth_latest_block_number": 1756230,
            "int_signed_transactions": [{
                "_id":"perc20-on-int-int-0",
                "broadcast":false,
                "int_tx_hash":"0x789b38f40824f6a162c7b1cbae8d5a66a871d891783cda97f399f21d7dcb54b2",
                "int_tx_amount":"1336",
                "int_tx_recipient":"0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC",
                "witnessed_timestamp":1661442375,
                "host_token_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "originating_tx_hash":"0x10cbe65e33d7a145b0807de618f8b638d34aaba307725d468dd3e516e9f24d85",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address":"0x5eb802abe474290aacc1ef2786431e1ff6c03191",
                "destination_chain_id":"0x00e4b170",
                "int_signed_tx":"f9036b808504a817c8008306ddd094a83446f219baec0b6fd6b3031c5a49a54543045b80b90304dcdc7dd00000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60000000000000000000000000000000000000000000000000000000000000538000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000240030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000030d6b500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014000e4b1700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003c0ff330000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786665646665323631366562333636316362386665643237383266356630636339316435396463616300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a3078666564666532363136656233363631636238666564323738326635663063633931643539646361630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029a0291b447edbcbd36347501a6e20ce7cac471330100fd88b8a34fc1a326851c9aea04a217c3f9b01ae1d6ada699e70e091eca18d4f68ac19d46c26d395b72de20a7c",
                "any_sender_nonce":null,
                "int_account_nonce":0,
                "int_latest_block_number":11544277,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null,
                "any_sender_tx":null
            }]
        });
        let expected_result = EthOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = EthOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_batch_submit_evm_blocks_successfully_even_if_one_already_in_db() {
        let db = get_test_database();
        let router_address = convert_hex_to_eth_address("0x0e1c8524b1D1891B201ffC7BB58a82c96f8Fc4F6").unwrap();
        let vault_address = convert_hex_to_eth_address("0x97B8CAA7aCe2daA7995bF679f5dA30aF187897DE").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        let int_init_block = get_sample_int_init_block_json_string();
        // NOTE: Initialize the ETH side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_sepolia_init_block_json_string(),
            &EthChainId::Sepolia,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::Erc20OnInt,
            true, // NOTE is_native
        )
        .unwrap();
        // NOTE: Initialize the INT side of the core...
        initialize_evm_core_with_no_contract_tx(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE is_native
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("8549cf9b30276305de31fa7533938e7ce366d12a").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("d22ecd05f55019604c5484bdb55d6c78c631cd7a05cc31781900ce356186617e").unwrap(),
        )
        .unwrap();
        let db_utils = EvmDbUtils::new(&db);

        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address,);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key,);

        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_sepolia_token_dictionary_entry(), &db)
            .unwrap();

        let submission_string = get_sample_sepolia_peg_in_submission_string();
        let block_num = 1756230;

        // NOTE: This totally normal submission should succeed
        submit_eth_block_to_core(&db, &submission_string).unwrap();

        // NOTE: However it will fail a second time due to the block already being in the db...
        match submit_eth_block_to_core(&db, &submission_string) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(AppError::BlockAlreadyInDbError(e)) => assert_eq!(e.block_num, block_num),
            Err(e) => panic!("wrong error received: {e}"),
        };

        // NOTE: However if the same block forms part of a _batch_ of blocks, the
        // `BlockAlreadyInDbError` should be swallowed, and thus no errors.
        let batch = format!("[{submission_string},{submission_string},{submission_string}]");
        let result = submit_eth_blocks_to_core(&db, &batch);
        assert!(result.is_ok());
    }
}
