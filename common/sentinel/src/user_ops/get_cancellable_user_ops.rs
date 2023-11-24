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
                    // NOTE:User ops don't include/commit to their originating network IDs. User ops also go through
                    // an interim chain. So a movement from chain X to chain Y is acually two distinct user ops:
                    // `chain X -> interim chain` then `interim chain -> chain Y`.
                    //
                    // The second user ops' `underlyingAssetNetworkId` maintains a pointer to chain X, but the user
                    // op the sentinel needs to have witnessed for this second user op is on the _interim chain_.
                    // The op does _not_ track this network ID.
                    //
                    // As such, in order to determine if a user op is cancellable, we have to ensure that ALL the
                    // chains this sentinel is tracking are up to date to within  some allowable time delta. This
                    // condition met means we've have every chance to see the origin user op event, and thus we
                    // haven't seen it, so it can't be a valid op and thus is cancellable.
                    let is_cancellable = latest_block_infos.iter().all(|info| {
                        let latest_block_timestamp = *info.block_timestamp();
                        debug!("                    op uid: {uid}");
                        debug!("                network id: {}", info.network_id());
                        debug!("    latest block timestamp: {latest_block_timestamp}");
                        debug!("user op enqueued timestamp: {enqueued_timestamp}");
                        debug!("                 max delta: {max_delta}");

                        let r = if max_delta < enqueued_timestamp && latest_block_timestamp > 0 {
                            enqueued_timestamp - max_delta < latest_block_timestamp
                        } else {
                            false
                        };
                        debug!("         op is cancellable: {r}");
                        debug!(" op destination network id: {d_nid}");
                        r
                    });

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
