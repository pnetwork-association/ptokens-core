use std::{fmt, str::FromStr};

#[cfg(test)]
use bs58;
#[cfg(test)]
use common::types::Bytes;
use common::{
    constants::MAX_DATA_SENSITIVITY_LEVEL,
    crypto_utils::generate_random_private_key,
    errors::AppError,
    traits::DatabaseInterface,
    types::Result,
};
use secp256k1::{
    key::{PublicKey, SecretKey, ONE_KEY},
    Message,
    Secp256k1,
};

#[cfg(test)]
use crate::bitcoin_crate_alias::hashes::sha256d;
use crate::{
    bitcoin_crate_alias::{
        hashes::{sha256, Hash as HashTrait},
        util::base58,
    },
    eos_crypto::{eos_public_key::EosPublicKey, eos_signature::EosSignature},
    eos_database_utils::EosDbUtils,
    eos_types::EosNetwork,
};
#[derive(Clone, PartialEq, Eq)]
pub struct EosPrivateKey {
    pub compressed: bool,
    private_key: SecretKey,
    pub network: EosNetwork,
}

#[allow(dead_code)]
#[cfg(test)]
impl EosPrivateKey {
    fn to_bytes(&self) -> Bytes {
        self.private_key[..].to_vec()
    }

    fn get_checksummed_bytes(&self) -> Bytes {
        let prefixed_bytes = [vec![0x80], self.to_bytes()].concat();
        let hash = sha256d::Hash::hash(&prefixed_bytes).to_vec();
        [prefixed_bytes, hash[..4].to_vec()].concat()
    }

    fn to_wif(&self) -> String {
        // NOTE: EOS always uses compressed keys! See encoding here:
        // https://developers.eos.io/manuals/eos/v2.0/keosd/wallet-specification
        bs58::encode(self.get_checksummed_bytes()).into_string()
    }
}

impl EosPrivateKey {
    pub fn generate_random() -> Result<Self> {
        Ok(Self {
            compressed: false,
            network: EosNetwork::Mainnet,
            private_key: generate_random_private_key()?,
        })
    }

    pub fn to_public_key(&self) -> EosPublicKey {
        EosPublicKey {
            compressed: true,
            public_key: PublicKey::from_secret_key(&Secp256k1::new(), &self.private_key),
        }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        Ok(Self {
            compressed: false,
            network: EosNetwork::Mainnet, // NOTE: Since they're all same.
            private_key: SecretKey::from_slice(slice)?,
        })
    }

    #[cfg(not(feature = "ltc"))]
    pub fn from_wallet_import_format(wallet_import_formatted_key: &str) -> Result<EosPrivateKey> {
        let data = base58::from_check(wallet_import_formatted_key)?;
        let compressed = match data.len() {
            33 => false,
            34 => true,
            _ => return Err(AppError::Base58Error(base58::Error::InvalidLength(data.len()))),
        };
        let network = match data[0] {
            128 => EosNetwork::Mainnet,
            239 => EosNetwork::Testnet,
            x => return Err(AppError::Base58Error(base58::Error::InvalidAddressVersion(x))),
        };
        Ok(EosPrivateKey {
            compressed,
            network,
            private_key: SecretKey::from_slice(&data[1..33])?,
        })
    }

    #[cfg(feature = "ltc")]
    pub fn from_wallet_import_format(wallet_import_formatted_key: &str) -> Result<EosPrivateKey> {
        let data = base58::from_check(wallet_import_formatted_key)?;
        let compressed = match data.len() {
            33 => false,
            34 => true,
            _ => return Err(AppError::LitecoinBase58Error(base58::Error::InvalidLength(data.len()))),
        };
        let network = match data[0] {
            128 => EosNetwork::Mainnet,
            239 => EosNetwork::Testnet,
            x => return Err(AppError::LitecoinBase58Error(base58::Error::InvalidAddressVersion(x))),
        };
        Ok(EosPrivateKey {
            compressed,
            network,
            private_key: SecretKey::from_slice(&data[1..33])?,
        })
    }

    pub fn sign_hash(&self, hash: &[u8]) -> Result<EosSignature> {
        let msg = match Message::from_slice(hash) {
            Ok(msg) => msg,
            Err(err) => return Err(err.into()),
        };
        Ok(EosSignature::from(Secp256k1::sign_canonical(
            &Secp256k1::new(),
            &msg,
            &self.private_key,
        )))
    }

