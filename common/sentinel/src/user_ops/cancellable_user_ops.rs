use common::DatabaseInterface;
use common_chain_ids::EthChainId;
use derive_getters::Getters;
use derive_more::{Constructor, Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::{UserOp, UserOpError, UserOpList, UserOpState, UserOpStates, UserOps, USER_OP_CANCEL_TX_GAS_LIMIT};
use crate::{
    LatestBlockInfos,
    NetworkId,
    SentinelDbUtils,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};

#[derive(Clone, Debug, Eq, Default, PartialEq, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct CancellableUserOps(Vec<CancellableUserOp>);

impl TryFrom<WebSocketMessagesEncodable> for CancellableUserOps {
    type Error = SentinelError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        match m {
            WebSocketMessagesEncodable::Success(json) => Ok(serde_json::from_value(json)?),
            other => Err(WebSocketMessagesError::CannotConvert {
                from: format!("{other}"),
                to: "UserOps".to_string(),
            }
            .into()),
        }
    }
}

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

        debug!(
            "             uid: 0x{}",
            hex::encode(self.op().uid().unwrap_or_default().as_bytes())
        );
        debug!("          leeway: {LEEWAY}");
        debug!("     origin time: {origin_chain_timestamp}");
        debug!("   enqueued time: {enqueued_timestamp}");

        let origin_chain_is_beyond_enqueued_time =
            origin_chain_timestamp > LEEWAY && origin_chain_timestamp - LEEWAY >= enqueued_timestamp;

        if origin_chain_is_beyond_enqueued_time {
            info!("origin chain is beyond enqueued time and we've not seen a user send so op is cancellable");
        } else {
            info!("origin chain is not beyond the enqueud time so we cannot determine if it is cancellable");
        }

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

    fn get_enqueued_but_not_witnessed_nor_executed(ops: &UserOps) -> UserOps {
        UserOps::new(
            ops.iter()
                .filter(|op| op.has_been_enqueued() && op.has_not_been_witnessed() && op.has_not_been_executed())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

impl From<Vec<CancellableUserOps>> for CancellableUserOps {
    fn from(vec_of_vec_of_cancellable_ops: Vec<CancellableUserOps>) -> Self {
        let mut r = vec![];
        for vec_of_cancellable_ops in vec_of_vec_of_cancellable_ops.iter() {
            for cancellable_op in vec_of_cancellable_ops.iter() {
                r.push(cancellable_op.clone());
            }
        }
        Self::new(r)
    }
}

impl From<UserOps> for CancellableUserOps {
    fn from(ops: UserOps) -> Self {
        Self::get_enqueued_but_not_witnessed_nor_executed(&ops)
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
        let enqueued_states = op.get_enqueued_states();
        let enqueued_network_ids = enqueued_states.iter().map(|s| s.network_id()).collect::<Vec<_>>();

        // NOTE: A user op with the same uid _cannot_ be enqueued > 1 on the same chain, regardless
        // of the outcome of said queue operation, this is enforced by the hub smart contract.
        let mut cancelled_states = vec![UserOpStates::empty(); enqueued_network_ids.len()];
        for i in 0..enqueued_network_ids.len() {
            cancelled_states[i] = op.cancelled_states_for_network(&enqueued_network_ids[i]);
        }

        for i in 0..cancelled_states.len() {
            if cancelled_states[i].has_no_cancellation_by_sentinel(&enqueued_network_ids[i]) {
                cancellable_ops.push(CancellableUserOp::new(op.clone(), enqueued_states[i]));
            }
        }

        cancellable_ops
    }
}

impl UserOp {
    fn cancelled_states_for_network(&self, network_id: &NetworkId) -> UserOpStates {
        debug!("getting cancelled states for network: {network_id}");

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

    fn get_enqueued_states(&self) -> UserOpStates {
        info!("getting all enqueued states of user op");
        let mut enqueued_states: Vec<UserOpState> = vec![];

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

        UserOpStates::new(enqueued_states)
    }

    fn has_been_enqueued(&self) -> bool {
        self.state.is_enqueued() || self.previous_states.iter().any(|state| state.is_enqueued())
    }

    fn has_been_witnessed(&self) -> bool {
        self.state.is_witnessed() || self.previous_states.iter().any(|state| state.is_witnessed())
    }

    fn has_not_been_witnessed(&self) -> bool {
        !self.has_been_witnessed()
    }
}

impl UserOpStates {
    fn has_cancellation_by_sentinel(&self, network_id: &NetworkId) -> bool {
        let r = self.iter().any(|s| match s {
            UserOpState::Cancelled(state, actor) => state.network_id() == network_id && actor.is_sentinel(),
            _ => false,
        });
        debug!("has cancellation by sentinel on network {network_id}: {r}");
        r
    }

    fn has_no_cancellation_by_sentinel(&self, network_id: &NetworkId) -> bool {
        !self.has_cancellation_by_sentinel(network_id)
    }
}

#[cfg(test)]
mod tests {
    use common::get_test_database;
    use ethereum_types::H256 as EthHash;

    use super::{UserOp, UserOpList, *};
    use crate::{get_utc_timestamp, ActorType, LatestBlockInfo, SentinelDbUtils};

    #[test]
    fn should_get_cancellable_user_ops() {
        let db = get_test_database();
        let db_utils = SentinelDbUtils::new(&db);
        let mut op = UserOp::default();

        // NOTE: We're going to manually create an op that would be a candidate for cancellation,
        // by changing it's state to `Enqueued`. There will be no previous states, therefore no
        // `Witnessed` state, making it elegible for cancellation based on state. We'll need to
        // make more edits to make it cancellable temporally, however, so we'll do that later.
        let origin_network_id = NetworkId::try_from("bsc").unwrap();
        let enqueued_tx_hash = EthHash::random();
        let enqueued_timestamp = get_utc_timestamp().unwrap();
        let enqueued_network_id = NetworkId::try_from("eth").unwrap();
        op.origin_network_id = origin_network_id;
        op.state = UserOpState::enqueued(enqueued_network_id, enqueued_tx_hash, enqueued_timestamp);
        let mut list = UserOpList::default();
        list.process_op(op.clone(), &db_utils).unwrap();
        let latest_block_infos = LatestBlockInfos::default();
        let r1 = CancellableUserOps::get(&db_utils, latest_block_infos).unwrap();
        assert!(r1.is_empty());

        // NOTE: Now, if we add some latest block info for the origin chain, but make it appear out
        // of sync, we should still get no cancellable ops;
        let mut bsc_latest_block_info = LatestBlockInfo::default();
        bsc_latest_block_info.network_id = origin_network_id;
        bsc_latest_block_info.block_timestamp =
            enqueued_timestamp - (60 * 60/* NOTE: an hour _behind_ the enqueued time */);
        let mut bsc_latest_block_infos = LatestBlockInfos::new(vec![bsc_latest_block_info.clone()]);
        let r2 = CancellableUserOps::get(&db_utils, bsc_latest_block_infos).unwrap();
        assert!(r2.is_empty());

        //NOTE Now let's set the origin chain to be in sync w/r/t the user op, meaning
        //the op becomes cancellable.
        bsc_latest_block_info.block_timestamp =
            enqueued_timestamp + (60 * 60/* NOTE: an hour _beyond_ the enqueued time */);
        bsc_latest_block_infos = LatestBlockInfos::new(vec![bsc_latest_block_info]);
        let r3 = CancellableUserOps::get(&db_utils, bsc_latest_block_infos.clone()).unwrap();
        assert!(!r3.is_empty());

        // NOTE: Let's assert its the expected op
        let expected_cancellable_op = CancellableUserOp::new(op.clone(), *op.state());
        assert_eq!(r3[0], expected_cancellable_op);

        // NOTE: Now lets add a cancellation state from a sentinel, but for a _different_ chain to
        // the where the op is enqueued. This should result in returning the same cancellable op as
        // before.
        let cancelled_tx_hash = EthHash::random();
        let cancelled_timestamp = get_utc_timestamp().unwrap();
        let cancelled_network_id = NetworkId::try_from("polygon").unwrap();
        let wrong_chain_cancelled_state =
            UserOpState::cancelled(cancelled_network_id, cancelled_tx_hash, cancelled_timestamp);
        assert!(matches!(
            wrong_chain_cancelled_state.actor_type(),
            Some(ActorType::Sentinel)
        ));
        op.state = wrong_chain_cancelled_state;
        list.process_op(op.clone(), &db_utils).unwrap(); //NOTE: Update the op in the db
        let r4 = CancellableUserOps::get(&db_utils, bsc_latest_block_infos.clone()).unwrap();
        assert!(!r4.is_empty());
        assert_eq!(r4[0], expected_cancellable_op);

        // NOTE: Now let's add a cancellation from a sentinel on the enqueued chain. This should
        // result in no cancellable user ops being returned.
        let enqueued_chain_cancelled_state =
            UserOpState::cancelled(enqueued_network_id, cancelled_tx_hash, cancelled_timestamp);
        op.state = enqueued_chain_cancelled_state;
        list.process_op(op.clone(), &db_utils).unwrap(); // NOTE Update the op in db
        let r5 = CancellableUserOps::get(&db_utils, bsc_latest_block_infos.clone()).unwrap();
        assert!(r5.is_empty());
    }

    #[test]
    fn should_get_multiple_cancellabe_ops_for_single_user_op_queued_on_multiple_chains() {
        let db = get_test_database();
        let db_utils = SentinelDbUtils::new(&db);
        let mut op = UserOp::default();
        let uid = op.uid().unwrap();
        let eth_network_id = NetworkId::try_from("eth").unwrap();
        let polygon_network_id = NetworkId::try_from("polygon").unwrap();
        let origin_network_id = NetworkId::try_from("bsc").unwrap();
        let enqueued_timestamp = get_utc_timestamp().unwrap();
        op.origin_network_id = origin_network_id;
        op.state = UserOpState::enqueued(eth_network_id, EthHash::random(), enqueued_timestamp);
        let mut list = UserOpList::default();
        list.process_op(op.clone(), &db_utils).unwrap();
        op.state = UserOpState::enqueued(polygon_network_id, EthHash::random(), enqueued_timestamp);
        list.process_op(op.clone(), &db_utils).unwrap();
        let latest_block_infos = LatestBlockInfos::default();
        let r1 = CancellableUserOps::get(&db_utils, latest_block_infos).unwrap();
        assert!(r1.is_empty());
        let mut bsc_latest_block_info = LatestBlockInfo::default();
        bsc_latest_block_info.network_id = origin_network_id;
        bsc_latest_block_info.block_timestamp =
            enqueued_timestamp + (60 * 60/* NOTE: an hour _beyond_ the enqueued time */);
        let bsc_latest_block_infos = LatestBlockInfos::new(vec![bsc_latest_block_info.clone()]);
        let r2 = CancellableUserOps::get(&db_utils, bsc_latest_block_infos).unwrap();
        assert_eq!(r2.len(), 2);
        assert_eq!(r2[0].op().uid().unwrap(), uid);
        assert_eq!(r2[1].op().uid().unwrap(), uid);
        let cancellation_network_ids = r2
            .iter()
            .map(|x| x.state().network_id().clone())
            .collect::<Vec<NetworkId>>();
        assert!(cancellation_network_ids.contains(&eth_network_id));
        assert!(cancellation_network_ids.contains(&polygon_network_id));
    }
}
