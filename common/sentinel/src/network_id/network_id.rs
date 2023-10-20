use std::{fmt, str::FromStr};

use common::{Byte, Bytes};
use common_chain_ids::EthChainId;
use common_metadata::{MetadataChainId, MetadataChainIdError};
use derive_more::{Constructor, Deref};
use ethabi::{encode as ethabi_encode, Token};
use ethereum_types::U256;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{NetworkIdError, NetworkIdVersion, ProtocolId};

const NUM_BYTES: usize = 4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deref, Serialize, Deserialize)]
pub struct Bytes4([u8; NUM_BYTES]);

impl TryFrom<Bytes> for Bytes4 {
    type Error = NetworkIdError;

    fn try_from(bs: Bytes) -> Result<Self, Self::Error> {
        let l = bs.len();
        if l < 4 {
            Err(NetworkIdError::NotEnoughBytes {
                expected: NUM_BYTES,
                got: l,
            })
        } else {
            Ok(Self([bs[0], bs[1], bs[2], bs[3]]))
        }
    }
}

impl fmt::Display for Bytes4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[derive(Clone, Debug, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NetworkId {
    chain_id: u64,       // FIXME make this a u64, since that's how it's encoded
    disambiguator: Byte, // NOTE: Can be rolled in case of collisions.
    protocol_id: ProtocolId,
    version: NetworkIdVersion,
}

impl NetworkId {
    pub fn to_hex(&self) -> Result<String, NetworkIdError> {
        Ok(self.to_bytes_4()?.to_string())
    }

    pub fn new(chain_id: u64, protocol_id: ProtocolId) -> Self {
        Self::new_v1(chain_id, protocol_id)
    }

    pub fn new_v1_for_evm(ecid: EthChainId) -> Self {
        Self::new_v1(ecid.to_u64(), ProtocolId::Ethereum)
    }

    fn new_v1(chain_id: u64, protocol_id: ProtocolId) -> Self {
        Self {
            chain_id,
            protocol_id,
            disambiguator: 0,
            version: NetworkIdVersion::V1,
        }
    }
}

impl NetworkId {
    fn abi_encode(&self) -> Bytes {
        ethabi_encode(&[
            Token::FixedBytes([<NetworkIdVersion as Into<u8>>::into(self.version)].to_vec()),
            Token::FixedBytes([<ProtocolId as Into<u8>>::into(self.protocol_id)].to_vec()),
            Token::Uint(U256::from(self.chain_id)),
            Token::FixedBytes([self.disambiguator].to_vec()),
        ])
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut hasher = Sha256::new();
        hasher.update(self.abi_encode());
        hasher.finalize().to_vec()
    }

    pub fn to_bytes_4(&self) -> Result<Bytes4, NetworkIdError> {
        Bytes4::try_from(self.to_bytes())
    }
}

impl TryFrom<&Vec<u8>> for NetworkId {
    type Error = NetworkIdError;

    fn try_from(bs: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_str(&hex::encode(bs))
    }
}

impl TryFrom<Vec<u8>> for NetworkId {
    type Error = NetworkIdError;

    fn try_from(bs: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_str(&hex::encode(bs))
    }
}

impl TryFrom<&[u8]> for NetworkId {
    type Error = NetworkIdError;

    fn try_from(bs: &[u8]) -> Result<Self, Self::Error> {
        Self::from_str(&hex::encode(bs))
    }
}

impl FromStr for NetworkId {
    type Err = NetworkIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "0xf9b459a1" | "f9b459a1" | "polygon" | "matic" => Ok(Self::new_v1_for_evm(EthChainId::PolygonMainnet)),
            // TODO others!
            other => Err(NetworkIdError::InvalidNetworkId(other.to_string())),
        }
    }
}

impl TryInto<Bytes4> for NetworkId {
    type Error = NetworkIdError;

    fn try_into(self) -> Result<Bytes4, NetworkIdError> {
        self.to_bytes_4()
    }
}

impl TryFrom<MetadataChainId> for NetworkId {
    type Error = MetadataChainIdError;

    fn try_from(m: MetadataChainId) -> Result<NetworkId, Self::Error> {
        NetworkId::try_from(&m)
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_bytes_4() {
            Err(e) => write!(f, "error converting network id to bytes: {e}"),
            Ok(b4) => {
                #[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
                struct Temp {
                    bytes: String,
                    chain_id: u64,
                    disambiguator: String,
                    protocol_id: ProtocolId,
                    version: NetworkIdVersion,
                }
                let t = Temp::new(
                    b4.to_string(),
                    self.chain_id,
                    format!("0x{:x}", self.disambiguator),
                    self.protocol_id,
                    self.version,
                );
                write!(f, "{}", json!(t))
            },
        }
    }
}

