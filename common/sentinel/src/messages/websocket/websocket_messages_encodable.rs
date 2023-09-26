use std::{fmt, str::FromStr};

use base64::{engine::general_purpose, Engine};
use common_metadata::MetadataChainId;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::WebSocketMessagesEncodableDbOps;
use crate::{
    SentinelError,
    UserOp,
    UserOpUniqueId,
    WebSocketMessagesError,
    WebSocketMessagesGetCancellableUserOpArgs,
    WebSocketMessagesInitArgs,
    WebSocketMessagesResetChainArgs,
    WebSocketMessagesSubmitArgs,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodable {
    Null,
    GetUserOps,
    GetUserOpList,
    Success(Json),
    RemoveUserOp(UserOpUniqueId),
    Error(WebSocketMessagesError),
    GetCoreState(Vec<MetadataChainId>),
    DbOps(WebSocketMessagesEncodableDbOps),
    Submit(Box<WebSocketMessagesSubmitArgs>),
    Initialize(Box<WebSocketMessagesInitArgs>),
    GetLatestBlockNumbers(Vec<MetadataChainId>),
    GetUserOpCancellationSiganture(Box<UserOp>),
    ResetChain(Box<WebSocketMessagesResetChainArgs>),
    GetCancellableUserOps(Box<WebSocketMessagesGetCancellableUserOpArgs>),
}

impl TryFrom<WebSocketMessagesEncodable> for Json {
    type Error = WebSocketMessagesError;

    fn try_from(w: WebSocketMessagesEncodable) -> Result<Json, Self::Error> {
        match w {
            WebSocketMessagesEncodable::Success(json) => Ok(json),
            other => Err(WebSocketMessagesError::CannotConvert {
                to: "json".to_string(),
                from: other.to_string(),
            }),
        }
    }
}

impl WebSocketMessagesEncodable {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

impl fmt::Display for WebSocketMessagesEncodable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "WebSocketMessagesEncodable::";
        let s = match self {
            Self::Null => "Null".to_string(),
            Self::DbOps(op) => format!("{op}"),
            Self::Error(e) => format!("Error: {e}"),
            Self::Submit(..) => "Submit".to_string(),
            Self::Success(_) => "Success".to_string(),
            Self::GetUserOps => "GetUserOps".to_string(),
            Self::Initialize(_) => "Initialize".to_string(),
            Self::ResetChain(_) => "ResetChain".to_string(),
            Self::GetUserOpList => "GetUserOpList".to_string(),
            Self::GetCoreState(..) => "GetCoreState".to_string(),
            Self::RemoveUserOp(_) => "RemoveUserOp".to_string(),
            Self::GetCancellableUserOps(_) => "GetCancellableUserOps".to_string(),
            Self::GetLatestBlockNumbers(..) => "GetLatestBlockNumbers".to_string(),
            Self::GetUserOpCancellationSiganture(..) => "GetUserOpCancellationSiganture".to_string(),
        };
        write!(f, "{prefix}{s}")
    }
}

impl TryFrom<String> for WebSocketMessagesEncodable {
    type Error = SentinelError;

    fn try_from(s: String) -> Result<Self, SentinelError> {
        Self::try_from(s.as_str())
    }
}

impl TryFrom<&str> for WebSocketMessagesEncodable {
    type Error = SentinelError;

    fn try_from(s: &str) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(&general_purpose::STANDARD_NO_PAD.decode(s)?)?)
    }
}

impl TryFrom<Vec<&str>> for WebSocketMessagesEncodable {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<&str>) -> Result<Self, WebSocketMessagesError> {
        Self::try_from(args.iter().map(|x| x.to_string()).collect::<Vec<_>>())
    }
}

impl TryFrom<Vec<String>> for WebSocketMessagesEncodable {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };
        let cmd = args[0].as_ref();
        match cmd {
            "init" | "initialize" => Ok(Self::Initialize(Box::new(WebSocketMessagesInitArgs::try_from(
                args[1..].to_vec(),
            )?))),
            "getCancellableUserOps" => Ok(Self::GetCancellableUserOps(Box::new(
                WebSocketMessagesGetCancellableUserOpArgs::try_from(args[1..].to_vec())?,
            ))),
            "reset" | "resetChain" => Ok(Self::ResetChain(Box::new(WebSocketMessagesResetChainArgs::try_from(
                args[1..].to_vec(),
            )?))),
            "removeUserOp" => {
                let uid = UserOpUniqueId::from_str(&args[1])?;
                Ok(Self::RemoveUserOp(uid))
            },
            "get" | "put" | "delete" => Ok(Self::DbOps(WebSocketMessagesEncodableDbOps::try_from(args)?)),
            _ => {
                warn!("cannot create WebSocketMessagesEncodable from args {args:?}");
                Err(WebSocketMessagesError::CannotCreate(args))
            },
        }
    }
}

impl TryInto<String> for WebSocketMessagesEncodable {
    type Error = SentinelError;

    fn try_into(self) -> Result<String, SentinelError> {
        Ok(general_purpose::STANDARD_NO_PAD.encode(serde_json::to_vec(&self)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_messages_encodable_should_make_serde_roundtrip() {
        let m = WebSocketMessagesEncodable::GetLatestBlockNumbers;
        let s: String = m.clone().try_into().unwrap();
        let expected_s = "IkdldExhdGVzdEJsb2NrTnVtYmVycyI";
        assert_eq!(s, expected_s);
        let r = WebSocketMessagesEncodable::try_from(s).unwrap();
        assert_eq!(r, m);
    }
}
