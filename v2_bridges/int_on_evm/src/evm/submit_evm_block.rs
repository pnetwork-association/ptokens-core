use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result, AppError};
use common_eth::{
    check_for_parent_of_evm_block_in_state,
    maybe_add_evm_block_and_receipts_to_db_and_return_state,
    maybe_increment_int_account_nonce_and_return_eth_state,
    maybe_remove_old_evm_tail_block_and_return_state,
    maybe_remove_receipts_from_evm_canon_block_and_return_state,
    maybe_update_evm_canon_block_hash_and_return_state,
    maybe_update_evm_linker_hash_and_return_state,
    maybe_update_evm_tail_block_hash_and_return_state,
    maybe_update_latest_evm_block_hash_and_return_state,
    parse_eth_submission_material_json_and_put_in_state,
    validate_evm_block_in_state,
    validate_receipts_in_state,
    EthState,
    EthSubmissionMaterialJson,
    EthSubmissionMaterialJsons,
};

use crate::evm::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::{
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_vault_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
    },
    filter_submission_material::filter_submission_material_for_redeem_events_in_state,
    filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
    filter_zero_value_tx_infos::filter_out_zero_value_eth_tx_infos_from_state,
    get_evm_output_json::{get_evm_output_json, EvmOutput, EvmOutputs},
    parse_tx_infos::maybe_parse_tx_info_from_canon_block_and_add_to_state,
    sign_txs::maybe_sign_eth_txs_and_add_to_evm_state,
};

fn submit_evm_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<EvmOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_evm_block_in_state)
        .and_then(|state| state.get_eth_evm_token_dictionary_and_add_to_state())
        .and_then(check_for_parent_of_evm_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_redeem_events_in_state)
        .and_then(maybe_add_evm_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_evm_block_hash_and_return_state)
        .and_then(maybe_update_evm_canon_block_hash_and_return_state)
        .and_then(maybe_update_evm_tail_block_hash_and_return_state)
        .and_then(maybe_update_evm_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_eth_tx_infos_from_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_sign_eth_txs_and_add_to_evm_state)
        .and_then(maybe_increment_int_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_evm_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_evm_canon_block_and_return_state)
        .and_then(get_evm_output_json)
}

