use std::{fmt, str::FromStr};

use common::{
    constants::THIRTY_TWO_ZERO_BYTES,
    crypto_utils::keccak_hash_bytes,
    errors::AppError,
    traits::ChainId,
    types::{Byte, Bytes, Result},
    utils::decode_hex_with_err_msg,
};
use ethereum_types::H256 as KeccakHash;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const EOS_CHAIN_ID_LENGTH_IN_BYTES: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum EosChainId {
    EosMainnet,
    TelosMainnet,
    EosJungleTestnet,
    UltraMainnet,
    UltraTestnet,
    FioMainnet,
    LibreTestnet,
    LibreMainnet,
    Unknown(Bytes),
}

impl Default for EosChainId {
    fn default() -> Self {
        Self::EosMainnet
    }
}

impl ChainId for EosChainId {
    fn keccak_hash(&self) -> Result<KeccakHash> {
        Ok(keccak_hash_bytes(&self.to_bytes()))
    }
}

lazy_static! {
    pub static ref EOS_MAINNET_BYTES: Bytes =
        hex::decode("aca376f206b8fc25a6ed44dbdc66547c36c6c33e3a119ffbeaef943642f0e906")
            .expect("✘ Invalid hex in `EOS_MAINNET_BYTES`");
    pub static ref PHOENIX_TESTNET_BYTES: Bytes =
        hex::decode("793cf401f9405530986c5fa206598ee3f4d38df7f12dcd1fccb986a5e3bdb05a")
            .expect("✘ Invalid hex in `PHOENIX_TESTNET_BYTES`");
    pub static ref PHOENIX_MAINNET_BYTES: Bytes =
        hex::decode("38b1d7815474d0c60683ecbea321d723e83f5da6ae5f1c1f9fecc69d9ba96465")
            .expect("✘ Invalid hex in `PHOENIX_MAINNET_BYTES`");
    pub static ref TELOS_MAINNET_BYTES: Bytes =
        hex::decode("4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11")
            .expect("✘ Invalid hex in `TELOS_MAINNET_BYTES`");
    pub static ref EOS_JUNGLE_TESTNET_BYTES: Bytes =
        hex::decode("e70aaab8997e1dfce58fbfac80cbbb8fecec7b99cf982a9444273cbc64c41473")
            .expect("✘ Invalid hex in `EOS_JUNGLE_TESTNET_BYTES`");
    pub static ref ULTRA_MAINNET_BYTES: Bytes =
        hex::decode("a9c481dfbc7d9506dc7e87e9a137c931b0a9303f64fd7a1d08b8230133920097")
            .expect("✘ Invalid hex in `ULTRA_MAINNET_BYTES`");
    pub static ref ULTRA_TESTNET_BYTES: Bytes =
        hex::decode("7fc56be645bb76ab9d747b53089f132dcb7681db06f0852cfa03eaf6f7ac80e9")
            .expect("✘ Invalid hex in `ULTRA_TESTNET_BYTES`");
    pub static ref FIO_MAINNET_BYTES: Bytes =
        hex::decode("21dcae42c0182200e93f954a074011f9048a7624c6fe81d3c9541a614a88bd1c")
            .expect("✘ Invalid hex in `FIO_MAINNET_BYTES`");
}

impl FromStr for EosChainId {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        decode_hex_with_err_msg(s, &format!("`EosChainId` error! Invalid hex: 0x{}", s))
            .and_then(|ref bytes| Self::from_bytes(bytes))
    }
}

impl EosChainId {
    pub fn unknown() -> Self {
        Self::Unknown(THIRTY_TWO_ZERO_BYTES.to_vec())
    }

