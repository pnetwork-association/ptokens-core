use std::{convert::From, fmt, str::FromStr};

use common::AppError;

#[derive(Clone, Debug)]
pub enum DefaultBlockParameter {
    Latest,
    Pending,
    Earliest,
    BlockNum(u64),
}

impl fmt::Display for DefaultBlockParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Pending => write!(f, "pending"),
            Self::Earliest => write!(f, "earliest"),
            Self::BlockNum(n) => write!(f, "0x{n:x}"),
        }
    }
}

impl FromStr for DefaultBlockParameter {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "latest" => Ok(Self::Latest),
            "pending" => Ok(Self::Pending),
            "earliest" => Ok(Self::Earliest),
            _ => Err(AppError::Custom(format!(
                "cannot convert `{s}` into `DefaultBlockParameter`"
            ))),
        }
    }
}

impl From<u64> for DefaultBlockParameter {
    fn from(n: u64) -> Self {
        Self::BlockNum(n)
    }
}

impl From<u32> for DefaultBlockParameter {
    fn from(n: u32) -> Self {
        Self::BlockNum(n as u64)
    }
}
