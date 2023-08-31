use std::{fmt, str::FromStr};

use common::{
    constants::THIRTY_TWO_ZERO_BYTES,
    traits::ChainId,
    types::{Byte, Bytes, Result},
    AppError,
};
use common_chain_ids::{AlgoChainId, BtcChainId, EosChainId, EthChainId};
use ethereum_types::H256 as KeccakHash;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::MetadataProtocolId;

pub const METADATA_CHAIN_ID_NUMBER_OF_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum MetadataChainId {
    EthereumMainnet,  // 0x005fe7f9
    EthereumRopsten,  // 0x0069c322
    EthereumRinkeby,  // 0x00f34368
    BitcoinMainnet,   // 0x01ec97de
    BitcoinTestnet,   // 0x018afeb2
    EosMainnet,       // 0x02e7261c
    TelosMainnet,     // 0x028c7109
    BscMainnet,       // 0x00e4b170
    EosJungleTestnet, // 0x0282317f
    XDaiMainnet,      // 0x00f1918e
    PolygonMainnet,   // 0x0075dd4c
    UltraMainnet,     // 0x02f9337d
    FioMainnet,       // 0x02174f20
    UltraTestnet,     // 0x02b5a4d6
    EthUnknown,       // 0x00000000
    BtcUnknown,       // 0x01000000
    EosUnknown,       // 0x02000000
    InterimChain,     // 0xffffffff
    ArbitrumMainnet,  // 0x00ce98c4
    LuxochainMainnet, // 0x00d5beb0
    FantomMainnet,    // 0x0022af98
    AlgorandMainnet,  // 0x03c38e67
    LibreTestnet,     // 0x02a75f2c
    LibreMainnet,     // 0x026776fa
    EthereumGoerli,   // 0x00b4f6c5
    EthereumSepolia,  // 0x0030d6b5
    LitecoinMainnet,  // 0x01840435
}

impl Default for MetadataChainId {
    fn default() -> Self {
        Self::InterimChain
    }
}

impl FromStr for MetadataChainId {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            // Algorand...
            "AlgorandMainnet" | "0x03c38e67" => Ok(Self::AlgorandMainnet),

            // Bitcoin...
            "BtcUnknown" | "0x01000000" => Ok(Self::BtcUnknown),
            "BitcoinMainnet" | "0x01ec97de" => Ok(Self::BitcoinMainnet),
            "BitcoinTestnet" | "0x018afeb2" => Ok(Self::BitcoinTestnet),

            // Eos...
            "EosMainnet" | "0x02e7261c" => Ok(Self::EosMainnet),
            "FioMainnet" | "0x02174f20" => Ok(Self::FioMainnet),
            "EosUnknown" | "0x02000000" => Ok(Self::EosUnknown),
            "TelosMainnet" | "0x028c7109" => Ok(Self::TelosMainnet),
            "UltraTestnet" | "0x02b5a4d6" => Ok(Self::UltraTestnet),
            "LibreTestnet" | "0x02a75f2c" => Ok(Self::LibreTestnet),
            "LibreMainnet" | "0x026776fa" => Ok(Self::LibreMainnet),
            "UltraMainnet" | "0x025d3c68" => Ok(Self::UltraMainnet),
            "EosJungleTestnet" | "0x0282317f" => Ok(Self::EosJungleTestnet),

            // Eth...
            "EthUnknown" | "0x00000000" => Ok(Self::EthUnknown),
            "BscMainnet" | "0x00e4b170" => Ok(Self::BscMainnet),
            "XDaiMainnet" | "0x00f1918e" => Ok(Self::XDaiMainnet),
            "InterimChain" | "0xffffffff" => Ok(Self::InterimChain),
            "FantomMainnet" | "0x0022af98" => Ok(Self::FantomMainnet),
            "EthereumGoerli" | "0x00b4f6c5" => Ok(Self::EthereumGoerli),
            "PolygonMainnet" | "0x0075dd4c" => Ok(Self::PolygonMainnet),
            "EthereumMainnet" | "0x005fe7f9" => Ok(Self::EthereumMainnet),
            "EthereumRinkeby" | "0x00f34368" => Ok(Self::EthereumRinkeby),
            "EthereumSepolia" | "0x0030d6b5" => Ok(Self::EthereumSepolia),
            "EthereumRopsten" | "0x0069c322" => Ok(Self::EthereumRopsten),
            "ArbritrumMainnet" | "0x00ce98c4" => Ok(Self::ArbitrumMainnet),
            "LuxochainMainnet" | "0x00d5beb0" => Ok(Self::LuxochainMainnet),

