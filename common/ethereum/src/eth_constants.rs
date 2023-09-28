pub const ZERO_ETH_VALUE: usize = 0;
pub const VALUE_FOR_MINTING_TX: usize = 0;
pub const ETH_WORD_SIZE_IN_BYTES: usize = 32;
pub const ETH_ADDRESS_SIZE_IN_BYTES: usize = 20;
pub const MAX_BYTES_FOR_ETH_USER_DATA: usize = 2000;
pub const ETH_CORE_IS_INITIALIZED_JSON: &str = "{eth_core_initialized:true}";
pub const EVM_CORE_IS_INITIALIZED_JSON: &str = "{evm_core_initialized:true}";

#[cfg(not(test))]
pub const ETH_TAIL_LENGTH: u64 = 100;

// NOTE: Tests use a smaller tail length so that we don't need to keep 100+ samples around!
#[cfg(test)]
pub const ETH_TAIL_LENGTH: u64 = 10;
