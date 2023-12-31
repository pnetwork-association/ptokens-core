use common::{
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
#[allow(unused_imports)]
use common_debug_signers::validate_debug_command_signature;
use common_eth::{EthDbUtils, EthDbUtilsExt, EvmDbUtils};
use function_name::named;
use serde_json::json;

#[named]
fn debug_set_gas_price_in_db<D: DatabaseInterface>(
    db: &D,
    gas_price: u64,
    is_for_eth: bool,
    core_type: &CoreType,
    _signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &gas_price, &is_for_eth, core_type)())
        .map(|_hash| {
            warn!("DEBUG FUNCTTION SIGNATURE VALIDATION DISABLED FOR GAS PRICE SETTER!");
            // FIXME To be reinstated once scripts running these debug functions are updated to
            // provided signatures.
            //validate_debug_command_signature(db, core_type, signature, &hash))
        })
        .and_then(|_| {
            if is_for_eth {
                EthDbUtils::new(db).put_eth_gas_price_in_db(gas_price)
            } else {
                EvmDbUtils::new(db).put_eth_gas_price_in_db(gas_price)
            }
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({"sucess":true,format!("new_{}_gas_price", if is_for_eth { "eth" } else { "evm" }):gas_price})
                .to_string(),
        ))
        .map(prepend_debug_output_marker_to_string)
}

/// Debug Set ETH Gas Price
///
/// This function sets the ETH gas price to use when making ETH transactions. It's unit is `Wei`.
pub fn debug_set_eth_gas_price<D: DatabaseInterface>(
    db: &D,
    gas_price: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Setting ETH gas price in db...");
    debug_set_gas_price_in_db(db, gas_price, true, core_type, signature)
}

/// Debug Set EVM Gas Price
///
/// This function sets the EVM gas price to use when making EVM transactions. It's unit is `Wei`.
pub fn debug_set_evm_gas_price<D: DatabaseInterface>(
    db: &D,
    gas_price: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Setting EVM gas price in db...");
    debug_set_gas_price_in_db(db, gas_price, false, core_type, signature)
}

#[cfg(test)]
mod tests {
    use common::test_utils::{get_test_database, DUMMY_DEBUG_COMMAND_SIGNATURE};

    use super::*;

    #[test]
    fn should_set_eth_gas_price_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let gas_price = 6;
        db_utils.put_eth_gas_price_in_db(gas_price).unwrap();
        assert_eq!(db_utils.get_eth_gas_price_from_db().unwrap(), gas_price);
        let new_gas_price = 4;
        let is_for_eth = true;
        debug_set_gas_price_in_db(
            &db,
            new_gas_price,
            is_for_eth,
            &CoreType::BtcOnInt,
            DUMMY_DEBUG_COMMAND_SIGNATURE,
        )
        .unwrap();
        assert_eq!(db_utils.get_eth_gas_price_from_db().unwrap(), new_gas_price);
    }
}
