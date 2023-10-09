use common::crypto_utils::keccak_hash_bytes;
use common_eth::{EthPrivateKey, EthSignature, EthSigningCapabilities};
use ethereum_types::Address as EthAddress;

use crate::SentinelError;

pub fn get_registration_signature(owner: &EthAddress, pk: &EthPrivateKey) -> Result<EthSignature, SentinelError> {
    debug!("getting registration signature over address {owner}...");
    Ok(pk.hash_and_sign_msg_with_eth_prefix(keccak_hash_bytes(owner.as_bytes()).as_bytes())?)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common_eth::convert_hex_to_eth_address;

    use super::*;

    #[test]
    fn should_get_registration_signature() {
        let pk = EthPrivateKey::from_str("cfc1fa2e6fd0ccaf97265f464a4d45628263c755bcbc813c18db93539194683c").unwrap();
        let address = convert_hex_to_eth_address("8bf5c77932b68e157434e08c73a5a669ddeaec30").unwrap();
        let sig = get_registration_signature(&address, &pk).unwrap().to_string();
        let expected_sig = "8e2e3bc606d583b9a768480320d0d49dce6132e87e42f99c288d977a074a073317944bdffd8495e3a1f2924eceb9ec8a27f5dde9456a0b0b1ad33d390404768d1b";
        assert_eq!(sig, expected_sig);
    }
}
