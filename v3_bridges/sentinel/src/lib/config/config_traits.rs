use common::BridgeSide;
use ethereum_types::Address as EthAddress;

pub trait ConfigT {
    fn side(&self) -> BridgeSide;
    fn is_validating(&self) -> bool;
    fn get_contract_addresses(&self) -> Vec<EthAddress>;
}
