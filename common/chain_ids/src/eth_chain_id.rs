use std::{fmt, str::FromStr};

use common::{
    crypto_utils::keccak_hash_bytes,
    errors::AppError,
    traits::ChainId,
    types::{Byte, Bytes, Result},
    utils::{convert_bytes_to_u64, convert_bytes_to_u8},
};
use ethereum_types::{H256 as KeccakHash, U256};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use super::ChainIdT;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum EthChainId {
    Goerli,
    Sepolia,
    Mainnet,
    Rinkeby,
    Ropsten,
    BscMainnet,
    XDaiMainnet,
    InterimChain,
    FantomMainnet,
    PolygonMainnet,
    ArbitrumMainnet,
    LuxochainMainnet,
    Unknown(u64),
}

impl ChainIdT for EthChainId {}

impl From<EthChainId> for U256 {
    fn from(x: EthChainId) -> Self {
        U256::from(x.to_u64())
    }
}

impl Default for EthChainId {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl ChainId for EthChainId {
    fn keccak_hash(&self) -> Result<KeccakHash> {
        Ok(keccak_hash_bytes(&self.to_bytes()?))
    }
}

impl FromStr for EthChainId {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        match &*s.to_lowercase() {
            "bscmainnet" | "bsc" | "56" => Ok(Self::BscMainnet),
            "ethereumgoerli" | "goerli" | "5" => Ok(Self::Goerli),
            "xdaimainnet" | "xdai" | "100" => Ok(Self::XDaiMainnet),
            "ethereumropsten" | "ropsten" | "3" => Ok(Self::Ropsten),
            "ethereumrinkeby" | "rinkeby" | "4" => Ok(Self::Rinkeby),
            "interimchain" | "interim" | "947" => Ok(Self::InterimChain),
            "fantommainnet" | "fantom" | "250" => Ok(Self::FantomMainnet),
            "ethereumsepolia" | "sepolia" | "11155111" => Ok(Self::Sepolia),
            "polygonmainnet" | "polygon" | "137" => Ok(Self::PolygonMainnet),
            "arbitrummainnet" | "arbitrum" | "42161" => Ok(Self::ArbitrumMainnet),
            "ethmainnet" | "ethereummainnet" | "mainnet" | "1" => Ok(Self::Mainnet),
            "luxomainnet" | "luxo" | "luxochain" | "110" => Ok(Self::LuxochainMainnet),
            _ => match s.parse::<u64>() {
                Ok(u_64) => Ok(Self::Unknown(u_64)),
                Err(_) => Err(format!("✘ Unrecognized ETH network: '{}'!", s).into()),
            },
        }
    }
}

const ARBITRUM_GAS_MULTIPLIER: usize = 10;
const ERC777_CHANGE_PNETWORK_GAS_LIMIT: usize = 30_000;
const ERC20_VAULT_MIGRATE_GAS_LIMIT: usize = 2_000_000;
const ERC777_MINT_WITH_DATA_GAS_LIMIT: usize = 450_000;
const ERC777_MINT_WITH_NO_DATA_GAS_LIMIT: usize = 180_000;
const ERC20_VAULT_PEGOUT_WITH_USER_DATA_GAS_LIMIT: usize = 450_000;
const ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT: usize = 100_000;
const ERC20_VAULT_PEGOUT_WITHOUT_USER_DATA_GAS_LIMIT: usize = 250_000;

impl EthChainId {
    pub fn get_erc777_change_pnetwork_gas_limit(&self) -> usize {
        match self {
            Self::ArbitrumMainnet => ARBITRUM_GAS_MULTIPLIER * ERC777_CHANGE_PNETWORK_GAS_LIMIT,
            _ => ERC777_CHANGE_PNETWORK_GAS_LIMIT,
        }
    }

    pub fn get_erc777_mint_with_data_gas_limit(&self) -> usize {
        match self {
            Self::ArbitrumMainnet => ARBITRUM_GAS_MULTIPLIER * ERC777_MINT_WITH_DATA_GAS_LIMIT,
            _ => ERC777_MINT_WITH_DATA_GAS_LIMIT,
        }
    }

    pub fn get_erc777_mint_with_no_data_gas_limit(&self) -> usize {
        match self {
            Self::ArbitrumMainnet => ARBITRUM_GAS_MULTIPLIER * ERC777_MINT_WITH_NO_DATA_GAS_LIMIT,
            _ => ERC777_MINT_WITH_NO_DATA_GAS_LIMIT,
        }
    }

