use ethereum_types::Address as EthAddress;

use crate::{
    chains::eos::eos_state::EosState,
    constants::SAFE_ETH_ADDRESS,
    erc20_on_eos::eos::redeem_info::{Erc20OnEosRedeemInfo, Erc20OnEosRedeemInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnEosRedeemInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token", "vault"
);
