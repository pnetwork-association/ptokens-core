use std::{fmt, str::FromStr};

use common::{
    errors::AppError,
    types::{Byte, Bytes, Result},
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Clone, Debug, EnumIter, Eq, PartialEq, Serialize, Deserialize)]
pub enum EthReceiptType {
    Legacy,
    EIP2718,
    EIP2930,
}

impl EthReceiptType {
    pub fn from_byte(byte: &Byte) -> Self {
        match byte {
            0x00 => Self::Legacy,
            0x01 => Self::EIP2930,
            0x02 => Self::EIP2718,
            _ => Self::Legacy,
        }
    }

    pub fn to_byte(&self) -> Byte {
        match self {
            Self::Legacy => 0x00,
            Self::EIP2930 => 0x01,
            Self::EIP2718 => 0x02,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        vec![self.to_byte()]
    }
}

impl fmt::Display for EthReceiptType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Legacy => "0x0",
            Self::EIP2930 => "0x1",
            Self::EIP2718 => "0x2",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for EthReceiptType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "0x0" | "0" => Ok(Self::Legacy),
            "EIP2930" | "0x1" | "1" => Ok(Self::EIP2930),
            "EIP2718" | "0x2" | "2" => Ok(Self::EIP2718),
            _ => Err(format!("Unrecognized ETH receipt type: {s}").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use common::types::Bytes;
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn should_make_receipt_types_byte_roundtrip() {
        let expected_results = EthReceiptType::iter().collect::<Vec<EthReceiptType>>();
        let bytes = EthReceiptType::iter()
            .map(|receipt_type| receipt_type.to_byte())
            .collect::<Bytes>();
        let results = bytes
            .iter()
            .map(EthReceiptType::from_byte)
            .collect::<Vec<EthReceiptType>>();
        assert_eq!(results, expected_results);
    }
}
