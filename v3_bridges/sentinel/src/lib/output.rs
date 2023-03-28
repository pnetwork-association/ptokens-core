use std::fmt;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{HostOutput, NativeOutput};

#[derive(Clone, Debug, Default, Constructor, Serialize, Deserialize)]
pub struct Output {
    native: NativeOutput,
    host: HostOutput,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = json!({
            "native": self.native.to_string(),
            "host": self.host.to_string(),
        })
        .to_string();

        write!(f, "{s}")
    }
}
