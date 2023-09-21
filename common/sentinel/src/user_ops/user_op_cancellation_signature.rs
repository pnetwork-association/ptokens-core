use common_eth::{EthSignature, ETH_SIGNATURE_NUM_BYTES};
use derive_getters::{Dissolve, Getters};
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{SentinelError, UserOpError, UserOpUniqueId, WebSocketMessagesEncodable};

type Bytes = Vec<u8>;

#[derive(Clone, Debug, Serialize, Deserialize, Deref)]
pub struct CancellationSignature(Bytes);

impl CancellationSignature {
    pub fn new(bs: Bytes) -> Result<Self, UserOpError> {
        let name = "UserOpCancellationSignature";
        debug!("creating `{name}` struct...");
        let l = bs.len();
        if l != ETH_SIGNATURE_NUM_BYTES {
            Err(UserOpError::NotEnoughBytes {
                got: l,
                expected: format!("{ETH_SIGNATURE_NUM_BYTES}"),
                location: name.to_string(),
            })
        } else {
            Ok(Self(bs))
        }
    }
}

#[derive(Clone, Debug, Constructor, Serialize, Deserialize, Getters, Dissolve)]
pub struct UserOpCancellationSignature {
    signer: EthAddress,
    uid: UserOpUniqueId,
    sig: CancellationSignature,
}

impl TryFrom<WebSocketMessagesEncodable> for UserOpCancellationSignature {
    type Error = SentinelError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<UserOpCancellationSignature, Self::Error> {
        debug!("trying to get `UserOpCancellationSignature` from `WebSocketMessagesEncodable`...");
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}

impl From<EthSignature> for CancellationSignature {
    fn from(es: EthSignature) -> CancellationSignature {
        CancellationSignature(es.0.to_vec())
    }
}
