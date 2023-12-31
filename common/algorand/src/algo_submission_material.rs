use std::{fmt, str::FromStr};

use common::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};
use derive_more::{Constructor, Deref};
use rust_algorand::{
    AlgorandAddress,
    AlgorandBlock,
    AlgorandBlockJson,
    AlgorandTransactionProof,
    AlgorandTransactionProofJson,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AlgoState;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct AlgoSubmissionMaterials(Vec<AlgoSubmissionMaterial>);

impl FromStr for AlgoSubmissionMaterials {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::new(
            AlgoSubmissionMaterialJsons::from_str(s)?
                .iter()
                .map(AlgoSubmissionMaterial::from_json)
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlgoSubmissionMaterial {
    pub block: AlgorandBlock,
    pub proofs: Vec<AlgorandTransactionProof>,
    pub expired_participation_accounts: Option<Vec<AlgorandAddress>>,
}

impl AlgoSubmissionMaterial {
    pub fn from_json(json: &AlgoSubmissionMaterialJson) -> Result<Self> {
        Ok(Self {
            block: AlgorandBlock::from_json(&json.block)?,
            proofs: json
                .proofs
                .iter()
                .map(|proof_json| Ok(AlgorandTransactionProof::from_json(proof_json)?))
                .collect::<Result<Vec<AlgorandTransactionProof>>>()?,
            expired_participation_accounts: match &json.expired_participation_accounts {
                Some(account_strs) => Some(
                    account_strs
                        .iter()
                        .map(|account| Ok(AlgorandAddress::from_str(account)?))
                        .collect::<Result<Vec<AlgorandAddress>>>()?,
                ),
                None => None,
            },
        })
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(&serde_json::from_slice::<AlgoSubmissionMaterialJson>(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.to_json()?)?)
    }

    pub fn to_json(&self) -> Result<AlgoSubmissionMaterialJson> {
        Ok(AlgoSubmissionMaterialJson {
            block: self.block.to_json()?,
            proofs: self
                .proofs
                .iter()
                .map(|proof| proof.to_json())
                .collect::<Vec<AlgorandTransactionProofJson>>(),
            expired_participation_accounts: self
                .expired_participation_accounts
                .as_ref()
                .map(|accounts| accounts.iter().map(|address| address.to_string()).collect()),
        })
    }

    pub fn remove_txs(&self) -> Self {
        let mut mutable_self = self.clone();
        mutable_self.block.transactions = None;
        mutable_self
    }
}

impl FromStr for AlgoSubmissionMaterial {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        AlgoSubmissionMaterialJson::from_str(s).and_then(|ref json| Self::from_json(json))
    }
}

impl fmt::Display for AlgoSubmissionMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_json() {
            Ok(json_struct) => write!(f, "{}", json!(json_struct)),
            Err(error) => write!(f, "Could not convert AlgorandBlock to json!: {}", error),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct AlgoSubmissionMaterialJsons(Vec<AlgoSubmissionMaterialJson>);

impl FromStr for AlgoSubmissionMaterialJsons {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::new(serde_json::from_str::<Vec<AlgoSubmissionMaterialJson>>(s)?))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlgoSubmissionMaterialJson {
    block: AlgorandBlockJson,
    proofs: Vec<AlgorandTransactionProofJson>,
    #[serde(rename = "expired-participation-accounts")]
    expired_participation_accounts: Option<Vec<String>>,
}

impl FromStr for AlgoSubmissionMaterialJson {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn parse_algo_submission_material_and_put_in_state<'a, D: DatabaseInterface>(
    submission_material: &str,
    state: AlgoState<'a, D>,
) -> Result<AlgoState<'a, D>> {
    state.add_algo_submission_material(&AlgoSubmissionMaterial::from_str(submission_material)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{get_sample_batch_submission_material, get_sample_submission_material_str_n};

    #[test]
    fn should_parse_submission_material_from_str() {
        let submission_material_str = get_sample_submission_material_str_n(0);
        let result = AlgoSubmissionMaterial::from_str(&submission_material_str);
        assert!(result.is_ok());
    }

    #[test]
    fn should_serde_submission_material_to_and_from_json() {
        let submission_material_str = get_sample_submission_material_str_n(0);
        let submission_material = AlgoSubmissionMaterial::from_str(&submission_material_str).unwrap();
        let json = submission_material.to_json().unwrap();
        let result = AlgoSubmissionMaterial::from_json(&json).unwrap();
        assert_eq!(result, submission_material)
    }

    #[test]
    fn should_serde_algo_submission_material_to_and_from_bytes() {
        let submission_material_str = get_sample_submission_material_str_n(0);
        let submission_material = AlgoSubmissionMaterial::from_str(&submission_material_str).unwrap();
        let bytes = submission_material.to_bytes().unwrap();
        let result = AlgoSubmissionMaterial::from_bytes(&bytes).unwrap();
        assert_eq!(result, submission_material);
    }

    #[test]
    fn should_parse_batch_submission_material_jsons_correctly() {
        let s = get_sample_batch_submission_material();
        let r = AlgoSubmissionMaterialJsons::from_str(&s);
        assert!(r.is_ok());
    }

    #[test]
    fn should_parse_batch_submission_material_correctly() {
        let s = get_sample_batch_submission_material();
        let r = AlgoSubmissionMaterials::from_str(&s);
        assert!(r.is_ok());
    }
}
