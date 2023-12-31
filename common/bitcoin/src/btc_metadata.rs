#[cfg(not(feature = "ltc"))]
use std::str::FromStr;

use common::types::{Byte, Bytes, Result};
use common_chain_ids::BtcChainId;
use common_metadata::{Metadata, MetadataAddress, MetadataChainId, MetadataProtocolId};
#[cfg(not(feature = "ltc"))]
use common_safe_addresses::safely_convert_str_to_btc_address;
#[cfg(feature = "ltc")]
use common_safe_addresses::safely_convert_str_to_ltc_address;

pub trait ToMetadata {
    fn get_user_data(&self) -> Option<Bytes>;

    fn get_originating_tx_address(&self) -> String;

    fn maybe_to_metadata_bytes(
        &self,
        btc_chain_id: &BtcChainId,
        max_data_length: usize,
        destination_protocol_id: &MetadataProtocolId,
    ) -> Result<Option<Bytes>>
    where
        Self: Sized,
    {
        self.maybe_to_metadata(btc_chain_id, max_data_length)
            .and_then(|maybe_metadata| match maybe_metadata {
                Some(metadata) => Ok(Some(metadata.to_bytes_for_protocol(destination_protocol_id)?)),
                None => Ok(None),
            })
    }

    fn maybe_to_metadata(&self, btc_chain_id: &BtcChainId, max_data_length: usize) -> Result<Option<Metadata>>
    where
        Self: Sized,
    {
        info!("✔ Maybe getting metadata from user data...");
        match self.get_user_data() {
            Some(ref user_data) => {
                if user_data.len() > max_data_length {
                    info!(
                        "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes!",
                        max_data_length
                    );
                    Ok(None)
                } else {
                    self.to_metadata(user_data, btc_chain_id)
                }
            },
            None => {
                info!("✘ No user data to wrap into metadata ∴ skipping this step!");
                Ok(None)
            },
        }
    }

    #[cfg(not(feature = "ltc"))]
    fn to_metadata(&self, user_data: &[Byte], btc_chain_id: &BtcChainId) -> Result<Option<Metadata>> {
        info!("✔ Getting metadata from user data...");
        Ok(Some(Metadata::new(
            user_data,
            &MetadataAddress::new_from_btc_address(
                &safely_convert_str_to_btc_address(&self.get_originating_tx_address()),
                &MetadataChainId::from_str(&btc_chain_id.to_string())?,
            )?,
        )))
    }

    #[cfg(feature = "ltc")]
    fn to_metadata(&self, user_data: &[Byte], _btc_chain_id: &BtcChainId) -> Result<Option<Metadata>> {
        info!("✔ Getting metadata from user data...");
        let ma = &MetadataAddress {
            // NOTE: We only support litecoin mainnet
            metadata_chain_id: MetadataChainId::LitecoinMainnet,
            address: safely_convert_str_to_ltc_address(&self.get_originating_tx_address()).to_string(),
        };
        Ok(Some(Metadata::new(user_data, ma)))
    }
}
