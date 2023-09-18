use std::fmt;

use serde::{Deserialize, Serialize};

type Bytes = Vec<u8>;

use super::websocket_messages_utils::check_num_args;
use crate::WebSocketMessagesError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodableDbOps {
    Get(Bytes),
    Delete(Bytes),
    Put(Bytes, Bytes),
}

impl TryFrom<Vec<String>> for WebSocketMessagesEncodableDbOps {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        debug!("trying `WebSocketMessagesEncodableDbOps` from `Vec<String>`...");
        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };

        let checked_args = check_num_args(2, args)?;
        let cmd = checked_args[0].as_ref();
        let k = hex::decode(&checked_args[1])?;
        match cmd {
            "get" => Ok(Self::Get(k)),
            "delete" => Ok(Self::Delete(k)),
            "put" => {
                let final_args = check_num_args(3, checked_args)?;
                let v = hex::decode(&final_args[2])?;
                Ok(Self::Put(k, v))
            },
            _ => {
                warn!("cannot create WebSocketMessagesEncodableDbOps from args {checked_args:?}");
                Err(WebSocketMessagesError::CannotCreate(checked_args))
            },
        }
    }
}

impl fmt::Display for WebSocketMessagesEncodableDbOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "WebSocketMessagesEncodableDbOps::";
        let s = match self {
            Self::Get(..) => "Get",
            Self::Put(..) => "Put",
            Self::Delete(..) => "Delete",
        };
        write!(f, "{prefix}{s}")
    }
}
