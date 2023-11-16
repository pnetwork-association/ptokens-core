use std::str::FromStr;

use common_sentinel::{DbIntegrity, SentinelError};
use derive_getters::Getters;
use serde::Deserialize;
use serde_json::Value as Json;

#[derive(Debug, Default, Deserialize, Getters)]
pub(super) struct DbJsonSuccessResponse {
    id: String,
    jsonrpc: String,
    result: Json,
}

#[derive(Debug, Default, Deserialize, Getters)]
pub(super) struct DbJsonErrorResponse {
    id: String,
    jsonrpc: String,
    error: DbJsonError,
}

#[derive(Debug, Default, Deserialize, Getters)]
pub(super) struct DbJsonError {
    code: String,
    message: ErrorObject,
}

#[derive(Debug, Default, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorObject {
    msg: String,
    db_integrity: String,
}

impl From<DbJsonSuccessResponse> for DbIntegrity {
    fn from(_res: DbJsonSuccessResponse) -> Self {
        // NOTE: A successful parsing of a success json response means the database integrity is
        // valid.
        Self::Valid
    }
}

impl TryFrom<DbJsonErrorResponse> for DbIntegrity {
    type Error = SentinelError;

    fn try_from(res: DbJsonErrorResponse) -> Result<Self, Self::Error> {
        Ok(DbIntegrity::from_str(res.error().message().db_integrity())?)
    }
}