            // Ltc...
            "LitecoinMainnet" | "0x01840435" => Ok(Self::LitecoinMainnet),

            _ => Err(format!("Unrecognised chain id: '{s}'").into()),
        }
    }
}

impl MetadataChainId {
    pub fn to_protocol_id(self) -> MetadataProtocolId {
        match self {
            Self::EosMainnet
            | Self::FioMainnet
            | Self::UltraMainnet
            | Self::UltraTestnet
            | Self::TelosMainnet
            | Self::LibreTestnet
            | Self::LibreMainnet
            | Self::EosJungleTestnet
            | Self::EosUnknown => MetadataProtocolId::Eos,
            Self::AlgorandMainnet => MetadataProtocolId::Algorand,
            Self::BitcoinMainnet | Self::BitcoinTestnet | Self::BtcUnknown | Self::LitecoinMainnet => {
                MetadataProtocolId::Bitcoin
            },
            Self::BscMainnet
            | Self::EthUnknown
            | Self::XDaiMainnet
            | Self::InterimChain
            | Self::FantomMainnet
            | Self::EthereumGoerli
            | Self::EthereumMainnet
            | Self::EthereumSepolia
            | Self::EthereumRinkeby
            | Self::EthereumRopsten
            | Self::ArbitrumMainnet
            | Self::LuxochainMainnet
            | Self::PolygonMainnet => MetadataProtocolId::Ethereum,
        }
    }

    fn to_chain_id(self) -> Box<dyn ChainId> {
        match self {
            Self::BtcUnknown => Box::new(BtcChainId::unknown()),
            Self::EosUnknown => Box::new(EosChainId::unknown()),
            Self::EthUnknown => Box::new(EthChainId::unknown()),
            Self::EthereumGoerli => Box::new(EthChainId::Goerli),
            Self::EosMainnet => Box::new(EosChainId::EosMainnet),
            Self::FioMainnet => Box::new(EosChainId::FioMainnet),
            Self::BscMainnet => Box::new(EthChainId::BscMainnet),
            Self::BitcoinMainnet => Box::new(BtcChainId::Bitcoin),
            Self::BitcoinTestnet => Box::new(BtcChainId::Testnet),
            Self::EthereumMainnet => Box::new(EthChainId::Mainnet),
            Self::EthereumRinkeby => Box::new(EthChainId::Rinkeby),
            Self::EthereumSepolia => Box::new(EthChainId::Sepolia),
            Self::EthereumRopsten => Box::new(EthChainId::Ropsten),
            Self::XDaiMainnet => Box::new(EthChainId::XDaiMainnet),
            Self::AlgorandMainnet => Box::new(AlgoChainId::Mainnet),
            Self::TelosMainnet => Box::new(EosChainId::TelosMainnet),
            Self::UltraMainnet => Box::new(EosChainId::UltraMainnet),
            Self::UltraTestnet => Box::new(EosChainId::UltraTestnet),
            Self::InterimChain => Box::new(EthChainId::InterimChain),
            Self::LibreTestnet => Box::new(EosChainId::LibreTestnet),
            Self::LibreMainnet => Box::new(EosChainId::LibreMainnet),
            Self::FantomMainnet => Box::new(EthChainId::FantomMainnet),
            Self::PolygonMainnet => Box::new(EthChainId::PolygonMainnet),
            Self::ArbitrumMainnet => Box::new(EthChainId::ArbitrumMainnet),
            Self::LuxochainMainnet => Box::new(EthChainId::LuxochainMainnet),
            Self::EosJungleTestnet => Box::new(EosChainId::EosJungleTestnet),
            // NOTE: This is how LTC is handled in the forked library it uses
            Self::LitecoinMainnet => Box::new(BtcChainId::Bitcoin),
        }
    }

