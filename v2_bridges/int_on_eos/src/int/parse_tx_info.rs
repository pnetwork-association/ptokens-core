use std::str::FromStr;

use common::{dictionaries::eos_eth::EosEthTokenDictionary, traits::DatabaseInterface, types::Result};
use common_chain_ids::EthChainId;
use common_eth::{
    Erc20VaultPegInEventParams,
    EthDbUtils,
    EthDbUtilsExt,
    EthLog,
    EthLogExt,
    EthLogs,
    EthReceipt,
    EthState,
    EthSubmissionMaterial,
    ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2,
};
use common_metadata::MetadataChainId;
use common_safe_addresses::safely_convert_str_to_eos_address;
use ethereum_types::Address as EthAddress;

use crate::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos};

impl IntOnEosEosTxInfos {
    fn is_log_erc20_vault_peg_in(log: &EthLog) -> bool {
        log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2)
    }

    pub fn is_token_supported(log: &EthLog, dictionary: &EosEthTokenDictionary) -> Result<bool> {
        Erc20VaultPegInEventParams::from_eth_log(log).map(|params| dictionary.is_token_supported(&params.token_address))
    }

    pub fn is_log_relevant(log: &EthLog, dictionary: &EosEthTokenDictionary) -> Result<bool> {
        let log_is_erc20_vault_peg_in = Self::is_log_erc20_vault_peg_in(log);
        let token_is_supported = Self::is_token_supported(log, dictionary)?;
        Ok(log_is_erc20_vault_peg_in && token_is_supported)
    }

    pub fn get_relevant_logs_from_receipt(receipt: &EthReceipt, dictionary: &EosEthTokenDictionary) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_relevant(log, dictionary), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
        router_address: &EthAddress,
        vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnEosEosTxInfos` from receipts...");
        Ok(Self::new(
            Self::get_relevant_logs_from_receipt(receipt, dictionary)
                .iter()
                .map(|log| {
                    let params = Erc20VaultPegInEventParams::from_eth_log(log)?;
                    let tx_info = IntOnEosEosTxInfo {
                        vault_address: *vault_address,
                        token_sender: params.token_sender,
                        originating_tx_hash: receipt.transaction_hash,
                        router_address: format!("0x{}", hex::encode(router_address)),
                        destination_address: safely_convert_str_to_eos_address(&params.destination_address).to_string(),
                        eos_token_address: dictionary
                            .get_eos_account_name_from_eth_token_address(&params.token_address)?,
                        eos_asset_amount: dictionary
                            .convert_u256_to_eos_asset_string(&params.token_address, &params.token_amount)?,
                        eth_token_address: params.token_address,
                        token_amount: params.token_amount,
                        user_data: params.user_data.clone(),
                        destination_chain_id: params.get_destination_chain_id()?,
                        origin_chain_id: MetadataChainId::from_str(&origin_chain_id.to_string())?,
                    };
                    info!("✔ Parsed `IntOnEosEosTxInfo`: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<IntOnEosEosTxInfo>>>()?,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        eos_eth_token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
        router_address: &EthAddress,
        vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnEosEosTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| {
                    Self::from_eth_receipt(
                        receipt,
                        eos_eth_token_dictionary,
                        origin_chain_id,
                        router_address,
                        vault_address,
                    )
                })
                .collect::<Result<Vec<IntOnEosEosTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<IntOnEosEosTxInfo>>>()
                .concat(),
        ))
    }
}

pub fn maybe_parse_eos_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `INT-on-EOS` EOS tx infos from canon block...");
    let eth_db_utils = EthDbUtils::new(state.db);
    eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|submission_material| {
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in canon block ∴ no tx info to parse!");
                Ok(state)
            } else {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                EosEthTokenDictionary::get_from_db(state.db)
                    .and_then(|dictionary| {
                        IntOnEosEosTxInfos::from_submission_material(
                            &submission_material,
                            &dictionary,
                            &eth_db_utils.get_eth_chain_id_from_db()?,
                            &eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                            &eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
                        )
                    })
                    .and_then(|infos| infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
            }
        })
}
