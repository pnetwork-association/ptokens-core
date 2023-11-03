use std::fmt;

use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

// NOTE: Debug signature handling for function calls that require them tend to be arranged thusly:
// First the call is made without a signature. This fails, but returns the information required to
// generate the correct signature (IE, the hash of the function inputs plus the nonce for the
// signing address etc). A second call to the fxn then appends the signature, which if valid,
// allows the function to run as expected. And thus an `Option` fits the above paradigm nicely. So
// this type encloses over that, giving a couple of helper methods for converting to and from it
// etc.

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
