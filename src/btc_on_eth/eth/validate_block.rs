use ethereum_types::H256;
use crate::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    constants::CORE_IS_VALIDATING,
    btc_on_eth::{
        crypto_utils::keccak_hash_bytes,
        eth::{
            eth_types::EthBlock,
            eth_state::EthState,
            rlp_codec::rlp_encode_block,
        },
    },
};

fn hash_block(block: &EthBlock) -> Result<H256> {
    trace!("block being encoded: {:?}", block);
    trace!("rlp encoded block: {}", hex::encode(rlp_encode_block(block)?));
    rlp_encode_block(block)
        .map(keccak_hash_bytes)
}

pub fn validate_block_header(block: &EthBlock) -> Result<bool> {
    hash_block(block)
        .map(|hash| {
            trace!("✔ Block hash from from block: {}", block.hash);
            trace!("✔ Calculated block hash: {}", hash);
            hash == block.hash
        })
}

pub fn validate_block_in_state<D>(state: EthState<D>) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    if CORE_IS_VALIDATING {
        info!("✔ Validating block header...");
        match validate_block_header(
            &state.get_eth_block_and_receipts()?.block
        )? {
            true => Ok(state),
            false => Err(AppError::Custom(
                "✘ Not accepting ETH block - header hash not valid!".to_string()
            )),
        }
    } else {
        info!("✔ Skipping ETH block header validaton!");
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_sample_invalid_block,
        get_sample_eth_block_and_receipts,
        get_valid_state_with_block_and_receipts,
    };

    #[test]
    fn should_hash_block() {
        let block = get_sample_eth_block_and_receipts().block;
        let result = hash_block(&block)
            .unwrap();
        assert_eq!(result, block.hash)
    }

    #[test]
    fn valid_block_header_should_return_true() {
        let block = get_sample_eth_block_and_receipts().block;
        let result = validate_block_header(&block)
            .unwrap();
        assert!(result);
    }

    #[test]
    fn invalid_block_header_should_return_true() {
        let invalid_block = get_sample_invalid_block();
        let result = validate_block_header(&invalid_block)
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn should_validate_block_in_state() {
        let state = get_valid_state_with_block_and_receipts()
            .unwrap();
        if validate_block_in_state(state).is_err() {
            panic!("Block in state should be valid!")
        }
    }

    #[cfg(not(feature="non-validating"))]
    #[test]
    fn should_fail_to_validate_invalid_block_in_state() {
        use crate::btc_on_eth::eth::eth_test_utils::{
            get_valid_state_with_invalid_block_and_receipts
        };
        let expected_error = "✘ Not accepting ETH block - header hash not valid!"
            .to_string();
        let state = get_valid_state_with_invalid_block_and_receipts()
            .unwrap();
        match validate_block_in_state(state) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            _ => panic!("Should not validate invalid block in state!")
        }
    }
}
