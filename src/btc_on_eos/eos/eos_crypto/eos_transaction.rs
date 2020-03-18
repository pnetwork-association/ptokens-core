use eos_primitives::{
    SerializeData,
    ActionTransfer,
    PermissionLevel,
    ActionPTokenMint,
    Action as EosAction,
    Transaction as EosTransaction,
};
use crate::btc_on_eos::{
    types::Result,
    eos::{
        eos_types::EosSignedTransaction,
        eos_crypto::eos_private_key::EosPrivateKey,
        eos_constants::{
            PBTC_TOKEN_NAME,
            PBTC_MINT_FXN_NAME,
        },
    },
};

fn get_peos_permission_level(
    actor: &str,
    permission_level: &str,
) -> Result<PermissionLevel> {
    Ok(PermissionLevel::from_str(actor, permission_level)?)
}

fn get_peos_transfer_action(
    to: &str,
    _from: &str,
    memo: &str,
    amount: &str,
) -> Result<ActionPTokenMint> {
    Ok(ActionPTokenMint::from_str(to, amount, memo)?)
}

fn get_eos_minting_action(
    to: &str,
    from: &str,
    memo: &str,
    actor: &str,
    amount: &str,
    permission_level: &str,
) -> Result<EosAction> {
    Ok(
        EosAction::from_str(
            "pbtctokenxxx",//PBTC_TOKEN_NAME, // this same as actor etc? FIXME!
            "issue",//PBTC_MINT_FXN_NAME,
            vec![get_peos_permission_level(actor, permission_level)?],
            get_peos_transfer_action(to, from, memo, amount)?,
        )?
    )
}

pub fn get_unsigned_peos_transaction(
    to: &str,
    from: &str,
    memo: &str,
    actor: &str,
    amount: &str,
    ref_block_num: u16,
    ref_block_prefix: u32,
    seconds_from_now: u32,
    permission_level: &str,
) -> Result<EosTransaction> {
    Ok(
        EosTransaction::new(
            seconds_from_now,
            ref_block_num,
            ref_block_prefix,
            vec![
                get_eos_minting_action(
                    to,
                    from,
                    memo,
                    actor,
                    amount,
                    permission_level,
                )?
            ]
        )
    )
}

pub fn sign_peos_transaction(
    to: &str,
    amount: &str,
    chain_id: &str,
    eos_private_key: &EosPrivateKey,
    unsigned_transaction: &EosTransaction,
) -> Result<EosSignedTransaction> {
    Ok(
        EosSignedTransaction::new(
            format!(
                "{}",
                eos_private_key
                    .sign_message_bytes(
                        &unsigned_transaction.get_signing_data(chain_id)?
                    )?
            ),
            hex::encode(
                &unsigned_transaction.to_serialize_data()[..]
            ).to_string(),
            to.to_string(),
            amount.to_string(),
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::eos::{
        eos_test_utils::{
            EOS_JUNGLE_CHAIN_ID,
            get_sample_eos_private_key_2,
        },
        eos_constants::{
            MEMO,
            PEOS_ACCOUNT_NAME,
            PEOS_ACCOUNT_ACTOR,
            EOS_MAX_EXPIRATION_SECS,
            PEOS_ACCOUNT_PERMISSION_LEVEL,
        },
    };

    #[test]
    fn should_sign_minting_tx_correctly() {
        let to = "provtestable";
        let amount = "1.00000042 PFFF";
        let ref_block_num = 44391;
        let ref_block_prefix = 1355491504;
        let unsigned_transaction = get_unsigned_peos_transaction(
            to,
            PEOS_ACCOUNT_NAME,
            MEMO,
            PEOS_ACCOUNT_ACTOR,
            amount,
            ref_block_num,
            ref_block_prefix,
            EOS_MAX_EXPIRATION_SECS,
            PEOS_ACCOUNT_PERMISSION_LEVEL,
        ).unwrap();
        let pk = EosPrivateKey::from_slice(
            &hex::decode(
            "0bc331469a2c834b26ff3af7a72e3faab3ee806c368e7a8008f57904237c6057"
            ).unwrap()
        ).unwrap();
        let result = sign_peos_transaction(
            to,
            amount,
            EOS_JUNGLE_CHAIN_ID,
            &pk,
            &unsigned_transaction,
        )
            .unwrap()
            .transaction;
        // NOTE: First 4 bytes are the timestamp (8 hex chars...)
        // NOTE: Signature not deterministic ∴ we don't test it.
        // NOTE: Real tx broadcast here: https://jungle.bloks.io/transaction/45c8e6256a3e380b455648d43d0d10ffc8278c3bf428508b8de8e4e3155f7957
        let expected_result = "67adb028cb500000000001d07b9f0ad28cf2a90000000000a5317601d07b9f0ad28cf2a900000000a8ed32322ea0e23119abbce9ad2ae1f50500000000085046464600000015425443202d3e207042544320636f6d706c6574652100".to_string();
        let result_without_timestamp = &result[8..];
        assert_eq!(result_without_timestamp, expected_result);
    }
}