    pub fn get_erc20_vault_migrate_gas_limit(&self) -> usize {
        match self {
            Self::ArbitrumMainnet => ARBITRUM_GAS_MULTIPLIER * ERC20_VAULT_MIGRATE_GAS_LIMIT,
            _ => ERC20_VAULT_MIGRATE_GAS_LIMIT,
        }
    }

    pub fn get_erc20_vault_pegout_without_user_data_gas_limit(&self) -> usize {
        match self {
            Self::ArbitrumMainnet => ARBITRUM_GAS_MULTIPLIER * ERC20_VAULT_PEGOUT_WITHOUT_USER_DATA_GAS_LIMIT,
            _ => ERC20_VAULT_PEGOUT_WITHOUT_USER_DATA_GAS_LIMIT,
        }
    }

    pub fn get_erc20_vault_pegout_with_user_data_gas_limit(&self) -> usize {
        match self {
            Self::ArbitrumMainnet => ARBITRUM_GAS_MULTIPLIER * ERC20_VAULT_PEGOUT_WITH_USER_DATA_GAS_LIMIT,
            _ => ERC20_VAULT_PEGOUT_WITH_USER_DATA_GAS_LIMIT,
        }
    }

    pub fn get_erc20_vault_change_supported_token_gas_limit(&self) -> usize {
        match self {
            Self::ArbitrumMainnet => ARBITRUM_GAS_MULTIPLIER * ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
            _ => ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
        }
    }

    pub fn unknown() -> Self {
        Self::Unknown(0)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        match self {
            // NOTE: The following explicit ones are for legacy reasons...
            Self::Mainnet => Ok(vec![0x01]),
            Self::Rinkeby => Ok(vec![0x04]),
            Self::Ropsten => Ok(vec![0x03]),
            Self::BscMainnet => Ok(vec![0x38]),
            Self::XDaiMainnet => Ok(vec![0x64]),
            Self::PolygonMainnet => Ok(vec![0x89]),
            // NOTE: ...and the rest are encoded thusly.
            _ => Ok(self.to_u64().to_le_bytes().to_vec()),
        }
    }

    fn from_unsigned_int<T: Into<u64>>(i: T) -> Result<Self> {
        let needle: u64 = i.into();
        match needle {
            5 => Ok(Self::Goerli),
            1 => Ok(Self::Mainnet),
            3 => Ok(Self::Ropsten),
            4 => Ok(Self::Rinkeby),
            56 => Ok(Self::BscMainnet),
            100 => Ok(Self::XDaiMainnet),
            11155111 => Ok(Self::Sepolia),
            947 => Ok(Self::InterimChain),
            250 => Ok(Self::FantomMainnet),
            137 => Ok(Self::PolygonMainnet),
            110 => Ok(Self::LuxochainMainnet),
            42161 => Ok(Self::ArbitrumMainnet),
            _ => {
                info!("✔ Using unknown ETH chain ID: {}", needle);
                Ok(Self::Unknown(needle))
            },
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        let hex = hex::encode(bytes);
        if bytes.len() == 1 {
            info!("✔ Getting `EthChainId` from legacy byte: 0x{}", hex);
            convert_bytes_to_u8(bytes).and_then(Self::from_unsigned_int)
        } else {
            info!("✔ Getting `EthChainId` from bytes: 0x{}", hex);
            convert_bytes_to_u64(bytes).and_then(Self::from_unsigned_int)
        }
    }

    pub fn to_u64(&self) -> u64 {
        match self {
            Self::Goerli => 5,
            Self::Mainnet => 1,
            Self::Ropsten => 3,
            Self::Rinkeby => 4,
            Self::BscMainnet => 56,
            Self::XDaiMainnet => 100,
            Self::InterimChain => 947,
            Self::Sepolia => 11155111,
            Self::FantomMainnet => 250,
            Self::PolygonMainnet => 137,
            Self::Unknown(u_64) => *u_64,
            Self::LuxochainMainnet => 110,
            Self::ArbitrumMainnet => 42161,
        }
    }

    pub fn get_all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().filter(|chain_id| !chain_id.is_unknown()).collect()
    }

    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

#[cfg(test)]
impl EthChainId {
    fn to_hex(&self) -> Result<String> {
        self.to_bytes().map(|ref bytes| hex::encode(bytes))
    }

