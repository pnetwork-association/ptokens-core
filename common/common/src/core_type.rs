#![allow(dead_code)]
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    errors::AppError,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

lazy_static! {
    pub static ref CORE_IS_INITIALIZED_MARKER: Bytes = vec![1u8];
    pub static ref HOST_CORE_IS_INITIALIZED_DB_KEY: [u8; 32] =
        crate::utils::get_prefixed_db_key("host_core_is_initialized_db_key");
    pub static ref NATIVE_CORE_IS_INITIALIZED_DB_KEY: [u8; 32] =
        crate::utils::get_prefixed_db_key("native_core_is_initialized_db_key");
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum CoreType {
    BtcOnEth,
    BtcOnInt,
    IntOnEos,
    IntOnEvm,
    EosOnInt,
    BtcOnEos,
    EosOnEth,
    IntOnAlgo,
    Erc20OnEos,
    Erc20OnInt,
    Erc20OnEvm,
    V3(V3CoreType),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum V3CoreType {
    EvmOnInt,
    IntOnEvm,
}

impl FromStr for V3CoreType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "int_on_evm" | "int-on-evm" | "intonevm" => Ok(Self::IntOnEvm),
            "evm_on_int" | "evm-on-int" | "evmonint" => Ok(Self::EvmOnInt),
            _ => Err(format!("Unrecognized v3 core type: {s}").into()),
        }
    }
}

impl V3CoreType {
    fn get_host_symbol(&self) -> String {
        self.to_string().split('_').collect::<Vec<_>>()[2].into()
    }

    fn get_native_symbol(&self) -> String {
        self.to_string().split('_').collect::<Vec<_>>()[0].into()
    }
}

impl fmt::Display for V3CoreType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::EvmOnInt => "EVM_ON_INT",
            Self::IntOnEvm => "Int_ON_EVM",
        };
        write!(f, "{}", s)
    }
}

impl Default for V3CoreType {
    fn default() -> Self {
        Self::EvmOnInt
    }
}

impl CoreType {
    pub fn to_v3_core_type(&self) -> Result<V3CoreType> {
        match self {
            Self::V3(c) => Ok(*c),
            _ => Err(AppError::Custom(format!("Cannot convert `{self}` to v3 core type!"))),
        }
    }

    pub fn initialize_native_core<D: DatabaseInterface>(db: &D) -> Result<()> {
        info!("✔ Initializing NATIVE core...");
        db.put(
            NATIVE_CORE_IS_INITIALIZED_DB_KEY.to_vec(),
            CORE_IS_INITIALIZED_MARKER.clone(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn initialize_host_core<D: DatabaseInterface>(db: &D) -> Result<()> {
        info!("✔ Initializing HOST core...");
        db.put(
            HOST_CORE_IS_INITIALIZED_DB_KEY.to_vec(),
            CORE_IS_INITIALIZED_MARKER.clone(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_host_symbol(&self) -> String {
        match self {
            Self::V3(core_type) => core_type.get_host_symbol(),
            _ => self.to_string().split('_').collect::<Vec<_>>()[2].into(),
        }
    }

    fn get_native_symbol(&self) -> String {
        match self {
            Self::V3(core_type) => core_type.get_native_symbol(),
            _ => self.to_string().split('_').collect::<Vec<_>>()[0].into(),
        }
    }

    pub fn as_db_key_prefix(&self) -> String {
        self.to_string().to_lowercase().replace('_', "-")
    }

    fn core_is_initialized<D: DatabaseInterface>(db: &D, is_native: bool) -> bool {
        db.get(
            if is_native {
                NATIVE_CORE_IS_INITIALIZED_DB_KEY.to_vec()
            } else {
                HOST_CORE_IS_INITIALIZED_DB_KEY.to_vec()
            },
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .is_ok()
    }

    pub fn native_core_is_initialized<D: DatabaseInterface>(db: &D) -> bool {
        Self::core_is_initialized(db, true)
    }

    pub fn host_core_is_initialized<D: DatabaseInterface>(db: &D) -> bool {
        Self::core_is_initialized(db, false)
    }

    pub fn check_is_initialized<D: DatabaseInterface>(db: &D) -> Result<()> {
        if !Self::native_core_is_initialized(db) {
            Err("NATIVE side of core is not initialized!".into())
        } else if !Self::host_core_is_initialized(db) {
            Err("HOST side of core is not initialized!".into())
        } else {
            info!("✔ Core is initialized!");
            Ok(())
        }
    }
}

impl Default for CoreType {
    fn default() -> Self {
        Self::BtcOnInt
    }
}

impl fmt::Display for CoreType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = match self {
            Self::BtcOnEth => "BTC_ON_ETH".into(),
            Self::BtcOnEos => "BTC_ON_EOS".into(),
            Self::EosOnEth => "EOS_ON_ETH".into(),
            Self::BtcOnInt => "BTC_ON_INT".into(),
            Self::IntOnEos => "INT_ON_EOS".into(),
            Self::EosOnInt => "EOS_ON_INT".into(),
            Self::IntOnEvm => "INT_ON_EVM".into(),
            Self::IntOnAlgo => "INT_ON_ALGO".into(),
            Self::Erc20OnEos => "ERC20_ON_EOS".into(),
            Self::Erc20OnEvm => "ERC20_ON_EVM".into(),
            Self::Erc20OnInt => "ERC20_ON_INT".into(),
            Self::V3(v3_core_type) => format!("V3_{}", v3_core_type),
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn should_get_core_type_as_db_key_prefix() {
        let expected_results = vec![
            "btc-on-eth",
            "btc-on-int",
            "int-on-eos",
            "int-on-evm",
            "eos-on-int",
            "btc-on-eos",
            "eos-on-eth",
            "int-on-algo",
            "erc20-on-eos",
            "erc20-on-int",
            "erc20-on-evm",
            "v3-evm-on-int",
            "v3-int-on-evm",
        ];
        CoreType::iter()
            .zip(expected_results.iter())
            .for_each(|(core_type, expected_result)| assert_eq!(&core_type.as_db_key_prefix(), *expected_result))
    }

    #[test]
    fn should_get_native_symbol_from_core_type() {
        let expected_results = vec![
            "BTC", "BTC", "INT", "INT", "EOS", "BTC", "EOS", "INT", "ERC20", "ERC20", "ERC20", "EVM", "INT",
        ];
        CoreType::iter()
            .zip(expected_results.iter())
            .for_each(|(core_type, expected_result)| assert_eq!(&core_type.get_native_symbol(), *expected_result))
    }

    #[test]
    fn should_get_host_symbol_from_core_type() {
        let expected_results = vec![
            "ETH", "INT", "EOS", "EVM", "INT", "EOS", "ETH", "ALGO", "EOS", "INT", "EVM", "INT", "EVM",
        ];
        CoreType::iter()
            .zip(expected_results.iter())
            .for_each(|(core_type, expected_result)| assert_eq!(&core_type.get_host_symbol(), *expected_result))
    }
}
