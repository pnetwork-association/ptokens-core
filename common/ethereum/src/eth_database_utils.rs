use common::{
    constants::{MAX_DATA_SENSITIVITY_LEVEL, MIN_DATA_SENSITIVITY_LEVEL},
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, DataSensitivity, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};
use common_chain_ids::EthChainId;
use ethereum_types::{Address as EthAddress, H256 as EthHash};

use crate::{convert_h256_to_bytes, AnySenderSigningParams, EthPrivateKey, EthSigningParams, EthSubmissionMaterial};

pub fn put_u64_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], u_64: u64) -> Result<()> {
    debug!("✔ Putting `u64` of {} in db...", u_64);
    db.put(key.to_vec(), u_64.to_le_bytes().to_vec(), None)
}

pub fn get_u64_from_db<D: DatabaseInterface>(db: &D, key: &[Byte]) -> Result<u64> {
    debug!("✔ Getting `u64` from db...");
    db.get(key.to_vec(), None)
        .and_then(|ref bytes| convert_bytes_to_u64(bytes))
}

create_db_utils_with_getters!(
    "Eth";
    "_CHAIN_ID_KEY" => "eth-chain-id",
    "_GAS_PRICE_KEY" => "eth-gas-price",
    "_ADDRESS_KEY" => "eth-address-key",
    "_LINKER_HASH_KEY" => "linker-hash-key",
    "_ACCOUNT_NONCE_KEY" => "eth-account-nonce",
    "_ANY_SENDER_NONCE_KEY" => "any-sender-nonce",
    "_PRIVATE_KEY_DB_KEY" => "eth-private-key-key",
    "_PTOKEN_GENESIS_HASH_KEY" => "provable-ptoken",
    "_CANON_BLOCK_HASH_KEY" => "canon-block-hash-key",
    "_ANCHOR_BLOCK_HASH_KEY" => "anchor-block-hash-key",
    "_LATEST_BLOCK_HASH_KEY" => "latest-block-hash-key",
    "_TAIL_BLOCK_HASH_KEY" => "eth-tail-block-hash-key",
    "_CANON_TO_TIP_LENGTH_KEY" => "canon-to-tip-length-key",
    "_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "eth-smart-contract",
    "_ERC777_PROXY_CONTRACT_ADDRESS_KEY" => "erc-777-proxy-contract-address-key",
    "_ROUTER_SMART_CONTRACT_ADDRESS_KEY" => "eth-router-smart-contract-address-key",
    "_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "eos-on-eth-smart-contract-address-key",
    "_INT_ON_EOS_SMART_CONTRACT_ADDRESS_KEY" => "int-on-eos-smart-contract-address-key",
    "_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY" => "erc20-on-eos-smart-contract-address-key",
    "_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "eth-int-on-evm-smart-contract-address-key",
    "_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "erc20-on-evm-eth-smart-contract-address-key",
    "_INT_ON_ALGO_SMART_CONTRACT_ADDRESS_KEY" => "int-on-algo-smart-contract-address-key"
);

create_db_utils_with_getters!(
    "Evm";
    "_CHAIN_ID_KEY" => "evm-chain-id",
    "_GAS_PRICE_KEY" => "evm-gas-price",
    "_ADDRESS_KEY" => "evm-address-key",
    "_LINKER_HASH_KEY" => "evm-linker-hash-key",
    "_ACCOUNT_NONCE_KEY" => "evm-account-nonce",
    "_PRIVATE_KEY_DB_KEY" => "evm-private-key-key",
    "_ANY_SENDER_NONCE_KEY" => "evm-any-sender-nonce",
    "_TAIL_BLOCK_HASH_KEY" => "evm-tail-block-hash-key",
    "_PTOKEN_GENESIS_HASH_KEY" => "evm-provable-ptoken",
    "_CANON_BLOCK_HASH_KEY" => "evm-canon-block-hash-key",
    "_ANCHOR_BLOCK_HASH_KEY" => "evm-anchor-block-hash-key",
    "_LATEST_BLOCK_HASH_KEY" => "evm-latest-block-hash-key",
    "_CANON_TO_TIP_LENGTH_KEY" => "evm-canon-to-tip-length-key",
    "_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "evm-smart-contract",
    "_ROUTER_SMART_CONTRACT_ADDRESS_KEY" => "eth-router-smart-contract-address-key",
    "_ERC777_PROXY_CONTRACT_ADDRESS_KEY" => "evm-erc-777-proxy-contract-address-key",
    "_INT_ON_EOS_SMART_CONTRACT_ADDRESS_KEY" => "int-on-eos-smart-contract-address-key",
    "_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "evm-eos-on-eth-smart-contract-address-key",
    "_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "eth-int-on-evm-smart-contract-address-key",
    "_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY" => "evm-erc20-on-eos-smart-contract-address-key",
    "_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "evm-erc20-on-evm-smart-contract-address-key",
    "_INT_ON_ALGO_SMART_CONTRACT_ADDRESS_KEY" => "evm-int-on-algo-smart-contract-address-key"
);

macro_rules! impl_eth_db_utils_ext {
    ($prefix:ident, $is_for_eth:expr) => {
        paste! {
            impl<D: DatabaseInterface> EthDbUtilsExt<D> for [< $prefix:camel DbUtils>]<'_, D> {
                fn get_db(&self) -> &D {
                    self.get_db()
                }

                fn get_is_for_evm(&self) -> bool {
                    !$is_for_eth
                }

                fn get_any_sender_nonce_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _any_sender_nonce_key>]()
                }

                fn get_router_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _router_smart_contract_address_key>]()
                }

                fn get_erc20_on_evm_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _erc20_on_evm_smart_contract_address_key>]()
                }

                fn get_int_on_eos_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _int_on_eos_smart_contract_address_key>]()
                }

                fn get_eos_on_eth_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _eos_on_eth_smart_contract_address_key>]()
                }

                fn get_erc20_on_eos_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _erc20_on_eos_smart_contract_address_key>]()
                }

                fn get_btc_on_eth_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _btc_on_eth_smart_contract_address_key>]()
                }

                fn get_erc777_proxy_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _erc777_proxy_contract_address_key>]()
                }

                fn get_eth_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _address_key>]()
                }

                fn get_eth_private_key_db_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _private_key_db_key>]()
                }

                fn get_eth_chain_id_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _chain_id_key>]()
                }

                fn get_eth_account_nonce_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _account_nonce_key>]()
                }

                fn get_eth_gas_price_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _gas_price_key>]()
                }

                fn get_eth_linker_hash_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _linker_hash_key>]()
                }

                fn get_eth_tail_block_hash_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _tail_block_hash_key>]()
                }

                fn get_eth_canon_block_hash_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _canon_block_hash_key>]()
                }

                fn get_eth_latest_block_hash_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _latest_block_hash_key>]()
                }

                fn get_eth_anchor_block_hash_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _anchor_block_hash_key>]()
                }

                fn get_eth_canon_to_tip_length_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _canon_to_tip_length_key>]()
                }

                fn get_int_on_evm_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _int_on_evm_smart_contract_address_key>]()
                }

                fn get_int_on_algo_smart_contract_address_key(&self) -> Bytes {
                    self.[< get_ $prefix:lower _int_on_algo_smart_contract_address_key>]()
                }
            }
        }
    };
}

