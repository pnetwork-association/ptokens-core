use common::{Byte, Bytes};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use super::{NetworkIdError, NetworkIdVersion, ProtocolId};

const NUM_BYTES: usize = 4;

pub struct Bytes4([u8; NUM_BYTES]);

impl TryFrom<Bytes> for Bytes4 {
    type Error = NetworkIdError;

    fn try_from(bs: Bytes) -> Result<Self, Self::Error> {
        let l = bs.len();
        bs.try_into().map_err(|_| NetworkIdError::NotEnoughBytes {
            expected: NUM_BYTES,
            got: l,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct NetworkId<T: std::convert::AsRef<[u8]>> {
    chain_id: T,
    disambiguator: Byte, // NOTE: Can be rolled in case of collisions.
    protocol_id: ProtocolId,
    version: NetworkIdVersion,
}

impl<T: std::convert::AsRef<[u8]>> NetworkId<T> {
    pub fn to_bytes_4(&self) -> Result<Bytes4, NetworkIdError> {
        let mut hasher = Sha3_256::new();
        hasher.update([<NetworkIdVersion as Into<u8>>::into(self.version)]);
        hasher.update([<ProtocolId as Into<u8>>::into(self.protocol_id)]);
        hasher.update(&self.chain_id);
        hasher.update([self.disambiguator]);
        let hash = hasher.finalize();
        Bytes4::try_from(hash.to_vec())
    }
}
