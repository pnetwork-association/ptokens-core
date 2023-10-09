use common::{crypto_utils::keccak_hash_bytes, DatabaseInterface, MAX_DATA_SENSITIVITY_LEVEL};
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::Address as EthAddress;
use function_name::named;

use super::ChainError;
use crate::EthPrivateKey;

#[derive(Debug, Clone, PartialEq, Eq, Getters, Constructor)]
pub struct ChainDbUtils<'a, D: DatabaseInterface> {
    db: &'a D,
}

const DB_KEY_PREFIX: &str = "ChainDbUtils::";

impl<'a, D: DatabaseInterface> ChainDbUtils<'a, D> {
    #[named]
    fn pk_db_key(&self) -> Vec<u8> {
        let s = format!("{DB_KEY_PREFIX}{}", function_name!());
        keccak_hash_bytes(s.as_bytes()).as_bytes().to_vec()
    }

    pub fn put_pk(&self, pk: &EthPrivateKey) -> Result<(), ChainError> {
        pk.write_to_database(self.db(), &self.pk_db_key()).map_err(|e| {
            error!("{e}");
            ChainError::CouldNotPutPkInDb
        })?;
        Ok(())
    }

    pub fn get_pk(&self) -> Result<EthPrivateKey, ChainError> {
        self.db()
            .get(self.pk_db_key(), MAX_DATA_SENSITIVITY_LEVEL)
            .and_then(|ref bs| EthPrivateKey::from_slice(bs))
            .map_err(|e| {
                error!("{e}");
                ChainError::DbGet("EthPrivateKey".to_string())
            })
    }

    pub fn get_signing_address(&self) -> Result<EthAddress, ChainError> {
        self.get_pk().map(|pk| pk.to_address())
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;

    #[test]
    fn should_get_pk_db_key() {
        let db = get_test_database();
        let db_utils = ChainDbUtils::new(&db);
        let key = hex::encode(db_utils.pk_db_key());
        let expected_key = "709a4eefe510dc3c3df2265c9d5a109ba8e666882e917c4aeb3df67fdfadefcf";
        assert_eq!(key, expected_key);
    }
}
