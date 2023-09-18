use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{EthState, EthSubmissionMaterial};
use common_eth_debug::reset_eth_chain;

use crate::{ProcessorOutput, SentinelError, UserOps};

pub fn reset_chain<D: DatabaseInterface>(
    db: &D,
    confs: u64,
    side: BridgeSide,
    sub_mat: EthSubmissionMaterial,
) -> Result<ProcessorOutput, SentinelError> {
    info!("resetting chain for side {side}...");
    let block_num = sub_mat.get_block_number()?.as_u64();
    let state = EthState::new_with_sub_mat(db, sub_mat);
    let is_for_eth = side.is_native();
    let _ = reset_eth_chain(state, confs, is_for_eth)?;
    let r = ProcessorOutput::new(side, block_num, UserOps::empty())?;
    Ok(r)
}
