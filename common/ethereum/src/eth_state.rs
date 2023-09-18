use common::{
    dictionaries::{eos_eth::EosEthTokenDictionary, eth_evm::EthEvmTokenDictionary, evm_algo::EvmAlgoTokenDictionary},
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::{get_no_overwrite_state_err, get_not_in_state_err},
};
use ethereum_types::{H256 as EthHash, U256};
use rust_algorand::AlgorandTxGroup;

use crate::{EthDbUtils, EthSubmissionMaterial, EthTransactions, EvmDbUtils};

pub trait EthStateCompatible<D: DatabaseInterface> {
    fn get_eth_db_utils(&self) -> &EthDbUtils<D>;
    fn get_evm_db_utils(&self) -> &EvmDbUtils<D>;
    fn get_sub_mat(&self) -> Result<&EthSubmissionMaterial>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthState<'a, D: DatabaseInterface> {
    pub db: &'a D,
    pub tx_infos: Bytes,
    pub signed_txs: Bytes,
    pub misc: Option<String>,
    pub btc_utxos_and_values: Bytes,
    pub eth_db_utils: EthDbUtils<'a, D>,
    pub evm_db_utils: EvmDbUtils<'a, D>,
    pub int_on_evm_int_signed_txs: EthTransactions,
    pub int_on_evm_evm_signed_txs: EthTransactions,
    pub erc20_on_evm_evm_signed_txs: EthTransactions,
    pub erc20_on_evm_eth_signed_txs: EthTransactions,
    pub erc20_on_int_int_signed_txs: EthTransactions,
    pub erc20_on_int_eth_signed_txs: EthTransactions,
    pub algo_signed_txs: Vec<(String, AlgorandTxGroup)>,
    pub eth_submission_material: Option<EthSubmissionMaterial>,
    pub eos_eth_token_dictionary: Option<EosEthTokenDictionary>,
    pub eth_evm_token_dictionary: Option<EthEvmTokenDictionary>,
    pub evm_algo_token_dictionary: Option<EvmAlgoTokenDictionary>,
}

impl<'a, D: DatabaseInterface> EthStateCompatible<D> for EthState<'a, D> {
    fn get_eth_db_utils(&self) -> &EthDbUtils<D> {
        &self.eth_db_utils
    }

    fn get_evm_db_utils(&self) -> &EvmDbUtils<D> {
        &self.evm_db_utils
    }

    fn get_sub_mat(&self) -> Result<&EthSubmissionMaterial> {
        self.get_eth_submission_material()
    }
}

