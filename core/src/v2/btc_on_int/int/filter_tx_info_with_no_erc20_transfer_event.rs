use crate::tx_infos::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos};

impl_to_erc20_token_event!(BtcOnIntBtcTxInfo, amount_in_wei, to, from, token_address);

make_erc20_token_event_filterer!(EthState<D>, eth_db_utils, BtcOnIntBtcTxInfos);
