use common::Bytes;
use common_chain_ids::EthChainId;
use common_eth::{encode_fxn_call, EthPrivateKey, EthTransaction};
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, U256};

use super::{Challenge, ChallengeResponseSignatureInfo, ChallengesError};
use crate::{ActorType, NetworkId};

const RESPONSE_FXN_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"actor\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"challenger\",\"type\":\"address\"},{\"internalType\":\"enum IPNetworkHub.ActorTypes\",\"name\":\"actorType\",\"type\":\"uint8\"},{\"internalType\":\"uint64\",\"name\":\"timestamp\",\"type\":\"uint64\"},{\"internalType\":\"bytes4\",\"name\":\"networkId\",\"type\":\"bytes4\"}],\"internalType\":\"struct IPNetworkHub.Challenge\",\"name\":\"challenge\",\"type\":\"tuple\"},{\"internalType\":\"enum IPNetworkHub.ActorTypes\",\"name\":\"actorType\",\"type\":\"uint8\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\"}],\"name\":\"solveChallenge\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

impl Challenge {
    pub fn check_affordability(&self, balance: U256, gas_limit: usize, gas_price: u64) -> Result<(), ChallengesError> {
        let cost = Self::get_tx_cost(gas_limit, gas_price);
        if balance > cost {
            Ok(())
        } else {
            Err(ChallengesError::InsufficientBalance {
                have: balance,
                need: cost,
            })
        }
    }

    fn get_tx_cost(gas_limit: usize, gas_price: u64) -> U256 {
        U256::from(gas_limit as u64 * gas_price)
    }

    fn encode_solve_challenge_fxn_data(
        &self,
        sig_info: &ChallengeResponseSignatureInfo,
    ) -> Result<Bytes, ChallengesError> {
        debug!("encoding `solveChallenge` fxn data...");
        let challenge = self.to_eth_abi_token()?;
        let actor_type = EthAbiToken::Uint(ActorType::Sentinel.into());
        let inclusion_proof = EthAbiToken::from(sig_info.proof());
        let signature: EthAbiToken = sig_info.sig().clone().into();

        let r = encode_fxn_call(RESPONSE_FXN_ABI, "solveChallenge", &[
            challenge,
            actor_type,
            inclusion_proof,
            signature,
        ])?;

        Ok(r)
    }

    pub fn to_solve_challenge_tx(
        self,
        nonce: u64,
        gas_price: u64,
        gas_limit: usize,
        network_id: &NetworkId,
        pnetwork_hub: &EthAddress,
        broadcaster_pk: &EthPrivateKey,
        sig_info: &ChallengeResponseSignatureInfo,
    ) -> Result<EthTransaction, ChallengesError> {
        let value = 0;
        let ecid = EthChainId::try_from(network_id)?;
        let data = self.encode_solve_challenge_fxn_data(sig_info)?;
        debug!("nonce: {nonce}");
        debug!("gas_price: {gas_price}");
        debug!("gas_limit: {gas_limit}");
        debug!("pnetwork_hub: {pnetwork_hub}");
        debug!("eth_chain_id: {ecid}");
        debug!("tx signer: {}", broadcaster_pk.to_address());
        debug!("cancellation sig: {}", sig_info.sig().clone().to_string());
        Ok(
            EthTransaction::new_unsigned(data, nonce, value, *pnetwork_hub, &ecid, gas_limit, gas_price)
                .sign(broadcaster_pk)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::test_utils::get_sample_challenge;

    #[test]
    fn should_encode_challenge_data() {
        let challenge = get_sample_challenge();
        let sig_info = ChallengeResponseSignatureInfo::default();
        let result = challenge.encode_solve_challenge_fxn_data(&sig_info);
        assert!(result.is_ok());
    }
}
