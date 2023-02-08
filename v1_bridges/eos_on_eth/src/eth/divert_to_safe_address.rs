use common::{chains::eth::EthState, safe_addresses::SAFE_EOS_ADDRESS_STR, traits::DatabaseInterface, types::Result};

use crate::eth::eos_tx_info::{EosOnEthEosTxInfo, EosOnEthEosTxInfos};

create_safe_address_diversion_fxns_v2!(
    "EosOnEthEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String =>"token"
);
