use std::{fmt, str::FromStr};

use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{
    SentinelError,
    UserOpUniqueId,
    WebSocketMessagesError,
    WebSocketMessagesInitArgs,
    WebSocketMessagesResetChainArgs,
    WebSocketMessagesSubmitArgs,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodable {
    Null,
    GetUserOps,
    GetCoreState,
    GetUserOpList,
    Success(Json),
    GetLatestBlockNumbers,
    GetCancellableUserOps(u64),
    RemoveUserOp(UserOpUniqueId),
    Error(WebSocketMessagesError),
    Submit(Box<WebSocketMessagesSubmitArgs>),
    Initialize(Box<WebSocketMessagesInitArgs>),
    ResetChain(Box<WebSocketMessagesResetChainArgs>),
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
            // NOTE: We can't get a ref of this, so otherwise we'd have to convert all others
            // `.to_string()` individually.
            Self::Error(e) => format!("Error: {e}"),
            others => {
                let s = match others {
                    Self::Null => "Null",
                    Self::Submit(..) => "Submit",
                    Self::Success(_) => "Success",
                    Self::GetUserOps => "GetUserOps",
                    Self::Initialize(_) => "Initialize",
                    Self::ResetChain(_) => "ResetChain",
                    Self::GetCoreState => "GetCoreState",
                    Self::GetUserOpList => "GetUserOpList",
                    Self::RemoveUserOp(_) => "RemoveUserOp",
                    Self::GetLatestBlockNumbers => "GetLatestBlockNumbers",
                    Self::GetCancellableUserOps(_) => "GetCancellableUserOps",
                    Self::Error(_) => "we should never reach this match arm of this",
                };
                s.to_string()
            },
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
            "reset" | "resetChain" => Ok(Self::ResetChain(Box::new(WebSocketMessagesResetChainArgs::try_from(
                args[1..].to_vec(),
            )?))),
            "removeUserOp" => {
                let uid = UserOpUniqueId::from_str(&args[1])?;
                Ok(Self::RemoveUserOp(uid))
            },
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
        let m = WebSocketMessagesEncodable::GetCoreState;
        let s: String = m.clone().try_into().unwrap();
        let expected_s = "IkdldEVuY2xhdmVTdGF0ZSI";
        assert_eq!(s, expected_s);
        let r = WebSocketMessagesEncodable::try_from(s).unwrap();
        assert_eq!(r, m);
    }

    #[test]
    fn should_get_init_message_from_string_of_args() {
        let args = vec!["init", "true", "true", "EthereumMainnet", "10", "BscMainnet", "100"];
        let r = WebSocketMessagesEncodable::try_from(args);
        assert!(r.is_ok());
    }
}
