use common::{
    traits::DatabaseInterface,
    types::Result,
    utils::{decode_hex_with_err_msg, is_hex},
};
use serde_json::{json, Value as JsonValue};

use crate::{EthDbUtils, EthDbUtilsExt, EthSignature, EthSigningCapabilities, EvmDbUtils};

fn encode_eth_signed_message_as_json(message: &str, signature: &EthSignature) -> JsonValue {
    info!("✔ Encoding eth signed message as json...");
    json!({"message": message, "signature": format!("0x{}", hex::encode(&signature[..]))})
}

fn sign_message_with_no_eth_prefix<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    message: &str,
) -> Result<String> {
    db_utils
        .get_eth_private_key_from_db()
        .and_then(|key| key.hash_and_sign_msg(message.as_bytes()))
        .map(|signature| encode_eth_signed_message_as_json(message, &signature).to_string())
}

fn sign_message_with_eth_prefix<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    message: &str,
) -> Result<String> {
    db_utils
        .get_eth_private_key_from_db()
        .and_then(|key| key.hash_and_sign_msg_with_eth_prefix(message.as_bytes()))
        .map(|signature| encode_eth_signed_message_as_json(message, &signature).to_string())
}

fn sign_hex_message_with_eth_prefix<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    message: &str,
) -> Result<String> {
    let bytes = decode_hex_with_err_msg(message, "Message to sign is NOT valid hex!")?;
    db_utils
        .get_eth_private_key_from_db()
        .and_then(|key| key.hash_and_sign_msg_with_eth_prefix(&bytes))
        .map(|signature| encode_eth_signed_message_as_json(message, &signature).to_string())
}

pub fn sign_ascii_msg_with_no_prefix<D: DatabaseInterface>(db: &D, message: &str, is_for_eth: bool) -> Result<String> {
    info!("✔ Checking message is valid ASCII...");
    if !message.is_ascii() {
        return Err("Non-ASCII message passed. Only valid ASCII messages are supported.".into());
    }
    info!("✔ Checking message is not valid HEX...");
    if is_hex(message) {
        return Err("HEX message passed. Signing HEX messages without prefix is not allowed.".into());
    }
    if is_for_eth {
        sign_message_with_no_eth_prefix(&EthDbUtils::new(db), message)
    } else {
        sign_message_with_no_eth_prefix(&EvmDbUtils::new(db), message)
    }
}

pub fn sign_ascii_msg_with_prefix<D: DatabaseInterface>(db: &D, message: &str, is_for_eth: bool) -> Result<String> {
    info!("✔ Checking message is valid ASCII...");
    if !message.is_ascii() {
        return Err("Non-ASCII message passed. Only valid ASCII messages are supported.".into());
    }
    if is_for_eth {
        sign_message_with_eth_prefix(&EthDbUtils::new(db), message)
    } else {
        sign_message_with_eth_prefix(&EvmDbUtils::new(db), message)
    }
}

pub fn sign_hex_msg_with_prefix<D: DatabaseInterface>(db: &D, message: &str, is_for_eth: bool) -> Result<String> {
    if is_for_eth {
        sign_hex_message_with_eth_prefix(&EthDbUtils::new(db), message)
    } else {
        sign_hex_message_with_eth_prefix(&EvmDbUtils::new(db), message)
    }
}

/// # Sign ASCII Message With ETH Key
///
/// Signs a given ASCII message with the ETH private key from the encrypted database. The function first
/// checks if the message to be signed is valid ASCII, and errors if not. It also checks if message is valid HEX,
/// and errors if it is. This signing function uses a recoverable `secp256k1` signature scheme
/// with NO prefix prepended to the message.
pub fn sign_ascii_msg_with_eth_key_with_no_prefix<D: DatabaseInterface>(db: &D, message: &str) -> Result<String> {
    info!("✔ Signing ASCII message with ETH key & no prefix...");
    sign_ascii_msg_with_no_prefix(db, message, true)
}

/// # Sign ASCII Message With EVM Key
///
/// Signs a given ASCII message with the ETH private key from the encrypted database. The function first
/// checks if the message to be signed is valid ASCII, and errors if not. It also checks if message is valid HEX,
/// and errors if it is. This signing function uses a recoverable `secp256k1` signature scheme
/// with NO prefix prepended to the message.
pub fn sign_ascii_msg_with_evm_key_with_no_prefix<D: DatabaseInterface>(db: &D, message: &str) -> Result<String> {
    info!("✔ Signing ASCII message with EVM key & no prefix...");
    sign_ascii_msg_with_no_prefix(db, message, false)
}