    pub fn to_hex(self) -> Result<String> {
        Ok(format!("0x{}", hex::encode(self.to_bytes()?)))
    }

    fn to_keccak_hash(self) -> Result<KeccakHash> {
        match self {
            Self::EthUnknown | Self::EosUnknown | Self::BtcUnknown => {
                Ok(KeccakHash::from_slice(&THIRTY_TWO_ZERO_BYTES.to_vec()))
            },
            _ => self.to_chain_id().keccak_hash(),
        }
    }

    fn to_first_three_bytes_of_keccak_hash(self) -> Result<Bytes> {
        match self {
            Self::LitecoinMainnet => Ok(vec![]),
            _ => Ok(self.to_keccak_hash()?[..3].to_vec()),
        }
    }

    pub fn to_bytes(self) -> Result<Bytes> {
        match self {
            Self::InterimChain => Ok(vec![0xff, 0xff, 0xff, 0xff]),
            Self::LitecoinMainnet => {
                // NOTE: Litecoin is handled via a feature flag in the `/common/bitcoin` crate.
                // That crate uses a forked `bitcoin` lib to handle litecoin, however the underlying
                // bitcoin based chain ID does _not_ change, and thus the hash of the bytes of that
                // ID would match bitcoin too. So instead we just use three random bytes in the
                // defintion of a litecoin metadata chain ID.
                let random_bytes = vec![0x84, 0x04, 0x35];
                Ok([vec![self.to_protocol_id().to_byte()], random_bytes].concat())
            },
            _ => Ok([
                vec![self.to_protocol_id().to_byte()],
                self.to_first_three_bytes_of_keccak_hash()?,
            ]
            .concat()),
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        let number_of_bytes = bytes.len();
        if number_of_bytes != METADATA_CHAIN_ID_NUMBER_OF_BYTES {
            Err(format!(
                "Expected {} bytes for metadata chain ID, got {} instead!",
                METADATA_CHAIN_ID_NUMBER_OF_BYTES, number_of_bytes
            )
            .into())
        } else {
            let maybe_self = Self::get_all()
                .iter()
                .map(|id| match id.to_bytes() {
                    Err(_) => None,
                    Ok(id_bytes) => {
                        if id_bytes == bytes {
                            Some(*id)
                        } else {
                            None
                        }
                    },
                })
                .filter(Option::is_some)
                .collect::<Vec<Option<Self>>>();
            match maybe_self.len() {
                1 => maybe_self[0].ok_or_else(|| "Failed to unwrap `maybe_self` from option!".into()),
                0 => Err(format!("Unrecognized bytes for `MetadataChainId`: 0x{}", hex::encode(bytes)).into()),
                _ => Err("`MetadataChainId` collision! > 1 chain ID has the same 1st 3 bytes when hashed!".into()),
            }
        }
    }

    #[cfg(test)]
    fn print_all() {
        Self::get_all().iter().for_each(|id| println!("{}", id))
    }

    fn get_all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
}

impl fmt::Display for MetadataChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hex = self.to_hex().unwrap_or_else(|_| "Could not unwrap hex!".to_string());
        match self {
            Self::EthUnknown => write!(f, "EthUnknown: {}", hex),
            Self::EosUnknown => write!(f, "EosUnknown: {}", hex),
            Self::BtcUnknown => write!(f, "BtcUnknown: {}", hex),
            Self::EosMainnet => write!(f, "Eos Mainnet: {}", hex),
            Self::FioMainnet => write!(f, "FIO Mainnet: {}", hex),
            Self::XDaiMainnet => write!(f, "xDai Mainnet: {}", hex),
            Self::TelosMainnet => write!(f, "Telos Mainnet: {}", hex),
            Self::UltraTestnet => write!(f, "Ultra Testnet: {}", hex),
            Self::UltraMainnet => write!(f, "Ultra Mainnet: {}", hex),
            Self::InterimChain => write!(f, "Interim Chain: {}", hex),
            Self::LibreTestnet => write!(f, "Libre Testnet: {}", hex),
            Self::LibreMainnet => write!(f, "Libre Mainnet: {}", hex),
            Self::FantomMainnet => write!(f, "Fantom Mainnet: {}", hex),
            Self::EthereumGoerli => write!(f, "Goerli Testnet: {}", hex),
            Self::BitcoinMainnet => write!(f, "Bitcoin Mainnet: {}", hex),
            Self::PolygonMainnet => write!(f, "Polygon Mainnet: {}", hex),
            Self::BitcoinTestnet => write!(f, "Bitcoin Testnet: {}", hex),
            Self::AlgorandMainnet => write!(f, "AlgorandMainnet: {}", hex),
            Self::LitecoinMainnet => write!(f, "LitecoinMainnet: {}", hex),
            Self::EthereumSepolia => write!(f, "Sepolia Testnet: {}", hex),
            Self::ArbitrumMainnet => write!(f, "Arbitrum Mainnet: {}", hex),
            Self::EthereumMainnet => write!(f, "Ethereum Mainnet: {}", hex),
            Self::EthereumRinkeby => write!(f, "Ethereum Rinkeby: {}", hex),
            Self::EthereumRopsten => write!(f, "Ethereum Ropsten: {}", hex),
            Self::LuxochainMainnet => write!(f, "Luxochain Mainnet: {}", hex),
            Self::EosJungleTestnet => write!(f, "EOS Jungle Testnet: {}", hex),
            Self::BscMainnet => write!(f, "Binance Chain (BSC) Mainnet: {}", hex),
        }
    }
}

