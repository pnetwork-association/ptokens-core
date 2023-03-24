use common_eth::convert_hex_to_h256;
use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};

use crate::{ConfigT, SentinelError};

macro_rules! setup_topics {
    ($($name:ident => $hex:expr),* $(,)?) => {
        $(
            lazy_static! {
                static ref $name: EthHash = convert_hex_to_h256(&$hex)
                    .expect(&format!("Converting from hex shouldn't fail for {}", stringify!($name)));
            }
        )*
    }
}

setup_topics!(
    // NOTE: On the host side we have an token contract, which when a relayer calls the mint function
    // will emit a minted event. When a user pegs out, the peg out event emitted.
    HOST_MINTED_TOPIC => "2fe5be0146f74c5bce36c0b80911af6c7d86ff27e89d5cfa61fc681327954e5d",
    HOST_PEG_OUT_TOPIC => "dd56da0e6e7b301867b3632876d707f60c7cbf4b06f9ae191c67ea016cc5bf31",

    // NOTE: On the native side we have a vault. When a user pegs in, a peg in event is fired. When a
    // relayer pegs out, there is not event, only the erc20 transfer from the vault address.
    NATIVE_PEG_IN_TOPIC => "c03be660a5421fb17c93895da9db564bd4485d475f0d8b3175f7d55ed421bebb",
    NATIVE_ERC20_TRANSFER_TOPIC => "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
);

#[derive(Clone, Debug, Default, Constructor, Serialize, Deserialize)]
pub struct AddressAndTopic {
    pub(crate) address: EthAddress,
    pub(crate) topic: EthHash,
}

pub trait AddressesAndTopicsT {
    fn from_config<C: ConfigT>(config: &C) -> Result<Self, SentinelError>
    where
        Self: Sized;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Constructor, IntoIterator)]
pub struct NativeAddressesAndTopics(Vec<AddressAndTopic>);

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Constructor, IntoIterator)]
pub struct HostAddressesAndTopics(Vec<AddressAndTopic>);

impl AddressesAndTopicsT for NativeAddressesAndTopics {
    fn from_config<C: ConfigT>(config: &C) -> Result<Self, SentinelError> {
        if config.side().is_host() {
            return Err(SentinelError::Custom(
                "Cannot create native addresses and topics from host config!".into(),
            ));
        }
        let mut r: Vec<AddressAndTopic> = vec![];
        let addresses = config.get_contract_addresses();
        for address in addresses {
            r.push(AddressAndTopic::new(address, *NATIVE_PEG_IN_TOPIC));
            r.push(AddressAndTopic::new(address, *NATIVE_ERC20_TRANSFER_TOPIC));
        }
        Ok(Self::new(r))
    }
}

impl AddressesAndTopicsT for HostAddressesAndTopics {
    fn from_config<C: ConfigT>(config: &C) -> Result<Self, SentinelError> {
        if config.side().is_native() {
            return Err(SentinelError::Custom(
                "Cannot create host addresses and topics from host config!".into(),
            ));
        }
        let mut r: Vec<AddressAndTopic> = vec![];
        let addresses = config.get_contract_addresses();
        for address in addresses {
            r.push(AddressAndTopic::new(address, *HOST_MINTED_TOPIC));
            r.push(AddressAndTopic::new(address, *HOST_PEG_OUT_TOPIC));
        }
        Ok(Self::new(r))
    }
}

#[cfg(test)]
mod tests {
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::config::{ContractInfo, ContractInfos, HostConfig, NativeConfig};

    fn get_sample_contract_infos() -> ContractInfos {
        let name = "pBTC".to_string();
        let address = convert_hex_to_eth_address("0xedB86cd455ef3ca43f0e227e00469C3bDFA40628").unwrap();
        let contract_info = ContractInfo::new(name, address);
        ContractInfos::new(vec![contract_info])
    }

    #[test]
    fn should_get_native_addresses_and_topics_from_native_config() {
        let mut config = NativeConfig::default();
        config.set_contract_infos(get_sample_contract_infos());
        let result = NativeAddressesAndTopics::from_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_addresses_and_topics_from_host_config() {
        let mut config = HostConfig::default();
        config.set_contract_infos(get_sample_contract_infos());
        let result = HostAddressesAndTopics::from_config(&config);
        assert!(result.is_ok());
    }
}
