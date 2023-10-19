use common::crypto_utils::keccak_hash_bytes;
use common_eth::{EthPrivateKey, EthSignature, EthSigningCapabilities};
use ethereum_types::{U256, Address as EthAddress};
use ethabi::{Token as EthAbiToken, encode as eth_abi_encode};

use crate::SentinelError;

pub fn get_registration_signature(owner: &EthAddress, nonce: u64, pk: &EthPrivateKey) -> Result<EthSignature, SentinelError> {
    debug!("getting registration signature over address {owner}...");
    let bs = eth_abi_encode(&[EthAbiToken::Address(*owner), EthAbiToken::Uint(U256::from(nonce))]);
    Ok(pk.hash_and_sign_msg_with_eth_prefix(keccak_hash_bytes(&bs).as_bytes())?)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common_eth::convert_hex_to_eth_address;

    use super::*;

    #[test]
    fn should_get_registration_signature() {
        let nonce = 1337;
        let pk = EthPrivateKey::from_str("cfc1fa2e6fd0ccaf97265f464a4d45628263c755bcbc813c18db93539194683c").unwrap();
        let address = convert_hex_to_eth_address("8bf5c77932b68e157434e08c73a5a669ddeaec30").unwrap();
        let sig = get_registration_signature(&address, nonce, &pk).unwrap().to_string();
        let expected_sig = "0523deec98ceed455def55123b3846f679e713dead685caddbb5f91c6116d06914e90e32e34fc7e2ad7165e90abb5702a94d82c131312ddc84771b37cba022611c";
        assert_eq!(sig, expected_sig);
    }
}
