use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::UserOpError;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum UserOpVersion {
    V0 = 0,
}

impl UserOpVersion {
    pub(super) fn latest() -> Self {
        Self::V0
    }
}

impl Default for UserOpVersion {
    fn default() -> Self {
        Self::V0
    }
}

impl fmt::Display for UserOpVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::V0 => "0",
        };
        write!(f, "{s}")
    }
}

impl FromStr for UserOpVersion {
    type Err = UserOpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "0" | "v0" => Ok(Self::V0),
            other => Err(UserOpError::UnrecognizedVersion(other.into())),
        }
    }
}
