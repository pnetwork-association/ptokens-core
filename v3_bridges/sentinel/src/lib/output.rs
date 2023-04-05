use std::fmt;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::{HostOutput, NativeOutput, UserOps};

#[derive(Clone, Debug, Default, Constructor, Serialize, Deserialize)]
pub struct Output {
    host_timestamp: u64,
    native_timestamp: u64,
    host_latest_block_num: u64,
    native_latest_block_num: u64,
    host_unmatched_user_ops: UserOps,
    native_unmatched_user_ops: UserOps,
}

impl Output {
    pub fn host_unmatched_user_ops(&self) -> UserOps {
        self.host_unmatched_user_ops.clone()
    }

    pub fn native_unmatched_user_ops(&self) -> UserOps {
        self.native_unmatched_user_ops.clone()
    }
}

impl From<(&NativeOutput, &HostOutput)> for Output {
    fn from((n, h): (&NativeOutput, &HostOutput)) -> Self {
        Self {
            host_timestamp: h.get_timestamp(),
            native_timestamp: n.get_timestamp(),
            host_latest_block_num: h.get_latest_block_num(),
            native_latest_block_num: n.get_latest_block_num(),
            host_unmatched_user_ops: h.get_host_unmatched_user_ops(),
            native_unmatched_user_ops: n.get_native_unmatched_user_ops(),
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
