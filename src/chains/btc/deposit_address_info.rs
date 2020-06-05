use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
};
use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::{
    types::Result,
    errors::AppError,
    chains::btc::btc_utils::convert_hex_to_sha256_hash,
};

pub type DepositInfoList = Vec<DepositAddressInfo>;
pub type DepositAddressJsonList = Vec<DepositAddressInfoJson>;
pub type DepositInfoHashMap =  HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositAddressListVersion {
    V0,
    V1,
}

impl DepositAddressListVersion {
    pub fn from_maybe_string(maybe_string: &Option<String>) -> Result<Self> {
        match maybe_string {
            None => Ok(DepositAddressListVersion::V0),
            Some(version_string) => DepositAddressListVersion::from_string(version_string.clone()),
        }
    }

    pub fn from_string(version_string: String) -> Result<Self> {
        match version_string.chars().next() {
            Some('0') => Ok(DepositAddressListVersion::V0),
            Some('1') => Ok(DepositAddressListVersion::V1),
            _ => Err(AppError::Custom(format!("✘ Deposit address list version unrecognized: {}", version_string)))
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DepositAddressListVersion::V0 => "0".to_string(),
            DepositAddressListVersion::V1 => "1".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: Option<String>,
    pub eth_address: Option<String>, // NOTE: For legacy reasons.
    pub btc_deposit_address: String,
    pub version: Option<String>,
    pub address_and_nonce_hash: Option<String>,
    pub eth_address_and_nonce_hash: Option<String>, // NOTE: Ibid.
}

impl DepositAddressInfoJson {
    #[cfg(test)]
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
        version: Option<String>,
    ) -> Result<Self> {
        match DepositAddressListVersion::from_maybe_string(&version)? {
            DepositAddressListVersion::V0 => Ok(DepositAddressInfoJson {
                nonce,
                version,
                address: None,
                btc_deposit_address,
                eth_address: Some(address),
                address_and_nonce_hash: None,
                eth_address_and_nonce_hash: Some(address_and_nonce_hash)
            }),
            DepositAddressListVersion::V1 => Ok(DepositAddressInfoJson {
                nonce,
                version,
                eth_address: None,
                btc_deposit_address,
                address: Some(address),
                eth_address_and_nonce_hash: None,
                address_and_nonce_hash: Some(address_and_nonce_hash),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepositAddressInfo {
    pub nonce: u64,
    pub address: String,
    pub commitment_hash: sha256d::Hash,
    pub btc_deposit_address: BtcAddress,
    pub version: DepositAddressListVersion,
}

impl DepositAddressInfo {
    fn get_missing_field_err_msg(field_name: &str) -> String {
        format!("✘ No '{}' field in deposit address info json!", field_name)
    }

    fn extract_address_and_nonce_hash_string_from_json(
        deposit_address_info_json: &DepositAddressInfoJson
    ) -> Result<String> {
        match DepositAddressListVersion::from_maybe_string(&deposit_address_info_json.version)? {
            DepositAddressListVersion::V0 => match &deposit_address_info_json.eth_address_and_nonce_hash {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("eth_address_and_nonce_hash"))),
            },
            DepositAddressListVersion::V1 => match &deposit_address_info_json.address_and_nonce_hash {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("address_and_nonce_hash"))),
            },
        }
    }

    fn extract_address_and_nonce_hash_from_json(
        deposit_address_info_json: &DepositAddressInfoJson
    ) -> Result<sha256d::Hash> {
        Self::extract_address_and_nonce_hash_string_from_json(deposit_address_info_json)
            .and_then(|hex| convert_hex_to_sha256_hash(&hex))
    }

    fn extract_address_string_from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<String> {
        match DepositAddressListVersion::from_maybe_string(&deposit_address_info_json.version)? {
            DepositAddressListVersion::V0 => match &deposit_address_info_json.eth_address {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("eth_address"))),
            },
            DepositAddressListVersion::V1 => match &deposit_address_info_json.address {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("address"))),
            }
        }
    }

    pub fn from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<Self> {
        Ok(DepositAddressInfo {
            nonce: deposit_address_info_json.nonce.clone(),
            address: Self::extract_address_string_from_json(deposit_address_info_json)?,
            btc_deposit_address: BtcAddress::from_str(&deposit_address_info_json.btc_deposit_address)?,
            commitment_hash: Self::extract_address_and_nonce_hash_from_json(deposit_address_info_json)?,
            version: DepositAddressListVersion::from_maybe_string(&deposit_address_info_json.version)?,
        })
    }

    pub fn to_json(&self) -> DepositAddressInfoJson {
        let hash_string = hex::encode(self.commitment_hash);
        DepositAddressInfoJson {
            nonce: self.nonce,
            version: Some(self.version.to_string()),
            btc_deposit_address: self.btc_deposit_address.to_string(),
            address: match self.version {
                DepositAddressListVersion::V0 => None,
                DepositAddressListVersion::V1 => Some(self.address.clone()),
            },
            eth_address: match self.version {
                DepositAddressListVersion::V0 => Some(self.address.clone()),
                DepositAddressListVersion::V1 => None,
            },
            eth_address_and_nonce_hash: match self.version {
                DepositAddressListVersion::V0 => Some(hash_string.clone()),
                DepositAddressListVersion::V1 => None,
            },
            address_and_nonce_hash: match self.version {
                DepositAddressListVersion::V0 => None,
                DepositAddressListVersion::V1 => Some(hash_string),
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_err_if_json_is_v1_and_has_no_address_and_nonce_hash_key() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let eth_address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let eth_address = None;
        let address_and_nonce_hash = None;
        let version = Some("1".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'address_and_nonce_hash' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_err_if_json_is_v0_and_has_no_eth_address_field() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let eth_address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let eth_address = None;
        let address_and_nonce_hash = None;
        let version = Some("0".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'eth_address' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_err_if_json_is_v1_and_has_no_address_field() {
        let nonce = 1578079722;
        let eth_address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("1".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'address' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_err_if_json_is_v0_and_has_no_eth_address_and_nonce_hash() {
        let nonce = 1578079722;
        let eth_address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("0".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'eth_address_and_nonce_hash' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn deposit_info_should_be_v0_if_version_field_missing() {
        let nonce = 1578079722;
        let eth_address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let eth_address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let version = None;
        let address = None;
        let address_and_nonce_hash = None;
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let result = DepositAddressInfo::from_json(&deposit_json).unwrap();
        assert_eq!(result.version, DepositAddressListVersion::V0);
    }

    #[test]
    fn should_convert_deposit_info_json_to_deposit_info() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let eth_address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("1.0.0".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        if let Err(e) = DepositAddressInfo::from_json(&deposit_json) {
            panic!("Error parsing deposit info json: {}", e);
        }
    }
}
