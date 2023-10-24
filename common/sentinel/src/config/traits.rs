use ethereum_types::Address as EthAddress;

pub trait ConfigT {
    fn gas_limit(&self) -> usize;
    fn is_validating(&self) -> bool;
    fn gas_price(&self) -> Option<u64>;
    fn pnetwork_hub(&self) -> EthAddress;
    fn pre_filter_receipts(&self) -> bool;
}
