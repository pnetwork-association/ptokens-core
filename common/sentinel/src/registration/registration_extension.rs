use common_chain_ids::EthChainId;
use common_eth::{encode_fxn_call, EthPrivateKey, EthTransaction};
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, U256};

use crate::{NetworkId, SentinelError};

const REGISTRATION_EXTENSION_GAS_LIMIT: usize = 100_000;

const REGISTRATION_ABI_FRAGMENT: &str = "[{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"duration\",\"type\":\"uint64\"}],\"name\":\"increaseSentinelRegistrationDuration\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

pub fn get_registration_extension_tx(
    nonce: u64,
    duration: u64,
    gas_price: u64,
    network_id: NetworkId,
    owner_pk: &EthPrivateKey,
    registration_manager: EthAddress,
) -> Result<EthTransaction, SentinelError> {
    debug!("getting registation extension tx...");
    let value = 0;
    let ecid = EthChainId::try_from(network_id)?;
    let data = encode_fxn_call(REGISTRATION_ABI_FRAGMENT, "increaseSentinelRegistrationDuration", &[
        EthAbiToken::Uint(U256::from(duration)),
    ])?;
    Ok(EthTransaction::new_unsigned(
        data,
        nonce,
        value,
        registration_manager,
        &ecid,
        REGISTRATION_EXTENSION_GAS_LIMIT,
        gas_price,
    )
    .sign(owner_pk)?)
}