#[cfg(test)]
mod tests {
    use common::AppError;

    use super::*;

    #[test]
    fn should_print_all_ids() {
        MetadataChainId::print_all();
    }

    #[test]
    fn should_perform_metadata_chain_ids_bytes_round_trip() {
        MetadataChainId::get_all().iter().for_each(|id| {
            let byte = id.to_bytes().unwrap();
            let result = MetadataChainId::from_bytes(&byte).unwrap();
            assert_eq!(&result, id);
        });
    }

    #[test]
    fn all_chain_ids_should_be_unique() {
        let mut ids_as_bytes = MetadataChainId::get_all()
            .iter()
            .map(|id| id.to_bytes().unwrap())
            .collect::<Vec<Bytes>>();
        ids_as_bytes.sort();
        let length_before_dedup = ids_as_bytes.len();
        ids_as_bytes.dedup();
        let length_after_dedup = ids_as_bytes.len();
        assert_eq!(length_before_dedup, length_after_dedup);
    }

    #[test]
    fn should_get_metadata_chain_id_from_bytes_correctly() {
        #[rustfmt::skip]
        let chain_ids_bytes = vec![
            "005fe7f9", "0069c322", "00f34368", "01ec97de",
            "018afeb2", "02e7261c", "028c7109", "00e4b170",
            "0282317f", "00f1918e", "0075dd4c", "02f9337d",
            "02174f20", "02b5a4d6", "00000000", "01000000",
            "02000000", "ffffffff", "00ce98c4", "00d5beb0",
            "0022af98", "03c38e67", "02a75f2c", "026776fa",
            "00b4f6c5", "0030d6b5", "01840435",
        ]
        .iter()
        .map(|ref hex| hex::decode(hex).unwrap())
        .collect::<Vec<Bytes>>();
        let result = chain_ids_bytes
            .iter()
            .map(|bytes| MetadataChainId::from_bytes(bytes))
            .collect::<Result<Vec<MetadataChainId>>>()
            .unwrap();
        let expected_result = MetadataChainId::get_all();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_error_when_getting_metadata_chain_id_due_to_wrong_number_of_bytes() {
        let bytes = vec![];
        let number_of_bytes = bytes.len();
        assert_ne!(number_of_bytes, METADATA_CHAIN_ID_NUMBER_OF_BYTES);
        let expected_error = format!(
            "Expected {} bytes for metadata chain ID, got {} instead!",
            METADATA_CHAIN_ID_NUMBER_OF_BYTES, number_of_bytes
        );
        match MetadataChainId::from_bytes(&bytes) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        };
    }
}
