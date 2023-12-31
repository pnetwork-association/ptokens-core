#[cfg(test)]
use std::str::from_utf8;
use std::str::FromStr;

use bitcoin::Address as BtcAddress;
#[cfg(test)]
use common::types::Byte;
use common::{types::Bytes, utils::strip_hex_prefix, Result};
use common_safe_addresses::{
    safely_convert_str_to_algo_address,
    safely_convert_str_to_btc_address,
    safely_convert_str_to_eos_address,
    safely_convert_str_to_eth_address,
    safely_convert_str_to_ltc_address,
};
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;
#[cfg(test)]
use litecoin::Address as LtcAddress;
use rust_algorand::{AlgorandAddress, AlgorandAppId};
use serde::{Deserialize, Serialize};

use crate::{MetadataChainId, MetadataProtocolId};

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataAddress {
    pub address: String,
    pub metadata_chain_id: MetadataChainId,
}

impl MetadataAddress {
    pub fn new(address: &str, metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let parsed_address = match metadata_chain_id.to_protocol_id() {
            MetadataProtocolId::Bitcoin => {
                // NOTE: We don't differentiate the MetadataProtocolId for these two chains...
                if metadata_chain_id == &MetadataChainId::LitecoinMainnet {
                    info!("✔ Getting `MetadataAddress` for a LTC address...");
                    safely_convert_str_to_ltc_address(address).to_string()
                } else {
                    info!("✔ Getting `MetadataAddress` for a BTC address...");
                    safely_convert_str_to_btc_address(address).to_string()
                }
            },
            MetadataProtocolId::Ethereum => {
                info!("✔ Getting `MetadataAddress` for an ETH address...");
                format!("0x{}", hex::encode(safely_convert_str_to_eth_address(address)))
            },
            MetadataProtocolId::Eos => {
                info!("✔ Getting `MetadataAddress` for an EOS address...");
                safely_convert_str_to_eos_address(address).to_string()
            },
            MetadataProtocolId::Algorand => {
                info!("✔ Getting `MetadataAddress` for an ALGO address...");
                match AlgorandAppId::from_str(address) {
                    Ok(app_id) => {
                        info!("Algorand metadata address is actually an application ID: '{}'!", app_id);
                        app_id.to_string()
                    },
                    Err(_) => safely_convert_str_to_algo_address(address).to_string(),
                }
            },
        };
        let metadata_address = Self {
            address: parsed_address,
            metadata_chain_id: *metadata_chain_id,
        };
        info!("✔ Successfully parsed `metadata_address`: {:?}", metadata_address);
        Ok(metadata_address)
    }

    fn get_err_msg(protocol: MetadataProtocolId) -> String {
        let symbol = protocol.to_symbol();
        format!(
            "`MetadataAddress` error - {} address supplied with non-{} chain ID!",
            symbol, symbol
        )
    }

