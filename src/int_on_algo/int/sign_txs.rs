use rust_algorand::{
    AlgorandAddress,
    AlgorandApplicationArg,
    AlgorandHash,
    AlgorandKeys,
    AlgorandTransaction,
    AlgorandTxGroup,
    MicroAlgos,
};

use crate::{
    chains::{
        algo::{
            algo_constants::ALGO_MAX_FOREIGN_ITEMS,
            algo_signed_group_txs::{AlgoSignedGroupTx, AlgoSignedGroupTxs},
            algo_user_data::AlgoUserData,
        },
        eth::eth_state::EthState,
    },
    int_on_algo::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos},
    metadata::metadata_traits::ToMetadata,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

impl IntOnAlgoAlgoTxInfo {
    fn maybe_to_metadata_bytes(&self) -> Result<Option<Bytes>> {
        let metadata_bytes = if self.user_data.is_empty() {
            vec![]
        } else {
            self.to_metadata_bytes()?
        };
        if metadata_bytes.is_empty() {
            debug!("✔ No user data ∴ not wrapping in metadata!");
            Ok(None)
        } else {
            debug!("✔ Signing with metadata : 0x{}", hex::encode(&metadata_bytes));
            Ok(Some(metadata_bytes))
        }
    }

    fn get_asset_transfer_tx(
        &self,
        fee: &MicroAlgos,
        first_valid: u64,
        genesis_hash: &AlgorandHash,
        sender: &AlgorandAddress,
        last_valid: Option<u64>,
    ) -> Result<AlgorandTransaction> {
        Ok(AlgorandTransaction::asset_transfer(
            self.algo_asset_id,
            *fee,
            self.host_token_amount.as_u64(),
            self.maybe_to_metadata_bytes()?,
            first_valid,
            *sender,
            *genesis_hash,
            last_valid,
            self.issuance_manager_app_id.to_address()?,
        )?)
    }

    fn to_user_peg_in_signed_group_tx(
        &self,
        fee: &MicroAlgos,
        first_valid: u64,
        genesis_hash: &AlgorandHash,
        sender: &AlgorandAddress,
        private_key: &AlgorandKeys,
    ) -> Result<AlgoSignedGroupTx> {
        info!(
            "✔ Signing ALGO group transaction for a user peg-in with tx info: {:?}",
            self
        );
        let last_valid = None;

        // NOTE: First we transfer the asset in question to the issuance manager app...
        let asset_transfer_tx = self.get_asset_transfer_tx(fee, first_valid, genesis_hash, sender, last_valid)?;

        // NOTE: Next we call the issuance manager app, with the ASA in question as one of
        // the foreign assets, and the final destination (as set by the user) as an account.
        // In this case, the app is simply a forwarder, and so completes the asset transfer
        // to the final user address.
        let foreign_apps = None;
        let destination_address = self.get_destination_address()?;
        let accounts = Some(vec![destination_address]);
        let foreign_assets = Some(vec![self.algo_asset_id]);
        let application_args = Some(vec![
            AlgorandApplicationArg::from("issue"),
            AlgorandApplicationArg::from(destination_address),
        ]);
        let app_call_tx = AlgorandTransaction::application_call_noop(
            self.issuance_manager_app_id.to_u64(),
            *fee,
            first_valid,
            *sender,
            *genesis_hash,
            last_valid,
            application_args,
            accounts,
            foreign_apps,
            foreign_assets,
        )?;

        let group_tx = AlgorandTxGroup::new(&vec![asset_transfer_tx, app_call_tx])?;

        Ok(AlgoSignedGroupTx::new(
            group_tx.sign_transactions(&[private_key])?,
            group_tx,
        ))
    }

