use common::{
    core_type::CoreType,
    crypto_utils::keccak_hash_bytes,
    types::{Bytes, Result},
    utils::strip_hex_prefix,
};
use common_eth::EthSignature;
use ethereum_types::{Address as EthAddress, H256};

use crate::debug_signatory::DebugSignatory;

impl DebugSignatory {
    fn get_eth_prefixed_message_bytes(message: &str) -> Bytes {
        let eth_prefixed_message_bytes =
            keccak_hash_bytes(format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message).as_bytes())[..]
                .to_vec();
        debug!(
            "ETH prefixed message bytes: {}",
            hex::encode(&eth_prefixed_message_bytes)
        );
        eth_prefixed_message_bytes
    }

    fn hash_with_eth_prefix(
        &self,
        core_type: &CoreType,
        debug_command_hash: &H256,
        use_hex_prefix: bool,
    ) -> Result<H256> {
        self.hash_to_hex(core_type, debug_command_hash)
            .map(|message| {
                let stripped_message = strip_hex_prefix(&message);
                if use_hex_prefix {
                    format!("0x{}", stripped_message)
                } else {
                    stripped_message
                }
            })
            .map(|message| Self::get_eth_prefixed_message_bytes(&message))
            .map(|bytes| H256::from_slice(&bytes))
    }

    fn recover_signer_addresses_using_correct_eth_prefixing_scheme(
        &self,
        signature: &EthSignature,
        debug_command_hash: &H256,
    ) -> Result<Vec<EthAddress>> {
        // NOTE: Prior versions of the ethereum crate in this workspace had a bug in how it handled
        // the eth prefix when signing messages. This function works to recover correct addresses
        // from signatures made with the non-bugged version of ths signer...
        info!("recovering addresses using correct ETH prefixed signature scheme...");
        let bs = debug_command_hash.as_bytes();
        let eth_msg_prefix = b"\x19Ethereum Signed Message:\n";
        let msg = [eth_msg_prefix, format!("{}", bs.len()).as_bytes(), bs].concat();
        let hash = keccak_hash_bytes(&msg);
        let addresses = signature.recover_both_signer_addresses(&hash)?;
        debug!("recovered addresses: {:?}", addresses);
        Ok(addresses)
    }

    fn recover_signer_addresses(
        &self,
        signature: &EthSignature,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<Vec<EthAddress>> {
        info!("✔ Recovering signature with NO ETH prefix...");
        let addresses = signature.recover_both_signer_addresses(&self.hash(core_type, debug_command_hash)?)?;
        debug!("Recovered addresses: {:?}", addresses);
        Ok(addresses)
    }

    fn recover_signer_addresses_using_eth_prefix_and_hex_prefix(
        &self,
        signature: &EthSignature,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<Vec<EthAddress>> {
        info!("✔ Recovering signature with ETH prefix AND hex prefix...");
        let addresses = signature.recover_both_signer_addresses(&self.hash_with_eth_prefix(
            core_type,
            debug_command_hash,
            true,
        )?)?;
        debug!("Recovered addresses: {:?}", addresses);
        Ok(addresses)
    }

    fn recover_signer_addresses_using_eth_prefix_and_no_hex_prefix(
        &self,
        signature: &EthSignature,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<Vec<EthAddress>> {
        info!("✔ Recovering signature with ETH prefix and NO hex prefix...");
        let addresses = signature.recover_both_signer_addresses(&self.hash_with_eth_prefix(
            core_type,
            debug_command_hash,
            false,
        )?)?;
        debug!("Recovered addresses: {:?}", addresses);
        Ok(addresses)
    }

    fn unsafely_recover_signer_addresses_from_ethersjs_signature(
        &self,
        signature: &EthSignature,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<Vec<EthAddress>> {
        info!("✔ Attempting to unsafely recover ethersjs-style signature...");
        self.hash_to_hex(core_type, debug_command_hash)
            .and_then(|hex| Ok(hex::decode(strip_hex_prefix(&hex))?))
            .map(|bytes| {
                // NOTE: So apparently the ethersjs signing fxn for some reason will convert
                // hex to UTF8 regardless of whether it's valid or not, and then form the message
                // to be signed around _that_. We can only do the same here using unsafe rust :/
                let message: String;
                unsafe {
                    message = std::str::from_utf8_unchecked(&bytes).to_string();
                }
                message
            })
            .map(|message| Self::get_eth_prefixed_message_bytes(&message))
            .and_then(|bytes| signature.recover_both_signer_addresses(&H256::from_slice(&bytes)))
            .map(|addresses| {
                debug!("Recovered addresses: {:?}", addresses);
                addresses
            })
    }

    #[allow(clippy::if_same_then_else)]
    pub fn validate(&self, signature: &EthSignature, core_type: &CoreType, debug_command_hash: &H256) -> Result<()> {
        debug!("validating debug signature over debug command hash: 0x{debug_command_hash}");
        let needle = self.eth_address;
        let haystack = [
            self.recover_signer_addresses(signature, core_type, debug_command_hash)?,
            self.recover_signer_addresses_using_correct_eth_prefixing_scheme(signature, debug_command_hash)?,
            self.recover_signer_addresses_using_eth_prefix_and_hex_prefix(signature, core_type, debug_command_hash)?,
            self.recover_signer_addresses_using_eth_prefix_and_no_hex_prefix(signature, core_type, debug_command_hash)?,
        ]
        .concat();

        if haystack.contains(&needle) {
            Ok(())
        } else if self
            .unsafely_recover_signer_addresses_from_ethersjs_signature(signature, core_type, debug_command_hash)?
            .contains(&needle)
        {
            // NOTE: We run this check in a separate arm since it uses `unsafe` code so we don't
            // want it running by default every time we validate a debug signature!
            Ok(())
        } else {
            Err("Could not validate debug signature!".into())
        }
    }
}

#[cfg(test)]
use common_eth::{EthPrivateKey, EthSigningCapabilities};

#[cfg(test)]
impl DebugSignatory {
    pub fn sign_with_eth_prefix(
        &self,
        pk: &EthPrivateKey,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<EthSignature> {
        self.hash_with_eth_prefix(core_type, debug_command_hash, false)
            .and_then(|hash| pk.sign_hash_and_set_eth_recovery_param(hash))
    }

    #[cfg(test)]
    pub fn sign(&self, pk: &EthPrivateKey, core_type: &CoreType, debug_command_hash: &H256) -> Result<EthSignature> {
        self.hash(core_type, debug_command_hash)
            .and_then(|hash| pk.sign_hash_and_set_eth_recovery_param(hash))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::errors::AppError;
    use common_eth::convert_hex_to_eth_address;
    use ethereum_types::H256 as EthHash;

    use super::*;
    use crate::test_utils::{get_sample_debug_command_hash, get_sample_debug_signatory, get_sample_private_key};

    #[test]
    fn should_sign_debug_signatory_hash() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let result = hex::encode(signatory.sign(&pk, &core_type, &debug_command_hash).unwrap().to_vec());
        let expected_result = "7ee719e38908f63a26aca7a3957b7573156e01e9a853ada12ae877789d4c95a06334c13f397b050a5318867325cf60ebdcce1145c9f786ca522cb3d9bc9ab5a91b";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_recover_correct_address_from_signature_over_debug_signatory_hash() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign(&pk, &core_type, &debug_command_hash).unwrap();
        let result = signatory
            .recover_signer_addresses(&signature, &core_type, &debug_command_hash)
            .unwrap();
        let expected_result = pk.to_public_key().to_address();
        assert!(result.contains(&expected_result));
    }

    #[test]
    fn should_validate_debug_signature() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign(&pk, &core_type, &debug_command_hash).unwrap();
        let result = signatory.validate(&signature, &core_type, &debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_sign_with_eth_prefix() {
        let core_type = CoreType::BtcOnInt;
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let pk = get_sample_private_key();
        let result = debug_signatory
            .sign_with_eth_prefix(&pk, &core_type, &debug_command_hash)
            .unwrap();
        let expected_result = EthSignature::from_str("9af3fde7df5987f7d43b4b27948d089eac4495595e731891fb3b9e7a78eca0a151fc6bfa50f75783e65197c4a1ff07969112df0b6b1d3e72538c566aed4dab221b").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_recover_address_from_eth_prefixed_signature() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory
            .sign_with_eth_prefix(&pk, &core_type, &debug_command_hash)
            .unwrap();
        let result = signatory
            .recover_signer_addresses_using_eth_prefix_and_no_hex_prefix(&signature, &core_type, &debug_command_hash)
            .unwrap();
        let expected_result = pk.to_public_key().to_address();
        assert!(result.contains(&expected_result));
    }

    #[test]
    fn should_validate_with_eth_prefix_signature() {
        let core_type = CoreType::BtcOnInt;
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let pk = get_sample_private_key();
        let eth_prefixed_signature = debug_signatory
            .sign_with_eth_prefix(&pk, &core_type, &debug_command_hash)
            .unwrap();
        let result = debug_signatory.validate(&eth_prefixed_signature, &core_type, &debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_to_validate_signature_using_wrong_eth_address() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = debug_signatory.sign(&pk, &core_type, &debug_command_hash).unwrap();
        let wrong_debug_signatory = DebugSignatory::random();
        assert_ne!(wrong_debug_signatory.eth_address, pk.to_public_key().to_address());
        let expected_error = "Could not validate debug signature!".to_string();
        match wrong_debug_signatory.validate(&signature, &core_type, &debug_command_hash) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_validate_ethersjs_style_signature() {
        let core_type = CoreType::BtcOnEth;
        let debug_command_hash = keccak_hash_bytes(&serde_json::to_vec("debug_get_all_db_keys").unwrap());
        let eth_address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let debug_signatory = DebugSignatory::new("some_name", &eth_address);
        let signature = EthSignature::from_str("0xc716ec191db45cf73f6dbf6a6983d3706654339cd190700904e4f5c11ed3002f4f6d5e6056ab64b192ca8360c82b9a13ef81317c216489f97e367ed9cadca73e1b").unwrap();
        let result = debug_signatory.validate(&signature, &core_type, &debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_validate_ethersjs_style_signature_2() {
        let core_type = CoreType::BtcOnEth;
        let debug_command_hash = keccak_hash_bytes(&serde_json::to_vec("debug_get_all_db_keys").unwrap());
        let eth_address = convert_hex_to_eth_address("0xAf61eE43b6B8Ce7B511f6105AD2Ab516fC646205").unwrap();
        let debug_signatory = DebugSignatory {
            name: "alain1".to_string(),
            eth_address,
            nonce: 3,
        };
        let ethersjs_signature = EthSignature::from_str("0x8a49a58622d13e99e19275c63fefe92e8e9788aa9ee772950998b4eefa3be1f23dccf6f2324adc093ff812f07dfb266f5d0f9788d9ac2c762a4cc6b8e3a14e0800").unwrap();
        let ethersjs_result = debug_signatory.validate(&ethersjs_signature, &core_type, &debug_command_hash);
        assert!(ethersjs_result.is_ok());
    }

    #[test]
    fn should_validate_mycrypto_style_signature_2() {
        let core_type = CoreType::BtcOnEth;
        let debug_command_hash = keccak_hash_bytes(&serde_json::to_vec("debug_get_all_db_keys").unwrap());
        let eth_address = convert_hex_to_eth_address("0xAf61eE43b6B8Ce7B511f6105AD2Ab516fC646205").unwrap();
        let debug_signatory = DebugSignatory {
            name: "alain1".to_string(),
            eth_address,
            nonce: 3,
        };
        let mycrypto_signature = EthSignature::from_str("0xce752850af5f3e2c9fbe58586581d25fc410d1dd4f5734538c91e632ed5879c77563b9d3e4d28feec585b8d359cf47c17088818c0220b6ddf3ca97c666e84dec01").unwrap(); //mycrypto
        let mycrypto_result = debug_signatory.validate(&mycrypto_signature, &core_type, &debug_command_hash);
        assert!(mycrypto_result.is_ok());
    }

    #[test]
    fn should_validate_v3_signature() {
        use simple_logger;
        simple_logger::init().unwrap();
        let core_type = CoreType::V3Strongbox;
        let debug_command_hash =
            EthHash::from_str("0x29bc8048453c48ed8e3a4826a7a1a2b0de8b6c0faaa19052b2d0a183a834b4cc").unwrap();
        let eth_address = EthAddress::from_str("0xdeb1e28d9f05543c03b1685ca248f236fc783a26").unwrap();
        let debug_signatory = DebugSignatory {
            name: "user".to_string(),
            eth_address,
            nonce: 0,
        };
        let signature = EthSignature::from_str("e4667602615ae06ea3159aa6ccff43fb26a66ec59852c38896a7acc138714f1209a860eef6a0474a93623103fc211ecef3c4fbe61084381a7ed7f93d943fdd1d1b").unwrap();
        let r = debug_signatory.validate(&signature, &core_type, &debug_command_hash);
        assert!(r.is_ok());
    }
}
