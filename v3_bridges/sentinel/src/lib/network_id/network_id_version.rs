use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::NetworkIdError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkIdVersion {
    V1,
}

impl Default for NetworkIdVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl FromStr for NetworkIdVersion {
    type Err = NetworkIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "1" | "one" | "v1" | "01" | "0x01" => Ok(Self::V1),
            _ => Err(NetworkIdError::InvalidNetworkId(s.into())),
        }
    }
}

impl From<NetworkIdVersion> for u8 {
    fn from(v: NetworkIdVersion) -> u8 {
        match v {
            NetworkIdVersion::V1 => 1,
        }
    }
}

impl TryFrom<u8> for NetworkIdVersion {
    type Error = NetworkIdError;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        Self::from_str(&hex::encode([x]))
    }
}

impl fmt::Display for NetworkIdVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::V1 => "v1",
        };
        write!(f, "{s}")
    }
}
