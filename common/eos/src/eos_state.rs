use common::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::get_not_in_state_err,
};

pub use crate::bitcoin_crate_alias::blockdata::transaction::Transaction as BtcTransaction;
use crate::{
    eos_action_proofs::EosActionProofs,
    eos_block_header::EosBlockHeaderV2,
    eos_database_utils::EosDbUtils,
    eos_global_sequences::{GlobalSequences, ProcessedGlobalSequences},
    eos_incremerkle::Incremerkles,
    eos_producer_schedule::EosProducerScheduleV2,
    eos_submission_material::EosSubmissionMaterial,
    eos_types::Checksum256s,
    protocol_features::EnabledFeatures,
};

make_state_setters_and_getters!(
    EosState;
    "active_schedule" => EosProducerScheduleV2,
    "eos_eth_token_dictionary" => EosEthTokenDictionary
);

#[derive(Clone, Debug, PartialEq)]
pub struct EosState<'a, D: DatabaseInterface> {
    pub db: &'a D,
    pub tx_infos: Bytes,
    pub eth_signed_txs: Bytes,
    pub block_num: Option<u64>,
    #[cfg(not(feature = "spring_1-0"))]
    pub incremerkles: Incremerkles,
    pub producer_signature: String,
    pub btc_utxos_and_values: Bytes,
    global_sequences: GlobalSequences,
    pub action_proofs: EosActionProofs,
    pub interim_block_ids: Checksum256s,
    pub eos_db_utils: EosDbUtils<'a, D>,
    pub block_header: Option<EosBlockHeaderV2>,
    pub btc_on_eos_signed_txs: Vec<BtcTransaction>,
    pub processed_tx_ids: ProcessedGlobalSequences,
    pub enabled_protocol_features: EnabledFeatures,
    pub active_schedule: Option<EosProducerScheduleV2>,
    eos_eth_token_dictionary: Option<EosEthTokenDictionary>,
}

impl<'a, D: DatabaseInterface> EosState<'a, D> {
    pub fn init(db: &'a D) -> EosState<'a, D> {
        EosState {
            db,
            block_num: None,
            tx_infos: vec![],
            block_header: None,
            action_proofs: vec![],
            eth_signed_txs: vec![],
            active_schedule: None,
            interim_block_ids: vec![],
            btc_utxos_and_values: vec![],
            btc_on_eos_signed_txs: vec![],
            eos_eth_token_dictionary: None,
            eos_db_utils: EosDbUtils::new(db),
            producer_signature: String::new(),
            #[cfg(not(feature = "spring_1-0"))]
            incremerkles: Incremerkles::default(),
            global_sequences: GlobalSequences::default(),
            enabled_protocol_features: EnabledFeatures::init(),
            processed_tx_ids: ProcessedGlobalSequences::new(vec![]),
        }
    }

    pub fn add_btc_utxos_and_values(mut self, bytes: Bytes) -> Self {
        self.btc_utxos_and_values = bytes;
        self
    }

    pub fn get_eos_eth_token_dictionary_and_add_to_state(mut self) -> Result<Self> {
        info!("✔ Getting `EosERc20Dictionary` and adding to EOS state...");
        EosEthTokenDictionary::get_from_db(self.db).map(|dictionary| {
            self.eos_eth_token_dictionary = Some(dictionary);
            self
        })
    }

    pub fn add_global_sequences(mut self, global_sequences: GlobalSequences) -> Self {
        self.global_sequences = global_sequences;
        self
    }

    pub fn add_tx_infos(mut self, infos: Bytes) -> Self {
        info!("✔ Adding tx infos to state...");
        self.tx_infos = infos;
        self
    }

    pub fn add_btc_on_eos_signed_txs(mut self, btc_on_eos_signed_txs: Vec<BtcTransaction>) -> Result<EosState<'a, D>> {
        self.btc_on_eos_signed_txs = btc_on_eos_signed_txs;
        Ok(self)
    }

    pub fn add_eth_signed_txs(mut self, txs: Bytes) -> Self {
        self.eth_signed_txs = txs;
        self
    }

    #[cfg(not(feature = "spring_1-0"))]
    pub fn add_incremerkles(mut self, incremerkles: Incremerkles) -> EosState<'a, D> {
        self.incremerkles = incremerkles;
        self
    }

    pub fn add_submission_material(mut self, submission_material: EosSubmissionMaterial) -> Result<EosState<'a, D>> {
        self.block_num = Some(submission_material.block_num);
        self.action_proofs = submission_material.action_proofs;
        self.block_header = Some(submission_material.block_header);
        self.interim_block_ids = submission_material.interim_block_ids;
        self.producer_signature = submission_material.producer_signature;
        Ok(self)
    }

    pub fn add_processed_tx_ids(mut self, tx_ids: ProcessedGlobalSequences) -> Result<Self> {
        self.processed_tx_ids = tx_ids;
        Ok(self)
    }

    pub fn add_enabled_protocol_features(mut self, enabled_protocol_features: EnabledFeatures) -> Result<Self> {
        self.enabled_protocol_features = enabled_protocol_features;
        Ok(self)
    }

    pub fn get_eos_block_header(&self) -> Result<&EosBlockHeaderV2> {
        match self.block_header {
            Some(ref block_header) => Ok(block_header),
            None => Err(get_not_in_state_err("block_header").into()),
        }
    }

    pub fn get_eos_block_num(&self) -> Result<u64> {
        match self.block_num {
            Some(num) => Ok(num),
            None => Err(get_not_in_state_err("block_num").into()),
        }
    }

    pub fn replace_btc_on_eos_signed_txs(mut self, replacements: Vec<BtcTransaction>) -> Self {
        info!("✔ Replacing signed BTC txs infos in state...");
        self.btc_on_eos_signed_txs = replacements;
        self
    }

    pub fn replace_action_proofs(mut self, replacements: EosActionProofs) -> Result<EosState<'a, D>> {
        info!("✔ Replacing `action_proofs` in state...");
        self.action_proofs = replacements;
        Ok(self)
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        self.global_sequences.clone()
    }
}
