use std::str::FromStr;

use common_eth::EthPrivateKey;
use common_sentinel::{
    get_registration_extension_tx,
    EthRpcMessages,
    EthRpcSenders,
    NetworkId,
    SentinelConfig,
    SentinelError,
};
use ethereum_types::Address as EthAddress;
use serde_json::{json, Value as Json};

use crate::rpc_server::{RpcCalls, RpcParams};

// TODO Take whatever other params are required to maybe broadcast this signature too?

// NOTE: The registration extension tx is handled soley by the owner of the sentinel and does not
// require and input from the sentinel's TEE protected signing key.

impl RpcCalls {
    pub(crate) async fn handle_get_registration_extension_tx(
        _config: SentinelConfig,
        params: RpcParams,
        pk: EthPrivateKey,
        eth_rpc_senders: EthRpcSenders,
    ) -> Result<Json, SentinelError> {
        const REQUIRED_NUM_PARAMS: usize = 3;

        // NOTE: If another arg is passed in let's use it as a gas price.
        let maybe_gas_price = if params.len() > REQUIRED_NUM_PARAMS {
            let p = params[REQUIRED_NUM_PARAMS + 1].clone().parse::<u64>()?;
            Some(p)
        } else {
            None
        };

        let checked_params = Self::check_params(params, REQUIRED_NUM_PARAMS)?;

        let duration = checked_params[0].parse::<u64>()?; // TODO sanity check?
        let network_id = NetworkId::from_str(&checked_params[1])?;

        let registration_manager = EthAddress::from_str(&checked_params[2])?;

        let sender = eth_rpc_senders.sender(&network_id)?;

        let address = pk.to_address();
        let (nonce_msg, nonce_rx) = EthRpcMessages::get_nonce_msg(network_id, address);
        sender.send(nonce_msg).await?;
        let nonce = nonce_rx.await??;

        let gas_price = if let Some(p) = maybe_gas_price {
            p
        } else {
            let (gas_price_msg, gas_price_rx) = EthRpcMessages::get_gas_price_msg(network_id);
            sender.send(gas_price_msg).await?;
            gas_price_rx.await??
        };

        let signed_tx =
            get_registration_extension_tx(nonce, duration, gas_price, network_id, &pk, registration_manager)?;

        let json = json!({
            "nonce": nonce,
            "duration": duration,
            "gas_price": gas_price,
            "signer": format!("0x{}", hex::encode(address)),
            "registration_manager": format!("0x{}", hex::encode(registration_manager)),
            "tx": format!("0x{}", signed_tx.serialize_hex()),
        });

        Ok(json)
    }
}
