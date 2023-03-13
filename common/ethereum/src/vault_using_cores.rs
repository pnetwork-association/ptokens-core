use common::{traits::DatabaseInterface, types::Result, CoreType, V3CoreType};
use ethereum_types::Address as EthAddress;
use strum_macros::EnumIter;

use crate::eth_database_utils::EthDbUtilsExt;

#[derive(Clone, Debug, Eq, PartialEq, EnumIter)]
pub enum VaultUsingCores {
    IntOnEvm,
    IntOnAlgo,
    Erc20OnEos,
    Erc20OnEvm,
    Erc20OnInt,
    IntOnEos,
    V3(V3CoreType),
}

impl VaultUsingCores {
    pub fn from_core_type(core_type: &CoreType) -> Result<Self> {
        match core_type {
            CoreType::IntOnEos => Ok(Self::IntOnEos),
            CoreType::IntOnEvm => Ok(Self::IntOnEvm),
            CoreType::IntOnAlgo => Ok(Self::IntOnAlgo),
            CoreType::Erc20OnEos => Ok(Self::Erc20OnEos),
            CoreType::Erc20OnEvm => Ok(Self::Erc20OnEvm),
            CoreType::Erc20OnInt => Ok(Self::Erc20OnInt),
            CoreType::V3(v3_core_type) => match v3_core_type {
                V3CoreType::EvmOnInt => Ok(Self::V3(V3CoreType::EvmOnInt)),
                V3CoreType::IntOnEvm => Ok(Self::V3(V3CoreType::IntOnEvm)),
            },
            _ => Err(format!("Core type '{core_type}' does not appear to be a vault using core").into()),
        }
    }

    pub fn get_vault_contract<D: DatabaseInterface, E: EthDbUtilsExt<D>>(&self, db_utils: &E) -> Result<EthAddress> {
        match self {
            Self::IntOnAlgo => db_utils.get_int_on_algo_smart_contract_address(),
            Self::IntOnEos => db_utils.get_int_on_eos_smart_contract_address_from_db(),
            Self::Erc20OnEos => db_utils.get_erc20_on_eos_smart_contract_address_from_db(),
            Self::Erc20OnInt => db_utils.get_erc20_on_int_smart_contract_address_from_db(),
            Self::IntOnEvm | Self::V3(V3CoreType::IntOnEvm) => db_utils.get_int_on_evm_smart_contract_address_from_db(),
            Self::Erc20OnEvm | Self::V3(V3CoreType::EvmOnInt) => {
                db_utils.get_erc20_on_evm_smart_contract_address_from_db()
            },
        }
    }

    pub fn put_vault_contract_in_db<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
        &self,
        db_utils: &E,
        address: &EthAddress,
    ) -> Result<()> {
        match self {
            Self::IntOnEos => db_utils.put_int_on_eos_smart_contract_address_in_db(address),
            Self::IntOnAlgo => db_utils.put_int_on_algo_smart_contract_address_in_db(address),
            Self::Erc20OnEos => db_utils.put_erc20_on_eos_smart_contract_address_in_db(address),
            Self::Erc20OnInt => db_utils.put_erc20_on_int_smart_contract_address_in_db(address),
            Self::IntOnEvm | Self::V3(V3CoreType::IntOnEvm) => {
                db_utils.put_int_on_evm_smart_contract_address_in_db(address)
            },
            Self::Erc20OnEvm | Self::V3(V3CoreType::EvmOnInt) => {
                db_utils.put_erc20_on_evm_smart_contract_address_in_db(address)
            },
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
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{EthDbUtils, EthPrivateKey};

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
