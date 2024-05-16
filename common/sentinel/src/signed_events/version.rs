use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignedEventVersion {
    V1,
}

impl Default for SignedEventVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl SignedEventVersion {
    pub fn current() -> Self {
        Self::V1
    }
}
