use common::DatabaseInterface;
use common_chain_ids::EthChainId;
use derive_getters::Getters;
use derive_more::{Constructor, Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::{UserOp, UserOpError, UserOpList, UserOpState, UserOpStates, UserOps, USER_OP_CANCEL_TX_GAS_LIMIT};
use crate::{LatestBlockInfos, NetworkId, SentinelDbUtils, SentinelError};

#[derive(Clone, Debug, Eq, Default, PartialEq, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct CancellableUserOps(Vec<CancellableUserOp>);

#[derive(Clone, Debug, Eq, Default, PartialEq, Serialize, Deserialize, Constructor, Getters)]
pub struct CancellableUserOp {
    op: UserOp,
    state: UserOpState,
}

impl From<Vec<CancellableUserOp>> for CancellableUserOps {
    fn from(v: Vec<CancellableUserOp>) -> Self {
        Self::new(v)
    }
}

impl CancellableUserOp {
    pub fn cancellation_gas_limit(&self) -> Result<usize, UserOpError> {
        let nid = self.enqueued_network_id()?;
        let ecid = EthChainId::try_from(nid)?;
        match ecid {
            EthChainId::XDaiMainnet => Ok(200_000),
            EthChainId::ArbitrumMainnet => Ok(2_000_000),
            _ => Ok(USER_OP_CANCEL_TX_GAS_LIMIT as usize),
        }
    }

    fn enqueued_state(&self) -> Result<UserOpState, UserOpError> {
        let s = *self.state();
        if s.is_enqueued() {
            Ok(s)
        } else {
            // NOTE: We should never reach here since the `CancellableUserOps` is built via
            // the `From` defined below, but better to have it handled here just in case.
            Err(UserOpError::CancellableUserOpIsNotEnqueued(self.op.uid()?))
        }
    }

    fn enqueued_network_id(&self) -> Result<NetworkId, UserOpError> {
        self.enqueued_state().map(|s| s.network_id())
    }

    fn enqueued_block_timestamp(&self) -> Result<u64, UserOpError> {
        self.enqueued_state().and_then(|s| s.block_timestamp())
    }

    fn origin_chain_is_in_sync(&self, latest_block_infos: &LatestBlockInfos) -> bool {
        info!("checking if user op is cancellable w/r/t origin chain sync status...");
        // NOTE: To account for block timestamps not being entirely reliable & help during race conditions
        // between the cancellation thread and related syncer threads
        const LEEWAY: u64 = 90; // NOTE: 1m 30s
                                //
        let origin_network_id = self.op().origin_network_id();
        let enqueued_timestamp = match self.enqueued_block_timestamp() {
            Ok(t) => t,
            _ => {
                warn!("cannot cancel user op due to no enqueue timestamp ");
                return false;
            },
        };

        let origin_chain_timestamp = match latest_block_infos.get_for(origin_network_id) {
            Ok(info) => *info.block_timestamp(),
            _ => {
                warn!("cannot cancel user op due to no chain data for its origin network: {origin_network_id}");
                return false;
            },
        };

        debug!("          leeway: {LEEWAY}");
        debug!("     origin time: {origin_chain_timestamp}");
        debug!("   enqueued time: {enqueued_timestamp}");

        let origin_chain_is_beyond_enqueued_time =
            origin_chain_timestamp > LEEWAY && origin_chain_timestamp - LEEWAY >= enqueued_timestamp;

        if origin_chain_is_beyond_enqueued_time {
            info!("origin chain is beyond enqueued time and we've not seen a user send so op is cancellable");
        };

        origin_chain_is_beyond_enqueued_time
    }
}

impl CancellableUserOps {
    pub fn get<D: DatabaseInterface>(
        db_utils: &SentinelDbUtils<D>,
        latest_block_infos: LatestBlockInfos,
    ) -> Result<Self, SentinelError> {
        const NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY: usize = 20; // TODO make configurable?
        let list = UserOpList::get(db_utils);

        if list.is_empty() {
            return Ok(Self::empty());
        };

        Ok(
            Self::from(list.get_up_to_last_x_ops(db_utils, NUM_PAST_OPS_TO_CHECK_FOR_CANCELLABILITY)?)
                .iter()
                .filter(|cancellable_op| cancellable_op.origin_chain_is_in_sync(&latest_block_infos))
                .cloned()
                .collect::<Vec<CancellableUserOp>>()
                .into(),
        )
    }

    fn empty() -> Self {
        Self::default()
    }

    fn get_enqueued_but_neither_witnessed_nor_cancelled_nor_executed_ops(ops: &UserOps) -> UserOps {
        UserOps::new(
            ops.iter()
                .filter(|op| {
                    op.has_been_enqueued()
                        && op.has_not_been_witnessed()
                        && op.has_not_been_cancelled()
                        && op.has_not_been_executed()
                })
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

impl From<Vec<CancellableUserOps>> for CancellableUserOps {
    fn from(v: Vec<CancellableUserOps>) -> Self {
        CancellableUserOps::new(v.into_iter().map(|x| x[0].clone()).collect())
    }
}

impl From<UserOps> for CancellableUserOps {
    fn from(ops: UserOps) -> Self {
        Self::get_enqueued_but_neither_witnessed_nor_cancelled_nor_executed_ops(&ops)
            .iter()
            .map(CancellableUserOps::from)
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<&UserOp> for CancellableUserOps {
    fn from(op: &UserOp) -> Self {
        let mut cancellable_ops = Self::empty();

        // NOTE: A single user op can result in > 1 cancellable user ops, since it could have been
        // spuriously enqueued on multiple chains.
        let enqueued_states = match op.get_enqueued_states() {
            Ok(states) => states,
            _ => {
                debug!("user op has not been enqueued anywwhere so is not cancellable");
                return cancellable_ops;
            },
        };

        let enqueued_network_ids = enqueued_states.iter().map(|s| s.network_id()).collect::<Vec<_>>();

        // NOTE: A user op with the same uid _cannot_ be enqueued > 1 on the same chain, regardless
        // of the outcome of said queue operation, this is enforced by the hub smart contract.
        let mut cancelled_states = vec![UserOpStates::empty(); enqueued_network_ids.len()];
        for i in 0..enqueued_network_ids.len() {
            cancelled_states[i] = op.cancelled_states_for_network(&enqueued_network_ids[i]);
        }

        for i in 0..cancelled_states.len() {
            if cancelled_states[i].has_no_cancellation_by_sentinel() {
                cancellable_ops.push(CancellableUserOp::new(op.clone(), enqueued_states[i]));
            }
        }

        cancellable_ops
    }
}

impl UserOp {
    fn cancelled_states_for_network(&self, network_id: &NetworkId) -> UserOpStates {
        let mut r = vec![];
        if self.state.is_cancelled() && self.state().network_id() == *network_id {
            r.push(self.state);
        };
        for state in self.previous_states() {
            if state.is_cancelled() && state.network_id() == *network_id {
                r.push(self.state);
            }
        }
        UserOpStates::new(r)
    }

    fn has_been_cancelled(&self) -> bool {
        todo!("this");
    }

    fn has_not_been_cancelled(&self) -> bool {
        !self.has_been_cancelled()
    }

    fn get_enqueued_states(&self) -> Result<UserOpStates, UserOpError> {
        let e = UserOpError::HasNotBeenEnqueued;

        let mut enqueued_states: Vec<UserOpState> = vec![];

        if self.has_not_been_enqueued() {
            return Err(e);
        };

        if self.state.is_enqueued() {
            enqueued_states.push(self.state);
        };

        for state in self.previous_states.iter() {
            if state.is_enqueued() {
                enqueued_states.push(*state)
            }
        }

        debug!("enqueued states before deduplicating: {enqueued_states:?}");
        enqueued_states.sort();
        enqueued_states.dedup();
        debug!("enqueued states after deduplicating: {enqueued_states:?}");

        Ok(UserOpStates::new(enqueued_states))
    }

    fn has_been_enqueued(&self) -> bool {
        self.state.is_enqueued() || self.previous_states.iter().any(|state| state.is_enqueued())
    }

    fn has_not_been_enqueued(&self) -> bool {
        !self.has_been_enqueued()
    }

    fn has_been_witnessed(&self) -> bool {
        self.state.is_witnessed() || self.previous_states.iter().any(|state| state.is_witnessed())
    }

    fn has_not_been_witnessed(&self) -> bool {
        !self.has_been_witnessed()
    }
}

impl UserOpStates {
    fn has_cancellation_by_sentinel(&self) -> bool {
        self.iter().any(|s| match s {
            UserOpState::Cancelled(_, actor) => actor.is_sentinel(),
            _ => false,
        })
    }

    fn has_no_cancellation_by_sentinel(&self) -> bool {
        !self.has_cancellation_by_sentinel()
    }
}