impl_eth_db_utils_ext!(Eth, true);
impl_eth_db_utils_ext!(Evm, false);

pub trait EthDbUtilsExt<D: DatabaseInterface> {
    fn get_db(&self) -> &D;
    fn get_is_for_evm(&self) -> bool;
    fn get_eth_address_key(&self) -> Bytes;
    fn get_eth_chain_id_key(&self) -> Bytes;
    fn get_eth_gas_price_key(&self) -> Bytes;
    fn get_eth_linker_hash_key(&self) -> Bytes;
    fn get_any_sender_nonce_key(&self) -> Bytes;
    fn get_eth_account_nonce_key(&self) -> Bytes;
    fn get_eth_private_key_db_key(&self) -> Bytes;
    fn get_eth_tail_block_hash_key(&self) -> Bytes;
    fn get_eth_canon_block_hash_key(&self) -> Bytes;
    fn get_eth_latest_block_hash_key(&self) -> Bytes;
    fn get_eth_anchor_block_hash_key(&self) -> Bytes;
    fn get_eth_canon_to_tip_length_key(&self) -> Bytes;
    fn get_router_smart_contract_address_key(&self) -> Bytes;
    fn get_erc777_proxy_contract_address_key(&self) -> Bytes;
    fn get_int_on_evm_smart_contract_address_key(&self) -> Bytes;
    fn get_eos_on_eth_smart_contract_address_key(&self) -> Bytes;
    fn get_btc_on_eth_smart_contract_address_key(&self) -> Bytes;
    fn get_int_on_eos_smart_contract_address_key(&self) -> Bytes;
    fn get_int_on_algo_smart_contract_address_key(&self) -> Bytes;
    fn get_erc20_on_evm_smart_contract_address_key(&self) -> Bytes;
    fn get_erc20_on_eos_smart_contract_address_key(&self) -> Bytes;

    fn get_is_for_eth(&self) -> bool {
        !self.get_is_for_evm()
    }

    fn delete_block_by_block_hash(&self, block: &EthSubmissionMaterial) -> Result<()> {
        let key = self.normalize_key(block.get_block_hash()?.as_bytes().to_vec());
        debug!("Deleting block by blockhash under key: 0x{}", hex::encode(&key));
        self.get_db().delete(key)
    }

    fn get_signing_params_from_db(&self) -> Result<EthSigningParams> {
        debug!("✔ Getting signing params from db...");
        Ok(EthSigningParams {
            gas_price: self.get_eth_gas_price_from_db()?,
            chain_id: self.get_eth_chain_id_from_db()?,
            eth_private_key: self.get_eth_private_key_from_db()?,
            eth_account_nonce: self.get_eth_account_nonce_from_db()?,
            smart_contract_address: self.get_btc_on_eth_smart_contract_address_from_db()?,
        })
    }

    fn get_any_sender_signing_params_from_db(&self) -> Result<AnySenderSigningParams> {
        debug!("✔ Getting AnySender signing params from db...");
        Ok(AnySenderSigningParams {
            chain_id: self.get_eth_chain_id_from_db()?,
            eth_private_key: self.get_eth_private_key_from_db()?,
            any_sender_nonce: self.get_any_sender_nonce_from_db()?,
            public_eth_address: self.get_public_eth_address_from_db()?,
            erc777_proxy_address: self.get_erc777_proxy_contract_address_from_db()?,
        })
    }

