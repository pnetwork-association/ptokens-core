#![allow(dead_code)] // FIXME RM!

use ethereum_types::{Address as EthAddress, H256 as EthHash};

use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_crypto::eth_private_key::EthPrivateKey,
        eth_submission_material::EthSubmissionMaterial,
        eth_types::{AnySenderSigningParams, EthSigningParams},
        eth_utils::{convert_bytes_to_h256, convert_h256_to_bytes},
    },
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    database_utils::{get_u64_from_db, put_u64_in_db},
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, DataSensitivity, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};

#[derive(Debug)]
pub struct EthDatabseUtils {
    any_sender_nonce_key: Bytes,
    btc_on_eth_smart_contract_address_key: Bytes,
    eos_on_eth_smart_contract_address_key: Bytes,
    erc20_on_eos_smart_contract_address_key: Bytes,
    erc20_on_evm_smart_contract_address_key: Bytes,
    erc777_proxy_contact_address_key: Bytes,
    eth_account_nonce_key: Bytes,
    eth_address_key: Bytes,
    eth_anchor_block_hash_key: Bytes,
    eth_canon_block_hash_key: Bytes,
    eth_canon_to_tip_length_key: Bytes,
    eth_chain_id_key: Bytes,
    eth_gas_price_key: Bytes,
    eth_latest_block_hash_key: Bytes,
    eth_linker_hash_key: Bytes,
    eth_private_key_db_key: Bytes,
    eth_tail_block_hash_key: Bytes,
}

impl EthDatabseUtils {
    pub fn new_for_eth() -> Self {
        use crate::chains::eth::eth_constants::{
            ANY_SENDER_NONCE_KEY,
            BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY,
            EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY,
            ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY,
            ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY,
            ERC777_PROXY_CONTACT_ADDRESS_KEY,
            ETH_ACCOUNT_NONCE_KEY,
            ETH_ADDRESS_KEY,
            ETH_ANCHOR_BLOCK_HASH_KEY,
            ETH_CANON_BLOCK_HASH_KEY,
            ETH_CANON_TO_TIP_LENGTH_KEY,
            ETH_CHAIN_ID_KEY,
            ETH_GAS_PRICE_KEY,
            ETH_LATEST_BLOCK_HASH_KEY,
            ETH_LINKER_HASH_KEY,
            ETH_PRIVATE_KEY_DB_KEY,
            ETH_TAIL_BLOCK_HASH_KEY,
        };
        Self {
            any_sender_nonce_key: ANY_SENDER_NONCE_KEY.to_vec(),
            btc_on_eth_smart_contract_address_key: BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY.to_vec(),
            eos_on_eth_smart_contract_address_key: EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY.to_vec(),
            erc20_on_eos_smart_contract_address_key: ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY.to_vec(),
            erc20_on_evm_smart_contract_address_key: ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY.to_vec(),
            erc777_proxy_contact_address_key: ERC777_PROXY_CONTACT_ADDRESS_KEY.to_vec(),
            eth_account_nonce_key: ETH_ACCOUNT_NONCE_KEY.to_vec(),
            eth_address_key: ETH_ADDRESS_KEY.to_vec(),
            eth_anchor_block_hash_key: ETH_ANCHOR_BLOCK_HASH_KEY.to_vec(),
            eth_canon_block_hash_key: ETH_CANON_BLOCK_HASH_KEY.to_vec(),
            eth_canon_to_tip_length_key: ETH_CANON_TO_TIP_LENGTH_KEY.to_vec(),
            eth_chain_id_key: ETH_CHAIN_ID_KEY.to_vec(),
            eth_gas_price_key: ETH_GAS_PRICE_KEY.to_vec(),
            eth_latest_block_hash_key: ETH_LATEST_BLOCK_HASH_KEY.to_vec(),
            eth_linker_hash_key: ETH_LINKER_HASH_KEY.to_vec(),
            eth_private_key_db_key: ETH_PRIVATE_KEY_DB_KEY.to_vec(),
            eth_tail_block_hash_key: ETH_TAIL_BLOCK_HASH_KEY.to_vec(),
        }
    }

