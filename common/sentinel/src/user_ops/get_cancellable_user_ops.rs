use common::DatabaseInterface;

use super::{UserOp, UserOpList, UserOps};
use crate::{DbUtilsT, LatestBlockInfos, SentinelDbUtils, SentinelError};

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
        latest_block_infos: LatestBlockInfos,
    ) -> Result<UserOps, SentinelError> {
        if self.is_empty() {
            return Ok(UserOps::empty());
        };

        self.get_up_to_last_x_ops(db_utils, NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY)
            .map(|ops| ops.get_enqueued_but_not_witnessed())
            .and_then(|potentially_cancellable_ops| {
                debug!(
                    "num ops queued but not witnessed: {}",
                    potentially_cancellable_ops.len()
                );
                let mut cancellable_ops: Vec<UserOp> = vec![];

                for op in potentially_cancellable_ops.iter() {
                    let uid = op.uid_hex()?;
                    let d_nid = op.destination_network_id();
                    let enqueued_timestamp = op.enqueued_timestamp()?;

                    // NOTE:User ops don't include their origin network IDs, meaning we have to
                    // ensure _all_ other chains this sentinel works with are within the max
                    // allowable delta in order to conclude whether or not an operation is
                    // cancellable.
                    let is_cancellable = latest_block_infos.iter().all(|info| {
                        let latest_block_timestamp = *info.block_timestamp();
                        debug!("                network id: {}", info.network_id());
                        debug!("    latest block timestamp: {latest_block_timestamp}");
                        debug!("user op enqueued timestamp: {enqueued_timestamp}");
                        debug!("                 max delta: {max_delta}");

                        let r = if max_delta > enqueued_timestamp && latest_block_timestamp > 0 {
                            enqueued_timestamp - max_delta < latest_block_timestamp
                        } else {
                            false
                        };
                        debug!("         op is cancellable: {r}");
                        r
                    });

                    debug!(
                        "op uid: {}, max_delta {}, destination: {}, enqueued_timestamp: {}, is_cancellable: {}",
                        uid, max_delta, d_nid, enqueued_timestamp, is_cancellable,
                    );

                    if is_cancellable {
                        cancellable_ops.push(op.clone())
                    }
                }

                let r = UserOps::new(cancellable_ops);
                debug!("num cancellable ops: {}", r.len());
                Ok(r)
            })
    }
}
