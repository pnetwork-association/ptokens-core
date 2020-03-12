use eos_primitives::{
    Action as EosAction,
    BlockHeader as EosBlockHeader,
    ActionReceipt as EosActionReceipt,
};

use crate::btc_on_eos::eos::eos_crypto::eos_signature::EosSignature;

pub type EosAmount = String;
pub type EosAddress = String;
pub type ActionProof = MerkleProof;
pub type MerkleProof = Vec<String>;
pub type EosAddresses = Vec<String>;
pub type ActionProofs = MerkleProofs;
pub type EosAmounts = Vec<EosAmount>;
pub type MerkleProofs = Vec<ActionProof>;
pub type EosSignatures = Vec<EosSignature>;
pub type ActionProofJsons = Vec<ActionProofJson>;
pub type Sha256HashedMessage = secp256k1::Message;
pub type EosSignedTransactions= Vec<EosSignedTransaction>;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum EosNetwork {
    Mainnet,
    Testnet,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct EosSignedTransaction {
    pub nonce: usize,
    pub amount: String,
    pub recipient: String,
    pub signature: String,
    pub transaction: String,
}

impl EosSignedTransaction {
    pub fn new(
        signature: String,
        transaction: String,
        nonce: usize,
        recipient: String,
        amount: String,
    ) -> EosSignedTransaction {
        EosSignedTransaction {
            signature,
            transaction,
            nonce,
            amount,
            recipient,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterial {
    pub action_proofs: ActionProofs,
    pub block_header: EosBlockHeader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterialJson {
    pub block_header: EosBlockHeaderJson,
    pub action_proofs: ActionProofJsons,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosBlockHeaderJson {
    pub confirmed: u16,
    pub producer: String,
    pub previous: String,
    pub block_id: String,
    pub block_num: usize,
    pub timestamp: String,
    pub action_mroot: String,
    pub schedule_version: u32,
    pub transaction_mroot: String,
    pub producer_signature: String,
    pub header_extension: Option<Vec<String>>,
    pub new_producers: Option<ProducerScheduleJson>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProducerScheduleJson {
    pub version: u32,
    pub producers: Vec<ProducerKeyJson>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProducerKeyJson {
    pub producer_name: String, // To become AccountName
    pub block_signing_key: String, // To become public key
}

#[derive(Debug)]
pub struct EosRawTxData {
    pub sender: String,
    pub mint_nonce: u64,
    pub receiver: String,
    pub asset_amount: u64,
    pub asset_name: String,
    pub eth_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionProofJson {
    pub tx_id: String,
    pub block_id: String,
    pub action_index: usize,
    pub action_digest: String,
    pub action_proof: ActionProof,
    pub serialized_action: String,
    pub action_receipt_digest: String,
    pub serialized_action_receipt: String,
}
