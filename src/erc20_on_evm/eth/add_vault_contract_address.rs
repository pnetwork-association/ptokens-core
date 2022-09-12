use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_utils::convert_hex_to_eth_address,
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

/// # Maybe Add Vault Contract Address
///
/// This function will add a passed in ETH contract address to the encrypted database since the ETH
/// initialization no longer creates one. Until this step has been carried out after ETH core
/// initialization, the `get_enclave_state` command will error with a message telling you to call
/// this function.
///
/// ### BEWARE:
/// This vault contract setter can only be set ONCE. Further attempts to do so will not succeed.
pub fn maybe_add_vault_contract_address<D: DatabaseInterface>(db: &D, hex_address: &str) -> Result<String> {
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| convert_hex_to_eth_address(hex_address))
        .and_then(|ref address| EthDbUtils::new(db).put_erc20_on_evm_smart_contract_address_in_db(address))
        .and_then(|_| db.end_transaction())
        .map(|_| "{add_vault_address_success:true}".to_string())
}
