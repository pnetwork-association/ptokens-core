use common::BridgeSide;
use ethereum_types::Address as EthAddress;

pub trait ConfigT {
    fn gas_limit(&self) -> usize;
    fn side(&self) -> BridgeSide;
    fn is_validating(&self) -> bool;
    fn gas_price(&self) -> Option<u64>;
    fn pnetwork_hub(&self) -> EthAddress;
}