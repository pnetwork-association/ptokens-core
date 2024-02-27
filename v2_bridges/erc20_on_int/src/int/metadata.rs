use std::str::FromStr;

use common::types::{Bytes, Result};
use common_eth::MAX_BYTES_FOR_ETH_USER_DATA;
use common_metadata::{Metadata, MetadataAddress, MetadataChainId, MetadataProtocolId};

use crate::int::eth_tx_info::Erc20OnIntEthTxInfo;

impl Erc20OnIntEthTxInfo {
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

        let origin_address = if cfg!(feature = "include-origin-tx-details") && self.metadata_event.is_some() {
            debug!("using origin tx details in metadata");
            let metadata = self
                .metadata_event
                .as_ref()
                .expect("this not to fail due to preceeding line");

            MetadataAddress::new(metadata.origin_address(), metadata.origin_chain_id())?
        } else {
            MetadataAddress::new_from_eth_address(
                &self.token_sender,
                &MetadataChainId::from_str(&self.origin_chain_id.to_string())?,
            )?
        };

        Ok(Metadata::new(&user_data, &origin_address))
    }

    pub fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}
