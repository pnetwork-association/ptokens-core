use std::fmt;

use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Deref, Constructor, Serialize, Deserialize)]
pub struct DebugSignature(Option<String>);

impl fmt::Display for DebugSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = if let Some(ref s) = self.0 {
            s.to_string()
        } else {
            String::default()
        };
        write!(f, "{s}")
    }
}

impl From<Option<&String>> for DebugSignature {
    fn from(v: Option<&String>) -> Self {
        Self::new(v.cloned())
    }
}

impl From<&str> for DebugSignature {
    fn from(s: &str) -> Self {
        let o = match s {
            "" => None,
            other => Some(other.to_string()),
        };
        Self::new(o)
    }
}
