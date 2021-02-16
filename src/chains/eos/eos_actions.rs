#![allow(non_snake_case)]
use std::str::FromStr;

use eos_primitives::{AccountName as EosAccountName, Asset as EosAsset, NumBytes, Read, SerializeData, Write};

use crate::types::Bytes;

#[eosio_core_root_path = "::eos_primitives"]
#[derive(Clone, Debug, Default, Read, Write, NumBytes)]
pub struct PTokenMintAction {
    pub to: EosAccountName,
    pub quantity: EosAsset,
    pub memo: String,
}

impl PTokenMintAction {
    pub fn new(to: EosAccountName, quantity: EosAsset, memo: &str) -> Self {
        PTokenMintAction {
            to,
            quantity,
            memo: memo.into(),
        }
    }

    pub fn from_str(to: &str, quantity: &str, memo: &str) -> crate::Result<Self> {
        Ok(Self::new(
            EosAccountName::from_str(to)?,
            EosAsset::from_str(quantity)?,
            memo,
        ))
    }
}

#[derive(Clone, Debug, Read, Write, NumBytes, Default)]
#[eosio_core_root_path = "::eos_primitives"]
pub struct PTokenPegOutAction {
    pub tokenContract: EosAccountName,
    pub quantity: EosAsset,
    pub recipient: EosAccountName,
    pub metadata: Bytes,
}

impl PTokenPegOutAction {
    pub fn from_str(token_contract: &str, quantity: &str, recipient: &str, metadata: &[u8]) -> crate::Result<Self> {
        Ok(Self {
            metadata: metadata.to_vec(),
            quantity: EosAsset::from_str(quantity)?,
            recipient: EosAccountName::from_str(recipient)?,
            tokenContract: EosAccountName::from_str(token_contract)?,
        })
    }
}

impl SerializeData for PTokenMintAction {}
impl SerializeData for PTokenPegOutAction {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_ptoken_mint_action_from_str() {
        let result = PTokenMintAction::from_str("whateverxxx", "1.000 EOS", "a memo");
        assert!(result.is_ok());
    }

    #[test]
    fn should_crate_ptoken_peg_out_action_from_str() {
        let result =
            PTokenPegOutAction::from_str("whateverxxx", "1.000 EOS", "whateveryyyy", &vec![0x1, 0x3, 0x3, 0x7]);
        assert!(result.is_ok());
    }
}
