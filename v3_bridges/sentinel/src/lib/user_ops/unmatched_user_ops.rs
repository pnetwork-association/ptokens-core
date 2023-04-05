use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use super::UserOps;

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor, Serialize, Deserialize)]
pub struct UnmatchedUserOps {
    native: UserOps,
    host: UserOps,
}
