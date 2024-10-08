use common::{
    traits::DatabaseInterface,
    types::{Byte, Bytes, NoneError, Result},
};
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

use crate::eos_database_utils::EosDbUtils;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProtocolFeature {
    feature_name: String,
    feature_hash: String,
}

impl ProtocolFeature {
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    pub fn new(name: &str, feature_hash: &str) -> Self {
        ProtocolFeature {
            feature_name: name.to_string(),
            feature_hash: feature_hash.to_string(),
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(hex::decode(&self.feature_hash)?)
    }
}

#[derive(Clone, Debug, PartialEq, Default, Eq, Deref, Serialize, Deserialize, Constructor)]
pub struct EnabledFeatures(Vec<ProtocolFeature>);

impl EnabledFeatures {
    pub fn init() -> Self {
        EnabledFeatures(vec![])
    }

    pub fn remove(mut self, feature_hash: &[Byte]) -> Result<Self> {
        if self.does_not_contain(feature_hash) {
            debug!("Feature hash not in enabled features ∴ doing nothing!");
            return Ok(self);
        };
        AVAILABLE_FEATURES
            .get_feature_from_hash(feature_hash)
            .and_then(|feature| {
                self.0.remove(
                    self.0
                        .iter()
                        .position(|x| x == &feature)
                        .ok_or(NoneError("Could not unwrap EOS protocol feature while removing!"))?,
                );
                Ok(self)
            })
    }

    pub fn add_new(mut self, feature_hash: &[Byte]) -> Result<Self> {
        AVAILABLE_FEATURES
            .check_contains(feature_hash)
            .and_then(|_| AVAILABLE_FEATURES.get_feature_from_hash(feature_hash))
            .and_then(|feature| {
                info!("✔ Adding feature: {}", feature.to_json()?);
                self.0.push(feature);
                Ok(self)
            })
    }

    fn add_multi(mut self, feature_hashes: &mut Vec<Bytes>) -> Result<Self> {
        info!("✔ Adding multiple features...");
        // NOTE: Sort the passed in feature hashes so we can remove duplications...
        feature_hashes.sort();
        feature_hashes.dedup();

        // NOTE: Ensure each of the feature hashes actually exist...
        feature_hashes
            .iter()
            .try_for_each(|feature_hash| AVAILABLE_FEATURES.check_contains(feature_hash))?;

        // NOTE: Get the features from the hashes...
        let features = feature_hashes
            .iter()
            .filter_map(|hash| AVAILABLE_FEATURES.maybe_get_feature_from_hash(hash));

        // NOTE: And finally, add them to self...
        for feature in features {
            info!("✔ Adding feature: {}", feature.to_json()?);
            self.0.push(feature);
        }
        Ok(self)
    }

    fn contains(&self, feature_hash: &[Byte]) -> bool {
        let hash = hex::encode(feature_hash);
        self.0.iter().any(|e| e.feature_hash == hash)
    }

    fn does_not_contain(&self, feature_hash: &[Byte]) -> bool {
        !self.contains(feature_hash)
    }

    pub fn is_enabled(&self, feature_hash: &[Byte]) -> bool {
        AVAILABLE_FEATURES.contains(feature_hash) && self.contains(feature_hash)
    }

