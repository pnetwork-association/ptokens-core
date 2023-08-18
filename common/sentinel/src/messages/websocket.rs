use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

use crate::SentinelError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessages {
    GetNativeLatestBlockNum,
}

impl WebSocketMessages {
    fn encode(&self) -> Result<String, SentinelError> {
        Ok(general_purpose::STANDARD_NO_PAD.encode(serde_json::to_vec(self)?))
    }

    fn decode(s: &str) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(&general_purpose::STANDARD_NO_PAD.decode(s)?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_serde_roundtrip() {
        let m = WebSocketMessages::GetNativeLatestBlockNum;
        let s = m.encode().unwrap();
        let expected_s = "IkdldE5hdGl2ZUxhdGVzdEJsb2NrTnVtIg";
        assert_eq!(s, expected_s);
        let r = WebSocketMessages::decode(&s).unwrap();
        assert_eq!(r, m);
    }
}