impl<'a, D: DatabaseInterface> EthState<'a, D> {
    pub fn init(db: &'a D) -> EthState<'a, D> {
        EthState {
            db,
            misc: None,
            tx_infos: vec![],
            signed_txs: vec![],
            algo_signed_txs: vec![],
            btc_utxos_and_values: vec![],
            eth_submission_material: None,
            eth_evm_token_dictionary: None,
            eos_eth_token_dictionary: None,
            evm_algo_token_dictionary: None,
            eth_db_utils: EthDbUtils::new(db),
            evm_db_utils: EvmDbUtils::new(db),
            int_on_evm_int_signed_txs: EthTransactions::new(vec![]),
            int_on_evm_evm_signed_txs: EthTransactions::new(vec![]),
            erc20_on_evm_evm_signed_txs: EthTransactions::new(vec![]),
            erc20_on_evm_eth_signed_txs: EthTransactions::new(vec![]),
            erc20_on_int_int_signed_txs: EthTransactions::new(vec![]),
            erc20_on_int_eth_signed_txs: EthTransactions::new(vec![]),
        }
    }

    pub fn new_with_sub_mat(db: &'a D, s: EthSubmissionMaterial) -> Self {
        let mut state = Self::init(db);
        state.eth_submission_material = Some(s);
        state
    }

    pub fn get_eth_submission_material(&self) -> Result<&EthSubmissionMaterial> {
        match self.eth_submission_material {
            Some(ref eth_submission_material) => Ok(eth_submission_material),
            None => Err(get_not_in_state_err("eth_submission_material").into()),
        }
    }

    pub fn get_eos_eth_token_dictionary_from_db_and_add_to_state(self) -> Result<Self> {
        info!("✔ Getting `EosErc20Dictionary` and adding to ETH state...");
        EosEthTokenDictionary::get_from_db(self.db).and_then(|dictionary| self.add_eos_eth_token_dictionary(dictionary))
    }

    pub fn get_evm_algo_token_dictionary_and_add_to_state(self) -> Result<Self> {
        info!("✔ Getting `EvmAlgoTokenDictionary` and adding to ETH state...");
        EvmAlgoTokenDictionary::get_from_db(self.db).and_then(|dictionary| self.add_evm_algo_dictionary(dictionary))
    }

    pub fn get_eth_evm_token_dictionary_and_add_to_state(self) -> Result<Self> {
        info!("✔ Getting `EthEvmTokenDictionary` and adding to ETH state...");
        EthEvmTokenDictionary::get_from_db(self.db).and_then(|dictionary| self.add_eth_evm_token_dictionary(dictionary))
    }

    pub fn add_algo_txs(mut self, txs: Vec<(String, AlgorandTxGroup)>) -> Self {
        self.algo_signed_txs = txs;
        self
    }

    pub fn get_eth_evm_token_dictionary(&self) -> Result<&EthEvmTokenDictionary> {
        match self.eth_evm_token_dictionary {
            Some(ref dictionary) => Ok(dictionary),
            None => Err(get_not_in_state_err("eth_evm_token_dictionary").into()),
        }
    }

    pub fn get_evm_algo_token_dictionary(&self) -> Result<&EvmAlgoTokenDictionary> {
        match self.evm_algo_token_dictionary {
            Some(ref dictionary) => Ok(dictionary),
            None => Err(get_not_in_state_err("evm_algo_token_dictionary").into()),
        }
    }

    pub fn add_erc20_on_evm_eth_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_evm_eth_signed_txs.is_empty() {
            self.erc20_on_evm_eth_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("erc20_on_evm_eth_signed_txs").into())
        }
    }

    pub fn add_erc20_on_int_eth_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_int_eth_signed_txs.is_empty() {
            self.erc20_on_int_eth_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("erc20_on_int_eth_signed_txs").into())
        }
    }

    pub fn add_erc20_on_int_int_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_int_int_signed_txs.is_empty() {
            self.erc20_on_int_int_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("erc20_on_int_int_signed_txs").into())
        }
    }

    pub fn add_int_on_evm_int_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.int_on_evm_int_signed_txs.is_empty() {
            self.int_on_evm_int_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("int_on_evm_int_signed_txs").into())
        }
    }

    pub fn add_int_on_evm_evm_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.int_on_evm_evm_signed_txs.is_empty() {
            self.int_on_evm_evm_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("int_on_evm_evm_signed_txs").into())
        }
    }

    pub fn add_eth_submission_material(mut self, eth_submission_material: EthSubmissionMaterial) -> Result<Self> {
        match self.eth_submission_material {
            Some(_) => Err(get_no_overwrite_state_err("eth_submission_material").into()),
            None => {
                self.eth_submission_material = Some(eth_submission_material);
                Ok(self)
            },
        }
    }

    pub fn add_tx_infos(mut self, infos: Bytes) -> Self {
        self.tx_infos = infos;
        self
    }

    pub fn add_erc20_on_evm_evm_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_evm_evm_signed_txs.is_empty() {
            self.erc20_on_evm_evm_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("evm_transaction").into())
        }
    }

    pub fn update_eth_submission_material(
        mut self,
        new_eth_submission_material: EthSubmissionMaterial,
    ) -> Result<Self> {
        self.eth_submission_material = Some(new_eth_submission_material);
        Ok(self)
    }

    pub fn get_parent_hash(&self) -> Result<EthHash> {
        self.get_eth_submission_material()?.get_parent_hash()
    }

    pub fn get_block_num(&self) -> Result<U256> {
        self.get_eth_submission_material()?.get_block_number()
    }

    fn add_eos_eth_token_dictionary(mut self, dictionary: EosEthTokenDictionary) -> Result<Self> {
        match self.eos_eth_token_dictionary {
            Some(_) => Err(get_no_overwrite_state_err("eos_eth_token_dictionary").into()),
            None => {
                self.eos_eth_token_dictionary = Some(dictionary);
                Ok(self)
            },
        }
    }

    pub fn get_eos_eth_token_dictionary(&self) -> Result<&EosEthTokenDictionary> {
        match self.eos_eth_token_dictionary {
            Some(ref dictionary) => Ok(dictionary),
            None => Err(get_not_in_state_err("eos_eth_token_dictionary").into()),
        }
    }

    fn add_eth_evm_token_dictionary(mut self, dictionary: EthEvmTokenDictionary) -> Result<Self> {
        match self.eth_evm_token_dictionary {
            Some(_) => Err(get_no_overwrite_state_err("eth_evm_token_dictionary").into()),
            None => {
                self.eth_evm_token_dictionary = Some(dictionary);
                Ok(self)
            },
        }
    }

    pub fn add_evm_algo_dictionary(mut self, dictionary: EvmAlgoTokenDictionary) -> Result<Self> {
        match self.evm_algo_token_dictionary {
            Some(_) => Err(get_no_overwrite_state_err("evm_algo_token_dictionary").into()),
            None => {
                self.evm_algo_token_dictionary = Some(dictionary);
                Ok(self)
            },
        }
    }

    pub fn add_signed_txs(mut self, txs: Bytes) -> Result<Self> {
        if !self.signed_txs.is_empty() {
            Err(get_no_overwrite_state_err("signed_txs").into())
        } else {
            self.signed_txs = txs;
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database};

    use super::*;
    use crate::test_utils::{
        get_expected_block,
        get_expected_receipt,
        get_sample_eth_submission_material,
        get_sample_eth_submission_material_n,
        SAMPLE_RECEIPT_INDEX,
    };

    #[test]
    fn should_fail_to_get_eth_submission_material_in_state() {
        let expected_error = get_not_in_state_err("eth_submission_material");
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        match initial_state.get_eth_submission_material() {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Eth block should not be in state yet!"),
            _ => panic!("Wrong error received!"),
        };
    }

    #[test]
    fn should_add_eth_submission_material_state() {
        let expected_error = get_not_in_state_err("eth_submission_material");
        let eth_submission_material = get_sample_eth_submission_material();
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        match initial_state.get_eth_submission_material() {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Eth block should not be in state yet!"),
            _ => panic!("Wrong error received!"),
        };
        let updated_state = initial_state
            .add_eth_submission_material(eth_submission_material)
            .unwrap();
        let submission_material = updated_state.get_eth_submission_material().unwrap();
        let block = submission_material.get_block().unwrap();
        let receipt = submission_material.receipts.0[SAMPLE_RECEIPT_INDEX].clone();
        let expected_block = get_expected_block();
        let expected_receipt = get_expected_receipt();
        assert_eq!(block, expected_block);
        assert_eq!(receipt, expected_receipt);
    }

    #[test]
    fn should_err_when_overwriting_eth_submission_material_in_state() {
        let expected_error = get_no_overwrite_state_err("eth_submission_material");
        let eth_submission_material = get_sample_eth_submission_material();
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        let updated_state = initial_state
            .add_eth_submission_material(eth_submission_material.clone())
            .unwrap();
        match updated_state.add_eth_submission_material(eth_submission_material) {
            Ok(_) => panic!("Overwriting state should not have succeeded!"),
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            _ => panic!("Wrong error recieved!"),
        }
    }

    #[test]
    fn should_update_eth_submission_material() {
        let eth_submission_material_1 = get_sample_eth_submission_material_n(0).unwrap();
        let eth_submission_material_2 = get_sample_eth_submission_material_n(1).unwrap();
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        let updated_state = initial_state
            .add_eth_submission_material(eth_submission_material_1)
            .unwrap();
        let initial_state_block_num = updated_state
            .get_eth_submission_material()
            .unwrap()
            .get_block_number()
            .unwrap();
        let final_state = updated_state
            .update_eth_submission_material(eth_submission_material_2)
            .unwrap();
        let final_state_block_number = final_state
            .get_eth_submission_material()
            .unwrap()
            .get_block_number()
            .unwrap();
        assert_ne!(final_state_block_number, initial_state_block_num);
    }

    #[test]
    fn should_not_allow_overwrite_of_eth_evm_token_dictionary() {
        let db = get_test_database();
        let state = EthState::init(&db);
        let dictionary = EthEvmTokenDictionary::default();
        let updated_state = state.add_eth_evm_token_dictionary(dictionary.clone()).unwrap();
        let expected_error = get_no_overwrite_state_err("eth_evm_token_dictionary");
        match updated_state.add_eth_evm_token_dictionary(dictionary) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
