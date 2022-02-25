use crate::{
    chains::eth::eth_constants::MAX_BYTES_FOR_ETH_USER_DATA,
    erc20_on_int::eth::int_tx_info::Erc20OnIntIntTxInfo,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::ToMetadata,
        Metadata,
    },
    types::{Bytes, Result},
};

impl ToMetadata for Erc20OnIntIntTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        Ok(Metadata::new_v2(
            &user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id)?,
            &self.destination_chain_id,
            &MetadataAddress::new_from_eth_address(&self.destination_address, &self.destination_chain_id)?,
            None,
            None,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}