/// # Sign ASCII Message With ETH Key With Prefix
///
/// Signs a given ASCII message with the ETH private key from the encrypted database. The function first
/// checks if the message to be signed is valid ASCII, and errors if not. This signing function uses
/// a recoverable `secp256k1` signature scheme with the ethereum-specific prefix:
///
/// ```no_compile
/// "\x19Ethereum Signed Message:\n32"
/// ```
///
/// prepended to the message before signing.
pub fn sign_ascii_msg_with_eth_key_with_prefix<D: DatabaseInterface>(db: &D, message: &str) -> Result<String> {
    info!("✔ Signing ASCII message with ETH key & prefix...");
    sign_ascii_msg_with_prefix(db, message, true)
}

/// # Sign ASCII Message With EVM Key With Prefix
///
/// Signs a given ASCII message with the EVM private key from the encrypted database. The function first
/// checks if the message to be signed is valid ASCII, and errors if not. This signing function uses
/// a recoverable `secp256k1` signature scheme with the ethereum-specific prefix:
///
/// ```no_compile
/// "\x19Ethereum Signed Message:\n32"
/// ```
///
/// prepended to the message before signing.
pub fn sign_ascii_msg_with_evm_key_with_prefix<D: DatabaseInterface>(db: &D, message: &str) -> Result<String> {
    info!("✔ Signing ASCII message with EVM key & prefix...");
    sign_ascii_msg_with_prefix(db, message, false)
}

/// # Sign HEX Message With ETH Key
///
/// Signs a given HEX message with the ETH private key from the encrypted database. The function first
/// checks if the message to be signed is valid HEX, and errors if not. This signing function uses
/// a recoverable `secp256k1` signature scheme with the ethereum-specific prefix:
///
/// ```no_compile
/// "\x19Ethereum Signed Message:\n32"
/// ```
///
/// prepended to the message before signing.
pub fn sign_hex_msg_with_eth_key_with_prefix<D: DatabaseInterface>(db: &D, message: &str) -> Result<String> {
    info!("✔ Signing hex message with ETH key & prefix...");
    sign_hex_msg_with_prefix(db, message, true)
}

/// # Sign HEX Message With EVM Key
///
/// Signs a given HEX message with the EVM private key from the encrypted database. The function first
/// checks if the message to be signed is valid HEX, and errors if not. This signing function uses
/// a recoverable `secp256k1` signature scheme with the ethereum-specific prefix:
///
/// ```no_compile
/// "\x19Ethereum Signed Message:\n32"
/// ```
///
/// prepended to the message before signing.
pub fn sign_hex_msg_with_evm_key_with_prefix<D: DatabaseInterface>(db: &D, message: &str) -> Result<String> {
    info!("✔ Signing hex message with EVM key & prefix...");
    sign_hex_msg_with_prefix(db, message, false)
}

