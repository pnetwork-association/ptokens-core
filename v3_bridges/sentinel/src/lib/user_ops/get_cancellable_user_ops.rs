use common::{BridgeSide, DatabaseInterface};

use super::{UserOp, UserOpList, UserOps};
use crate::{DbUtilsT, SentinelDbUtils, SentinelError};

const NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY: usize = 10; // TODO make configurable?

impl UserOpList {
    fn get_up_to_last_x_ops<D: DatabaseInterface>(
        &self,
        db_utils: &SentinelDbUtils<D>,
        x: usize,
    ) -> Result<UserOps, SentinelError> {
        let num_ops = self.len();

        let idx = if num_ops >= x { num_ops - x } else { 0 };
        debug!("getting {} user ops from idx {idx} to {}", num_ops - idx, num_ops - 1);

        Ok(UserOps::new(
            self[idx..]
                .iter()
                .map(|entry| UserOp::get_from_db(db_utils, &entry.uid().into()))
                .collect::<Result<Vec<UserOp>, SentinelError>>()?,
        ))
    }

    pub fn get_cancellable_ops<D: DatabaseInterface>(
        &self,
        max_delta: u64,
        db_utils: &SentinelDbUtils<D>,
        n_latest_block_timestamp: u64,
        h_latest_block_timestamp: u64,
    ) -> Result<UserOps, SentinelError> {
        self.get_up_to_last_x_ops(db_utils, NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY)
            .map(|ops| ops.get_enqueued_but_not_witnessed())
            .and_then(|cancellable_ops| {
                let mut ops: Vec<UserOp> = vec![];
                for op in cancellable_ops.iter() {
                    let side = op.side();
                    let time = op.enqueued_timestamp()?;

                    let cancellable = match side {
                        // NOTE: Note that for host cancellations we need to ensure the _native_
                        // chain is in sync (within the allowable delta) and vice versa. This is so
                        // we've had every chance to see the witnessed event.
                        BridgeSide::Host => time - max_delta < n_latest_block_timestamp,
                        BridgeSide::Native => time - max_delta < h_latest_block_timestamp,
                    };
                    debug!(
                        "side: {side}, enqueued_timestamp: {time}, max_delta: {max_delta}, cancellable: {cancellable}"
                    );
                    if cancellable {
                        ops.push(op.clone())
                    }
                }

                Ok(UserOps::new(ops))
            })
    }
}
