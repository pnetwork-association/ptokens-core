use std::str::FromStr;

use base64::{engine::general_purpose, Engine};
use common_metadata::MetadataChainId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Responder, SentinelError};

#[derive(Error, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesError {
    #[error("core not initialized for chain id: {0}")]
    Uninitialized(MetadataChainId),

    #[error("core already initialized for chain id: {0}")]
    AlreadyInitialized(MetadataChainId),

    #[error("cannot create websocket message encodable from args: {0:?}")]
    CannotCreate(Vec<String>),

    #[error("cannot create websocket message encodable from {got} args, expected {expected}: {args:?}")]
    NotEnoughArgs {
        got: usize,
        expected: usize,
        args: Vec<String>,
    },

    #[error("could not parse u64 from {0}")]
    ParseInt(String),

    #[error("unrecognized chain id {0}")]
    UnrecognizedMetadataChainId(String),

    #[error("timed out - strongbox took longer than {0}ms to respond")]
    Timedout(u64),
}

pub type Confirmations = u64;

#[derive(Debug)]
pub struct WebSocketMessages(
    pub WebSocketMessagesEncodable,
    pub Responder<WebSocketMessagesEncodable>,
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodable {
    Error(WebSocketMessagesError),
    Success(String),
    GetLatestBlockNum(MetadataChainId),
    Initialize {
        host_id: MetadataChainId,
        host_confs: Confirmations,
        native_id: MetadataChainId,
        native_confs: Confirmations,
    },
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
            "init" | "initialize" => {
                let expected_num_args = 5;
                if args.len() != expected_num_args {
                    return Err(WebSocketMessagesError::NotEnoughArgs {
                        got: args.len(),
                        expected: expected_num_args,
                        args,
                    });
                }
                Ok(Self::Initialize {
                    host_id: MetadataChainId::from_str(&args[1])
                        .map_err(|_| WebSocketMessagesError::UnrecognizedMetadataChainId(args[1].clone()))?,
                    host_confs: args[2]
                        .parse::<Confirmations>()
                        .map_err(|_| WebSocketMessagesError::ParseInt(args[2].clone()))?,
                    native_id: MetadataChainId::from_str(&args[3])
                        .map_err(|_| WebSocketMessagesError::UnrecognizedMetadataChainId(args[3].clone()))?,
                    native_confs: args[4]
                        .parse::<Confirmations>()
                        .map_err(|_| WebSocketMessagesError::ParseInt(args[4].clone()))?,
                })
            },
            _ => {
                debug!("cannot create WebSocketMessagesEncodable from args {args:?}");
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
        let id = MetadataChainId::EthereumMainnet;
        let m = WebSocketMessagesEncodable::GetLatestBlockNum(id);
        let s: String = m.clone().try_into().unwrap();
        let expected_s = "eyJHZXRMYXRlc3RCbG9ja051bSI6IkV0aGVyZXVtTWFpbm5ldCJ9";
        assert_eq!(s, expected_s);
        let r = WebSocketMessagesEncodable::try_from(s).unwrap();
        assert_eq!(r, m);
    }

    #[test]
    fn should_get_init_message_from_string_of_args() {
        let args = vec!["init", "EthereumMainnet", "10", "BscMainnet", "100"];
        let r = WebSocketMessagesEncodable::try_from(args);
        assert!(r.is_ok());
        println!("r {r:?}")
    }
}
