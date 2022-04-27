pub(crate) mod metadata_address;
pub(crate) mod metadata_chain_id;
pub(crate) mod metadata_encoders;
pub(crate) mod metadata_protocol_id;
pub(crate) mod metadata_traits;
pub(crate) mod metadata_version;
pub(crate) mod test_utils;

use serde::{Deserialize, Serialize};

use crate::{
    metadata::{
        metadata_address::MetadataAddress,
        metadata_chain_id::MetadataChainId,
        metadata_version::MetadataVersion,
    },
    types::{Byte, Bytes, NoneError, Result},
};

/// Metadata V1 Specification per @bertani:
/// [
///     uint8 versionByte,
///     bytes userData,
///     bytes4 originProtocol <bytes1 originProtocolId + bytes3 keccak256(originChainId)[:3]>,
///     origin sender
/// ]
///
/// The v2 specification expands this to enclude destination address and chain IDs, along with
/// protocol options and receipt as further places to encode pertinent data.
///
/// The v3 specification affects how the ETH encoding of the metadata works. It changes the address
/// types from native EVM addresses, to strings, in order to be more flexible.
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub version: MetadataVersion,
    pub user_data: Bytes,
    pub origin_chain_id: MetadataChainId,
    pub origin_address: MetadataAddress, // NOTE: The above is redundant, since it's in here!
    pub destination_chain_id: Option<MetadataChainId>,
    pub destination_address: Option<MetadataAddress>, // NOTE: Ibid.
    pub protocol_options: Option<Bytes>,
    pub protocol_receipt: Option<Bytes>,
}

impl Metadata {
    pub fn get_destination_chain_id(&self) -> Result<MetadataChainId> {
        match self.version {
            MetadataVersion::V1 => Err("Cannot get destination chain ID from v1 metadata!".into()),
            _ => self
                .destination_chain_id
                .ok_or(NoneError("Error getting destinaction chain ID!")),
        }
    }

    pub fn new(user_data: &[Byte], origin_address: &MetadataAddress) -> Self {
        Self::new_v1(user_data, origin_address)
    }

    fn new_v1(user_data: &[Byte], origin_address: &MetadataAddress) -> Self {
        Self {
            version: MetadataVersion::V1,
            user_data: user_data.to_vec(),
            origin_address: origin_address.clone(),
            origin_chain_id: origin_address.metadata_chain_id,
            destination_chain_id: None,
            destination_address: None,
            protocol_options: None,
            protocol_receipt: None,
        }
    }

    pub fn new_v2(
        user_data: &[Byte],
        origin_address: &MetadataAddress,
        destination_address: &MetadataAddress,
        protocol_options: Option<Bytes>,
        protocol_receipt: Option<Bytes>,
    ) -> Self {
        Self {
            protocol_options,
            protocol_receipt,
            version: MetadataVersion::V2,
            user_data: user_data.to_vec(),
            origin_address: origin_address.clone(),
            origin_chain_id: origin_address.metadata_chain_id,
            destination_address: Some(destination_address.clone()),
            destination_chain_id: Some(destination_address.metadata_chain_id),
        }
    }

    pub fn new_v3(
        user_data: &[Byte],
        origin_address: &MetadataAddress,
        destination_address: &MetadataAddress,
        protocol_options: Option<Bytes>,
        protocol_receipt: Option<Bytes>,
    ) -> Self {
        info!("✔ Getting v3 metadata...");
        Self {
            protocol_options,
            protocol_receipt,
            version: MetadataVersion::V3,
            user_data: user_data.to_vec(),
            origin_address: origin_address.clone(),
            origin_chain_id: origin_address.metadata_chain_id,
            destination_address: Some(destination_address.clone()),
            destination_chain_id: Some(destination_address.metadata_chain_id),
        }
    }
}
