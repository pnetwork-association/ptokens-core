use std::fmt;

use common::{Byte, Bytes};
use common_chain_ids::EthChainId;
use common_metadata::{MetadataChainId, MetadataChainIdError};
use derive_more::Deref;
use ethabi::{encode as ethabi_encode, Token};
use ethereum_types::U256;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{NetworkIdError, NetworkIdVersion, ProtocolId};

const NUM_BYTES: usize = 4;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deref)]
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NetworkId {
    chain_id: u64,       // FIXME make this a u64, since that's how it's encoded
    disambiguator: Byte, // NOTE: Can be rolled in case of collisions.
    protocol_id: ProtocolId,
    version: NetworkIdVersion,
}

impl NetworkId {
    pub fn new(chain_id: u64, protocol_id: ProtocolId) -> Self {
        Self::new_v1(chain_id, protocol_id)
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

impl TryInto<Bytes4> for NetworkId {
    type Error = NetworkIdError;

    fn try_into(self) -> Result<Bytes4, NetworkIdError> {
        self.to_bytes_4()
    }
}

impl TryFrom<MetadataChainId> for NetworkId {
    type Error = MetadataChainIdError;

    fn try_from(m: MetadataChainId) -> Result<NetworkId, Self::Error> {
        match m {
            MetadataChainId::EthUnknown => Ok(NetworkId::new(EthChainId::Unknown(0).to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::BscMainnet => Ok(NetworkId::new(EthChainId::BscMainnet.to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::XDaiMainnet => Ok(NetworkId::new(EthChainId::XDaiMainnet.to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::FantomMainnet => {
                Ok(NetworkId::new(EthChainId::FantomMainnet.to_u64(), ProtocolId::Ethereum))
            },
            MetadataChainId::InterimChain => {
                Ok(NetworkId::new(EthChainId::InterimChain.to_u64(), ProtocolId::Ethereum))
            },
            MetadataChainId::EthereumGoerli => Ok(NetworkId::new(EthChainId::Goerli.to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::PolygonMainnet => Ok(NetworkId::new(
                EthChainId::PolygonMainnet.to_u64(),
                ProtocolId::Ethereum,
            )),
            MetadataChainId::EthereumSepolia => Ok(NetworkId::new(EthChainId::Sepolia.to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::ArbitrumMainnet => Ok(NetworkId::new(
                EthChainId::ArbitrumMainnet.to_u64(),
                ProtocolId::Ethereum,
            )),
            MetadataChainId::EthereumMainnet => Ok(NetworkId::new(EthChainId::Mainnet.to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::EthereumRopsten => Ok(NetworkId::new(EthChainId::Ropsten.to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::EthereumRinkeby => Ok(NetworkId::new(EthChainId::Rinkeby.to_u64(), ProtocolId::Ethereum)),
            MetadataChainId::LuxochainMainnet => Ok(NetworkId::new(
                EthChainId::LuxochainMainnet.to_u64(),
                ProtocolId::Ethereum,
            )),
            mcid => Err(Self::Error::CannotConvertTo(mcid, "NetworkId".to_string())),
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
        let network_id = NetworkId::new(chain_id, protocol_id);
        let result = network_id.to_bytes_4().unwrap();
        let expected_result = Bytes4([212, 27, 28, 91]);
        assert_eq!(result, expected_result);
    }
}
