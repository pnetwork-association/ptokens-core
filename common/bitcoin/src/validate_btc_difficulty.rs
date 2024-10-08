use common::{traits::DatabaseInterface, types::Result};

#[cfg(not(feature = "ltc"))]
use crate::bitcoin_crate_alias::blockdata::block::BlockHeader as BtcBlockHeader;
#[cfg(feature = "ltc")]
use crate::bitcoin_crate_alias::blockdata::block::Header as BtcBlockHeader;
use crate::{bitcoin_crate_alias::network::constants::Network as BtcNetwork, BtcState};

#[cfg(not(feature = "ltc"))]
fn check_difficulty_is_above_threshold(
    threshold: u64,
    btc_block_header: &BtcBlockHeader,
    network: BtcNetwork,
) -> Result<()> {
    // NOTE: Network not configurable in difficulty calculation ∵ all members
    // of the enum return the same value from underlying lib!
    info!("✔ Checking BTC block difficulty is above threshold...");
    let difficulty = btc_block_header.difficulty(network);
    if network != BtcNetwork::Bitcoin {
        warn!("not on mainnet - skipping difficulty check");
        Ok(())
    } else if difficulty >= threshold {
        info!("✔ BTC block difficulty is above threshold");
        Ok(())
    } else {
        let msg = format!("difficulty of {difficulty} is below threshold of {threshold}");
        warn!("{msg}");
        Err(msg.into())
    }
}

#[cfg(feature = "ltc")]
fn check_difficulty_is_above_threshold(
    threshold: u64,
    btc_block_header: &BtcBlockHeader,
    network: BtcNetwork,
) -> Result<()> {
    // NOTE: Network not configurable in difficulty calculation ∵ all members
    // of the enum return the same value from underlying lib!
    info!("✔ Checking BTC block difficulty is above threshold...");
    let difficulty = btc_block_header.difficulty();
    if network != BtcNetwork::Bitcoin {
        warn!("not on mainnet - skipping difficulty check");
        Ok(())
    } else if difficulty >= threshold.into() {
        info!("✔ BTC block difficulty is above threshold");
        Ok(())
    } else {
        let msg = format!("difficulty of {difficulty} is below threshold of {threshold}");
        warn!("{msg}");
        Err(msg.into())
    }
}

pub fn validate_difficulty_of_btc_block_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    if cfg!(feature = "non-validating") {
        info!("✔ Skipping BTC block difficulty validation!");
        Ok(state)
    } else {
        info!("✔ Validating BTC block difficulty...");
        check_difficulty_is_above_threshold(
            state.btc_db_utils.get_btc_difficulty_from_db()?,
            &state.get_btc_block_and_id()?.block.header,
            state.btc_db_utils.get_btc_network_from_db()?,
        )
        .and(Ok(state))
    }
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_btc_block_and_id;

    #[test]
    fn should_not_err_if_difficulty_is_above_threshold() {
        let block_header = get_sample_btc_block_and_id().unwrap().block.header;
        let threshold: u64 = 1;
        check_difficulty_is_above_threshold(threshold, &block_header, BtcNetwork::Bitcoin).unwrap();
    }

    #[test]
    fn should_err_if_difficulty_is_below_threshold() {
        let block_header = get_sample_btc_block_and_id().unwrap().block.header;
        let threshold = u64::MAX;
        assert!(check_difficulty_is_above_threshold(threshold, &block_header, BtcNetwork::Bitcoin).is_err());
    }

    #[test]
    #[cfg(not(feature = "ltc"))]
    fn should_skip_difficulty_check_if_not_on_mainnet() {
        let threshold = 0;
        let block_header = get_sample_btc_block_and_id().unwrap().block.header;
        let network = BtcNetwork::Testnet;
        let difficulty = block_header.difficulty(network);
        assert!(difficulty > threshold);
        assert!(check_difficulty_is_above_threshold(threshold, &block_header, network,).is_ok());
    }
}