    fn put_eth_canon_to_tip_length_in_db(&self, length: u64) -> Result<()> {
        debug!("✔ Putting ETH canon-to-tip length of {} in db...", length);
        self.get_db().put(
            self.get_eth_canon_to_tip_length_key(),
            convert_u64_to_bytes(length),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_eth_canon_to_tip_length_from_db(&self) -> Result<u64> {
        info!("✔ Getting ETH canon-to-tip length from db...");
        self.get_db()
            .get(self.get_eth_canon_to_tip_length_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    fn put_eth_canon_block_in_db(&self, eth_submission_material: &EthSubmissionMaterial) -> Result<()> {
        info!("✔ Putting ETH canon block in db...");
        self.put_special_eth_block_in_db(eth_submission_material, "canon")
    }

    fn put_eth_latest_block_hash_in_db(&self, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH latest block hash in db...");
        self.put_special_eth_hash_in_db("latest", eth_hash)
    }

    fn put_eth_anchor_block_hash_in_db(&self, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH anchor block hash in db...");
        self.put_special_eth_hash_in_db("anchor", eth_hash)
    }

    fn put_eth_canon_block_hash_in_db(&self, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH canon block hash in db...");
        self.put_special_eth_hash_in_db("canon", eth_hash)
    }

    fn put_eth_tail_block_hash_in_db(&self, eth_hash: &EthHash) -> Result<()> {
        info!("✔ Putting ETH tail block hash in db...");
        self.put_special_eth_hash_in_db("tail", eth_hash)
    }

    fn put_eth_linker_hash_in_db(&self, eth_hash: EthHash) -> Result<()> {
        info!("✔ Putting ETH linker hash in db...");
        self.put_special_eth_hash_in_db("linker", &eth_hash)
    }

    fn put_special_eth_block_in_db(
        &self,
        eth_submission_material: &EthSubmissionMaterial,
        block_type: &str,
    ) -> Result<()> {
        debug!("✔ Putting ETH special block in db of type: {}", block_type);
        self.put_eth_submission_material_in_db(eth_submission_material)
            .and_then(|_| self.put_special_eth_hash_in_db(block_type, &eth_submission_material.get_block_hash()?))
    }

    fn get_linker_hash_or_genesis_hash(&self) -> Result<EthHash> {
        match self.get_special_eth_hash_from_db("linker") {
            Ok(hash) => Ok(hash),
            Err(_) => {
                info!("✔ No linker-hash set yet, using pToken genesis hash...");
                Ok(EthHash::from_slice(&ETH_PTOKEN_GENESIS_HASH_KEY[..]))
            },
        }
    }

    fn put_special_eth_hash_in_db(&self, hash_type: &str, hash: &EthHash) -> Result<()> {
        let key = match hash_type {
            "linker" => Ok(self.get_eth_linker_hash_key()),
            "canon" => Ok(self.get_eth_canon_block_hash_key()),
            "tail" => Ok(self.get_eth_tail_block_hash_key()),
            "anchor" => Ok(self.get_eth_anchor_block_hash_key()),
            "latest" => Ok(self.get_eth_latest_block_hash_key()),
            _ => Err(AppError::Custom(format!(
                "✘ Cannot store special ETH hash of type: {}!",
                hash_type
            ))),
        }?;
        self.put_eth_hash_in_db(&key, hash)
    }

    fn get_latest_eth_block_number(&self) -> Result<usize> {
        info!("✔ Getting latest ETH block number from db...");
        match self.get_special_eth_block_from_db("latest") {
            Ok(result) => Ok(result.get_block_number()?.as_usize()),
            Err(e) => Err(e),
        }
    }

    fn get_latest_eth_block_timestamp(&self) -> Result<u64> {
        info!("✔ Getting latest ETH block number from db...");
        match self.get_special_eth_block_from_db("latest") {
            Ok(result) => Ok(result.get_timestamp().as_secs()),
            Err(e) => Err(e),
        }
    }

    fn get_eth_tail_block_from_db(&self) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH tail block from db...");
        self.get_special_eth_block_from_db("tail")
    }

    fn get_eth_latest_block_from_db(&self) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH latest block from db...");
        self.get_special_eth_block_from_db("latest")
    }

    fn get_eth_anchor_block_from_db(&self) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH anchor block from db...");
        self.get_special_eth_block_from_db("anchor")
    }

    fn get_eth_canon_block_from_db(&self) -> Result<EthSubmissionMaterial> {
        info!("✔ Getting ETH canon block from db...");
        self.get_special_eth_block_from_db("canon")
    }

    fn get_eth_anchor_block_hash_from_db(&self) -> Result<EthHash> {
        info!("✔ Getting ETH anchor block hash from db...");
        self.get_special_eth_hash_from_db("anchor")
    }

    fn get_special_eth_hash_from_db(&self, hash_type: &str) -> Result<EthHash> {
        let key = match hash_type {
            "linker" => Ok(self.get_eth_linker_hash_key()),
            "canon" => Ok(self.get_eth_canon_block_hash_key()),
            "tail" => Ok(self.get_eth_tail_block_hash_key()),
            "anchor" => Ok(self.get_eth_anchor_block_hash_key()),
            "latest" => Ok(self.get_eth_latest_block_hash_key()),
            _ => Err(AppError::Custom(format!(
                "✘ Cannot get ETH special hash of type: {}!",
                hash_type
            ))),
        }?;
        debug!("✔ Getting special ETH hash from db of type: {}", hash_type);
        self.get_eth_hash_from_db(&key.to_vec())
    }

    #[cfg(test)]
    fn get_special_hashes(&self) -> SpecialHashes {
        SpecialHashes {
            linker: self.get_linker_hash_or_genesis_hash().unwrap(),
            tail: self.get_special_eth_hash_from_db("tail").unwrap(),
            canon: self.get_special_eth_hash_from_db("canon").unwrap(),
            anchor: self.get_special_eth_hash_from_db("anchor").unwrap(),
            latest: self.get_special_eth_hash_from_db("latest").unwrap(),
        }
    }

    fn get_eth_hash_from_db(&self, key: &[Byte]) -> Result<EthHash> {
        debug!("✔ Getting ETH hash from db under key: {}", hex::encode(key));
        self.get_db()
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(|bytes| EthHash::from_slice(&self.normalize_key(bytes)))
    }

    fn get_special_eth_block_from_db(&self, block_type: &str) -> Result<EthSubmissionMaterial> {
        self.get_special_eth_hash_from_db(block_type)
            .and_then(|block_hash| self.get_submission_material_from_db(&block_hash))
    }

    fn put_eth_hash_in_db(&self, key: &[Byte], eth_hash: &EthHash) -> Result<()> {
        self.get_db().put(
            key.to_vec(),
            self.normalize_key(convert_h256_to_bytes(*eth_hash)),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn eth_block_exists_in_db(&self, block_hash: &EthHash) -> bool {
        info!(
            "✔ Checking for existence of ETH block: {}",
            hex::encode(block_hash.as_bytes())
        );
        self.key_exists_in_db(
            &self.normalize_key(block_hash.as_bytes().to_vec()),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn put_eth_submission_material_in_db(&self, eth_submission_material: &EthSubmissionMaterial) -> Result<()> {
        let key = self.normalize_key(convert_h256_to_bytes(eth_submission_material.get_block_hash()?));
        debug!("✔ Adding block to database under key: {:?}", hex::encode(&key));
        self.get_db().put(
            key,
            eth_submission_material.remove_block().to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn maybe_get_parent_eth_submission_material(&self, block_hash: &EthHash) -> Option<EthSubmissionMaterial> {
        trace!("✔ Maybe getting parent ETH block from db...");
        self.maybe_get_nth_ancestor_eth_submission_material(block_hash, 1)
            .ok()?
    }

    fn maybe_get_nth_ancestor_eth_submission_material(
        &self,
        block_hash: &EthHash,
        n: u64,
    ) -> Result<Option<EthSubmissionMaterial>> {
        trace!("✔ Getting {}th ancestor ETH block from db...", n);
        match self.maybe_get_eth_submission_material_from_db(block_hash) {
            None => Ok(None),
            Some(block_and_receipts) => match n {
                0 => Ok(Some(block_and_receipts)),
                _ => self.maybe_get_nth_ancestor_eth_submission_material(&block_and_receipts.get_parent_hash()?, n - 1),
            },
        }
    }

    fn maybe_get_eth_submission_material_from_db(&self, block_hash: &EthHash) -> Option<EthSubmissionMaterial> {
        let key = self.normalize_key(convert_h256_to_bytes(*block_hash));
        trace!(
            "✔ Maybe getting ETH block and receipts from db under hash: {}",
            hex::encode(&key)
        );
        match self.get_db().get(key, MIN_DATA_SENSITIVITY_LEVEL) {
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

    fn reverse_endianess(bytes: Bytes) -> Bytes {
        trace!("Reversing endianess of bytes: 0x{}", hex::encode(&bytes));
        // NOTE: We switch the endianness of the block hash for EVM bridges
        // to avoid DB collisions w/ ETH<->ETH bridges.
        let mut reversed_bytes = bytes;
        reversed_bytes.reverse();
        reversed_bytes.to_vec()
    }

    fn normalize_key(&self, key: Bytes) -> Bytes {
        if self.get_is_for_evm() {
            Self::reverse_endianess(key)
        } else {
            key
        }
    }

    fn get_submission_material_from_db(&self, block_hash: &EthHash) -> Result<EthSubmissionMaterial> {
        debug!("✔ Getting ETH block and receipts from db...");
        self.get_db()
            .get(
                self.normalize_key(convert_h256_to_bytes(*block_hash)),
                MIN_DATA_SENSITIVITY_LEVEL,
            )
            .and_then(|bytes| EthSubmissionMaterial::from_bytes(&bytes))
    }

    fn key_exists_in_db(&self, key: &[Byte], sensitivity: DataSensitivity) -> bool {
        debug!("✔ Checking for existence of key: {}", hex::encode(key));
        self.get_db().get(key.to_vec(), sensitivity).is_ok()
    }

    fn put_eth_gas_price_in_db(&self, gas_price: u64) -> Result<()> {
        debug!("✔ Putting ETH gas price of {} in db...", gas_price);
        self.get_db().put(
            self.get_eth_gas_price_key(),
            gas_price.to_le_bytes().to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_eth_gas_price_from_db(&self) -> Result<u64> {
        debug!("✔ Getting ETH gas price from db...");
        self.get_db()
            .get(self.get_eth_gas_price_key(), MIN_DATA_SENSITIVITY_LEVEL)
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

    fn put_eth_account_nonce_in_db(&self, nonce: u64) -> Result<()> {
        debug!("✔ Putting ETH account nonce of {} in db...", nonce);
        put_u64_in_db(self.get_db(), &self.get_eth_account_nonce_key(), nonce)
    }

    fn get_eth_account_nonce_from_db(&self) -> Result<u64> {
        debug!("✔ Getting ETH account nonce from db...");
        get_u64_from_db(self.get_db(), &self.get_eth_account_nonce_key())
    }

    fn increment_eth_account_nonce_in_db(&self, amount_to_increment_by: u64) -> Result<()> {
        debug!("✔ Incrementing ETH account nonce in db...");
        self.get_eth_account_nonce_from_db()
            .and_then(|nonce| self.put_eth_account_nonce_in_db(nonce + amount_to_increment_by))
    }

    fn put_eth_chain_id_in_db(&self, chain_id: &EthChainId) -> Result<()> {
        info!("✔ Putting `EthChainId` in db: {}", chain_id);
        self.get_db().put(
            self.get_eth_chain_id_key(),
            chain_id.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_eth_chain_id_from_db(&self) -> Result<EthChainId> {
        debug!("✔ Getting ETH `chain_id` from db...");
        self.get_db()
            .get(self.get_eth_chain_id_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|ref bytes| EthChainId::from_bytes(bytes))
    }

    fn put_eth_private_key_in_db(&self, pk: &EthPrivateKey) -> Result<()> {
        debug!("✔ Putting ETH private key in db...");
        pk.write_to_database(self.get_db(), &self.get_eth_private_key_db_key())
    }

    fn get_eth_private_key_from_db(&self) -> Result<EthPrivateKey> {
        debug!("✔ Getting ETH private key from db...");
        self.get_db()
            .get(self.get_eth_private_key_db_key(), MAX_DATA_SENSITIVITY_LEVEL)
            .and_then(|pk_bytes| {
                let mut array = [0; 32];
                array.copy_from_slice(&pk_bytes[..32]);
                EthPrivateKey::from_slice(&array)
            })
    }

    fn get_eos_on_eth_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        info!("✔ Getting 'EOS_ON_ETH' smart-contract address from db...");
        Ok(self
            .get_eth_address_from_db(&self.get_eos_on_eth_smart_contract_address_key())
            .unwrap_or_else(|_| EthAddress::zero()))
    }

    fn get_int_on_eos_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        info!("✔ Getting 'INT-on-EOS' smart-contract address from db...");
        self.get_eth_address_from_db(&self.get_int_on_eos_smart_contract_address_key())
            .map_err(|_| "No `INT-on-EOS` smart-contract address in DB! Did you forget to set it?".into())
    }

    fn put_int_on_eos_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        match self.get_int_on_eos_smart_contract_address_from_db() {
            Ok(address) => Err(format!("`INT-on-EOS smart contract set already 0x{}!", hex::encode(address)).into()),
            _ => {
                info!("✔ Putting `INT-on-EOS` smart-contract address in db...");
                self.put_eth_address_in_db(&self.get_int_on_eos_smart_contract_address_key(), address)
            },
        }
    }

    fn get_erc777_proxy_contract_address_from_db(&self) -> Result<EthAddress> {
        debug!("✔ Getting ERC777 proxy contract address from db...");
        match self
            .get_db()
            .get(self.get_erc777_proxy_contract_address_key(), MIN_DATA_SENSITIVITY_LEVEL)
        {
            Ok(address_bytes) => Ok(EthAddress::from_slice(&address_bytes[..])),
            Err(_) => {
                debug!("✘ No ERC777 proxy address in db, defaulting to zero ETH address!");
                Ok(EthAddress::zero())
            },
        }
    }

    #[allow(dead_code)]
    fn put_erc777_proxy_contract_address_in_db(&self, proxy_contract_address: &EthAddress) -> Result<()> {
        debug!("✔ Putting ERC777 proxy contract address in db...");
        self.put_eth_address_in_db(&self.get_erc777_proxy_contract_address_key(), proxy_contract_address)
    }

    fn get_btc_on_eth_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        info!("✔ Getting ETH ERC777 smart-contract address from db...");
        self.get_eth_address_from_db(&self.get_btc_on_eth_smart_contract_address_key())
            .map_err(|_| "No ERC777 contract address in DB! Did you forget to set it?".into())
    }

    fn get_btc_on_int_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        // NOTE: This is just an alias!
        info!("✔ Getting `BtcOnInt` ETH ERC777 smart-contract address from db...");
        self.get_btc_on_eth_smart_contract_address_from_db()
    }

    fn put_btc_on_eth_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        // NOTE: This is just an alias!
        match self.get_btc_on_int_smart_contract_address_from_db() {
            Ok(address) => Err(format!("ERC777 address already set to 0x{}!", hex::encode(address)).into()),
            _ => {
                info!("✔ Putting ETH ERC777 smart-contract address in db...");
                self.put_eth_address_in_db(&self.get_btc_on_eth_smart_contract_address_key(), address)
            },
        }
    }

    fn put_btc_on_int_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        match self.get_btc_on_eth_smart_contract_address_from_db() {
            Ok(address) => Err(format!("ERC777 address already set to 0x{}!", hex::encode(address)).into()),
            _ => {
                info!("✔ Putting ETH smart-contract address in db...");
                self.put_eth_address_in_db(&self.get_btc_on_eth_smart_contract_address_key(), address)
            },
        }
    }

    fn get_erc20_on_eos_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        info!("✔ Getting `pERC20-on-EOS` smart-contract address from db...");
        self.get_eth_address_from_db(&self.get_erc20_on_eos_smart_contract_address_key())
            .map_err(|_| "No `erc20-on-eos` vault contract address in DB! Did you forget to set it?".into())
    }

    fn put_eth_router_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        match self.get_eth_router_smart_contract_address_from_db() {
            Ok(address) => Err(format!("Router address already set to 0x{}!", hex::encode(address)).into()),
            _ => {
                info!("✔ Putting ETH router smart-contract address in db...");
                self.put_eth_address_in_db(&self.get_router_smart_contract_address_key(), address)
            },
        }
    }

    fn get_eth_router_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        info!("✔ Getting eth router smart-contract address from db...");
        self.get_eth_address_from_db(&self.get_router_smart_contract_address_key())
            .map_err(|_| "No router contract address in DB! Did you forget to set it?".into())
    }

    fn put_erc20_on_eos_smart_contract_address_in_db(&self, smart_contract_address: &EthAddress) -> Result<()> {
        match self.get_erc20_on_eos_smart_contract_address_from_db() {
            Ok(address) => Err(format!(
                "`erc20-on-eos` vault address is already set to {}!",
                hex::encode(address)
            )
            .into()),
            _ => {
                info!("✔ Putting 'ERC20-on-EOS` smart-contract address in db...");
                self.put_eth_address_in_db(
                    &self.get_erc20_on_eos_smart_contract_address_key(),
                    smart_contract_address,
                )
            },
        }
    }

    fn put_eos_on_eth_smart_contract_address_in_db(&self, smart_contract_address: &EthAddress) -> Result<()> {
        debug!("✔ Putting 'EOS_ON_ETH' smart-contract address in db...");
        self.put_eth_address_in_db(
            &self.get_eos_on_eth_smart_contract_address_key(),
            smart_contract_address,
        )
    }

    fn get_eth_address_from_db(&self, key: &[Byte]) -> Result<EthAddress> {
        self.get_db()
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(|address_bytes| EthAddress::from_slice(&address_bytes[..]))
    }

    fn get_erc20_on_evm_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        info!("✔ Getting `ERC20_ON_EVM` smart-contract address from db...");
        if self.get_is_for_evm() {
            info!("✔ DB utils are for EVM, meaning there's no vault on this side of the bridge!");
            Ok(EthAddress::zero())
        } else {
            self.get_eth_address_from_db(&self.get_erc20_on_evm_smart_contract_address_key())
                .map_err(|_| "No `erc20-on-evm` vault contract address in DB! Did you forget to set it?".into())
        }
    }

    fn put_erc20_on_evm_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        if self.get_erc20_on_evm_smart_contract_address_from_db().is_ok() {
            Err("`ERC20-on-EVM`Vault contract address already set!".into())
        } else {
            info!("✔ Putting `ERC20-on-EVM` vault contract address in db...");
            self.put_eth_address_in_db(&self.get_erc20_on_evm_smart_contract_address_key(), address)
        }
    }

    fn get_erc20_on_int_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        info!("✔ Putting `erc20-on-int` vault contract in db using `erc20-on-evm` key...");
        self.get_erc20_on_evm_smart_contract_address_from_db()
    }

    fn put_erc20_on_int_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        info!("✔ Getting `erc20-on-int` vault contract from db using `erc20-on-evm` key...");
        self.put_erc20_on_evm_smart_contract_address_in_db(address)
    }