#[cfg(test)]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database};

    use super::*;
    use crate::{eth_database_utils::EthDbUtils, test_utils::get_sample_eth_private_key};

    #[test]
    fn ascii_signer_should_return_error_if_message_is_not_valid_ascii() {
        let is_for_eth = true;
        let db = get_test_database();
        let message = "Grüße, 🦀";
        assert!(sign_ascii_msg_with_no_prefix(&db, message, is_for_eth).is_err());
        assert!(sign_ascii_msg_with_prefix(&db, message, is_for_eth).is_err());
    }

    #[test]
    fn ascii_signer_with_no_prefix_should_return_error_if_message_is_valid_hex() {
        let is_for_eth = true;
        let db = get_test_database();
        let hex_message = "0x5A0b54D5dc17e0AadC383d2db43B0a0D3E029c4c";
        let hex_message_no_prefix = "4d261b7d3101e9ff7e37f63449be8a9a1affef87e4952900dbb84ee3c29f45f3";
        let expected_error = "✘ HEX message passed. Signing HEX messages without prefix is not allowed.".to_string();
        assert_eq!(
            sign_ascii_msg_with_no_prefix(&db, hex_message, is_for_eth)
                .unwrap_err()
                .to_string(),
            expected_error
        );
        assert_eq!(
            sign_ascii_msg_with_no_prefix(&db, hex_message_no_prefix, is_for_eth)
                .unwrap_err()
                .to_string(),
            expected_error
        );
    }

    #[test]
    fn ascii_signer_with_prefix_should_sign_valid_hex() {
        let is_for_eth = true;
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let eth_private_key = get_sample_eth_private_key();
        eth_db_utils.put_eth_private_key_in_db(&eth_private_key).unwrap();
        let message = "0x5A0b54D5dc17e0AadC383d2db43B0a0D3E029c4c";
        let expected_result = json!({
            "message": "0x5A0b54D5dc17e0AadC383d2db43B0a0D3E029c4c",
            "signature": "0xfd1c8dce0d75ee249e612dc4be61774bfab906447f34300c715baab979d8ed5c7d2f209193055ed5243ca187163450279746feda5aaa9966184e731e68d7e70f1b"
        }).to_string();
        let result = sign_ascii_msg_with_prefix(&db, message, is_for_eth).unwrap();
        assert_eq!(result, expected_result, "✘ Message signature is invalid!")
    }

    #[test]
    fn should_sign_ascii_msg_with_no_prefix() {
        let is_for_eth = true;
        let db = get_test_database();
        let eth_private_key = get_sample_eth_private_key();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_private_key_in_db(&eth_private_key).unwrap();
        let message = "Arbitrary message";
        let expected_result = json!({
            "message": "Arbitrary message",
            "signature": "0x15a75ee16c085117190c8efbcd349cd5a1a8014fe454954d0e1a80210e3d5b7c1a455fba5da51471045e53e297f6d0837099aba65d4d5c5b98ae60fa42ca443d00"
        }).to_string();
        let result = sign_ascii_msg_with_no_prefix(&db, message, is_for_eth).unwrap();
        assert_eq!(result, expected_result, "✘ Message signature is invalid!")
    }

    #[test]
    fn should_encode_eth_signed_message_as_json() {
        let expected_result = json!({
            "message": "Arbitrary message",
            "signature": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        });
        let result = encode_eth_signed_message_as_json("Arbitrary message", &EthSignature::new([0u8; 65]));
        assert_eq!(result, expected_result, "✘ Message signature json is invalid!")
    }

    #[test]
    fn should_sign_hex_msg_with_prefix() {
        let is_for_eth = true;
        let db = get_test_database();
        let eth_private_key = get_sample_eth_private_key();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_private_key_in_db(&eth_private_key).unwrap();
        let hex_to_sign = "0xc0ffee";
        let result = sign_hex_msg_with_prefix(&db, hex_to_sign, is_for_eth).unwrap();
        let expected_result = json!({
            "message":"0xc0ffee",
            "signature":"0x23593235b38bed5249f727bd12493628d2ef907c28134650459dc0713cb6ed853ae408db7df527b34ac71142d03ccbd41f4d8e08f18c315887b4e5cdb7294d481c"}).to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_sign_invalid_hex_msg_with_prefix() {
        let is_for_eth = true;
        let db = get_test_database();
        let eth_private_key = get_sample_eth_private_key();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_private_key_in_db(&eth_private_key).unwrap();
        let invalid_hex_to_sign = "0xcoffee";
        let expected_err = "Message to sign is NOT valid hex! Invalid character \'o\' at position 1";
        match sign_hex_msg_with_prefix(&db, invalid_hex_to_sign, is_for_eth) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Ok(_) => panic!("Should not have succeeded!"),
            Err(_) => panic!("Got wrong error!"),
        };
    }

    #[test]
    fn should_sign_ascii_msg_with_prefix() {
        let is_for_eth = true;
        let db = get_test_database();
        let eth_private_key = get_sample_eth_private_key();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_private_key_in_db(&eth_private_key).unwrap();
        let message = "Arbitrary message";
        let expected_result = json!({
            "message": "Arbitrary message",
            "signature": "0x4e8fc473b281ad03f8a98429de6205abaafc48731ccf6c6fe9b1f8d918a2e3de69bb2e1e5e479440796ac305f4112900707f633c69d12954fa712d195cfceae41c"
        }).to_string();
        let result = sign_ascii_msg_with_prefix(&db, message, is_for_eth).unwrap();
        assert_eq!(result, expected_result, "✘ Message signature is invalid!")
    }
}