    pub fn sign_message_bytes(&self, message_slice: &[u8]) -> Result<EosSignature> {
        let msg_hash = sha256::Hash::hash(message_slice);
        self.sign_hash(&msg_hash)
    }

    pub fn write_to_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
        debug!("✔ Putting EOS private key in db...");
        db.put(
            EosDbUtils::new(db).get_eos_private_key_db_key(),
            self.private_key[..].to_vec(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        debug!("✔ Getting EOS private key from db...");
        db.get(
            EosDbUtils::new(db).get_eos_private_key_db_key(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
        .and_then(|bytes| Self::from_slice(&bytes[..]))
    }
}

impl fmt::Display for EosPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "✘ Refusing to print private key!")
    }
}

impl FromStr for EosPrivateKey {
    type Err = AppError;

    fn from_str(s: &str) -> Result<EosPrivateKey> {
        EosPrivateKey::from_wallet_import_format(s)
    }
}

impl fmt::Debug for EosPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[private key data]")
    }
}

impl Drop for EosPrivateKey {
    fn drop(&mut self) {
        unsafe { ::std::ptr::write_volatile(&mut self.private_key, ONE_KEY) };
    }
}

#[cfg(test)]
mod test {
    use common::test_utils::get_sample_message_to_sign_bytes;

    use super::*;
    use crate::{
        bitcoin_crate_alias::hashes::{sha256, Hash as HashTrait},
        eos_crypto::eos_private_key::EosPrivateKey,
        eos_test_utils::{
            get_sample_eos_private_key,
            get_sample_eos_private_key_str,
            get_sample_eos_public_key,
            get_sample_eos_public_key_str,
            get_sample_eos_signature,
        },
    };

    #[test]
    fn should_generate_random_eos_crypto() {
        if let Err(e) = EosPrivateKey::generate_random() {
            panic!("Error generating random key: {}", e);
        }
    }

    #[test]
    fn should_get_secret_key_from_wallet_import_format() {
        let wif = get_sample_eos_private_key_str();
        let private_key = EosPrivateKey::from_wallet_import_format(wif).unwrap();
        let expected_public_key = get_sample_eos_public_key();
        let result = private_key.to_public_key();
        assert_eq!(result, expected_public_key);
    }

    #[test]
    fn should_get_secret_key_from_string() {
        let string = get_sample_eos_private_key_str();
        let private_key = EosPrivateKey::from_str(string).unwrap();
        let expected_public_key = get_sample_eos_public_key();
        let result = private_key.to_public_key();
        assert_eq!(result, expected_public_key);
    }

    #[test]
    fn should_sign_message() {
        let private_key = get_sample_eos_private_key();
        let expected_result = get_sample_eos_signature();
        let message_bytes = get_sample_message_to_sign_bytes();
        let result = private_key.sign_message_bytes(message_bytes).unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_sign_hash() {
        let private_key = get_sample_eos_private_key();
        let expected_result = get_sample_eos_signature();
        let message_bytes = get_sample_message_to_sign_bytes();
        let hash = &sha256::Hash::hash(message_bytes);
        let result = private_key.sign_hash(hash).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn signature_should_be_canonical() {
        let eos_private_key = get_sample_eos_private_key();
        let message_bytes = get_sample_message_to_sign_bytes();
        let hash = &sha256::Hash::hash(message_bytes);
        let msg = Message::from_slice(hash).unwrap();
        let signature = Secp256k1::sign_canonical(&Secp256k1::new(), &msg, &eos_private_key.private_key);
        assert!(signature.is_canonical());
    }

    #[test]
    #[ignore] // NOTE: Expensive test!
    fn signatures_should_be_canonical() {
        let eos_private_key = get_sample_eos_private_key();
        let message_bytes = get_sample_message_to_sign_bytes();
        let mut hash = sha256::Hash::hash(message_bytes);
        for _ in 0..10_000 {
            hash = sha256::Hash::hash(&hash);
            let msg = Message::from_slice(&hash).unwrap();
            let signature = Secp256k1::sign_canonical(&Secp256k1::new(), &msg, &eos_private_key.private_key);
            assert!(signature.is_canonical());
        }
    }

    #[test]
    fn should_convert_private_to_public_correctly() {
        let expected_result = get_sample_eos_public_key_str();
        let private_key = get_sample_eos_private_key();
        let result = private_key.to_public_key().to_string();
        assert_eq!(result, expected_result);
    }
}
