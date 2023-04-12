use std::fmt;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::{HostOutput, NativeOutput, UserOps};

#[derive(Clone, Debug, Default, Constructor, Serialize, Deserialize)]
pub struct Output {
    user_ops: UserOps,
    host_timestamp: u64,
    native_timestamp: u64,
    host_latest_block_num: u64,
    native_latest_block_num: u64,
}

impl Output {
    pub fn user_ops(&self) -> UserOps {
        self.user_ops.clone()
    }
}

impl From<(&NativeOutput, &HostOutput)> for Output {
    fn from((n, h): (&NativeOutput, &HostOutput)) -> Self {
        let host_timestamp = h.timestamp();
        let native_timestamp = n.timestamp();
        let user_ops = if native_timestamp > host_timestamp {
            n.user_ops()
        } else {
            h.user_ops()
        };
        Self {
            user_ops,
            host_timestamp,
            native_timestamp,
            host_latest_block_num: h.latest_block_num(),
            native_latest_block_num: n.latest_block_num(),
        }
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error pretty printing `Output` json: {e}"),
        }
    }
}
