use secp256k1::key::SecretKey;
use std::{
    fmt,
    fmt::Write,
    str::FromStr,
};
use secp256k1::{
    Message,
    Secp256k1,
};
use bitcoin_hashes::{
    sha256,
    Hash as HashTrait
};
use crate::btc_on_eos::{
    base58,
    types::Result,
    errors::AppError,
    crypto_utils::generate_random_private_key,
    eos::{
        eos_types::EosNetwork,
        eos_crypto::eos_signature::EosSignature,
    },
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct EosPrivateKey {
    pub compressed: bool,
    pub network: EosNetwork,
    pub private_key: SecretKey,
}

impl EosPrivateKey {
    pub fn generate_random() -> Result<Self> {
        Ok(
            Self {
                compressed: false,
                network: EosNetwork::Mainnet,
                private_key: generate_random_private_key()?,
            }
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.private_key[..].to_vec()
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        Ok(
            Self {
                compressed: false,
                network: EosNetwork::Mainnet, // NOTE: Since they're all same.
                private_key: SecretKey::from_slice(&slice)?
            }
        )
    }

    pub fn convert_secret_key_to_wallet_import_format(
        &self,
        fmt: &mut dyn Write
    ) -> fmt::Result {
        let mut ret = [0; 34];
        ret[0] = match self.network {
            EosNetwork::Mainnet => 128,
            EosNetwork::Testnet => 239,
        };
        ret[1..33].copy_from_slice(&self.private_key[..]);
        let privkey = if self.compressed {
            ret[33] = 1;
            base58::check_encode_slice(&ret[..])
        } else {
            base58::check_encode_slice(&ret[..33])
        };
        fmt.write_str(&privkey)
    }

    pub fn from_wallet_import_format(
        wallet_import_formatted_key: &str
    ) -> Result<EosPrivateKey> {
        let data = base58::from_check(wallet_import_formatted_key)?;
        let compressed = match data.len() {
            33 => false,
            34 => true,
            _ => {
                return Err(AppError::Base58Error(
                    base58::Error::InvalidLength(data.len())
                ))
            }
        };
        let network = match data[0] {
            128 => EosNetwork::Mainnet,
            239 => EosNetwork::Testnet,
            x => {
                return Err(AppError::Base58Error(
                    base58::Error::InvalidVersion(vec![x])
                ))
            }
        };
        Ok(
            EosPrivateKey {
                compressed,
                network,
                private_key: SecretKey::from_slice(&data[1..33])?,
            }
        )
    }

    pub fn sign_hash(&self, hash: &[u8]) -> Result<EosSignature> {
        let msg = match Message::from_slice(&hash) {
            Ok(msg) => msg,
            Err(err) => return Err(err.into()),
        };
        Ok(
            EosSignature::from(
                Secp256k1::sign_canonical(
                    &Secp256k1::new(),
                    &msg,
                    &self.private_key
                )
            )
        )
    }

    pub fn sign_message_bytes(
        &self,
        message_slice: &[u8]
    ) -> Result<EosSignature> {
        let msg_hash = sha256::Hash::hash(&message_slice);
        self.sign_hash(&msg_hash)
    }
}

impl fmt::Display for EosPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.convert_secret_key_to_wallet_import_format(f)
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

#[cfg(test)]
mod test {
    use super::*;
    use bitcoin_hashes::{
        sha256,
        Hash as HashTrait
    };
    use crate::btc_on_eos::{
        test_utils::get_sample_message_to_sign_bytes,
        eos::{
            eos_crypto::{
                eos_public_key::EosPublicKey,
                eos_private_key::EosPrivateKey,
            },
            eos_test_utils::{
                get_sample_eos_signature,
                get_sample_eos_public_key,
                get_sample_eos_private_key,
                get_sample_eos_private_key_str,
                get_sample_eos_private_key_bytes,
            },
        },
    };

    #[test]
    fn should_generate_random_eos_crypto() {
        if let Err(e) = EosPrivateKey::generate_random() {
            panic!("Error generating random key: {}", e);
        }
    }

    #[test]
    fn should_get_pk_from_slice() {
        let pk_bytes = get_sample_eos_private_key()
            .to_bytes();
        let result = EosPrivateKey::from_slice(&pk_bytes)
            .unwrap();
        assert!(result.to_bytes() == pk_bytes);
    }

    #[test]
    fn should_convert_secret_key_to_bytes_correctly() {
        let expected_bytes = get_sample_eos_private_key_bytes();
        let wif = get_sample_eos_private_key_str();
        let private_key = EosPrivateKey::from_wallet_import_format(wif)
            .unwrap();
        let result = private_key.to_bytes();
        assert!(result == expected_bytes);
    }

    #[test]
    fn should_get_secret_key_from_wallet_import_format() {
        let wif = get_sample_eos_private_key_str();
        let private_key = EosPrivateKey::from_wallet_import_format(wif)
            .unwrap();
        let expected_public_key = get_sample_eos_public_key();
        let result = EosPublicKey::from(&private_key);
        assert!(result == expected_public_key);
    }

    #[test]
    fn should_get_secret_key_from_string() {
        let string = get_sample_eos_private_key_str();
        let private_key = EosPrivateKey::from_str(string)
            .unwrap();
        let expected_public_key = get_sample_eos_public_key();
        let result = EosPublicKey::from(&private_key);
        assert!(result == expected_public_key);
    }

    #[test]
    fn should_sign_message() {
        let private_key = get_sample_eos_private_key();
        let expected_result = get_sample_eos_signature();
        let message_bytes = get_sample_message_to_sign_bytes();
        let result = private_key
            .sign_message_bytes(message_bytes)
            .unwrap();
        assert!(result == expected_result)
    }

    #[test]
    fn should_sign_hash() {
        let private_key = get_sample_eos_private_key();
        let expected_result = get_sample_eos_signature();
        let message_bytes = get_sample_message_to_sign_bytes();
        let hash = &sha256::Hash::hash(message_bytes);
        let result = private_key
            .sign_hash(hash)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn signature_should_be_canonical() {
        let eos_private_key = get_sample_eos_private_key();
        let message_bytes = get_sample_message_to_sign_bytes();
        let hash = &sha256::Hash::hash(message_bytes);
        let msg = Message::from_slice(&hash).unwrap();
        let signature = Secp256k1::sign_canonical(
            &Secp256k1::new(),
            &msg,
            &eos_private_key.private_key
        );
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
            let signature = Secp256k1::sign_canonical(
                &Secp256k1::new(),
                &msg,
                &eos_private_key.private_key
            );
            assert!(signature.is_canonical());
        }
    }
}