/// # Submit EVM Block to Core
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_evm_block_to_core<D: DatabaseInterface>(db: &D, block: &str) -> Result<String> {
    info!("✔ Submitting EVM block to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJson::from_str(block))
        .and_then(|json| submit_evm_block(db, &json))
        .and_then(|output| {
            db.end_transaction()?;
            Ok(output.to_string())
        })
}

/// # Submit EVM Blocks to Core
///
/// Submit multiple EVM blocks to the core. See `submit_evm_block_to_core` for more information.
pub fn submit_evm_blocks_to_core<D: DatabaseInterface>(db: &D, blocks: &str) -> Result<String> {
    info!("✔ Batch submitting EVM blocks to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJsons::from_str(blocks))
        .and_then(|jsons| {
            let mut outputs = vec![];

            for json in jsons.iter() {
                match submit_evm_block(db, json) {
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
        .map(EvmOutputs::new)
        .and_then(|outputs| {
            db.end_transaction()?;
            Ok(outputs.to_output().to_string())
        })
}

#[cfg(test)]
mod tests {
    use std::{fs::read_to_string, str::FromStr};

    use common::{dictionaries::eth_evm::EthEvmTokenDictionary, test_utils::get_test_database};
    use common_chain_ids::EthChainId;
    use common_eth::{
        convert_hex_to_eth_address,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        initialize_evm_core_with_no_contract_tx,
        parse_eth_submission_material_and_put_in_state,
        EthDbUtils,
        EthDbUtilsExt,
        EthPrivateKey,
        VaultUsingCores,
    };
    use common_eth_debug::reset_eth_chain;
    use serde_json::json;

    use super::*;
    use crate::{
        evm::get_evm_output_json::EvmOutput,
        test_utils::{
            get_sample_evm_goerli_init_block_json_string,
            get_sample_evm_init_block_json_string,
            get_sample_int_init_block_json_string,
            get_sample_router_address,
            get_sample_token_dictionary_entry,
            get_sample_token_dictionary_entry_1,
        },
    };

    #[test]
    fn should_submit_evm_rinkeby_block_with_peg_out_successfully() {
        let db = get_test_database();
        let router_address = get_sample_router_address();
        let vault_address = convert_hex_to_eth_address("0x010e1e6f6c360da7e3d62479b6b9d717b3e114ca").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        // NOTE: Initialize the INT side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_int_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEvm,
            true, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Initialize the EVM side of the core...
        initialize_evm_core_with_no_contract_tx(
            &get_sample_evm_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("0x7f00f3d16bf2581105c35d1aacec3fe3d0c441f0").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("8d8f01916c70ff01244200f1768b9fb246158714ac05dc34cb6fca71798075a5").unwrap(),
        )
        .unwrap();
        let db_utils = EthDbUtils::new(&db);
        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        // NOTE: Set the nonce to match that used during the test...
        let evm_nonce = 0;
        db_utils.put_eth_account_nonce_in_db(evm_nonce).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key);
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), evm_nonce);
        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry(), &db)
            .unwrap();
        let is_for_eth = false;
        // NOTE Save the token dictionary into the db...
        // NOTE: Bring the chain up to the block prior to the block containing a peg-in...
        reset_eth_chain(
            parse_eth_submission_material_and_put_in_state(
                &read_to_string("src/test_utils/before-peg-out-1-block.json").unwrap(),
                EthState::init(&db),
            )
            .unwrap(),
            confirmations,
            is_for_eth,
        )
        .unwrap();
        let submission_string = read_to_string("src/test_utils/peg-out-block-1.json").unwrap();
        // NOTE: Finally, submit the block containing the peg in....
        let output = submit_evm_block_to_core(&db, &submission_string).unwrap();
        let expected_result_json = json!({
            "evm_latest_block_number": 11572292,
            "int_signed_transactions": [{
                "_id": "pint-on-evm-int-0",
                "broadcast": false,
                "int_tx_hash": "0x9b8c6a306e07271f9b3c732654806ec83b80a34f2ab2c2fe14bd151adb55dc06",
                "int_tx_amount": "665",
                "int_tx_recipient": "0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC",
                "witnessed_timestamp": 1638902000,
                "host_token_address": "0xdd9f905a34a6c507c7d68384985905cf5eb032e9",
                "originating_tx_hash": "0x61ac238ba14d7f8bc1fff8546f61127d9b44be6955819adb0f9861da6723bef1",
                "originating_address": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address": "0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "int_signed_tx": "f9034b808504a817c8008306ddd094010e1e6f6c360da7e3d62479b6b9d717b3e114ca80b902e4229654690000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6000000000000000000000000a83446f219baec0b6fd6b3031c5a49a54543045b000000000000000000000000000000000000000000000000000000000000029900000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000240030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014000f343680000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003decaff0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786665646665323631366562333636316362386665643237383266356630636339316435396463616300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a307866656466653236313665623336363163623866656432373832663566306363393164353964636163000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029a0e39e25beba0d32491a451e6b4ebe930e4169f4a18a824b870b5f1dfe5ec718c3a03143da213b6227eb4cd2c55f1d2a9b475fa84c879a24dd3e809e0b167285d07b",
                "any_sender_nonce": null,
                "int_account_nonce": 0,
                "int_latest_block_number": 11544951,
                "broadcast_tx_hash": null,
                "broadcast_timestamp": null,
                "any_sender_tx": null,
                "destination_chain_id": "0x00f34368",
            }]
        });
        let expected_result = EvmOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = EvmOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_submit_evm_goerli_block_with_peg_out_successfully() {
        let db = get_test_database();
        let router_address = get_sample_router_address();
        let vault_address = convert_hex_to_eth_address("0x010e1e6f6c360da7e3d62479b6b9d717b3e114ca").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;

        // NOTE: Initialize the INT side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_int_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEvm,
            true, // NOTE: is_native
        )
        .unwrap();

        // NOTE: Initialize the EVM side of the core...
        initialize_evm_core_with_no_contract_tx(
            &get_sample_evm_goerli_init_block_json_string(),
            &EthChainId::Goerli,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("0x7f00f3d16bf2581105c35d1aacec3fe3d0c441f0").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("8d8f01916c70ff01244200f1768b9fb246158714ac05dc34cb6fca71798075a5").unwrap(),
        )
        .unwrap();
        let db_utils = EthDbUtils::new(&db);
        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key);

        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Finally, submit the block containing the peg in....
        let submission_string = read_to_string("src/test_utils/goerli-core-peg-out-block.json").unwrap();
        let output = submit_evm_block_to_core(&db, &submission_string).unwrap();
        let expected_result_json = json!({
            "evm_latest_block_number": 7464238,
            "int_signed_transactions": [{
                "_id":"pint-on-evm-int-0",
                "broadcast":false,
                "int_tx_hash":"0x57e812d2e7c25e4c990a7bd601e65027f37f7d47e670fe2f5dbc5b2cd513b60c",
                "int_tx_amount":"1334",
                "int_tx_recipient":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "witnessed_timestamp":1661357369,
                "host_token_address":"0xf8c69b3a5db2e5384a0332325f5931cd5aa4aada",
                "originating_tx_hash":"0x1823814ab29df921fc32f7a25158a0c4221a072f167162037f8ccf43fde12fb8",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "destination_chain_id":"0x00e4b170",
                "int_signed_tx":"f9034b808504a817c8008306ddd094010e1e6f6c360da7e3d62479b6b9d717b3e114ca80b902e4229654690000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6000000000000000000000000a83446f219baec0b6fd6b3031c5a49a54543045b0000000000000000000000000000000000000000000000000000000000000536000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002400300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000b4f6c500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014000e4b1700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003decaff0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786665646665323631366562333636316362386665643237383266356630636339316435396463616300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a307866656466653236313665623336363163623866656432373832663566306363393164353964636163000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029a0101eed95c4557dfa2038c602060681a8acb0c1e8cf01e6a4e30211c5b3a0a3c7a03036c7903d12c0f466b8c8182101ed4cabdefdeb45eee2d68b0cf503788d934b",
                "any_sender_nonce":null,
                "int_account_nonce":0,
                "int_latest_block_number":11544951,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null,
                "any_sender_tx":null,
            }]
        });
        let expected_result = EvmOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = EvmOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_batch_submit_evm_blocks_successfully_even_if_one_already_in_db() {
        let db = get_test_database();
        let router_address = get_sample_router_address();
        let vault_address = convert_hex_to_eth_address("0x010e1e6f6c360da7e3d62479b6b9d717b3e114ca").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;

        // NOTE: Initialize the INT side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_int_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEvm,
            true, // NOTE: is_native
        )
        .unwrap();

        // NOTE: Initialize the EVM side of the core...
        initialize_evm_core_with_no_contract_tx(
            &get_sample_evm_goerli_init_block_json_string(),
            &EthChainId::Goerli,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
        )
        .unwrap();

        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Submit a block normally, which should be successful
        let submission_string = read_to_string("src/test_utils/goerli-core-peg-out-block.json").unwrap();
        let block_num = 7464238;
        submit_evm_block_to_core(&db, &submission_string).unwrap();

        // NOTE: However it will fail a second time due to the block already being in the db...
        match submit_evm_block_to_core(&db, &submission_string) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(AppError::BlockAlreadyInDbError(e)) => assert_eq!(e.block_num, block_num),
            Err(e) => panic!("wrong error received: {e}"),
        };

        // NOTE: However if the same block forms part of a _batch_ of blocks, the
        // `BlockAlreadyInDbError` should be swallowed, and thus no errors.
        let batch = format!("[{submission_string},{submission_string},{submission_string}]");
        let result = submit_evm_blocks_to_core(&db, &batch);
        assert!(result.is_ok());
    }
}
