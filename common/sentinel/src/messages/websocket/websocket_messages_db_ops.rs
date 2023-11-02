use std::fmt;

use serde::{Deserialize, Serialize};

type Bytes = Vec<u8>;

use common::strip_hex_prefix;

use super::websocket_messages_utils::check_num_args;
use crate::{DebugSignature, WebSocketMessagesError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodableDbOps {
    Get(Bytes, DebugSignature),
    Delete(Bytes, DebugSignature),
    Put(Bytes, Bytes, DebugSignature),
}

impl TryFrom<Vec<String>> for WebSocketMessagesEncodableDbOps {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        debug!("trying `WebSocketMessagesEncodableDbOps` from `Vec<String>`...");
        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };

        const MIN_NUM_ARGS: usize = 2;
        let checked_args = check_num_args(MIN_NUM_ARGS, args)?;
        let cmd = checked_args[0].as_ref();
        let k = hex::decode(strip_hex_prefix(&checked_args[1]))?;

        match cmd {
            "get" => Ok(Self::Get(k, checked_args.last().into())),
            "delete" => Ok(Self::Delete(k, checked_args.last().into())),
            "put" => {
                let final_args = check_num_args(3, checked_args)?;
                let v = hex::decode(&final_args[2])?;
                let maybe_sig = final_args.last().into();
                Ok(Self::Put(k, v, maybe_sig))
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