    fn to_keccak_hash_hex(&self) -> Result<String> {
        self.keccak_hash().map(|ref bytes| hex::encode(bytes))
    }
}

impl fmt::Display for EthChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unknown(_) => write!(f, "EthUnknown"),
            Self::Goerli => write!(f, "EthereumGoerli"),
            Self::BscMainnet => write!(f, "BscMainnet"),
            Self::Sepolia => write!(f, "SepoliaTestnet"),
            Self::Rinkeby => write!(f, "EthereumRinkeby"),
            Self::Ropsten => write!(f, "EthereumRopsten"),
            Self::XDaiMainnet => write!(f, "XDaiMainnet"),
            Self::Mainnet => write!(f, "EthereumMainnet"),
            Self::InterimChain => write!(f, "InterimChain"),
            Self::FantomMainnet => write!(f, "FantomMainnet"),
            Self::PolygonMainnet => write!(f, "PolygonMainnet"),
            Self::ArbitrumMainnet => write!(f, "ArbritrumMainnet"),
            Self::LuxochainMainnet => write!(f, "LuxochainMainnet"),
        }
    }
}

impl TryFrom<u64> for EthChainId {
    type Error = AppError;

    fn try_from(u_64: u64) -> Result<Self> {
        Self::from_bytes(&u_64.to_le_bytes())
    }
}

impl TryFrom<u8> for EthChainId {
    type Error = AppError;

    fn try_from(u_8: u8) -> Result<Self> {
        Self::try_from(u_8 as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_u64_roundtrip_for_all_eth_chain_ids() {
        let ids = EthChainId::get_all();
        let bytes = ids.iter().map(|id| id.to_u64()).collect::<Vec<u64>>();
        let result = bytes
            .iter()
            .map(|byte| EthChainId::try_from(*byte))
            .collect::<Result<Vec<EthChainId>>>()
            .unwrap();
        assert_eq!(result, ids);
    }

    #[test]
    fn should_make_bytes_roundtrip_for_all_eth_chain_ids() {
        let ids = EthChainId::get_all();
        let vec_of_bytes = ids
            .iter()
            .map(|id| id.to_bytes())
            .collect::<Result<Vec<Bytes>>>()
            .unwrap();
        let result = vec_of_bytes
            .iter()
            .map(|bytes| EthChainId::from_bytes(bytes))
            .collect::<Result<Vec<EthChainId>>>()
            .unwrap();
        assert_eq!(result, ids);
    }

    fn get_all_legacy() -> Vec<EthChainId> {
        vec![
            EthChainId::Mainnet,
            EthChainId::Rinkeby,
            EthChainId::Ropsten,
            EthChainId::BscMainnet,
            EthChainId::XDaiMainnet,
            EthChainId::PolygonMainnet,
        ]
    }

    fn get_legacy_chain_ids_hex<'a>() -> Vec<&'a str> {
        vec!["01", "04", "03", "38", "64", "89"]
    }

    fn get_legacy_chain_ids_keccak_hashes<'a>() -> Vec<&'a str> {
        vec![
            "5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2",
            "f343681465b9efe82c933c3e8748c70cb8aa06539c361de20f72eac04e766393",
            "69c322e3248a5dfc29d73c5b0553b0185a35cd5bb6386747517ef7e53b15e287",
            "e4b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c10",
            "f1918e8562236eb17adc8502332f4c9c82bc14e19bfc0aa10ab674ff75b3d2f3",
            "75dd4ce35898634c43d8e291c5edc041d288f0c0a531e92d5528804add589d1f",
        ]
    }

    #[test]
    fn should_get_all_chain_id_legacy_bytes() {
        let legacy_chain_ids = get_all_legacy();
        let chain_ids_hex = legacy_chain_ids
            .iter()
            .map(|id| id.to_hex())
            .collect::<Result<Vec<String>>>()
            .unwrap();
        let expected_chain_ids_hex = get_legacy_chain_ids_hex();
        chain_ids_hex
            .iter()
            .enumerate()
            .for_each(|(i, chain_id_hex)| assert_eq!(chain_id_hex, expected_chain_ids_hex[i]));
    }

    #[test]
    fn shuld_get_all_chain_id_legacy_keccak_hashes() {
        let legacy_chain_ids = get_all_legacy();
        let chain_ids_keccak_hashes = legacy_chain_ids
            .iter()
            .map(|id| id.to_keccak_hash_hex())
            .collect::<Result<Vec<String>>>()
            .unwrap();
        let expected_chain_ids_keccak_hashes = get_legacy_chain_ids_keccak_hashes();
        chain_ids_keccak_hashes
            .iter()
            .enumerate()
            .for_each(|(i, chain_id_hex)| assert_eq!(chain_id_hex, expected_chain_ids_keccak_hashes[i]));
    }
}
