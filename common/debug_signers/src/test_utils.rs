use common_eth::{convert_hex_to_eth_address, EthPrivateKey};
use ethereum_types::{Address as EthAddress, H256};

use crate::{DebugSignatories, DebugSignatory};

pub fn get_sample_debug_signatory() -> DebugSignatory {
    DebugSignatory::new(
        "Some signer name",
        &convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap(),
    )
}

pub fn get_sample_debug_command_hash() -> H256 {
    H256::from_slice(&hex::decode("ed0568d9281086a914e10ee97eb5526e059683b8d6e4f922648e71d30cd794f1").unwrap())
}

pub fn get_sample_private_key() -> EthPrivateKey {
    EthPrivateKey::from_slice(&hex::decode("1a19efff597d68186bf41da2f57a90c550258d4ebfbee5d17f265f1ef89c842f").unwrap())
        .unwrap()
}

pub fn get_sample_debug_signatories() -> DebugSignatories {
    DebugSignatories::new(vec![
        DebugSignatory::new("one", &EthAddress::random()),
        DebugSignatory::new("two", &EthAddress::random()).increment_nonce(),
        DebugSignatory::new("three", &EthAddress::random())
            .increment_nonce()
            .increment_nonce(),
    ])
}

pub fn get_n_random_debug_signatories(n: usize) -> DebugSignatories {
    DebugSignatories::new(
        vec![0; n]
            .iter()
            .map(|_| DebugSignatory::random())
            .collect::<Vec<DebugSignatory>>(),
    )
}