    pub fn get_signing_params_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthSigningParams> {
        trace!("✔ Getting signing params from db...");
        Ok(EthSigningParams {
            gas_price: self.get_eth_gas_price_from_db(db)?,
            chain_id: self.get_eth_chain_id_from_db(db)?,
            eth_private_key: self.get_eth_private_key_from_db(db)?,
            eth_account_nonce: self.get_eth_account_nonce_from_db(db)?,
            smart_contract_address: self.get_erc777_contract_address_from_db(db)?,
        })
    }

    pub fn get_any_sender_signing_params_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<AnySenderSigningParams> {
        trace!("✔ Getting AnySender signing params from db...");
        Ok(AnySenderSigningParams {
            chain_id: self.get_eth_chain_id_from_db(db)?,
            eth_private_key: self.get_eth_private_key_from_db(db)?,
            any_sender_nonce: self.get_any_sender_nonce_from_db(db)?,
            public_eth_address: self.get_public_eth_address_from_db(db)?,
            erc777_proxy_address: self.get_erc777_proxy_contract_address_from_db(db)?,
        })
    }

    pub fn put_eth_canon_to_tip_length_in_db<D: DatabaseInterface>(&self, db: &D, length: u64) -> Result<()> {
        debug!("✔ Putting ETH canon-to-tip length of {} in db...", length);
        db.put(
            self.eth_canon_to_tip_length_key.to_vec(),
            convert_u64_to_bytes(length),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_eth_canon_to_tip_length_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<u64> {
        info!("✔ Getting ETH canon-to-tip length from db...");
        db.get(self.eth_canon_to_tip_length_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    pub fn put_eth_canon_block_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        eth_submission_material: &EthSubmissionMaterial,
    ) -> Result<()> {
        info!("✔ Putting ETH canon block in db...");
        self.put_special_eth_block_in_db(db, eth_submission_material, "canon")
    }

    pub fn put_eth_latest_block_hash_in_db<D: DatabaseInterface>(&self, db: &D, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH latest block hash in db...");
        self.put_special_eth_hash_in_db(db, "latest", eth_hash)
    }

    pub fn put_eth_anchor_block_hash_in_db<D: DatabaseInterface>(&self, db: &D, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH anchor block hash in db...");
        self.put_special_eth_hash_in_db(db, "anchor", eth_hash)
    }

    pub fn put_eth_canon_block_hash_in_db<D: DatabaseInterface>(&self, db: &D, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH canon block hash in db...");
        self.put_special_eth_hash_in_db(db, "canon", eth_hash)
    }

    pub fn put_eth_tail_block_hash_in_db<D: DatabaseInterface>(&self, db: &D, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH tail block hash in db...");
        self.put_special_eth_hash_in_db(db, "tail", eth_hash)
    }

    pub fn put_eth_linker_hash_in_db<D: DatabaseInterface>(&self, db: &D, eth_hash: EthHash) -> Result<()> {
        info!("✔ Putting ETH linker hash in db...");
        self.put_special_eth_hash_in_db(db, "linker", &eth_hash)
    }

    pub fn put_special_eth_block_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        eth_submission_material: &EthSubmissionMaterial,
        block_type: &str,
    ) -> Result<()> {
        trace!("✔ Putting ETH special block in db of type: {}", block_type);
        Self::put_eth_submission_material_in_db(db, eth_submission_material)
            .and_then(|_| self.put_special_eth_hash_in_db(db, block_type, &eth_submission_material.get_block_hash()?))
    }

    pub fn put_special_eth_hash_in_db<D: DatabaseInterface>(&self, db: &D, hash_type: &str, hash: &EthHash) -> Result<()> {
        let key = match hash_type {
            "linker" => Ok(self.eth_linker_hash_key.to_vec()),
            "canon" => Ok(self.eth_canon_block_hash_key.to_vec()),
            "tail" => Ok(self.eth_tail_block_hash_key.to_vec()),
            "anchor" => Ok(self.eth_anchor_block_hash_key.to_vec()),
            "latest" => Ok(self.eth_latest_block_hash_key.to_vec()),
            _ => Err(AppError::Custom(format!(
                "✘ Cannot store special ETH hash of type: {}!",
                hash_type
            ))),
        }?;
        Self::put_eth_hash_in_db(db, &key, hash)
    }

    pub fn get_latest_eth_block_number<D: DatabaseInterface>(&self, db: &D) -> Result<usize> {
        info!("✔ Getting latest ETH block number from db...");
        match self.get_special_eth_block_from_db(db, "latest") {
            Ok(result) => Ok(result.get_block_number()?.as_usize()),
            Err(e) => Err(e),
        }
    }

    pub fn get_eth_tail_block_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH tail block from db...");
        self.get_special_eth_block_from_db(db, "tail")
    }

    pub fn get_eth_latest_block_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH latest block from db...");
        self.get_special_eth_block_from_db(db, "latest")
    }