    fn to_application_peg_in_signed_group_tx(
        &self,
        fee: &MicroAlgos,
        first_valid: u64,
        genesis_hash: &AlgorandHash,
        sender: &AlgorandAddress,
        private_key: &AlgorandKeys,
    ) -> Result<AlgoSignedGroupTx> {
        info!(
            "✔ Signing ALGO group transaction for an application peg-in with tx info: {:?}",
            self
        );
        let last_valid = None;

        // NOTE: First we transfer the asset in question to the issuance manager app...
        let asset_transfer_tx = self.get_asset_transfer_tx(fee, first_valid, genesis_hash, sender, last_valid)?;

        // NOTE: Now we assemble the ingredients for the application call tx...
        let destination_app_id = self.get_destination_app_id()?;
        let destination_address = destination_app_id.to_address()?;

        // NOTE: The user may have encoded some foreign accounts/apps into the `user_data` field...
        let decoded_user_data = AlgoUserData::from_bytes(&self.user_data).unwrap_or_default();
        let mut foreign_assets = [vec![self.algo_asset_id], decoded_user_data.to_asset_ids()]
            .concat()
            .to_vec();
        let mut foreign_accounts = [vec![destination_address], decoded_user_data.to_addresses()]
            .concat()
            .to_vec();
        let mut foreign_apps = [vec![destination_app_id.to_u64()], decoded_user_data.to_app_ids()]
            .concat()
            .to_vec();
        let application_args = Some(vec![
            AlgorandApplicationArg::from("issue"),
            AlgorandApplicationArg::from(destination_app_id.to_u64()),
        ]);

        // NOTE: Now we truncate to ensure we're not provisioning too many foreign items...
        foreign_apps.truncate(ALGO_MAX_FOREIGN_ITEMS);
        foreign_assets.truncate(ALGO_MAX_FOREIGN_ITEMS);
        foreign_accounts.truncate(ALGO_MAX_FOREIGN_ITEMS);

        // NOTE: Next we call the issuance manager app, with the ASA in question as one of
        // the foreign assets, and the final destination (as set by the user) as a foreign
        // account. In this case, the application will forward the ASA to the destination,
        // and call a hook in that application with the provided metadata (if extant).
        let app_call_tx = AlgorandTransaction::application_call_noop(
            self.issuance_manager_app_id.to_u64(),
            *fee,
            first_valid,
            *sender,
            *genesis_hash,
            last_valid,
            application_args,
            if foreign_accounts.is_empty() {
                None
            } else {
                Some(foreign_accounts)
            },
            if foreign_apps.is_empty() {
                None
            } else {
                Some(foreign_apps)
            },
            if foreign_assets.is_empty() {
                None
            } else {
                Some(foreign_assets)
            },
        )?;

        let group_tx = AlgorandTxGroup::new(&vec![asset_transfer_tx, app_call_tx])?;

        Ok(AlgoSignedGroupTx::new(
            group_tx.sign_transactions(&[private_key])?,
            group_tx,
        ))
    }

    pub fn to_algo_signed_group_tx(
        &self,
        fee: &MicroAlgos,
        first_valid: u64,
        genesis_hash: &AlgorandHash,
        sender: &AlgorandAddress,
        private_key: &AlgorandKeys,
    ) -> Result<AlgoSignedGroupTx> {
        if self.destination_is_app() {
            self.to_application_peg_in_signed_group_tx(fee, first_valid, genesis_hash, sender, private_key)
        } else {
            self.to_user_peg_in_signed_group_tx(fee, first_valid, genesis_hash, sender, private_key)
        }
    }
}

impl IntOnAlgoAlgoTxInfos {
    pub fn to_algo_signed_group_tx(
        &self,
        fee: &MicroAlgos,
        first_valid: u64,
        genesis_hash: &AlgorandHash,
        sender: &AlgorandAddress,
        private_key: &AlgorandKeys,
    ) -> Result<AlgoSignedGroupTxs> {
        info!("✔ Signing `erc20-on-int` INT transactions...");
        Ok(AlgoSignedGroupTxs::new(
            self.iter()
                .enumerate()
                .map(|(i, info)| {
                    info.to_algo_signed_group_tx(fee, first_valid + i as u64, genesis_hash, sender, private_key)
                })
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

pub fn maybe_sign_algo_txs_and_add_to_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    let tx_infos = state.int_on_algo_algo_tx_infos.clone();
    if tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no ALGO transactions to sign!");
        Ok(state)
    } else {
        tx_infos
            .to_algo_signed_group_tx(
                &state.algo_db_utils.get_algo_fee()?,
                state.get_eth_submission_material()?.get_algo_first_valid_round()?,
                &state.algo_db_utils.get_genesis_hash()?,
                &state.algo_db_utils.get_redeem_address()?,
                &state.algo_db_utils.get_algo_private_key()?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_algo_signed_group_txs(&signed_txs)
            })
    }
}