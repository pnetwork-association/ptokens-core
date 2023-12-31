use common::types::{Bytes, Result};
use common_eos::MAX_BYTES_FOR_EOS_USER_DATA;
use common_metadata::{Metadata, MetadataAddress, MetadataProtocolId};

use crate::int::eos_tx_info::EosOnIntEosTxInfo;

impl EosOnIntEosTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_EOS_USER_DATA {
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_EOS_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        Ok(Metadata::new_v3(
            &user_data,
            &MetadataAddress::new(&self.token_sender.to_string(), &self.origin_chain_id)?,
            &MetadataAddress::new(&self.destination_address.clone(), &self.destination_chain_id)?,
            None,
            None,
        ))
    }

    pub fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Eos)
    }
}