    pub fn get_eth_anchor_block_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH anchor block from db...");
        self.get_special_eth_block_from_db(db, "anchor")
    }

    pub fn get_eth_canon_block_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH canon block from db...");
        self.get_special_eth_block_from_db(db, "canon")
    }

    pub fn get_eth_anchor_block_hash_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthHash> {
        info!("✔ Getting ETH anchor block hash from db...");
        self.get_special_eth_hash_from_db(db, "anchor")
    }

    pub fn get_special_eth_hash_from_db<D: DatabaseInterface>(&self, db: &D, hash_type: &str) -> Result<EthHash> {
        let key = match hash_type {
            "linker" => Ok(self.eth_linker_hash_key.to_vec()),
            "canon" => Ok(self.eth_canon_block_hash_key.to_vec()),
            "tail" => Ok(self.eth_tail_block_hash_key.to_vec()),
            "anchor" => Ok(self.eth_anchor_block_hash_key.to_vec()),
            "latest" => Ok(self.eth_latest_block_hash_key.to_vec()),
            _ => Err(AppError::Custom(format!(
                "✘ Cannot get ETH special hash of type: {}!",
                hash_type
            ))),
        }?;
        trace!("✔ Getting special ETH hash from db of type: {}", hash_type);
        Self::get_eth_hash_from_db(db, &key.to_vec())
    }

    pub fn get_eth_hash_from_db<D: DatabaseInterface>(db: &D, key: &[Byte]) -> Result<EthHash> {
        trace!("✔ Getting ETH hash from db under key: {}", hex::encode(&key));

        db.get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(|bytes| EthHash::from_slice(&bytes))
    }

    pub fn get_special_eth_block_from_db<D: DatabaseInterface>(
        &self,
        db: &D,
        block_type: &str,
    ) -> Result<EthSubmissionMaterial> {
        self.get_special_eth_hash_from_db(db, block_type)
            .and_then(|block_hash| Self::get_submission_material_from_db(db, &block_hash))
    }

    pub fn put_eth_hash_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], eth_hash: &EthHash) -> Result<()> {
        db.put(
            key.to_vec(),
            convert_h256_to_bytes(*eth_hash),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn eth_block_exists_in_db<D: DatabaseInterface>(db: &D, block_hash: &EthHash) -> bool {
        info!(
            "✔ Checking for existence of ETH block: {}",
            hex::encode(block_hash.as_bytes().to_vec())
        );
        Self::key_exists_in_db(db, &block_hash.as_bytes().to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
    }

    pub fn get_hash_from_db_via_hash_key<D: DatabaseInterface>(db: &D, hash_key: EthHash) -> Result<Option<EthHash>> {
        match db.get(convert_h256_to_bytes(hash_key), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Ok(Some(convert_bytes_to_h256(&bytes)?)),
            Err(_) => Ok(None),
        }
    }

    pub fn put_eth_submission_material_in_db<D: DatabaseInterface>(
        db: &D,
        eth_submission_material: &EthSubmissionMaterial,
    ) -> Result<()> {
        let key = convert_h256_to_bytes(eth_submission_material.get_block_hash()?);
        trace!("✔ Adding block to database under key: {:?}", hex::encode(&key));
        db.put(
            key,
            eth_submission_material.remove_block().to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn maybe_get_parent_eth_submission_material<D: DatabaseInterface>(
        db: &D,
        block_hash: &EthHash,
    ) -> Option<EthSubmissionMaterial> {
        debug!("✔ Maybe getting parent ETH block from db...");
        Self::maybe_get_nth_ancestor_eth_submission_material(db, block_hash, 1).ok()?
    }

    pub fn maybe_get_nth_ancestor_eth_submission_material<D: DatabaseInterface>(
        db: &D,
        block_hash: &EthHash,
        n: u64,
    ) -> Result<Option<EthSubmissionMaterial>> {
        debug!("✔ Getting {}th ancestor ETH block from db...", n);
        match Self::maybe_get_eth_submission_material_from_db(db, block_hash) {
            None => Ok(None),
            Some(block_and_receipts) => match n {
                0 => Ok(Some(block_and_receipts)),
                _ => Self::maybe_get_nth_ancestor_eth_submission_material(
                    db,
                    &block_and_receipts.get_parent_hash()?,
                    n - 1,
                ),
            },
        }
    }

    pub fn maybe_get_eth_submission_material_from_db<D: DatabaseInterface>(
        db: &D,
        block_hash: &EthHash,
    ) -> Option<EthSubmissionMaterial> {
        trace!(
            "✔ Maybe getting ETH block and receipts from db under hash: {}",
            block_hash
        );
        match db.get(convert_h256_to_bytes(*block_hash), MIN_DATA_SENSITIVITY_LEVEL) {
            Err(_) => None,
            Ok(bytes) => match EthSubmissionMaterial::from_bytes(&bytes) {
                Ok(block_and_receipts) => {
                    trace!("✔ Decoded eth block and receipts from db!");
                    Some(block_and_receipts)
                },
                Err(_) => {
                    error!("✘ Failed to decode eth block and receipts from db!");
                    None
                },
            },
        }
    }

    pub fn get_submission_material_from_db<D: DatabaseInterface>(
        db: &D,
        block_hash: &EthHash,
    ) -> Result<EthSubmissionMaterial> {
        trace!("✔ Getting ETH block and receipts from db...");
        db.get(convert_h256_to_bytes(*block_hash), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| EthSubmissionMaterial::from_bytes(&bytes))
    }

    pub fn key_exists_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], sensitivity: DataSensitivity) -> bool {
        trace!("✔ Checking for existence of key: {}", hex::encode(key));
        db.get(key.to_vec(), sensitivity).is_ok()
    }

    pub fn put_eth_gas_price_in_db<D: DatabaseInterface>(&self, db: &D, gas_price: u64) -> Result<()> {
        trace!("✔ Putting ETH gas price of {} in db...", gas_price);
        db.put(
            self.eth_gas_price_key.to_vec(),
            gas_price.to_le_bytes().to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_eth_gas_price_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<u64> {
        trace!("✔ Getting ETH gas price from db...");
        db.get(self.eth_gas_price_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| match bytes.len() <= 8 {
                true => {
                    let mut array = [0; 8];
                    let bytes = &bytes[..array.len()];
                    array.copy_from_slice(bytes);
                    Ok(u64::from_le_bytes(array))
                },
                false => Err("✘ Too many bytes to convert to u64!".into()),
            })
    }

    pub fn get_eth_account_nonce_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<u64> {
        trace!("✔ Getting ETH account nonce from db...");
        get_u64_from_db(db, &self.eth_account_nonce_key.to_vec())
    }

    pub fn put_eth_account_nonce_in_db<D: DatabaseInterface>(&self, db: &D, nonce: u64) -> Result<()> {
        trace!("✔ Putting ETH account nonce of {} in db...", nonce);
        put_u64_in_db(db, &self.eth_account_nonce_key.to_vec(), nonce)
    }

    pub fn increment_eth_account_nonce_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        amount_to_increment_by: u64
    ) -> Result<()> {
        trace!("✔ Incrementing ETH account nonce in db...");
        self.get_eth_account_nonce_from_db(db)
            .and_then(|nonce| self.put_eth_account_nonce_in_db(db, nonce + amount_to_increment_by))
    }

    pub fn put_eth_chain_id_in_db<D: DatabaseInterface>(&self, db: &D, chain_id: &EthChainId) -> Result<()> {
        info!("✔ Putting `EthChainId` in db: {}", chain_id);
        db.put(
            self.eth_chain_id_key.to_vec(),
            chain_id.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_eth_chain_id_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthChainId> {
        trace!("✔ Getting ETH `chain_id` from db...");
        db.get(self.eth_chain_id_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|ref bytes| EthChainId::from_bytes(bytes))
    }

    pub fn put_eth_private_key_in_db<D: DatabaseInterface>(self, db: &D, pk: &EthPrivateKey) -> Result<()> {
        trace!("✔ Putting ETH private key in db...");
        pk.write_to_database(db, &self.eth_private_key_db_key.to_vec())
    }

    pub fn get_eth_private_key_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthPrivateKey> {
        trace!("✔ Getting ETH private key from db...");
        db.get(self.eth_private_key_db_key.to_vec(), Some(255)).and_then(|pk_bytes| {
            let mut array = [0; 32];
            array.copy_from_slice(&pk_bytes[..32]);
            EthPrivateKey::from_slice(&array)
        })
    }

    pub fn get_erc777_contract_address_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthAddress> {
        info!("✔ Getting ETH ERC777 smart-contract address from db...");
        Self::get_eth_address_from_db(db, &self.btc_on_eth_smart_contract_address_key)
            .map_err(|_| "No ERC777 contract address in DB! Did you forget to set it?".into())
    }

    pub fn get_erc20_on_eos_smart_contract_address_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthAddress> {
        info!("✔ Getting `pERC20-on-EOS` smart-contract address from db...");
        Self::get_eth_address_from_db(db, &self.erc20_on_eos_smart_contract_address_key)
            .map_err(|_| "No `erc20-on-eos` vault contract address in DB! Did you forget to set it?".into())
    }

    pub fn get_eos_on_eth_smart_contract_address_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthAddress> {
        info!("✔ Getting 'EOS_ON_ETH' smart-contract address from db...");
        Ok(
            Self::get_eth_address_from_db(db, &self.eos_on_eth_smart_contract_address_key)
                .unwrap_or_else(|_| EthAddress::zero()),
        )
    }

    pub fn get_erc20_on_evm_smart_contract_address_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthAddress> {
        info!("✔ Getting `ERC20_ON_EVM` smart-contract address from db...");
        Self::get_eth_address_from_db(db, &self.erc20_on_evm_smart_contract_address_key)
            .map_err(|_| "No `erc20-on-evm` vault contract address in DB! Did you forget to set it?".into())
    }

    fn get_eth_address_from_db<D: DatabaseInterface>(db: &D, key: &[Byte]) -> Result<EthAddress> {
        db.get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(|address_bytes| EthAddress::from_slice(&address_bytes[..]))
    }

    pub fn get_erc777_proxy_contract_address_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthAddress> {
        trace!("✔ Getting ERC777 proxy contract address from db...");
        match db.get(self.erc777_proxy_contact_address_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(address_bytes) => Ok(EthAddress::from_slice(&address_bytes[..])),
            Err(_) => {
                debug!("✘ No ERC777 proxy address in db, defaulting to zero ETH address!");
                Ok(EthAddress::zero())
            },
        }
    }

    #[allow(dead_code)]
    pub fn put_erc777_proxy_contract_address_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        proxy_contract_address: &EthAddress,
    ) -> Result<()> {
        trace!("✔ Putting ERC777 proxy contract address in db...");
        self.put_eth_address_in_db(db, &self.erc777_proxy_contact_address_key.to_vec(), proxy_contract_address)
    }

    pub fn put_btc_on_eth_smart_contract_address_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        address: &EthAddress,
    ) -> Result<()> {
        match self.get_erc777_contract_address_from_db(db) {
            Ok(address) => Err(format!("ERC777 address already set to 0x{}!", hex::encode(address)).into()),
            _ => {
                info!("✔ Putting ETH smart-contract address in db...");
                self.put_eth_address_in_db(db, &self.btc_on_eth_smart_contract_address_key, address)
            },
        }
    }

    pub fn put_erc20_on_eos_smart_contract_address_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        smart_contract_address: &EthAddress,
    ) -> Result<()> {
        match self.get_erc20_on_eos_smart_contract_address_from_db(db) {
            Ok(address) => Err(format!(
                "`erc20-on-eos` vault address is already set to {}!",
                hex::encode(address)
            )
            .into()),
            _ => {
                info!("✔ Putting 'ERC20-on-EOS` smart-contract address in db...");
                self.put_eth_address_in_db(
                    db,
                    &self.erc20_on_eos_smart_contract_address_key.to_vec(),
                    smart_contract_address,
                )
            },
        }
    }

    pub fn put_eos_on_eth_smart_contract_address_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        smart_contract_address: &EthAddress,
    ) -> Result<()> {
        trace!("✔ Putting 'EOS_ON_ETH' smart-contract address in db...");
        self.put_eth_address_in_db(
            db,
            &self.eos_on_eth_smart_contract_address_key.to_vec(),
            smart_contract_address,
        )
    }

    pub fn put_erc20_on_evm_smart_contract_address_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        address: &EthAddress,
    ) -> Result<()> {
        if self.get_erc20_on_evm_smart_contract_address_from_db(db).is_ok() {
            Err("`ERC20-on-EVM`Vault contract address already set!".into())
        } else {
            info!("✔ Putting `ERC20-on-EVM` vault contract address in db...");
            self.put_eth_address_in_db(db, &self.erc20_on_evm_smart_contract_address_key.to_vec(), address)
        }
    }

    pub fn get_public_eth_address_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<EthAddress> {
        trace!("✔ Getting public ETH address from db...");
        db.get(self.eth_address_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(|bytes| EthAddress::from_slice(&bytes))
    }

    pub fn put_public_eth_address_in_db<D: DatabaseInterface>(&self, db: &D, eth_address: &EthAddress) -> Result<()> {
        trace!("✔ Putting public ETH address in db...");
        db.put(
            self.eth_address_key.to_vec(),
            eth_address.as_bytes().to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn put_eth_address_in_db<D: DatabaseInterface>(&self, db: &D, key: &[Byte], eth_address: &EthAddress) -> Result<()> {
        db.put(
            key.to_vec(),
            eth_address.as_bytes().to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_any_sender_nonce_from_db<D: DatabaseInterface>(&self, db: &D) -> Result<u64> {
        trace!("✔ Getting AnySender nonce from db...");
        Ok(get_u64_from_db(db, &self.any_sender_nonce_key.to_vec()).unwrap_or_else(|_| {
            info!("✘ Could not find `AnySender` nonce in db, defaulting to `0`");
            0
        }))
    }

    pub fn put_any_sender_nonce_in_db<D: DatabaseInterface>(&self, db: &D, nonce: u64) -> Result<()> {
        trace!("✔ Putting AnySender nonce of {} in db...", nonce);
        put_u64_in_db(db, &self.any_sender_nonce_key.to_vec(), nonce)
    }

    pub fn increment_any_sender_nonce_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        amount_to_increment_by: u64
    ) -> Result<()> {
        trace!("✔ Incrementing AnySender nonce in db...");
        self.get_any_sender_nonce_from_db(db)
            .and_then(|nonce| self.put_any_sender_nonce_in_db(db, nonce + amount_to_increment_by))
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_test_utils::{
            get_sample_contract_address,
            get_sample_eth_address,
            get_sample_eth_private_key,
            get_sample_eth_submission_material,
            get_sample_eth_submission_material_n,
            get_sequential_eth_blocks_and_receipts,
        },
        test_utils::get_test_database,
    };

    #[test]
    fn non_existing_key_should_not_exist_in_db() {
        let db = get_test_database();
        let result = key_exists_in_db(&db, &ETH_ACCOUNT_NONCE_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(!result);
    }

    #[test]
    fn existing_key_should_exist_in_db() {
        let thing = vec![0xc0];
        let db = get_test_database();
        let key = *ETH_ACCOUNT_NONCE_KEY;
        db.put(key.to_vec(), thing, MIN_DATA_SENSITIVITY_LEVEL).unwrap();
        let result = key_exists_in_db(&db, &ETH_ACCOUNT_NONCE_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(result);
    }

    #[test]
    fn should_put_eth_gas_price_in_db() {
        let db = get_test_database();
        let gas_price = 20_000_000;
        put_eth_gas_price_in_db(&db, gas_price).unwrap();
        match get_eth_gas_price_from_db(&db) {
            Ok(gas_price_from_db) => assert_eq!(gas_price_from_db, gas_price),
            Err(e) => panic!("Error getting gas price from db: {}", e),
        }
    }

    #[test]
    fn should_put_chain_id_in_db() {
        let db = get_test_database();
        let chain_id = EthChainId::Rinkeby;
        put_eth_chain_id_in_db(&db, &chain_id).unwrap();
        let result = get_eth_chain_id_from_db(&db).unwrap();
        assert_eq!(result, chain_id);
    }

    #[test]
    fn should_save_nonce_to_db_and_get_nonce_from_db() {
        let db = get_test_database();
        let nonce = 1227;
        put_eth_account_nonce_in_db(&db, nonce).unwrap();
        match get_eth_account_nonce_from_db(&db) {
            Ok(nonce_from_db) => assert_eq!(nonce_from_db, nonce),
            Err(e) => panic!("Error getting nonce from db: {}", e),
        }
    }

    #[test]
    fn should_get_erc777_contract_address_from_db() {
        let db = get_test_database();
        let contract_address = get_sample_eth_address();
        put_btc_on_eth_smart_contract_address_in_db(&db, &contract_address).unwrap();
        let result = get_erc777_contract_address_from_db(&db).unwrap();
        assert_eq!(result, contract_address);
    }

    #[test]
    fn should_get_eth_pk_from_database() {
        let db = get_test_database();
        let eth_private_key = get_sample_eth_private_key();
        put_eth_private_key_in_db(&db, &eth_private_key).unwrap();
        match get_eth_private_key_from_db(&db) {
            Ok(pk) => assert_eq!(pk, eth_private_key),
            Err(e) => panic!("Error getting eth private key from db: {}", e),
        }
    }

    #[test]
    fn should_increment_eth_account_nonce_in_db() {
        let nonce = 666;
        let db = get_test_database();
        put_eth_account_nonce_in_db(&db, nonce).unwrap();
        let amount_to_increment_by: u64 = 671;
        increment_eth_account_nonce_in_db(&db, amount_to_increment_by).unwrap();
        match get_eth_account_nonce_from_db(&db) {
            Err(e) => panic!("Error getting nonce from db: {}", e),
            Ok(nonce_from_db) => assert_eq!(nonce_from_db, nonce + amount_to_increment_by),
        }
    }

    #[test]
    fn should_put_and_get_special_eth_hash_in_db() {
        let db = get_test_database();
        let hash_type = "linker";
        let hash = get_sample_eth_submission_material_n(1)
            .unwrap()
            .get_block_hash()
            .unwrap();
        put_special_eth_hash_in_db(&db, &hash_type, &hash).unwrap();
        match get_special_eth_hash_from_db(&db, hash_type) {
            Ok(hash_from_db) => assert_eq!(hash_from_db, hash),
            Err(e) => panic!("Error getting ETH special hash from db: {}", e),
        }
    }

    #[test]
    fn should_put_and_get_eth_hash_in_db() {
        let db = get_test_database();
        let hash_key = vec![6u8, 6u8, 6u8];
        let hash = get_sample_eth_submission_material_n(1)
            .unwrap()
            .get_block_hash()
            .unwrap();
        put_eth_hash_in_db(&db, &hash_key, &hash).unwrap();
        match get_eth_hash_from_db(&db, &hash_key) {
            Ok(hash_from_db) => assert_eq!(hash_from_db, hash),
            Err(e) => panic!("Error getting ETH hash from db: {}", e),
        }
    }

    #[test]
    fn should_put_and_get_special_eth_block_in_db() {
        let db = get_test_database();
        let block_type = "anchor";
        let submission_material = get_sample_eth_submission_material_n(1).unwrap();
        let expected_result = submission_material.remove_block();
        put_special_eth_block_in_db(&db, &submission_material, &block_type).unwrap();
        match get_special_eth_block_from_db(&db, block_type) {
            Ok(result) => assert_eq!(result, expected_result),
            Err(e) => panic!("Error getting ETH special submission_material from db: {}", e),
        }
    }

    #[test]
    fn should_get_submission_material_block_from_db() {
        let db = get_test_database();
        let submission_material = get_sample_eth_submission_material_n(1).unwrap();
        let expected_result = submission_material.remove_block();
        let block_hash = submission_material.get_block_hash().unwrap();
        put_eth_submission_material_in_db(&db, &submission_material).unwrap();
        match get_submission_material_from_db(&db, &block_hash) {
            Ok(result) => assert_eq!(result, expected_result),
            Err(e) => panic!("Error getting ETH submission_material from db: {}", e),
        }
    }

    #[test]
    fn should_put_eth_address_in_db() {
        let db = get_test_database();
        let key = ETH_ADDRESS_KEY.to_vec();
        let eth_address = get_sample_contract_address();
        let result = put_eth_address_in_db(&db, &key, &eth_address);
        assert!(result.is_ok());
    }

    #[test]
    fn should_put_and_get_public_eth_address_in_db() {
        let db = get_test_database();
        let eth_address = get_sample_contract_address();
        put_public_eth_address_in_db(&db, &eth_address).unwrap();
        match get_public_eth_address_from_db(&db) {
            Ok(eth_address_from_db) => assert_eq!(eth_address_from_db, eth_address),
            Err(e) => panic!("Error getting ETH address from db: {}", e),
        }
    }

    #[test]
    fn maybe_get_block_should_be_none_if_block_not_extant() {
        let db = get_test_database();
        let block_hash = get_sample_eth_submission_material_n(1)
            .unwrap()
            .get_block_hash()
            .unwrap();
        if maybe_get_eth_submission_material_from_db(&db, &block_hash).is_some() {
            panic!("Maybe getting none existing block should be 'None'");
        };
    }

    #[test]
    fn should_maybe_get_some_block_if_exists() {
        let db = get_test_database();
        let submission_material = get_sample_eth_submission_material_n(1).unwrap();
        let expected_result = submission_material.remove_block();
        let block_hash = submission_material.get_block_hash().unwrap();
        put_eth_submission_material_in_db(&db, &submission_material).unwrap();
        match maybe_get_eth_submission_material_from_db(&db, &block_hash) {
            None => panic!("`submission_material` should exist in db!"),
            Some(result) => assert_eq!(result, expected_result),
        };
    }

    #[test]
    fn should_return_none_if_no_parent_block_exists() {
        let db = get_test_database();
        let block = get_sample_eth_submission_material_n(1).unwrap();
        let block_hash = block.get_block_hash().unwrap();
        put_eth_submission_material_in_db(&db, &block).unwrap();
        let result = maybe_get_parent_eth_submission_material(&db, &block_hash);
        assert!(result.is_none());
    }

    #[test]
    fn should_maybe_get_parent_block_if_it_exists() {
        let db = get_test_database();
        let blocks = get_sequential_eth_blocks_and_receipts();
        let block = blocks[1].clone();
        let parent_block = blocks[0].clone();
        let expected_result = parent_block.remove_block();
        let block_hash = block.get_block_hash().unwrap();
        put_eth_submission_material_in_db(&db, &block).unwrap();
        put_eth_submission_material_in_db(&db, &parent_block).unwrap();
        match maybe_get_parent_eth_submission_material(&db, &block_hash) {
            None => panic!("Block should have parent in the DB!"),
            Some(result) => assert_eq!(result, expected_result),
        };
    }

    #[test]
    fn should_get_no_nth_ancestor_if_not_extant() {
        let ancestor_number = 3;
        let db = get_test_database();
        let block = get_sample_eth_submission_material_n(1).unwrap();
        let block_hash = block.get_block_hash().unwrap();
        put_eth_submission_material_in_db(&db, &block).unwrap();
        let result = maybe_get_nth_ancestor_eth_submission_material(&db, &block_hash, ancestor_number).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn should_get_nth_ancestor_if_extant() {
        let db = get_test_database();
        let blocks = get_sequential_eth_blocks_and_receipts();
        let block_hash = blocks[blocks.len() - 1].get_block_hash().unwrap();
        blocks
            .iter()
            .map(|block| put_eth_submission_material_in_db(&db, block))
            .collect::<Result<()>>()
            .unwrap();
        blocks.iter().enumerate().for_each(|(i, _)| {
            match maybe_get_nth_ancestor_eth_submission_material(&db, &block_hash, i as u64).unwrap() {
                None => panic!("Ancestor number {} should exist!", i),
                Some(ancestor) => assert_eq!(ancestor, blocks[blocks.len() - i - 1].remove_block()),
            }
        });
        let result = maybe_get_nth_ancestor_eth_submission_material(&db, &block_hash, blocks.len() as u64).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn saving_submission_material_should_remove_block() {
        let db = get_test_database();
        let submission_material = get_sample_eth_submission_material();
        let db_key = submission_material.get_block_hash().unwrap();
        assert!(submission_material.block.is_some());
        put_eth_submission_material_in_db(&db, &submission_material).unwrap();
        let result = get_submission_material_from_db(&db, &db_key).unwrap();
        assert!(result.block.is_none());
    }

    #[test]
    fn should_save_submission_material_if_block_already_removed() {
        let db = get_test_database();
        let submission_material = get_sample_eth_submission_material().remove_block();
        let db_key = submission_material.get_block_hash().unwrap();
        put_eth_submission_material_in_db(&db, &submission_material).unwrap();
        let result = get_submission_material_from_db(&db, &db_key).unwrap();
        assert_eq!(result, submission_material);
    }
}
*/
