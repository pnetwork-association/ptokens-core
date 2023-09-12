use common::BridgeSide;
use common_chain_ids::EthChainId;
use ethereum_types::Address as EthAddress;

pub trait ConfigT {
    fn gas_limit(&self) -> usize;
    fn side(&self) -> BridgeSide;
    fn is_validating(&self) -> bool;
    fn chain_id(&self) -> EthChainId;
    fn gas_price(&self) -> Option<u64>;
    fn pnetwork_hub(&self) -> EthAddress;
}