    pub fn is_not_enabled(&self, feature_hash: &[Byte]) -> bool {
        !self.is_enabled(feature_hash)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    fn check_all_features_are_available(&self) -> Result<Self> {
        self.iter()
            .try_for_each(|feature| AVAILABLE_FEATURES.check_contains(&feature.to_bytes()?))
            .and(Ok(self.clone()))
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        serde_json::from_slice::<Self>(bytes)?.check_all_features_are_available()
    }

    pub fn enable_multi<D: DatabaseInterface>(
        self,
        db_utils: &EosDbUtils<D>,
        feature_hashes: &mut Vec<Bytes>,
    ) -> Result<Self> {
        self.add_multi(feature_hashes).and_then(|updated_self| {
            db_utils.put_eos_enabled_protocol_features_in_db(&updated_self)?;
            Ok(updated_self)
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvailableFeatures(Vec<ProtocolFeature>);

impl AvailableFeatures {
    pub fn new(available_features: Vec<ProtocolFeature>) -> Self {
        AvailableFeatures(available_features)
    }

    pub fn contains(&self, feature_hash: &[Byte]) -> bool {
        let hash = hex::encode(feature_hash);
        self.0.iter().any(|e| e.feature_hash == hash)
    }

    pub fn check_contains(&self, feature_hash: &[Byte]) -> Result<()> {
        info!(
            "✔ Checking available features for feature hash {}",
            hex::encode(feature_hash)
        );
        match AVAILABLE_FEATURES.contains(feature_hash) {
            true => {
                info!("✔ Feature hash exists in available features!");
                Ok(())
            },
            false => Err(format!("Unrecognised feature hash: {}", hex::encode(feature_hash),).into()),
        }
    }

    fn get_known_feature_from_hash(&self, feature_hash: &[Byte]) -> ProtocolFeature {
        self.0
            .iter()
            .fold(ProtocolFeature::default(), |mut acc, protocol_feature| {
                if protocol_feature.feature_hash == hex::encode(feature_hash) {
                    acc = protocol_feature.clone();
                };
                acc
            })
    }

    fn maybe_get_feature_from_hash(&self, feature_hash: &[Byte]) -> Option<ProtocolFeature> {
        match self.contains(feature_hash) {
            true => Some(self.get_known_feature_from_hash(feature_hash)),
            false => {
                info!("Unrecognised feature hash: {}", hex::encode(feature_hash));
                None
            },
        }
    }

    fn get_feature_from_hash(&self, feature_hash: &[Byte]) -> Result<ProtocolFeature> {
        self.check_contains(feature_hash)
            .map(|_| self.get_known_feature_from_hash(feature_hash))
    }
}

lazy_static! {
    pub static ref AVAILABLE_FEATURES: AvailableFeatures = {
        AvailableFeatures::new(vec![ProtocolFeature::new(
            "WTMSIG_BLOCK_SIGNATURE",
            WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH,
        )])
    };
}

pub static WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH: &str =
    "299dcb6af692324b899b39f16d5a530a33062804e41f09dc97e9f156b4476707";

#[cfg(test)]
mod tests {
    use common::errors::AppError;

    use super::*;

    #[test]
    fn should_not_add_non_available_feature() {
        let feature_hash = [0u8; 32];
        assert!(!AVAILABLE_FEATURES.contains(&feature_hash));
        let enabled_features = EnabledFeatures::default();
        let expected_error = format!("Unrecognised feature hash: {}", hex::encode(feature_hash));
        match enabled_features.add_new(&feature_hash) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_add_available_feature() {
        let existing_feature_hash = hex::decode(WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH).unwrap();
        assert!(AVAILABLE_FEATURES.contains(&existing_feature_hash));
        let enabled_features = EnabledFeatures::default();
        let result = enabled_features.add_new(&existing_feature_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_multi_add_available_features() {
        let existing_feature_hash = hex::decode(WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH).unwrap();
        let existing_feature_hashes = [existing_feature_hash.clone(), existing_feature_hash];
        existing_feature_hashes
            .iter()
            .for_each(|hash| assert!(AVAILABLE_FEATURES.contains(hash)));
        let enabled_features = EnabledFeatures::default();
        let result =
            enabled_features.add_multi(&mut existing_feature_hashes.iter().map(|hash| hash.to_vec()).collect());
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_when_multi_addiing_non_available_features() {
        let unavailable_feature_hash = vec![0u8; 32];
        let existing_feature_hash = hex::decode(WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH).unwrap();
        let feature_hashes = [existing_feature_hash, unavailable_feature_hash.clone()];
        assert!(feature_hashes
            .iter()
            .map(|hash| AVAILABLE_FEATURES.contains(hash))
            .any(|x| !x));
        let enabled_features = EnabledFeatures::default();
        let expected_error = format!("Unrecognised feature hash: {}", hex::encode(unavailable_feature_hash));
        match enabled_features.add_multi(&mut feature_hashes.iter().map(|hash| hash.to_vec()).collect()) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_serde_enabled_features_to_and_from_bytes() {
        let features = EnabledFeatures::default();
        let updated_features = features
            .add_new(&hex::decode(WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH).unwrap())
            .unwrap();
        let bytes = updated_features.to_bytes().unwrap();
        let result = EnabledFeatures::from_bytes(&bytes).unwrap();
        assert_eq!(result, updated_features);
    }

    #[test]
    fn should_check_all_features_are_available() {
        let features = vec![ProtocolFeature::new(
            "WTMSIG_BLOCK_SIGATURE",
            WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH,
        )];
        let enabled_features = EnabledFeatures::new(features);
        let result = enabled_features.check_all_features_are_available();
        assert!(result.is_ok());
    }

    #[test]
    fn check_available_features_should_error_if_feature_is_unavailable() {
        let non_available_feature_hash = [0u8; 64].iter().fold(String::new(), |e, acc| format!("{}{}", e, acc));
        let features = vec![ProtocolFeature::new(
            "NON_AVAILABLE_FEATURE",
            &non_available_feature_hash,
        )];
        let enabled_features = EnabledFeatures::new(features);
        let expected_error = format!("Unrecognised feature hash: {}", non_available_feature_hash);
        match enabled_features.check_all_features_are_available() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