    fn get_int_on_evm_smart_contract_address_from_db(&self) -> Result<EthAddress> {
        // NOTE: This is an alias for the `erc20-on-evm` contract address
        info!("✔ Getting `int-on-evm` smart-contract address from db...");
        if self.get_is_for_evm() {
            info!("✔ DB utils are for EVM, meaning there's no vault on this side of the bridge!");
            Ok(EthAddress::zero())
        } else {
            self.get_erc20_on_evm_smart_contract_address_from_db()
                .map_err(|_| "No `int-on-evm` vault contract address in DB! Did you forget to set it?".into())
        }
    }

    fn put_int_on_evm_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        // NOTE: This is an alias for the `erc20-on-evm` contract address
        if self.get_int_on_evm_smart_contract_address_from_db().is_ok() {
            Err("`int-on-evm` vault contract address already set!".into())
        } else {
            info!("✔ Putting `int-on-ewvm` vault contract address in db...");
            self.put_eth_address_in_db(&self.get_erc20_on_evm_smart_contract_address_key(), address)
        }
    }

    fn get_public_eth_address_from_db(&self) -> Result<EthAddress> {
        debug!("✔ Getting public ETH address from db...");
        self.get_db()
            .get(self.get_eth_address_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(|bytes| EthAddress::from_slice(&bytes))
    }

    fn put_public_eth_address_in_db(&self, eth_address: &EthAddress) -> Result<()> {
        debug!("✔ Putting public ETH address in db...");
        self.get_db().put(
            self.get_eth_address_key(),
            eth_address.as_bytes().to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn put_eth_address_in_db(&self, key: &[Byte], eth_address: &EthAddress) -> Result<()> {
        self.get_db().put(
            key.to_vec(),
            eth_address.as_bytes().to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn put_any_sender_nonce_in_db(&self, nonce: u64) -> Result<()> {
        debug!("✔ Putting AnySender nonce of {} in db...", nonce);
        put_u64_in_db(self.get_db(), &self.get_any_sender_nonce_key(), nonce)
    }

    fn get_any_sender_nonce_from_db(&self) -> Result<u64> {
        debug!("✔ Getting AnySender nonce from db...");
        Ok(
            get_u64_from_db(self.get_db(), &self.get_any_sender_nonce_key()).unwrap_or_else(|_| {
                info!("✘ Could not find `AnySender` nonce in db, defaulting to `0`");
                0
            }),
        )
    }

    fn increment_any_sender_nonce_in_db(&self, amount_to_increment_by: u64) -> Result<()> {
        debug!("✔ Incrementing AnySender nonce in db...");
        self.get_any_sender_nonce_from_db()
            .and_then(|nonce| self.put_any_sender_nonce_in_db(nonce + amount_to_increment_by))
    }

    fn get_int_on_algo_smart_contract_address(&self) -> Result<EthAddress> {
        info!("✔ Getting `int-on-algo` smart contract address from db...");
        self.get_eth_address_from_db(&self.get_int_on_algo_smart_contract_address_key())
    }

    fn put_int_on_algo_smart_contract_address_in_db(&self, address: &EthAddress) -> Result<()> {
        self.put_eth_address_in_db(&self.get_int_on_algo_smart_contract_address_key(), address)
    }

    #[cfg(test)]
    fn get_hash_from_db_via_hash_key(&self, hash_key: EthHash) -> Result<Option<EthHash>> {
        use crate::test_utils::convert_bytes_to_h256;
        match self.get_db().get(
            self.normalize_key(convert_h256_to_bytes(hash_key)),
            MIN_DATA_SENSITIVITY_LEVEL,
        ) {
            Ok(bytes) => Ok(Some(convert_bytes_to_h256(&bytes)?)),
            Err(_) => Ok(None),
        }
    }

    #[cfg(test)]
    fn put_eth_latest_block_in_db(&self, submission_material: &EthSubmissionMaterial) -> Result<()> {
        info!("✔ Putting ETH latest block in db...");
        self.put_special_eth_block_in_db(submission_material, "latest")
    }

    #[cfg(test)]
    fn get_all(&self) -> Vec<Bytes> {
        vec![
            self.get_eth_address_key(),
            self.get_eth_chain_id_key(),
            self.get_eth_gas_price_key(),
            self.get_eth_linker_hash_key(),
            self.get_any_sender_nonce_key(),
            self.get_eth_account_nonce_key(),
            self.get_eth_private_key_db_key(),
            self.get_eth_tail_block_hash_key(),
            self.get_eth_canon_block_hash_key(),
            self.get_eth_anchor_block_hash_key(),
            self.get_eth_latest_block_hash_key(),
            self.get_eth_canon_to_tip_length_key(),
            self.get_erc777_proxy_contract_address_key(),
            self.get_btc_on_eth_smart_contract_address_key(),
            self.get_eos_on_eth_smart_contract_address_key(),
            self.get_int_on_algo_smart_contract_address_key(),
            self.get_erc20_on_eos_smart_contract_address_key(),
            self.get_erc20_on_evm_smart_contract_address_key(),
        ]
    }

    #[cfg(test)]
    fn get_all_as_hex_strings(&self) -> Vec<String> {
        self.get_all().iter().map(hex::encode).collect()
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SpecialHashes {
    pub tail: EthHash,
    pub canon: EthHash,
    pub linker: EthHash,
    pub anchor: EthHash,
    pub latest: EthHash,
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::test_utils::{
        get_sample_contract_address,
        get_sample_eth_address,
        get_sample_eth_private_key,
        get_sample_eth_submission_material,
        get_sample_eth_submission_material_n,
        get_sequential_eth_blocks_and_receipts,
    };
    #[test]
    fn non_existing_key_should_not_exist_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let result = db_utils.key_exists_in_db(&ETH_ACCOUNT_NONCE_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(!result);
    }

    #[test]
    fn existing_key_should_exist_in_db() {
        let thing = vec![0xc0];
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let key = *ETH_ACCOUNT_NONCE_KEY;
        db.put(key.to_vec(), thing, MIN_DATA_SENSITIVITY_LEVEL).unwrap();
        let result = db_utils.key_exists_in_db(&ETH_ACCOUNT_NONCE_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(result);
    }

    #[test]
    fn should_put_eth_gas_price_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let gas_price = 20_000_000;
        db_utils.put_eth_gas_price_in_db(gas_price).unwrap();
        match db_utils.get_eth_gas_price_from_db() {
            Ok(gas_price_from_db) => assert_eq!(gas_price_from_db, gas_price),
            Err(e) => panic!("Error getting gas price from db: {}", e),
        }
    }

    #[test]
    fn should_put_chain_id_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let chain_id = EthChainId::Rinkeby;
        db_utils.put_eth_chain_id_in_db(&chain_id).unwrap();
        let result = db_utils.get_eth_chain_id_from_db().unwrap();
        assert_eq!(result, chain_id);
    }

    #[test]
    fn should_save_nonce_to_db_and_get_nonce_from_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let nonce = 1227;
        db_utils.put_eth_account_nonce_in_db(nonce).unwrap();
        match db_utils.get_eth_account_nonce_from_db() {
            Ok(nonce_from_db) => assert_eq!(nonce_from_db, nonce),
            Err(e) => panic!("Error getting nonce from db: {}", e),
        }
    }

    #[test]
    fn should_get_btc_on_eth_smart_contract_address_from_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let contract_address = get_sample_eth_address();
        db_utils
            .put_btc_on_eth_smart_contract_address_in_db(&contract_address)
            .unwrap();
        let result = db_utils.get_btc_on_eth_smart_contract_address_from_db().unwrap();
        assert_eq!(result, contract_address);
    }

    #[test]
    fn should_get_eth_pk_from_database() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let eth_private_key = get_sample_eth_private_key();
        db_utils.put_eth_private_key_in_db(&eth_private_key).unwrap();
        match db_utils.get_eth_private_key_from_db() {
            Ok(pk) => assert_eq!(pk, eth_private_key),
            Err(e) => panic!("Error getting eth private key from db: {}", e),
        }
    }

    #[test]
    fn should_increment_eth_account_nonce_in_db() {
        let nonce = 666;
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        db_utils.put_eth_account_nonce_in_db(nonce).unwrap();
        let amount_to_increment_by: u64 = 671;
        db_utils
            .increment_eth_account_nonce_in_db(amount_to_increment_by)
            .unwrap();
        match db_utils.get_eth_account_nonce_from_db() {
            Err(e) => panic!("Error getting nonce from db: {}", e),
            Ok(nonce_from_db) => assert_eq!(nonce_from_db, nonce + amount_to_increment_by),
        }
    }

    #[test]
    fn should_put_and_get_special_eth_hash_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let hash_type = "linker";
        let hash = get_sample_eth_submission_material_n(1)
            .unwrap()
            .get_block_hash()
            .unwrap();
        db_utils.put_special_eth_hash_in_db(hash_type, &hash).unwrap();
        match db_utils.get_special_eth_hash_from_db(hash_type) {
            Ok(hash_from_db) => assert_eq!(hash_from_db, hash),
            Err(e) => panic!("Error getting ETH special hash from db: {}", e),
        }
    }

    #[test]
    fn should_put_and_get_eth_hash_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let hash_key = vec![6u8, 6u8, 6u8];
        let hash = get_sample_eth_submission_material_n(1)
            .unwrap()
            .get_block_hash()
            .unwrap();
        db_utils.put_eth_hash_in_db(&hash_key, &hash).unwrap();
        match db_utils.get_eth_hash_from_db(&hash_key) {
            Ok(hash_from_db) => assert_eq!(hash_from_db, hash),
            Err(e) => panic!("Error getting ETH hash from db: {}", e),
        }
    }

    #[test]
    fn should_put_and_get_special_eth_block_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let block_type = "anchor";
        let submission_material = get_sample_eth_submission_material_n(1).unwrap();
        let expected_result = submission_material.remove_block();
        db_utils
            .put_special_eth_block_in_db(&submission_material, block_type)
            .unwrap();
        match db_utils.get_special_eth_block_from_db(block_type) {
            Ok(result) => assert_eq!(result, expected_result),
            Err(e) => panic!("Error getting ETH special submission_material from db: {}", e),
        }
    }

    #[test]
    fn should_get_submission_material_block_from_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let submission_material = get_sample_eth_submission_material_n(1).unwrap();
        let expected_result = submission_material.remove_block();
        let block_hash = submission_material.get_block_hash().unwrap();
        db_utils
            .put_eth_submission_material_in_db(&submission_material)
            .unwrap();
        match db_utils.get_submission_material_from_db(&block_hash) {
            Ok(result) => assert_eq!(result, expected_result),
            Err(e) => panic!("Error getting ETH submission_material from db: {}", e),
        }
    }

    #[test]
    fn should_put_eth_address_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let key = ETH_ADDRESS_KEY.to_vec();
        let eth_address = get_sample_contract_address();
        let result = db_utils.put_eth_address_in_db(&key, &eth_address);
        assert!(result.is_ok());
    }

    #[test]
    fn should_put_and_get_public_eth_address_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let eth_address = get_sample_contract_address();
        db_utils.put_public_eth_address_in_db(&eth_address).unwrap();
        match db_utils.get_public_eth_address_from_db() {
            Ok(eth_address_from_db) => assert_eq!(eth_address_from_db, eth_address),
            Err(e) => panic!("Error getting ETH address from db: {}", e),
        }
    }

    #[test]
    fn maybe_get_block_should_be_none_if_block_not_extant() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let block_hash = get_sample_eth_submission_material_n(1)
            .unwrap()
            .get_block_hash()
            .unwrap();
        if db_utils
            .maybe_get_eth_submission_material_from_db(&block_hash)
            .is_some()
        {
            panic!("Maybe getting none existing block should be 'None'");
        };
    }

    #[test]
    fn should_maybe_get_some_block_if_exists() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let submission_material = get_sample_eth_submission_material_n(1).unwrap();
        let expected_result = submission_material.remove_block();
        let block_hash = submission_material.get_block_hash().unwrap();
        db_utils
            .put_eth_submission_material_in_db(&submission_material)
            .unwrap();
        match db_utils.maybe_get_eth_submission_material_from_db(&block_hash) {
            None => panic!("`submission_material` should exist in db!"),
            Some(result) => assert_eq!(result, expected_result),
        };
    }

    #[test]
    fn should_return_none_if_no_parent_block_exists() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let block = get_sample_eth_submission_material_n(1).unwrap();
        let block_hash = block.get_block_hash().unwrap();
        db_utils.put_eth_submission_material_in_db(&block).unwrap();
        let result = db_utils.maybe_get_parent_eth_submission_material(&block_hash);
        assert!(result.is_none());
    }

    #[test]
    fn should_maybe_get_parent_block_if_it_exists() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let block = blocks[1].clone();
        let parent_block = blocks[0].clone();
        let expected_result = parent_block.remove_block();
        let block_hash = block.get_block_hash().unwrap();
        db_utils.put_eth_submission_material_in_db(&block).unwrap();
        db_utils.put_eth_submission_material_in_db(&parent_block).unwrap();
        match db_utils.maybe_get_parent_eth_submission_material(&block_hash) {
            None => panic!("Block should have parent in the DB!"),
            Some(result) => assert_eq!(result, expected_result),
        };
    }

    #[test]
    fn should_get_no_nth_ancestor_if_not_extant() {
        let ancestor_number = 3;
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let block = get_sample_eth_submission_material_n(1).unwrap();
        let block_hash = block.get_block_hash().unwrap();
        db_utils.put_eth_submission_material_in_db(&block).unwrap();
        let result = db_utils
            .maybe_get_nth_ancestor_eth_submission_material(&block_hash, ancestor_number)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn should_get_nth_ancestor_if_extant() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let block_hash = blocks[blocks.len() - 1].get_block_hash().unwrap();
        blocks
            .iter()
            .try_for_each(|block| db_utils.put_eth_submission_material_in_db(block))
            .unwrap();
        blocks.iter().enumerate().for_each(|(i, _)| {
            match db_utils
                .maybe_get_nth_ancestor_eth_submission_material(&block_hash, i as u64)
                .unwrap()
            {
                None => panic!("Ancestor number {} should exist!", i),
                Some(ancestor) => assert_eq!(ancestor, blocks[blocks.len() - i - 1].remove_block()),
            }
        });
        let result = db_utils
            .maybe_get_nth_ancestor_eth_submission_material(&block_hash, blocks.len() as u64)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn saving_submission_material_should_remove_block() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let submission_material = get_sample_eth_submission_material();
        let db_key = submission_material.get_block_hash().unwrap();
        assert!(submission_material.block.is_some());
        db_utils
            .put_eth_submission_material_in_db(&submission_material)
            .unwrap();
        let result = db_utils.get_submission_material_from_db(&db_key).unwrap();
        assert!(result.block.is_none());
    }

    #[test]
    fn should_save_submission_material_if_block_already_removed() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let submission_material = get_sample_eth_submission_material().remove_block();
        let db_key = submission_material.get_block_hash().unwrap();
        db_utils
            .put_eth_submission_material_in_db(&submission_material)
            .unwrap();
        let result = db_utils.get_submission_material_from_db(&db_key).unwrap();
        assert_eq!(result, submission_material);
    }

    #[test]
    fn eth_db_keys_should_not_match_evm_db_keys() {
        let db = get_test_database();
        let eth_keys = EthDbUtils::new(&db).get_all_as_hex_strings();
        let evm_keys = EvmDbUtils::new(&db).get_all_as_hex_strings();
        eth_keys
            .iter()
            .zip(evm_keys.iter())
            .for_each(|(eth_key, evm_key)| assert_ne!(eth_key, evm_key));
    }

    #[test]
    fn eth_keys_should_all_be_different() {
        let db = get_test_database();
        let expected_result = EthDbUtils::new(&db).get_all().len();
        let mut db_keys = EthDbUtils::new(&db).get_all();
        db_keys.sort();
        db_keys.dedup();
        let result = db_keys.len();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn evm_keys_should_all_be_different() {
        let db = get_test_database();
        let expected_result = EvmDbUtils::new(&db).get_all().len();
        let mut db_keys = EvmDbUtils::new(&db).get_all();
        db_keys.sort();
        db_keys.dedup();
        let result = db_keys.len();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_linker_or_genesis_should_get_linker_hash_from_db_if_extant() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let linker_hash = EthHash::random();
        eth_db_utils.put_eth_linker_hash_in_db(linker_hash).unwrap();
        let result = eth_db_utils.get_linker_hash_or_genesis_hash().unwrap();
        assert_eq!(result, linker_hash);
    }

    #[test]
    fn get_linker_or_genesis_should_get_genesis_hash_if_linker_not_set() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let result = eth_db_utils.get_linker_hash_or_genesis_hash().unwrap();
        let expected_result = EthHash::from_slice(&ETH_PTOKEN_GENESIS_HASH_KEY[..]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_put_and_get_eth_router_smart_contract_address_from_db() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let eth_address = EthAddress::from_slice(&hex::decode("71A440EE9Fa7F99FB9a697e96eC7839B8A1643B8").unwrap());
        eth_db_utils
            .put_eth_router_smart_contract_address_in_db(&eth_address)
            .unwrap();
        let result = eth_db_utils.get_eth_router_smart_contract_address_from_db().unwrap();
        assert_eq!(result, eth_address);
    }

    #[test]
    fn eth_db_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = EthDatabaseKeysJson {
            HOST_CORE_IS_INITIALIZED_DB_KEY:
                "0271c9a9e186967bbd36c4eb36f47a94d3771ace3879f1bbf202842c89942999".to_string(),
            NATIVE_CORE_IS_INITIALIZED_DB_KEY:
                "afa4de60dc3ab1362c4b4acf9536393ece435e3e7951363c6ea87182939271f3".to_string(),
            ETH_ANY_SENDER_NONCE_KEY:
                "09feb18750877b8b216cf9dc0bf587dfc4d043620252e1a7a33353710939c2ae".to_string(),
            ETH_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
                "f2289049ab0275224d98f6f7d6b2e5c0b301167d04b83aa724024fcad81d61fc".to_string(),
            ETH_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
                "13a27c2fe10330e66ea6c562272bcbef4e7ebd003aed087dba387ac43a7f5fd4".to_string(),
            ETH_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY:
                "fb2788804c9b7b8c40b191f4da2e4db2602a2f1deaaefc052bf1d38220db1dcf".to_string(),
            ETH_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
                "7709f182e4be2554442ffb3637f3417dd75cef4ccb13942d2e35c5d6ace6c503".to_string(),
            ETH_ERC777_PROXY_CONTRACT_ADDRESS_KEY:
                "a2e7337756b00998e6efd72220477f4de76ceac441298d6770fff827837b27a6".to_string(),
            ETH_ACCOUNT_NONCE_KEY:
                "713a7d7396c523b7978cd822839e0186395053745941615b0370c0bb72b4dcf4".to_string(),
            ETH_ADDRESS_KEY:
                "bfd203dc3411da4e18d157e87b94507a428060618fcf3163357a1fabe93fba1a".to_string(),
            ETH_ANCHOR_BLOCK_HASH_KEY:
                "1087f2e9bfa897df4da210822cc94bcf77ee11396cf9d3cd247b06aeeb289737".to_string(),
            ETH_CANON_BLOCK_HASH_KEY:
                "c737daae274d21e37403be7d3d562c493332c381ee2b0f3fa0b2286af8b8e5c2".to_string(),
            ETH_CANON_TO_TIP_LENGTH_KEY:
                "192b7e4da694bf96fbc089656a3ba0f63f6263a95af257b693e8dee84334b38c".to_string(),
            ETH_CHAIN_ID_KEY:
                "47199e3b0ffc301baeedd4eb87ebf5ef3829496c8ab2660a6038a62e36e9222f".to_string(),
            ETH_GAS_PRICE_KEY:
                "ecf932d3aca97f12884bc42af7607469feba2206e8b1d37ed1328d477c747346".to_string(),
            ETH_LATEST_BLOCK_HASH_KEY:
                "8b39bef2b5b1e9564bb4a60c8211c32e2f94dc88cae8cfbaad42b2e7e527ea7a".to_string(),
            ETH_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
                "a1552e7ee400c2adf873879fc3efefea72db11307ad3c873506e1f3be8fd31db".to_string(),
            ETH_LINKER_HASH_KEY:
                "1c045b32a91a460a8a210de0a9b757da8fc21844f02399b558c3c87917122b58".to_string(),
            ETH_PRIVATE_KEY_DB_KEY:
                "eec538cafefe65e094e2e70364da2f2f6e752209e1974e38a9b23ca8ce22b73d".to_string(),
            ETH_TAIL_BLOCK_HASH_KEY:
                "539205e110a233c64f983acf425f1d2cf6cb6535a0241a3722a512690eeba758".to_string(),
            ETH_PTOKEN_GENESIS_HASH_KEY:
                "7eb2e65416dd107602495454d1ed094ae475cff2f3bfb2e2ae68a1c52bc0d66f".to_string(),
            ETH_ROUTER_SMART_CONTRACT_ADDRESS_KEY:
                "7e4ba9ad69fafede39d72a5e5d05953c4261d16ede043978031bc425d2e3b1d2".to_string(),
            ETH_INT_ON_EOS_SMART_CONTRACT_ADDRESS_KEY:
                "a4f5ee205467c1b48132db5fa365b15f18c4ef0eec315ed23c3a0947acd4f93a".to_string(),
            ETH_INT_ON_ALGO_SMART_CONTRACT_ADDRESS_KEY:
                "3eb7085064ca52e24da4e7cc9627806897eac04dbf6534f79d8b2be9ed69eba7".to_string(),
        };
        let result = EthDatabaseKeysJson::new();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn evm_db_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = EvmDatabaseKeysJson {
            HOST_CORE_IS_INITIALIZED_DB_KEY:
                "0271c9a9e186967bbd36c4eb36f47a94d3771ace3879f1bbf202842c89942999".to_string(),
            NATIVE_CORE_IS_INITIALIZED_DB_KEY:
                "afa4de60dc3ab1362c4b4acf9536393ece435e3e7951363c6ea87182939271f3".to_string(),
            EVM_ACCOUNT_NONCE_KEY:
               "ca7f0ab19900680d76625f41854791660729bfcaf7fede763d96d4c05916ec4c".to_string(),
            EVM_ADDRESS_KEY:
               "a1e0ede222d5df7500e8580bdf0f552b55e4f95a5a1585b059adbd1fab061d73".to_string(),
            EVM_ANCHOR_BLOCK_HASH_KEY:
               "0a28ac19c3f6ed77642240975ff3d553290e62785b9070e81fad38012d346bae".to_string(),
            EVM_ANY_SENDER_NONCE_KEY:
               "960d6c59b7c81545d0fcedd4a4e84102b306bef422b6f06b38c452df19b0673f".to_string(),
            EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
               "1a2270b3479ad2a676751ecbf17c8468ab64854d265d1ba8107e042e70a5c422".to_string(),
            EVM_CANON_BLOCK_HASH_KEY:
               "bc262de20ac1da20589be1d2464e9658bf9d5ab193ad65ff5be69008bbbc8ee2".to_string(),
            EVM_CANON_TO_TIP_LENGTH_KEY:
               "2ee78935508a7ae8327e1ec867d23813042f70e78ac5dafa05d00ed3a81eb7d7".to_string(),
            EVM_CHAIN_ID_KEY:
               "b302d7601e077a277f2d1e100c959ba2d63989531b47468bbeef4c9faa57d3c9".to_string(),
            EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
               "3afdaa0cf2f37afa64f93623c3b25778c9cde2f6a71af4818c78ab54c4731144".to_string(),
            EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY:
               "e06e403795bcba77bcaa7ae8e22a7149e69c7fe8eb7db5e81e4c80a268594fdb".to_string(),
            EVM_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
               "a7e4cd0d8bf1e96eaff6b8f74cb8786c834330f34cf209597ca988f5d724b4a7".to_string(),
            EVM_ERC777_PROXY_CONTRACT_ADDRESS_KEY:
               "0e5e8342356bb9f5b6f6b1a681c544c12838053a450bb97bed1d3a7a8e9a86ec".to_string(),
            EVM_GAS_PRICE_KEY:
               "b4dbeaf50ce099e52bd74571377dc97df7f25db7b981babcea4c0292035f58ba".to_string(),
            EVM_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
               "a1552e7ee400c2adf873879fc3efefea72db11307ad3c873506e1f3be8fd31db".to_string(),
            EVM_LATEST_BLOCK_HASH_KEY:
               "9a4dd10e7fc05b39c5c66698d808005e9bc678bf3d7816741b25ddddf93092a7".to_string(),
            EVM_LINKER_HASH_KEY:
               "b4ed69606ec2498bc6f8ea41a8ec6f46181d36617966c5083345115e0b7b964c".to_string(),
            EVM_PRIVATE_KEY_DB_KEY:
               "fa8338b621f949093c2880563aa678a8407ce0c78c1d75b9fec11768b042eba7".to_string(),
            EVM_PTOKEN_GENESIS_HASH_KEY:
               "2571ca7ce4ca58cbd74f2ec4d971bc90925a9c2305481798bab1a8a7e7ad67bc".to_string(),
            EVM_TAIL_BLOCK_HASH_KEY:
               "0bfa597048f0580d7782b60c89e596410b708ed843c5391f53fbfd6e947bccb4".to_string(),
            EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY:
                "7e4ba9ad69fafede39d72a5e5d05953c4261d16ede043978031bc425d2e3b1d2".to_string(),
            EVM_INT_ON_EOS_SMART_CONTRACT_ADDRESS_KEY:
                "a4f5ee205467c1b48132db5fa365b15f18c4ef0eec315ed23c3a0947acd4f93a".to_string(),
            EVM_INT_ON_ALGO_SMART_CONTRACT_ADDRESS_KEY:
                "edeeba4d99df6ec7bce9d35ca7e1f4affc13f5bdd637e59b979a7c980a5fa0af".to_string(),
        };
        let result = EvmDatabaseKeysJson::new();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn eth_router_smart_contract_addres_key_should_match_evm_router_smart_contract_address_key() {
        assert_eq!(
            *ETH_ROUTER_SMART_CONTRACT_ADDRESS_KEY,
            *EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY
        );
    }

    #[test]
    fn db_keys_should_have_no_collisions() {
        let db = get_test_database();
        let mut keys = vec![
            EthDbUtils::new(&db).get_all_as_hex_strings(),
            EvmDbUtils::new(&db).get_all_as_hex_strings(),
        ]
        .concat();
        let num_keys_before = keys.len();
        keys.sort();
        keys.dedup();
        let num_keys_after = keys.len();
        assert_eq!(num_keys_before, num_keys_after);
    }
}
