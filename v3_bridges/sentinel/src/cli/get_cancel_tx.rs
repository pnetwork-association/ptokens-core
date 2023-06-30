use common_eth::{convert_hex_to_h256, EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use common_rocksdb_database::get_db_at_path;
use ethereum_types::H256 as EthHash;
use lib::{
    check_init,
    get_gas_price,
    get_nonce,
    get_user_op_state,
    push_tx,
    ConfigT,
    DbUtilsT,
    SentinelConfig,
    SentinelDbUtils,
    SentinelError,
    UserOp,
    UserOpSmartContractState,
};
use serde_json::json;

#[derive(Clone, Debug, Default, Args)]
pub struct CancelTxArgs {
    /// User op identifaction hash
    uid: String,

    /// Nonce to use. If omitted, will call the endpoint to determine the nonce
    #[arg(long, short)]
    nonce: Option<u64>,

    /// Gas price to use. If omitted it will use the gas price suggested by the RPC
    #[arg(long)]
    gas_price: Option<u64>,

    /// Gas limit to use. If omitted it will use the default amount instead
    #[arg(long)]
    gas_limit: Option<usize>,

    /// Push the tx to the endpoint
    #[arg(long, short)]
    push: bool,
}

const SLEEP_TIME: u64 = 2000;

pub async fn get_cancel_tx(config: &SentinelConfig, args: &CancelTxArgs) -> Result<String, SentinelError> {
    let db = get_db_at_path(&config.get_db_path())?;
    let db_utils = SentinelDbUtils::new(&db);
    check_init(&db)?;

    let uid = convert_hex_to_h256(&args.uid)?;
    match UserOp::get_from_db(&db_utils, &uid.into()) {
        Err(e) => {
            warn!("{e}");
            Err(SentinelError::Custom(format!("no user op in db with uid {uid}")))
        },
        Ok(op) => {
            if !op.is_enqueued() {
                Err(SentinelError::Custom(
                    "user op is not enqueued, cannot cancel it".into(),
                ))
            } else {
                let side = op.destination_side();

                let h_db_utils = HostDbUtils::new(&db);
                let n_db_utils = NativeDbUtils::new(&db);

                let ws_client = if side.is_native() {
                    config.native().endpoints().get_first_ws_client().await?
                } else {
                    config.host().endpoints().get_first_ws_client().await?
                };

                let state_manager = if side.is_native() {
                    config.native().state_manager()
                } else {
                    config.host().state_manager()
                };

                let user_op_state = get_user_op_state(&op, &state_manager, &ws_client, SLEEP_TIME, side).await?;

                if user_op_state != UserOpSmartContractState::Enqueued {
                    return Err(SentinelError::Custom(format!(
                        "cannot cancel user op, user op state is {user_op_state}"
                    )));
                }

                debug!("user op state: {user_op_state}");

                let address = if side.is_native() {
                    n_db_utils.get_public_eth_address_from_db()?
                } else {
                    h_db_utils.get_public_eth_address_from_db()?
                };

                let nonce = if let Some(n) = args.nonce {
                    debug!("using passed in nonce {n}");
                    n
                } else {
                    let n = get_nonce(&ws_client, &address, SLEEP_TIME, side).await?;
                    debug!("using nonce from RPC {n}");
                    n
                };

                let gas_price = if let Some(p) = args.gas_price {
                    debug!("using passed in gas price {p}");
                    p
                } else {
                    let p = get_gas_price(&ws_client, SLEEP_TIME, side).await?;
                    debug!("using gas price from RPC: {p}");
                    p
                };

                let state_manager = if side.is_native() {
                    config.native().state_manager()
                } else {
                    config.host().state_manager()
                };

                let (chain_id, pk) = if side.is_native() {
                    (
                        n_db_utils.get_eth_chain_id_from_db()?,
                        n_db_utils.get_eth_private_key_from_db()?,
                    )
                } else {
                    (
                        h_db_utils.get_eth_chain_id_from_db()?,
                        h_db_utils.get_eth_private_key_from_db()?,
                    )
                };

                let gas_limit = if let Some(l) = args.gas_limit {
                    debug!("using passed in gas limit {l}");
                    l
                } else {
                    let l = UserOp::cancellation_gas_limit(&chain_id);
                    debug!("using gas limit {l}");
                    l
                };

                let tx = op.cancel(nonce, gas_price, &state_manager, gas_limit, &pk, &chain_id)?;
                let tx_hex = tx.serialize_hex();

                debug!("signed tx: 0x{tx_hex}");

                let tx_hash = if args.push {
                    info!("pushing tx to endpoint");
                    push_tx(&tx, &ws_client, SLEEP_TIME, side).await?
                } else {
                    info!("not pushing tx to endpoint");
                    EthHash::zero()
                };

                let r = json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "nonce": nonce,
                        "uid": args.uid,
                        "pushed": args.push,
                        "gas_price": gas_price,
                        "gas_limit": gas_limit,
                        "destination_side": side,
                        "signed_tx": format!("0x{tx_hex}"),
                        "tx_hash": format!("0x{}", hex::encode(tx_hash.as_bytes())),
                    }
                })
                .to_string();

                Ok(r)
            }
        },
    }
}
