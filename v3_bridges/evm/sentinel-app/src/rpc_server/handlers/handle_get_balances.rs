use std::str::FromStr;

use common_sentinel::{EthRpcMessages, EthRpcSenders, NetworkId, SentinelConfig, SentinelError};
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::rpc_server::{RpcCalls, RpcParams};

#[derive(Clone, Debug, Serialize, Deserialize, Deref, DerefMut, Constructor)]
struct Balances(Vec<Balance>);

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
#[serde(rename_all = "camelCase")]
struct Balance {
    balance: U256,
    network_id: NetworkId,
}

impl RpcCalls {
    pub(crate) async fn handle_get_balances(
        config: SentinelConfig,
        params: RpcParams,
        eth_rpc_senders: EthRpcSenders,
    ) -> Result<Json, SentinelError> {
        debug!("handling get balances...");

        let checked_params = Self::check_params(params, 1)?;
        let address = EthAddress::from_str(&checked_params[0])?;
        let network_ids = config.network_ids();

        let mut balances: Balances = Balances::new(vec![]);

        for id in network_ids {
            let sender = eth_rpc_senders.sender(&id)?;
            let (msg, rx) = EthRpcMessages::get_eth_balance_msg(id, address);
            sender.send(msg).await?;
            let b = rx.await??;
            let balance = Balance::new(b, id);
            balances.push(balance)
        }

        Ok(json!(balances))
    }
}
