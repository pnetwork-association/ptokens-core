use ethereum_types::Address as EthAddress;
use strum_macros::EnumIter;

use crate::{chains::eth::eth_database_utils::EthDbUtilsExt, traits::DatabaseInterface, types::Result};

#[derive(Clone, Debug, Eq, PartialEq, EnumIter)]
pub enum VaultUsingCores {
    IntOnEvm,
    IntOnAlgo,
    Erc20OnEos,
    Erc20OnEvm,
    Erc20OnInt,
    IntOnEos,
}

impl VaultUsingCores {
    pub fn get_vault_contract<D: DatabaseInterface, E: EthDbUtilsExt<D>>(&self, db_utils: &E) -> Result<EthAddress> {
        match self {
            Self::IntOnAlgo => db_utils.get_int_on_algo_smart_contract_address(),
            Self::IntOnEvm => db_utils.get_int_on_evm_smart_contract_address_from_db(),
            Self::IntOnEos => db_utils.get_int_on_eos_smart_contract_address_from_db(),
            Self::Erc20OnEos => db_utils.get_erc20_on_eos_smart_contract_address_from_db(),
            Self::Erc20OnEvm => db_utils.get_erc20_on_evm_smart_contract_address_from_db(),
            Self::Erc20OnInt => db_utils.get_erc20_on_int_smart_contract_address_from_db(),
        }
    }

    pub fn put_vault_contract_in_db<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
        &self,
        db_utils: &E,
        address: &EthAddress,
    ) -> Result<()> {
        match self {
            Self::IntOnEos => db_utils.put_int_on_eos_smart_contract_address_in_db(address),
            Self::IntOnEvm => db_utils.put_int_on_evm_smart_contract_address_in_db(address),
            Self::IntOnAlgo => db_utils.put_int_on_algo_smart_contract_address_in_db(address),
            Self::Erc20OnEos => db_utils.put_erc20_on_eos_smart_contract_address_in_db(address),
            Self::Erc20OnEvm => db_utils.put_erc20_on_evm_smart_contract_address_in_db(address),
            Self::Erc20OnInt => db_utils.put_erc20_on_int_smart_contract_address_in_db(address),
        }
    }

    #[cfg(test)]
    fn get_all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::{eth_crypto::eth_private_key::EthPrivateKey, eth_database_utils::EthDbUtils},
        test_utils::get_test_database,
    };

    #[test]
    fn should_get_and_put_all_vault_using_cores_smart_contract_addresses_in_db() {
        VaultUsingCores::get_all().iter().for_each(|vault_using_core| {
            let db = get_test_database();
            let db_utils = EthDbUtils::new(&db);
            let address = EthPrivateKey::generate_random().unwrap().to_public_key().to_address();
            vault_using_core.put_vault_contract_in_db(&db_utils, &address).unwrap();
            let result = vault_using_core.get_vault_contract(&db_utils).unwrap();
            assert_eq!(result, address);
        });
    }
}
