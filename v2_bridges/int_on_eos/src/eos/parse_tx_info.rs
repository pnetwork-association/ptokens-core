use std::str::FromStr;

use common::{
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::convert_bytes_to_u64,
};
use common_chain_ids::EosChainId;
use common_eos::{EosActionProof, EosState};
use common_eth::{EthDbUtils, EthDbUtilsExt};
use common_metadata::MetadataChainId;
use ethereum_types::{Address as EthAddress, U256};

use crate::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos};

impl IntOnEosIntTxInfos {
    pub fn from_action_proofs(
        action_proofs: &[EosActionProof],
        dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EosChainId,
        int_vault_address: &EthAddress,
        router_address: &EthAddress,
    ) -> Result<Self> {
        Ok(Self::new(
            action_proofs
                .iter()
                .map(|action_proof| {
                    IntOnEosIntTxInfo::from_action_proof(
                        action_proof,
                        dictionary,
                        origin_chain_id,
                        int_vault_address,
                        router_address,
                    )
                })
                .collect::<Result<Vec<IntOnEosIntTxInfo>>>()?,
        ))
    }
}

impl IntOnEosIntTxInfo {
    // NOTE: We should be decoding this from the actual hex blob in the proof, but since we don't
    // yet have an EOS codec, we can't deserialize it. Instead we rely on the json.
    fn get_destination_address_from_proof(proof: &EosActionProof) -> String {
        proof.action_json.data.memo.clone().unwrap_or_default()
    }

    // NOTE: Ibid.
    fn get_destination_chain_id_from_proof(proof: &EosActionProof) -> Result<MetadataChainId> {
        Ok(MetadataChainId::from_bytes(&hex::decode(
            proof.action_json.data.chain_id.clone().unwrap_or_default(),
        )?)
        .unwrap_or_default())
    }

    // NOTE: Ibid.
    fn get_user_data_from_proof(proof: &EosActionProof) -> Result<Bytes> {
        Ok(hex::decode(
            proof.action_json.data.user_data.clone().unwrap_or_default(),
        )?)
    }

    fn get_redeem_amount_from_proof(
        proof: &EosActionProof,
        dictionary_entry: &EosEthTokenDictionaryEntry,
    ) -> Result<U256> {
        proof
            .check_proof_action_data_length(15, "Not enough data to parse `IntOnEosIntTxInfo` amount from proof!")
            .and_then(|_| {
                Ok(dictionary_entry.convert_u64_to_eos_asset(convert_bytes_to_u64(&proof.action.data[8..=15])?))
            })
            .and_then(|eos_asset| dictionary_entry.convert_eos_asset_to_eth_amount(&eos_asset))
    }

    fn get_destination_chain_id_from_v1_action(
        origin_chain_id: &EosChainId,
        eos_address: &str,
    ) -> Result<MetadataChainId> {
        // NOTE: For some tokens we watch for legacy `redeem` actions as well as `redeem2` actions.
        // If we encounter the former, we need to be aware of it in order to determine its
        // destination chain ID, since the action itself does not contain that info.
        debug!("maybe getting destination chain id for v1 action...");
        match origin_chain_id {
            EosChainId::TelosMainnet => {
                let (token, cid): (&str, MetadataChainId) = match eos_address {
                    "btc.ptokens" => ("pBTC", MetadataChainId::BitcoinMainnet),
                    "eth.ptokens" => ("pWETH", MetadataChainId::EthereumMainnet),
                    "usdt.ptokens" => ("pUSDT", MetadataChainId::EthereumMainnet),
                    "usdc.ptokens" => ("pUSDC", MetadataChainId::EthereumMainnet),
                    _ => {
                        return Err(
                            format!("have no case for TELOS v1 redeem action from address: {eos_address}").into(),
                        )
                    },
                };
                warn!("{token} on TELOS v1 redeem action detected, using {cid} as destination chain ID");
                Ok(cid)
            },
            EosChainId::UltraMainnet => {
                // NOTE: We only ever supported ULTRA on ETH mainnet.
                warn!("ULTRA v1 redeem detected, defaulting to ETH mainnet as destination chain ID");
                Ok(MetadataChainId::EthereumMainnet)
            },
            EosChainId::EosMainnet => {
                let (token, cid): (&str, MetadataChainId) = match eos_address {
                    "wmbt.ptokens" => ("pWOMBAT", MetadataChainId::EthereumMainnet),
                    "btc.ptokens" => ("pBTC", MetadataChainId::BitcoinMainnet),
                    "eth.ptokens" => ("PETH", MetadataChainId::EthereumMainnet),
                    _ => return Err(format!("cannot handle EOS v1 redeem action from address: {eos_address}").into()),
                };
                warn!("{token} on EOS v1 redeem action detected, using {cid} as destination chain ID");
                Ok(cid)
            },
            _ => Err(format!("have no handling for v1 redeem actions from origin chain: {origin_chain_id}").into()),
        }
    }

    pub fn from_action_proof(
        proof: &EosActionProof,
        dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EosChainId,
        int_vault_address: &EthAddress,
        router_address: &EthAddress,
    ) -> Result<Self> {
        dictionary
            .get_entry_via_eos_address_and_symbol(
                &proof.get_eos_asset_symbol()?,
                &proof.get_action_eos_account().to_string(),
            )
            .and_then(|dictionary_entry| {
                let eos_address = dictionary_entry.eos_address.clone();
                let amount = Self::get_redeem_amount_from_proof(proof, &dictionary_entry)?;
                let eos_tx_amount = dictionary_entry.convert_u256_to_eos_asset_string(&amount)?;

                let destination_chain_id = if proof.is_v2_redeem() {
                    Self::get_destination_chain_id_from_proof(proof)
                } else {
                    Self::get_destination_chain_id_from_v1_action(origin_chain_id, &eos_address)
                }?;

                info!("✔ Converting action proof to `erc20-on-eos` redeem info...");
                Ok(Self {
                    amount,
                    eos_tx_amount,
                    destination_chain_id,
                    eos_token_address: eos_address,
                    originating_tx_id: proof.tx_id,
                    router_address: *router_address,
                    int_vault_address: *int_vault_address,
                    origin_address: proof.get_action_sender()?,
                    int_token_address: dictionary_entry.eth_address,
                    user_data: Self::get_user_data_from_proof(proof)?,
                    global_sequence: proof.action_receipt.global_sequence,
                    destination_address: Self::get_destination_address_from_proof(proof),
                    origin_chain_id: MetadataChainId::from_str(&origin_chain_id.to_string())?,
                })
            })
    }
}

pub fn maybe_parse_int_tx_infos_and_put_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Parsing `IntOnEosIntTxInfos` from actions data...");
    let eth_db_utils = EthDbUtils::new(state.db);
    IntOnEosIntTxInfos::from_action_proofs(
        &state.action_proofs,
        state.get_eos_eth_token_dictionary()?,
        &state.eos_db_utils.get_eos_chain_id_from_db()?,
        &eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
        &eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
    )
    .and_then(|infos| {
        let global_sequences = infos.get_global_sequences();
        info!("✔ Parsed {} `IntOnEosIntTxInfos`!", infos.len());
        let bytes = infos.to_bytes()?;
        Ok(state.add_tx_infos(bytes).add_global_sequences(global_sequences))
    })
}
