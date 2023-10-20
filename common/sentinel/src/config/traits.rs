use common::BridgeSide;
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;

use super::SentinelConfigError;

pub trait ConfigT {
    fn gas_limit(&self) -> usize;
    fn side(&self) -> BridgeSide;
    fn is_validating(&self) -> bool;
    fn gas_price(&self) -> Option<u64>;
    fn pnetwork_hub(&self) -> EthAddress;
    fn pre_filter_receipts(&self) -> bool;
    fn mcid(&self) -> Result<MetadataChainId, SentinelConfigError>;
    fn metadata_chain_id(&self) -> Result<MetadataChainId, SentinelConfigError>;
}