    pub fn new_from_eth_address(eth_address: &EthAddress, metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let protocol_id = metadata_chain_id.to_protocol_id();
        match protocol_id {
            MetadataProtocolId::Ethereum => Ok(Self {
                metadata_chain_id: *metadata_chain_id,
                address: hex::encode(eth_address),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn new_from_eos_address(eos_address: &EosAddress, metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let protocol_id = metadata_chain_id.to_protocol_id();
        match protocol_id {
            MetadataProtocolId::Eos => Ok(Self {
                metadata_chain_id: *metadata_chain_id,
                address: eos_address.to_string(),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn new_from_btc_address(btc_address: &BtcAddress, metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let protocol_id = metadata_chain_id.to_protocol_id();
        match protocol_id {
            MetadataProtocolId::Bitcoin => Ok(Self {
                metadata_chain_id: *metadata_chain_id,
                address: btc_address.to_string(),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        match self.metadata_chain_id.to_protocol_id() {
            MetadataProtocolId::Bitcoin => Ok(self.address.as_bytes().to_vec()),
            MetadataProtocolId::Ethereum => Ok(hex::decode(strip_hex_prefix(&self.address))?),
            MetadataProtocolId::Algorand => Ok(AlgorandAddress::from_str(&self.address)?.to_bytes()),
            MetadataProtocolId::Eos => Ok(EosAddress::from_str(&self.address)?.as_u64().to_le_bytes().to_vec()),
        }
    }
}

impl std::fmt::Display for MetadataAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.metadata_chain_id.to_protocol_id() {
            MetadataProtocolId::Ethereum => write!(f, "0x{}", strip_hex_prefix(&self.address)),
            _ => write!(f, "{}", self.address),
        }
    }
}

#[cfg(test)]
impl MetadataAddress {
    fn from_bytes(bytes: &[Byte], metadata_chain_id: &MetadataChainId) -> Result<Self> {
        match metadata_chain_id.to_protocol_id() {
            MetadataProtocolId::Eos => Self::from_bytes_for_eos(bytes, metadata_chain_id),
            MetadataProtocolId::Ethereum => Self::from_bytes_for_eth(bytes, metadata_chain_id),
            MetadataProtocolId::Algorand => Self::from_bytes_for_algo(bytes, metadata_chain_id),
            MetadataProtocolId::Bitcoin => {
                if metadata_chain_id == &MetadataChainId::LitecoinMainnet {
                    Self::from_bytes_for_ltc(bytes)
                } else {
                    Self::from_bytes_for_btc(bytes, metadata_chain_id)
                }
            },
        }
    }

    fn from_bytes_for_eth(bytes: &[Byte], metadata_chain_id: &MetadataChainId) -> Result<Self> {
        info!("✔ Attempting to create `MetadataAddress` from bytes for ETH...");
        let eth_address_size_in_bytes = 20;
        if bytes.len() == eth_address_size_in_bytes {
            Self::new_from_eth_address(&EthAddress::from_slice(bytes), metadata_chain_id)
        } else {
            Err("Incorrect number of bytes to convert to ETH address in `MetadataAddress`!".into())
        }
    }

    fn from_bytes_for_algo(bytes: &[Byte], metadata_chain_id: &MetadataChainId) -> Result<Self> {
        info!("✔ Attempting to create `MetadataAddress` from bytes for ALGO...");
        let algo_address_length_in_bytes: usize = 32;
        if bytes.len() == algo_address_length_in_bytes {
            Self::new(&AlgorandAddress::from_bytes(bytes)?.to_string(), metadata_chain_id)
        } else {
            Err("Incorrect number of bytes to convert to ALGO address in `MetadataAddress`!".into())
        }
    }

    fn from_bytes_for_btc(bytes: &[Byte], metadata_chain_id: &MetadataChainId) -> Result<Self> {
        info!("✔ Attempting to create `MetadataAddress` from bytes for BTC...");
        match from_utf8(bytes) {
            Err(err) => Err(format!("Error converting bytes to utf8 in `MetadataAddress`: {}", err).into()),
            Ok(btc_address_str) => match BtcAddress::from_str(btc_address_str) {
                Ok(ref a) => Self::new_from_btc_address(a, metadata_chain_id),
                Err(err) => Err(format!("Error converting bytes to BTC address in `MetadataAddress`: {}", err).into()),
            },
        }
    }

    fn from_bytes_for_ltc(bytes: &[Byte]) -> Result<Self> {
        info!("attempting to create `MetadataAddress` from bytes for LTC...");
        match from_utf8(bytes) {
            Err(err) => Err(format!("error converting bytes to utf8 in `MetadataAddress`: {}", err).into()),
            Ok(ltc_address_str) => match LtcAddress::from_str(ltc_address_str) {
                Ok(_) => Self::new(ltc_address_str, &MetadataChainId::LitecoinMainnet),
                Err(err) => Err(format!("Error converting bytes to LTC address in `MetadataAddress`: {}", err).into()),
            },
        }
    }

    fn from_bytes_for_eos(bytes: &[Byte], metadata_chain_id: &MetadataChainId) -> Result<Self> {
        info!("✔ Attempting to create `MetadataAddress` from bytes for EOS...");
        let eos_address_length_in_bytes = 8;
        let num_bytes = bytes.len();
        if num_bytes != eos_address_length_in_bytes {
            Err(format!(
                "Incorrect number of bytes for EOS address. Expected {}, got {}!",
                eos_address_length_in_bytes, num_bytes
            )
            .into())
        } else {
            Self::new_from_eos_address(
                &EosAddress::from(u64::from_le_bytes(bytes.try_into()?)),
                metadata_chain_id,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use common::utils::convert_hex_to_eth_address;

    use super::*;
    use crate::test_utils::{
        get_sample_algo_origin_address,
        get_sample_btc_address,
        get_sample_btc_origin_address,
        get_sample_eos_address,
        get_sample_eos_origin_address,
        get_sample_eth_address,
        get_sample_eth_origin_address,
    };

    #[test]
    fn should_get_metadata_address_from_eos_address() {
        let metadata_chain_id = MetadataChainId::TelosMainnet;
        let result = MetadataAddress::new_from_eos_address(&get_sample_eos_address(), &metadata_chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_metadata_address_from_btc_address() {
        let metadata_chain_id = MetadataChainId::BitcoinMainnet;
        let result = MetadataAddress::new_from_btc_address(&get_sample_btc_address(), &metadata_chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_metadata_address_from_eth_address() {
        let metadata_chain_id = MetadataChainId::EthereumRopsten;
        let result = MetadataAddress::new_from_eth_address(&get_sample_eth_address(), &metadata_chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_do_btc_address_bytes_roundtrip() {
        let metadata_address = get_sample_btc_origin_address();
        let metadata_chain_id = metadata_address.metadata_chain_id;
        let bytes = metadata_address.to_bytes().unwrap();
        let result = MetadataAddress::from_bytes(&bytes, &metadata_chain_id).unwrap();
        assert_eq!(result, metadata_address);
    }

    #[test]
    fn should_do_eth_address_bytes_roundtrip() {
        let metadata_address = get_sample_eth_origin_address();
        let metadata_chain_id = metadata_address.metadata_chain_id;
        let bytes = metadata_address.to_bytes().unwrap();
        let result = MetadataAddress::from_bytes(&bytes, &metadata_chain_id).unwrap();
        assert_eq!(result, metadata_address);
    }

    #[test]
    fn should_do_eos_address_bytes_roundtrip() {
        let metadata_address = get_sample_eos_origin_address();
        let metadata_chain_id = metadata_address.metadata_chain_id;
        let bytes = metadata_address.to_bytes().unwrap();
        let result = MetadataAddress::from_bytes(&bytes, &metadata_chain_id).unwrap();
        assert_eq!(result, metadata_address);
    }

    #[test]
    fn eth_metadata_address_should_add_hex_prefix() {
        let address_string = "0xea674fdde714fd979de3edf0f56aa9716b898ec8";
        let eth_address = convert_hex_to_eth_address(address_string).unwrap();
        let metadata_chain_id = MetadataChainId::EthereumMainnet;
        let metadata_address = MetadataAddress::new_from_eth_address(&eth_address, &metadata_chain_id).unwrap();
        assert_eq!(metadata_address.to_string(), address_string);
    }

    #[test]
    fn should_do_algo_address_bytes_roundtrip() {
        let metadata_address = get_sample_algo_origin_address();
        let metadata_chain_id = metadata_address.metadata_chain_id;
        let bytes = metadata_address.to_bytes().unwrap();
        let result = MetadataAddress::from_bytes(&bytes, &metadata_chain_id).unwrap();
        assert_eq!(result, metadata_address);
    }

    fn get_sample_ltc_address<'a>() -> &'a str {
        "ltc1qwhpua5f58j3jj6w5t2v4wjlfyndx3ql8ex6a5r"
    }

    #[test]
    fn ltc_address_should_pass_validation() {
        let ltc_address = get_sample_ltc_address();
        let metadata_chain_id = MetadataChainId::LitecoinMainnet;
        let result = MetadataAddress::new(ltc_address, &metadata_chain_id).unwrap();
        let expected_result = MetadataAddress {
            metadata_chain_id,
            address: ltc_address.to_string(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_do_ltc_address_bytes_roundtrip() {
        let ltc_address = get_sample_ltc_address();
        let id = MetadataChainId::LitecoinMainnet;
        let a = MetadataAddress::new(ltc_address, &id).unwrap();
        let bs = a.to_bytes().unwrap();
        let result = MetadataAddress::from_bytes(&bs, &id).unwrap();
        let expected_result = MetadataAddress {
            metadata_chain_id: id,
            address: ltc_address.to_string(),
        };
        assert_eq!(result, expected_result);
    }
}
