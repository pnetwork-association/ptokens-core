use common::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    core_type::CoreType,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};
use common_eth::EthSignature;
use common_safe_addresses::SAFE_ETH_ADDRESS;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map as JsonMap, Value as JsonValue};

use crate::DebugSignatory;

lazy_static! {
    pub static ref DEBUG_SIGNATORIES_DB_KEY: [u8; 32] = common::utils::get_prefixed_db_key("debug_signatories_db_key");
    pub static ref SAFE_DEBUG_SIGNATORIES: DebugSignatories =
        DebugSignatories::new(vec![DebugSignatory::new("safe_address", &SAFE_ETH_ADDRESS)]);
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct DebugSignatories(Vec<DebugSignatory>);

impl DebugSignatories {
    fn contains_name(&self, name: &str) -> bool {
        debug!("Checking if debug signatories contains name '{}'...", name);
        self.iter().any(|debug_signatory| debug_signatory.name() == name)
    }

    fn contains_eth_address(&self, eth_address: &str) -> bool {
        debug!("Checking if debug signatories contains address '{}'...", eth_address);
        self.iter()
            .any(|debug_signatory| debug_signatory.eth_address() == eth_address)
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        serde_json::from_slice::<Vec<Bytes>>(bytes)?
            .iter()
            .map(|bytes| DebugSignatory::from_bytes(bytes))
            .collect::<Result<Vec<DebugSignatory>>>()
            .map(Self)
    }

    fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(
            &self
                .iter()
                .cloned()
                .map(|debug_signatory| debug_signatory.to_bytes())
                .collect::<Result<Vec<Bytes>>>()?,
        )?)
    }

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        match db.get(DEBUG_SIGNATORIES_DB_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => Ok(Self::new(vec![])),
        }
    }

    pub fn put_in_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
        db.put(
            DEBUG_SIGNATORIES_DB_KEY.to_vec(),
            self.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn to_signature_info_json(
        &self,
        core_type: &CoreType,
        debug_command_hash: &H256,
        maybe_signature: Option<&EthSignature>,
    ) -> Result<JsonValue> {
        // NOTE: The `to_json` fxn for an individual signer uses these single-letter keys, so we
        // add a glossary to aid in understanding.
        let glossary_key = "glossary".to_string();
        let glossary_value = json!({
            "a": "the ETH `address` derived from the signer's private key",
            "n": "a `nonce` that's used to stop replays for a given address' signature",
            "h": "the final `hash` to sign if using a simple ETH signer like etherscan or MEW or mycrypto",
            "d": "the `debug` command hash, commiting to the debug command's arguments, used for EIP712 signing",
        });

        let error_key = "error".to_string();
        let error_value = if maybe_signature.is_some() && maybe_signature != Some(&EthSignature::empty()) {
            JsonValue::String("Could not validate signature!".to_string())
        } else {
            JsonValue::String("A signature is required to run this function!".to_string())
        };

        let core_type_key = "coreType".to_string();
        let core_type_value = JsonValue::String(core_type.to_string());

        let mut json_map = JsonMap::new();
        json_map.insert(error_key, error_value);
        json_map.insert(glossary_key, glossary_value);
        json_map.insert(core_type_key, core_type_value);

        self.iter()
            .try_fold(json_map, |mut map, debug_signatory| {
                map.insert(
                    debug_signatory.name.clone(),
                    debug_signatory.to_json(core_type, debug_command_hash)?,
                );
                Ok(map)
            })
            .map(JsonValue::Object)
    }

    fn add(&self, debug_signatory: &DebugSignatory) -> Self {
        info!("✔ Maybe adding debug signatory to list...");
        let mut mutable_self = self.0.clone();
        let name = debug_signatory.name();
        let eth_address = debug_signatory.eth_address();

        if debug_signatory.eth_address == *SAFE_ETH_ADDRESS {
            warn!("✘ Not adding debug signatory because it's address is the safe ETH address!");
            Self(mutable_self)
        } else if self.contains_name(&name) {
            warn!("✘ Debug signatory with name '{}' already in list!", name);
            Self(mutable_self)
        } else if self.contains_eth_address(&eth_address) {
            warn!("✘ Debug signatory with ETH address '{}' already in list!", eth_address);
            Self(mutable_self)
        } else {
            info!("✔ Adding debug signer to list!");
            mutable_self.push(debug_signatory.clone());
            Self(mutable_self)
        }
    }

    fn add_multi(&self, debug_signatories_to_add: &[DebugSignatory]) -> Self {
        debug_signatories_to_add
            .iter()
            .fold(self.clone(), |existing_debug_signatories: Self, debug_signatory| {
                existing_debug_signatories.add(debug_signatory)
            })
    }

    fn remove(&self, eth_address: &EthAddress) -> Self {
        Self(
            self.iter()
                .filter(|debug_signatory| debug_signatory.eth_address != *eth_address)
                .cloned()
                .collect(),
        )
    }

    fn get(&self, eth_address: &EthAddress) -> Result<DebugSignatory> {
        let signatories = self
            .iter()
            .filter(|debug_signatory| debug_signatory.eth_address == *eth_address)
            .cloned()
            .collect::<Vec<DebugSignatory>>();
        if signatories.is_empty() {
            Err(format!("Could not find debug signatory with address: '{}'!", eth_address).into())
        } else if signatories.len() > 1 {
            Err(format!("> 1 entry found with address: '{}'!", eth_address).into())
        } else {
            Ok(signatories[0].clone())
        }
    }

    fn replace(&self, debug_signatory: &DebugSignatory) -> Result<Self> {
        let eth_address = debug_signatory.eth_address;
        if self.get(&eth_address).is_ok() {
            Ok(self.remove(&eth_address).add(debug_signatory))
        } else {
            Err(format!("Cannot replace entry, none exists with eth address: '{}'!", eth_address).into())
        }
    }

    pub fn add_and_update_in_db<D: DatabaseInterface>(&self, db: &D, debug_signatory: &DebugSignatory) -> Result<()> {
        self.add(debug_signatory).put_in_db(db)
    }

    pub fn add_multi_and_update_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        debug_signatories_to_add: &[DebugSignatory],
    ) -> Result<()> {
        self.add_multi(debug_signatories_to_add).put_in_db(db)
    }

    pub fn remove_and_update_in_db<D: DatabaseInterface>(&self, db: &D, eth_address: &EthAddress) -> Result<()> {
        self.remove(eth_address).put_in_db(db)
    }

    fn increment_nonce_in_signatory_in_db<D: DatabaseInterface>(&self, db: &D, eth_address: &EthAddress) -> Result<()> {
        info!("✔ Incrementing nonce in debug signatory with address: {}", eth_address);
        self.get(eth_address)
            .map(|signatory| signatory.increment_nonce())
            .and_then(|signatory| self.replace(&signatory))
            .and_then(|debug_signatories| debug_signatories.put_in_db(db))
    }

    fn maybe_validate_signature_for_eth_address_and_increment_nonce_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        eth_address: &EthAddress,
        core_type: &CoreType,
        debug_command_hash: &H256,
        signature_str: &EthSignature,
    ) -> Result<()> {
        self.get(eth_address)
            .and_then(|signatory| signatory.validate(signature_str, core_type, debug_command_hash))
            .and_then(|_| self.increment_nonce_in_signatory_in_db(db, eth_address))
    }

    fn to_eth_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|signatory| signatory.eth_address).collect::<Vec<_>>()
    }

    pub fn maybe_validate_signature_and_increment_nonce_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        core_type: &CoreType,
        debug_command_hash: &H256,
        signature: &EthSignature,
    ) -> Result<()> {
        if self
            .to_eth_addresses()
            .iter()
            .filter_map(|eth_address| {
                match self.maybe_validate_signature_for_eth_address_and_increment_nonce_in_db(
                    db,
                    eth_address,
                    core_type,
                    debug_command_hash,
                    signature,
                ) {
                    Ok(_) => {
                        info!("✔ Signature valid for address: {}", eth_address);
                        Some(true)
                    },
                    Err(_) => {
                        warn!("✘ Signature NOT valid for address: {}", eth_address);
                        None
                    },
                }
            })
            .next()
            .is_none()
        {
            Err(self
                .to_signature_info_json(core_type, debug_command_hash, Some(signature))?
                .into())
        } else {
            Ok(())
        }
    }

    pub fn to_enclave_state_json(&self) -> JsonValue {
        json!(self
            .iter()
            .map(|debug_signatory| debug_signatory.to_enclave_state_json())
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database};
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::test_utils::{
        get_n_random_debug_signatories,
        get_sample_debug_command_hash,
        get_sample_debug_signatories,
        get_sample_private_key,
    };

    #[test]
    fn should_serde_debug_signatories_to_and_from_bytes() {
        let debug_signatories = get_sample_debug_signatories();
        let bytes = debug_signatories.to_bytes().unwrap();
        let result = DebugSignatories::from_bytes(&bytes).unwrap();
        assert_eq!(result, debug_signatories);
    }

    #[test]
    fn should_put_and_get_debug_signatories_in_and_from_db() {
        let debug_signatories = get_sample_debug_signatories();
        let db = get_test_database();
        debug_signatories.put_in_db(&db).unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, debug_signatories);
    }

    #[test]
    fn get_from_db_should_return_empty_signatories_if_none_in_db() {
        let db = get_test_database();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        let expected_result = DebugSignatories::new(vec![]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_debug_signatories_to_json() {
        let core_type = CoreType::BtcOnInt;
        let debug_signatories = get_sample_debug_signatories();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = None;
        let result = debug_signatories.to_signature_info_json(&core_type, &debug_command_hash, signature);
        assert!(result.is_ok());
    }

    #[test]
    fn should_add_debug_signatory() {
        let debug_signatories = get_sample_debug_signatories();
        let debug_signatory = DebugSignatory::default();
        assert!(!debug_signatories.contains(&debug_signatory));
        let result = debug_signatories.add(&debug_signatory);
        assert!(result.contains(&debug_signatory));
    }

    #[test]
    fn should_not_add_debug_signatory_if_extant() {
        let debug_signatory = DebugSignatory::random();
        let debug_signatories = DebugSignatories::new(vec![debug_signatory.clone()]);
        assert_eq!(debug_signatories.len(), 1);
        let result = debug_signatories.add(&debug_signatory);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn should_remove_debug_signatory() {
        let debug_signatories = get_sample_debug_signatories();
        let debug_signatory = debug_signatories[1].clone();
        assert!(debug_signatories.contains(&debug_signatory));
        let result = debug_signatories.remove(&debug_signatory.eth_address);
        assert!(!result.contains(&debug_signatory));
    }

    #[test]
    fn should_get_debug_signatory_from_signatories() {
        let debug_signatories = get_sample_debug_signatories();
        let expected_result = debug_signatories[2].clone();
        let eth_address = expected_result.eth_address;
        let result = debug_signatories.get(&eth_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_get_non_extant_debug_signatory_from_signatories() {
        let debug_signatories = get_sample_debug_signatories();
        let eth_address = EthAddress::random();
        let expected_error = format!("Could not find debug signatory with address: '{}'!", eth_address);
        match debug_signatories.get(&eth_address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_fail_to_replace_non_existent_entry() {
        let debug_signatories = get_sample_debug_signatories();
        let debug_signatory = DebugSignatory::random();
        let expected_error = format!(
            "Cannot replace entry, none exists with eth address: '{}'!",
            debug_signatory.eth_address
        );
        match debug_signatories.replace(&debug_signatory) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_replace_entry() {
        let debug_signatories = get_sample_debug_signatories();
        let index = 1;
        let mut debug_signatory = debug_signatories[index].clone();
        let eth_address = debug_signatory.eth_address;
        debug_signatory.nonce += 1;
        assert_ne!(debug_signatory, debug_signatories[index]);
        let updated_signatories = debug_signatories.replace(&debug_signatory).unwrap();
        let result = updated_signatories.get(&eth_address).unwrap();
        assert_eq!(result, debug_signatory);
    }

    #[test]
    fn should_add_and_update_in_db() {
        let db = get_test_database();
        let debug_signatory = DebugSignatory::random();
        let expected_result = DebugSignatories::new(vec![debug_signatory.clone()]);
        DebugSignatories::new(vec![])
            .add_and_update_in_db(&db, &debug_signatory)
            .unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_remove_and_update_in_db() {
        let db = get_test_database();
        let debug_signatories = get_n_random_debug_signatories(5);
        let signatory_to_remove = debug_signatories[2].eth_address;
        let expected_result = DebugSignatories(
            debug_signatories
                .iter()
                .filter(|debug_signatory| debug_signatory.eth_address != signatory_to_remove)
                .cloned()
                .collect(),
        );
        debug_signatories
            .remove_and_update_in_db(&db, &signatory_to_remove)
            .unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_increment_nonce_in_entry_in_db() {
        let debug_signatories = get_n_random_debug_signatories(5);
        let db = get_test_database();
        let index = 2;
        let debug_signatory = debug_signatories[index].clone();
        let eth_address = debug_signatory.eth_address;
        let nonce_before = debug_signatory.nonce;
        debug_signatories
            .increment_nonce_in_signatory_in_db(&db, &eth_address)
            .unwrap();
        let updated_signatories = DebugSignatories::get_from_db(&db).unwrap();
        let expected_result = nonce_before + 1;
        let result = updated_signatories.get(&eth_address).unwrap().nonce;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_error_if_entry_in_signatories_twice() {
        let debug_signatory = DebugSignatory::random();
        // NOTE: This is the only way we can create one with a duplicate in it.
        let debug_signatories = DebugSignatories(vec![debug_signatory.clone(), debug_signatory.clone()]);
        let eth_address = debug_signatory.eth_address;
        let expected_error = format!("> 1 entry found with address: '{}'!", eth_address);
        match debug_signatories.get(&eth_address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_maybe_validate_debug_signature_and_increment_nonce_in_db() {
        let db = get_test_database();
        let core_type = CoreType::BtcOnInt;
        let eth_address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let debug_command_hash = H256::random();
        let debug_signatory_1 = DebugSignatory::new("Some name", &eth_address);
        let debug_signatory_2 = DebugSignatory::random();
        let debug_signatory_3 = DebugSignatory::random();

        // NOTE: Assert the signatory we care about's nonce is 0.
        assert_eq!(debug_signatory_1.nonce, 0);
        let debug_signatories = DebugSignatories::new(vec![
            debug_signatory_2.clone(),
            debug_signatory_1.clone(),
            debug_signatory_3.clone(),
        ]);
        let pk = get_sample_private_key();
        // NOTE: Assert the private key is for the address we expect.
        assert_eq!(pk.to_public_key().to_address(), eth_address);

        // NOTE Now we sign the random `debug_command_hash`
        let signature = debug_signatory_1.sign(&pk, &core_type, &debug_command_hash).unwrap();

        // NOTE: Signature should be valid, and the nonce for this signatory should be incremented.
        debug_signatories
            .maybe_validate_signature_and_increment_nonce_in_db(&db, &core_type, &debug_command_hash, &signature)
            .unwrap();

        // NOTE: So lets assert that this signatory's nonce did indeed get updated in the db.
        let updated_signatories = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(updated_signatories.get(&eth_address).unwrap().nonce, 1);

        // NOTE: And finally, assert that the other two signatory's nonces did NOT get
        // incremented.
        assert_eq!(
            updated_signatories.get(&debug_signatory_2.eth_address).unwrap().nonce,
            debug_signatory_2.nonce
        );
        assert_eq!(
            updated_signatories.get(&debug_signatory_3.eth_address).unwrap().nonce,
            debug_signatory_3.nonce
        );
    }

    #[test]
    fn should_fail_to_maybe_validate_invalid_signature_and_thus_not_increment_nonce_in_db() {
        let db = get_test_database();
        let core_type = CoreType::BtcOnInt;
        let debug_signatories_before = get_sample_debug_signatories();
        let debug_command_hash = get_sample_debug_command_hash();
        debug_signatories_before.put_in_db(&db).unwrap();

        // NOTE: The signature is totally random...
        let random_signature = EthSignature::random().unwrap();

        // NOTE: And so it should error...
        let expected_error = debug_signatories_before
            .to_signature_info_json(&core_type, &debug_command_hash, Some(&random_signature))
            .unwrap();
        match debug_signatories_before.maybe_validate_signature_and_increment_nonce_in_db(
            &db,
            &core_type,
            &debug_command_hash,
            &random_signature,
        ) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Json(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }

        // NOTE: Finally, let's assert that no debug signatory nonces were incremented.
        let debug_signatories_after = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(debug_signatories_before, debug_signatories_after);
    }

    #[test]
    fn should_return_true_if_contains_name() {
        let debug_signatories = get_sample_debug_signatories();
        let result = debug_signatories.contains_name(&debug_signatories[0].name());
        assert!(result);
    }

    #[test]
    fn should_return_true_if_contains_eth_address() {
        let debug_signatories = get_sample_debug_signatories();
        let result = debug_signatories.contains_eth_address(&debug_signatories[0].eth_address());
        assert!(result);
    }

    #[test]
    fn should_return_false_if_does_not_contain_name() {
        let debug_signatories = get_sample_debug_signatories();
        let result = debug_signatories.contains_name("non-existent");
        assert!(!result);
    }

    #[test]
    fn should_return_false_if_does_not_contain_eth_address() {
        let debug_signatories = get_sample_debug_signatories();
        let result = debug_signatories.contains_eth_address("non-existent");
        assert!(!result);
    }
}
