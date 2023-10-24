use common::DatabaseInterface;

use super::{UserOp, UserOpList, UserOps};
use crate::{DbUtilsT, SentinelDbUtils, SentinelError};

const NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY: usize = 10; // TODO make configurable?

impl UserOpList {
    fn get_up_to_last_x_ops<D: DatabaseInterface>(
        &self,
        db_utils: &SentinelDbUtils<D>,
        x: usize,
    ) -> Result<UserOps, SentinelError> {
        if self.is_empty() || x == 0 {
            return Ok(UserOps::empty());
        };

        let num_ops = self.len();
        let num_ops_to_get = if x > num_ops { num_ops } else { x };
        let start_idx = num_ops - num_ops_to_get;

        debug!("getting {num_ops_to_get} user ops (from idx {start_idx} to {num_ops}");

        Ok(UserOps::new(
            self[start_idx..]
                .iter()
                .map(|entry| UserOp::get_from_db(db_utils, &entry.uid().into()))
                .collect::<Result<Vec<UserOp>, SentinelError>>()?,
        ))
    }

    pub fn get_cancellable_ops<D: DatabaseInterface>(
        &self,
        max_delta: u64,
        db_utils: &SentinelDbUtils<D>,
        // FIXME take vec of tuples of nid & timestamp
        n_latest_block_timestamp: u64,
        h_latest_block_timestamp: u64,
    ) -> Result<UserOps, SentinelError> {
        if self.is_empty() {
            return Ok(UserOps::empty());
        };

        self.get_up_to_last_x_ops(db_utils, NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY)
            .map(|ops| ops.get_enqueued_but_not_witnessed())
            .and_then(|potentially_cancellable_ops| {
                debug!("num ops queued but not witnessed: {}", potentially_cancellable_ops.len());
                debug!(
                    "max delta: {max_delta}, n_latest_timestamp: {n_latest_block_timestamp}, h_latest_block_timestamp: {h_latest_block_timestamp}"
                );

                let mut ops: Vec<UserOp> = vec![];
                /*
                for op in potentially_cancellable_ops.iter() {
                    let side = op.side();
                    let uid = op.uid_hex()?;
                    let time = op.enqueued_timestamp()?;

                    // FIXME check for underflow!

                    // TODO fix this to check ALL other chains. If any one is not in sync then we can't in good faith cancel a user op");

                    let is_cancellable = match side {
                        // NOTE: Note that for host cancellations we need to ensure the _native_
                        // chain is in sync (within the allowable delta) and vice versa. This is so
                        // we've had every chance to witness  the originationg `userSend` event.
                        BridgeSide::Host => time - max_delta < n_latest_block_timestamp,
                        BridgeSide::Native => time - max_delta < h_latest_block_timestamp,
                    };
                    debug!(
                        "op uid: {uid}, destination: {side}, enqueued_timestamp: {time}, is_cancellable: {is_cancellable}"
                    );

                    if is_cancellable {
                        ops.push(op.clone())
                    }
                }
                */

                let r = UserOps::new(ops);
                debug!("num cancellable ops: {}", r.len());
                Ok(r)
            })
    }
}