    pub fn to_hex(&self) -> String {
        match self {
            Self::EosMainnet => hex::encode(&*EOS_MAINNET_BYTES),
            Self::TelosMainnet => hex::encode(&*TELOS_MAINNET_BYTES),
            Self::EosJungleTestnet => hex::encode(&*EOS_JUNGLE_TESTNET_BYTES),
            Self::UltraMainnet => hex::encode(&*ULTRA_MAINNET_BYTES),
            Self::UltraTestnet => hex::encode(&*ULTRA_TESTNET_BYTES),
            Self::FioMainnet => hex::encode(&*FIO_MAINNET_BYTES),
            Self::LibreTestnet => hex::encode(&*PHOENIX_TESTNET_BYTES),
            Self::LibreMainnet => hex::encode(&*PHOENIX_MAINNET_BYTES),
            Self::Unknown(ref bytes) => hex::encode(bytes),
        }
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        info!("✔ Getting `EosChainId` from bytes: 0x{}", hex::encode(bytes));
        let maybe_self = Self::get_all()
            .iter()
            .map(|eos_chain_id| {
                let eos_chain_id_bytes = eos_chain_id.to_bytes();
                if eos_chain_id_bytes == bytes {
                    Some(eos_chain_id.clone())
                } else {
                    None
                }
            })
            .filter(Option::is_some)
            .collect::<Vec<Option<Self>>>();
        match maybe_self.len() {
            1 => maybe_self[0]
                .clone()
                .ok_or_else(|| "Failed to unwrap `maybe_self` from option!".into()),
            _ => {
                let num_bytes = bytes.len();
                match num_bytes {
                    EOS_CHAIN_ID_LENGTH_IN_BYTES => {
                        info!("✔ Using unknown EOS chain ID: 0x{}", hex::encode(bytes));
                        Ok(Self::Unknown(bytes.to_vec()))
                    },
                    _ => Err(format!(
                        "Incorrect number of bytes for `EosChainId`. Got {}, expected {}!",
                        num_bytes, EOS_CHAIN_ID_LENGTH_IN_BYTES
                    )
                    .into()),
                }
            },
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        match self {
            Self::EosMainnet => EOS_MAINNET_BYTES.to_vec(),
            Self::TelosMainnet => TELOS_MAINNET_BYTES.to_vec(),
            Self::EosJungleTestnet => EOS_JUNGLE_TESTNET_BYTES.to_vec(),
            Self::UltraMainnet => ULTRA_MAINNET_BYTES.to_vec(),
            Self::UltraTestnet => ULTRA_TESTNET_BYTES.to_vec(),
            Self::LibreTestnet => PHOENIX_TESTNET_BYTES.to_vec(),
            Self::LibreMainnet => PHOENIX_MAINNET_BYTES.to_vec(),
            Self::FioMainnet => FIO_MAINNET_BYTES.to_vec(),
            Self::Unknown(ref bytes) => bytes.to_vec(),
        }
    }

    fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }
}

impl fmt::Display for EosChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EosMainnet => write!(f, "EosMainnet"),
            Self::FioMainnet => write!(f, "FioMainnet"),
            Self::Unknown(_) => write!(f, "EosUnknown"),
            Self::TelosMainnet => write!(f, "TelosMainnet"),
            Self::UltraMainnet => write!(f, "UltraMainnet"),
            Self::UltraTestnet => write!(f, "UltraTestnet"),
            Self::LibreTestnet => write!(f, "LibreTestnet"),
            Self::LibreMainnet => write!(f, "LibreMainnet"),
            Self::EosJungleTestnet => write!(f, "EosJungleTestnet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use common::errors::AppError;

    use super::*;

    #[test]
    fn should_make_bytes_roundtrip_for_all_eos_chain_ids() {
        let ids = EosChainId::get_all();
        let vec_of_bytes = ids.iter().map(|id| id.to_bytes()).collect::<Vec<Bytes>>();
        let result = vec_of_bytes
            .iter()
            .map(|bytes| EosChainId::from_bytes(bytes))
            .collect::<Result<Vec<EosChainId>>>()
            .unwrap();
        assert_eq!(result, ids);
    }

    #[test]
    fn should_create_unknown_chain_id_if_bytes_unrecognised() {
        let unknown_chain_id_hex = "7013417c68fcf077c1ef0b8b800d1f91d1bbdb6f5e08e5e5f3d9020dc37cd2d5";
        let unknown_chain_id_bytes = hex::decode(unknown_chain_id_hex).unwrap();
        let result = EosChainId::from_bytes(&unknown_chain_id_bytes).unwrap();
        let expected_result = EosChainId::Unknown(unknown_chain_id_bytes);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn unknown_eos_chain_id_must_be_exactly_32_bytes() {
        let unknown_chain_id_bytes_too_short =
            hex::decode("7013417c68fcf077c1ef0b8b800d1f91d1bbdb6f5e08e5e5f3d9020dc37c").unwrap();
        let unknown_chain_id_bytes_too_long =
            hex::decode("7013417c68fcf077c1ef0b8b800d1f91d1bbdb6f5e08e5e5f3d9020dc37cd2d50000").unwrap();
        assert!(unknown_chain_id_bytes_too_short.len() < EOS_CHAIN_ID_LENGTH_IN_BYTES);
        assert!(unknown_chain_id_bytes_too_long.len() > EOS_CHAIN_ID_LENGTH_IN_BYTES);
        let expected_err_1 = format!(
            "Incorrect number of bytes for `EosChainId`. Got {}, expected {}!",
            unknown_chain_id_bytes_too_short.len(),
            EOS_CHAIN_ID_LENGTH_IN_BYTES
        );
        let expected_err_2 = format!(
            "Incorrect number of bytes for `EosChainId`. Got {}, expected {}!",
            unknown_chain_id_bytes_too_long.len(),
            EOS_CHAIN_ID_LENGTH_IN_BYTES
        );
        match EosChainId::from_bytes(&unknown_chain_id_bytes_too_short) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err_1),
            Err(_) => panic!("Wrong err received!"),
        };
        match EosChainId::from_bytes(&unknown_chain_id_bytes_too_long) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err_2),
            Err(_) => panic!("Wrong err received!"),
        };
    }
}