impl TryFrom<&MetadataChainId> for NetworkId {
    type Error = MetadataChainIdError;

    fn try_from(m: &MetadataChainId) -> Result<NetworkId, Self::Error> {
        match m {
            MetadataChainId::EthUnknown => Ok(NetworkId::new_v1_for_evm(EthChainId::Unknown(0))),
            MetadataChainId::BscMainnet => Ok(NetworkId::new_v1_for_evm(EthChainId::BscMainnet)),
            MetadataChainId::EthereumGoerli => Ok(NetworkId::new_v1_for_evm(EthChainId::Goerli)),
            MetadataChainId::EthereumMainnet => Ok(NetworkId::new_v1_for_evm(EthChainId::Mainnet)),
            MetadataChainId::EthereumRopsten => Ok(NetworkId::new_v1_for_evm(EthChainId::Ropsten)),
            MetadataChainId::EthereumRinkeby => Ok(NetworkId::new_v1_for_evm(EthChainId::Rinkeby)),
            MetadataChainId::EthereumSepolia => Ok(NetworkId::new_v1_for_evm(EthChainId::Sepolia)),
            MetadataChainId::XDaiMainnet => Ok(NetworkId::new_v1_for_evm(EthChainId::XDaiMainnet)),
            MetadataChainId::InterimChain => Ok(NetworkId::new_v1_for_evm(EthChainId::InterimChain)),
            MetadataChainId::FantomMainnet => Ok(NetworkId::new_v1_for_evm(EthChainId::FantomMainnet)),
            MetadataChainId::PolygonMainnet => Ok(NetworkId::new_v1_for_evm(EthChainId::PolygonMainnet)),
            MetadataChainId::ArbitrumMainnet => Ok(NetworkId::new_v1_for_evm(EthChainId::ArbitrumMainnet)),
            MetadataChainId::LuxochainMainnet => Ok(NetworkId::new_v1_for_evm(EthChainId::LuxochainMainnet)),
            mcid => Err(Self::Error::CannotConvertTo(*mcid, "NetworkId".to_string())),
        }
    }
}

impl TryFrom<NetworkId> for MetadataChainId {
    type Error = NetworkIdError;

    fn try_from(m: NetworkId) -> Result<MetadataChainId, Self::Error> {
        MetadataChainId::try_from(&m)
    }
}

impl TryFrom<&NetworkId> for MetadataChainId {
    type Error = NetworkIdError;

    fn try_from(m: &NetworkId) -> Result<MetadataChainId, Self::Error> {
        let err = NetworkIdError::CannotConvert {
            from: m.clone(),
            to: "MetadataChainId".to_string(),
        };
        if let Ok(ecid) = EthChainId::try_from(m.chain_id) {
            match ecid {
                EthChainId::BscMainnet => Ok(MetadataChainId::BscMainnet),
                EthChainId::Goerli => Ok(MetadataChainId::EthereumGoerli),
                EthChainId::Mainnet => Ok(MetadataChainId::EthereumMainnet),
                EthChainId::Ropsten => Ok(MetadataChainId::EthereumRopsten),
                EthChainId::Rinkeby => Ok(MetadataChainId::EthereumRinkeby),
                EthChainId::Sepolia => Ok(MetadataChainId::EthereumSepolia),
                EthChainId::XDaiMainnet => Ok(MetadataChainId::XDaiMainnet),
                EthChainId::InterimChain => Ok(MetadataChainId::InterimChain),
                EthChainId::FantomMainnet => Ok(MetadataChainId::FantomMainnet),
                EthChainId::PolygonMainnet => Ok(MetadataChainId::PolygonMainnet),
                EthChainId::ArbitrumMainnet => Ok(MetadataChainId::ArbitrumMainnet),
                EthChainId::LuxochainMainnet => Ok(MetadataChainId::LuxochainMainnet),
                EthChainId::Unknown(_) => Err(err),
            }
        } else {
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use common_chain_ids::EthChainId;

    use super::*;

    #[test]
    fn should_get_network_id_as_bytes_4_correctly() {
        let protocol_id = ProtocolId::Ethereum;
        let chain_id = EthChainId::XDaiMainnet;
        let network_id = NetworkId::new(chain_id.to_u64(), protocol_id);
        let result = network_id.to_bytes_4().unwrap();
        let expected_result = Bytes4([212, 27, 28, 91]);
        assert_eq!(result, expected_result);
    }
}
