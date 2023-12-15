use common::DatabaseInterface;

use super::{UserOp, UserOpList, UserOps};
use crate::{DbUtilsT, LatestBlockInfos, SentinelConfig, SentinelDbUtils, SentinelError};

const NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY: usize = 20; // TODO make configurable?

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
        config: &SentinelConfig,
        db_utils: &SentinelDbUtils<D>,
        latest_block_infos: LatestBlockInfos,
    ) -> Result<UserOps, SentinelError> {
        if self.is_empty() {
            return Ok(UserOps::empty());
        };

        self.get_up_to_last_x_ops(db_utils, NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY)
            .map(|ops| ops.get_enqueued_but_neither_witnessed_nor_cancelled_nor_executed())
            .and_then(|potentially_cancellable_ops| {
                debug!(
                    "ops that have been enqueued but neither witnessed, cancelled nor executed: {}",
                    potentially_cancellable_ops.len()
                );
                let mut cancellable_ops: Vec<UserOp> = vec![];

                for op in potentially_cancellable_ops.iter() {
                    let origin_network_id = op.origin_network_id();

                    let is_cancellable = match latest_block_infos.get_for(origin_network_id) {
                        Err(_) => {
                            warn!("cannot cancel user op due to no chain data for its origin network: {origin_network_id}");
                            false
                        },
                        Ok(info) => {
                            // NOTE: So user ops, once enqueued on a chain, are executable
                            // some `baseChallengePeriodDuration` later. It is during this
                            // window that we have time to see the as yet unseen original
                            // `userSend` event on the origin chain.
                            let origin_chain_latest_block_timestamp = *info.block_timestamp();
                            let uid = op.uid_hex()?;
                            let destination_network_id = op.destination_network_id();
                            let enqueued_timestamp = op.enqueued_block_timestamp()?;
                            let base_challenge_period_duration = config.base_challenge_period_duration(&destination_network_id)?;

                            debug!("                             op uid: {uid}");
                            debug!("            origin chain network id: {}", info.network_id());
                            debug!("     user op destination network id: {destination_network_id}");
                            debug!("origin chain latest block timestamp: {origin_chain_latest_block_timestamp}");
                            debug!("         user op enqueued timestamp: {enqueued_timestamp}");
                            debug!("     base challenge period duration: {base_challenge_period_duration}");

                            let op_is_cancellable = if base_challenge_period_duration < enqueued_timestamp && origin_chain_latest_block_timestamp > 0 {
                                let can_cancel = enqueued_timestamp - base_challenge_period_duration < origin_chain_latest_block_timestamp;
                                if !can_cancel {
                                    warn!("cannot cancel user op because its origin chain is not synced to within max delta of {base_challenge_period_duration}s");
                                }
                                can_cancel
                            } else {
                                debug!("cannot peform user op cancellability calculation due to over/underflows");
                                false
                            };
                            info!("op is cancellable: {op_is_cancellable}");
                            op_is_cancellable
                        },
                    };

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
