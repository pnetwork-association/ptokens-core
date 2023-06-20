use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
#[cfg(test)]
use strum::IntoEnumIterator;

use super::NetworkIdError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum ProtocolId {
    Ethereum = 0,
    Bitcoin  = 1,
    Eos      = 2,
    Algorand = 3,
}

#[cfg(test)]
impl ProtocolId {
    fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }
}

impl Default for ProtocolId {
    fn default() -> Self {
        Self::Ethereum
    }
}

impl FromStr for ProtocolId {
    type Err = NetworkIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "0" | "ethereum" | "eth" | "0x00" | "0x0" | "00" => Ok(Self::Ethereum),
            "1" | "bitcoin" | "btc" | "0x01" | "0x1" | "01" => Ok(Self::Bitcoin),
            "2" | "eos" | "0x02" | "0x2" | "02" => Ok(Self::Eos),
            "3" | "algorand" | "algo" | "0x03" | "0x3" | "03" => Ok(Self::Algorand),
            _ => Err(NetworkIdError::InvalidProtocolId(s.into())),
        }
    }
}

impl From<ProtocolId> for u8 {
    fn from(id: ProtocolId) -> u8 {
        match id {
            ProtocolId::Ethereum => 0,
            ProtocolId::Bitcoin => 1,
            ProtocolId::Eos => 2,
            ProtocolId::Algorand => 3,
        }
    }
}

impl TryFrom<&u8> for ProtocolId {
    type Error = NetworkIdError;

    fn try_from(x: &u8) -> Result<Self, Self::Error> {
        Self::from_str(&hex::encode([*x]))
    }
}

impl fmt::Display for ProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Eos => "EOS",
            Self::Bitcoin => "BTC",
            Self::Ethereum => "ETH",
            Self::Algorand => "ALGO",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_perform_protocol_ids_bytes_round_trip() {
        ProtocolId::get_all().into_iter().for_each(|id| {
            let byte: u8 = id.into();
            let result = ProtocolId::try_from(&byte).unwrap();
            assert_eq!(result, id);
        });
    }
}
