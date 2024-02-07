use common::types::{Bytes, Result};
use common_eth::MAX_BYTES_FOR_ETH_USER_DATA;
use common_metadata::{Metadata, MetadataAddress, MetadataProtocolId};
use common_safe_addresses::safely_convert_str_to_eth_address;

use crate::int::evm_tx_info::IntOnEvmEvmTxInfo;

impl IntOnEvmEvmTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            info!(
                "`user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };

        // NOTE: In this case the token sender is the router address, and the origin chain id
        // is that of the interim chain, IE 0xffffffff
        let mut origin_address = MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id)?;

        if cfg!(feature = "include-origin-tx-details") && self.metadata_event.is_some() {
            // NOTE: Here we use the router's metadata event to get the original sender & chain ID details
            let metadata = self
                .metadata_event
                .as_ref()
                .expect("this not to fail due to preceeding line");

            origin_address = MetadataAddress::new(metadata.origin_address(), metadata.origin_chain_id())?;
        };

        Ok(Metadata::new_v2(
            &user_data,
            &origin_address,
            &MetadataAddress::new_from_eth_address(
                &safely_convert_str_to_eth_address(&self.destination_address),
                &self.destination_chain_id,
            )?,
            None,
            None,
        ))
    }

    pub fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}
