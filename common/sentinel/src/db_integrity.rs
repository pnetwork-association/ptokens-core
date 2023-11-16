use std::{fmt, str::FromStr};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbIntegrityError {
    #[error("cannot parse `DbIntegrity` from str '{0}'")]
    FromStr(String),
}

pub enum DbIntegrity {
    Valid        = 0,
    Invalid      = 1,
    NoHash       = 2,
    NoState      = 3,
    FirstRun     = 4,
    Unverifiable = 5,
    HashWritten  = 6,
    NoSignature  = 7,
}

impl Default for DbIntegrity {
    fn default() -> Self {
        Self::Invalid
    }
}

impl fmt::Display for DbIntegrity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Valid => "Valid",
            Self::NoHash => "NoHash",
            Self::NoState => "NoState",
            Self::Invalid => "invalid",
            Self::FirstRun => "FirstRun",
            Self::NoSignature => "NoState",
            Self::HashWritten => "HashWritten",
            Self::Unverifiable => "Unverifiable",
        };
        write!(f, "{s}")
    }
}

impl FromStr for DbIntegrity {
    type Err = DbIntegrityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "valid" => Ok(Self::Valid),
            "invalid" => Ok(Self::Invalid),
            "nohash" | "no_hash" => Ok(Self::NoHash),
            "unverifiable" => Ok(Self::Unverifiable),
            "nostate" | "no_state" => Ok(Self::NoState),
            "firstrun" | "first_run" => Ok(Self::FirstRun),
            "hashwritten" | "hash_written" => Ok(Self::HashWritten),
            "nosignature" | "no_signature" => Ok(Self::NoSignature),
            other => Err(DbIntegrityError::FromStr(other.into())),
        }
    }
}
