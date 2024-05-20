use std::str::FromStr;

use common_sentinel::{DbIntegrity, SentinelError};
use derive_getters::Getters;
use serde::Deserialize;
use serde_json::Value as Json;

// NOTE: Some `dead_code` lint suppression is used because we don't need all the fields in these
// jsons (eg, in the success case, just successfully parsing the json is enough for us to continue
// execution elsewhere). The fields remain however in case we need them in future.

pub(super) enum DbJsonResponse {
    Error(DbJsonErrorResponse),
    Success(DbJsonSuccessResponse),
}

impl TryFrom<&str> for DbJsonResponse {
    type Error = SentinelError;

    fn try_from(s: &str) -> Result<Self, SentinelError> {
        debug!("attempting to parse `DbJsonResponse` from str {s}");

        match serde_json::from_str::<DbJsonSuccessResponse>(s) {
            Ok(r) => Ok(Self::Success(r)),
            Err(_) => match serde_json::from_str::<DbJsonErrorResponse>(s) {
                Ok(r) => Ok(Self::Error(r)),
                Err(e) => Err(e.into()),
            },
        }
    }
}

impl TryFrom<DbJsonResponse> for DbIntegrity {
    type Error = SentinelError;

    fn try_from(r: DbJsonResponse) -> Result<Self, Self::Error> {
        match r {
            DbJsonResponse::Error(j) => Self::try_from(j),
            DbJsonResponse::Success(j) => Ok(Self::from(j)),
        }
    }
}

#[derive(Debug, Default, Deserialize, Getters)]
#[allow(dead_code)]
pub(super) struct DbJsonSuccessResponse {
    id: u64,
    jsonrpc: String,
    result: Json,
}

#[derive(Debug, Default, Deserialize, Getters)]
pub(super) struct DbJsonErrorResponse {
    #[allow(dead_code)]
    id: u64,
    #[allow(dead_code)]
    jsonrpc: String,
    error: DbJsonError,
}

#[derive(Debug, Default, Deserialize, Getters)]
pub(super) struct DbJsonError {
    #[allow(dead_code)]
    code: u64,
    message: ErrorObject,
}

#[derive(Debug, Default, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorObject {
    #[allow(dead_code)]
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
