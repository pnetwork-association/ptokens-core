use common::DatabaseInterface;

use super::{UserOp, UserOpList, UserOps};
use crate::{get_utc_timestamp, DbUtilsT, LatestBlockInfos, NetworkId, SentinelDbUtils, SentinelError};

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

    fn op_is_cancellable(latest_block_infos: &LatestBlockInfos, origin_network_id: &NetworkId, e_t: u64) -> bool {
        info!("checking if user op is cancellable...");
        const LEEWAY: u64 = 10; // NOTE: To account for block timestamps not being entirely reliable
        let c_t = get_utc_timestamp().unwrap_or_default();

        let o_t = match latest_block_infos.get_for(origin_network_id) {
            Ok(info) => *info.block_timestamp(),
            _ => {
                warn!("cannot cancel user op due to no chain data for its origin network: {origin_network_id}");
                return false;
            },
        };

        debug!("     origin time: {o_t}");
        debug!("    current time: {c_t}");
        debug!("   enqueued time: {e_t}");

        let origin_chain_is_beyond_enqueued_time = o_t > LEEWAY && o_t - LEEWAY >= e_t;
        if origin_chain_is_beyond_enqueued_time {
            info!("origin chain is beyond enqueued time and we've not seen a user send so op is cancellable");
            return true;
        }

        let origin_chain_is_in_sync = c_t != 0 && o_t + LEEWAY <= c_t;
        if origin_chain_is_in_sync {
            info!("origin chain is in sync and we've not seen a user send so op is cancellable");
            true
        } else {
            info!("origin chain is not in sync so we don't know if there was a user send or not, so op is not cancellable");
            false
        }
    }

    pub fn get_cancellable_ops<D: DatabaseInterface>(
        &self,
        db_utils: &SentinelDbUtils<D>,
        latest_block_infos: LatestBlockInfos,
    ) -> Result<UserOps, SentinelError> {
        if self.is_empty() {
            return Ok(UserOps::empty());
        };

        self.get_up_to_last_x_ops(db_utils, NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY)
            .map(|ops| ops.get_enqueued_but_neither_witnessed_nor_cancelled_nor_executed())
            .and_then(|ops| {
                debug!(
                    "ops that are enqueued but neither witnessed, cancelled nor executed: {}",
                    ops.len()
                );
                let mut cancellable_ops: Vec<UserOp> = vec![];
                for op in ops.iter() {
                    let origin_time = op.origin_network_id();
                    let enqueued_time = op.enqueued_block_timestamp()?;
                    if Self::op_is_cancellable(&latest_block_infos, origin_time, enqueued_time) {
                        cancellable_ops.push(op.clone())
                    };
                }
                let r = UserOps::new(cancellable_ops);
                debug!("num cancellable ops: {}", r.len());
                Ok(r)
            })
    }
}
