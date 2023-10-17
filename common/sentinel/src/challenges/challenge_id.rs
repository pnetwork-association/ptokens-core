use common::crypto_utils::keccak_hash_bytes;
use ethabi::{encode as eth_abi_encode, Token as EthAbiToken};
use ethereum_types::{H256 as EthHash, U256};

use super::{Challenge, ChallengesError};
use crate::NetworkId;

impl Challenge {
    pub(super) fn to_eth_abi_token(self) -> Result<EthAbiToken, ChallengesError> {
        // NOTE: Structs in solidity get encoded in tuples
        let actor_type: u8 = self.actor().actor_type().into();

        Ok(EthAbiToken::Tuple(vec![
            EthAbiToken::Uint(*self.nonce()),
            EthAbiToken::Address(*self.actor().actor_address()),
            EthAbiToken::Address(*self.challenger_address()),
            EthAbiToken::Uint(U256::from(actor_type)),
            EthAbiToken::Uint(U256::from_big_endian(&self.timestamp().to_be_bytes())),
            EthAbiToken::FixedBytes(NetworkId::try_from(self.mcid())?.to_bytes_4()?.to_vec()),
        ]))
    }

    pub(super) fn abi_encode(&self) -> Result<Vec<u8>, ChallengesError> {
        Ok(eth_abi_encode(&[self.to_eth_abi_token()?]))
    }

    pub(super) fn hash(&self) -> Result<EthHash, ChallengesError> {
        Ok(keccak_hash_bytes(&self.abi_encode()?))
    }

    // NOTE: Just a synonym of the `hash` fxn
    pub fn id(&self) -> Result<EthHash, ChallengesError> {
        self.hash()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common_metadata::MetadataChainId;
    use ethereum_types::{Address as EthAddress, U256};

    use super::*;
    use crate::{Actor, ActorType};

    #[test]
    fn should_get_expected_challenge_id() {
        let c = Challenge::new(
            U256::from(0),
            Actor::new(
                ActorType::Sentinel,
                EthAddress::from_str("0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f").unwrap(),
            ),
            1697545846,
            MetadataChainId::BscMainnet,
            EthAddress::from_str("0x1CBd3b2770909D4e10f157cABC84C7264073C9Ec").unwrap(),
        );
        let encoding = hex::encode(c.abi_encode().unwrap());
        let expected_encoding = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000023618e81e3f5cdf7f54c3d65f7fbc0abf5b21e8f0000000000000000000000001cbd3b2770909d4e10f157cabc84c7264073c9ec000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000652e7e765aca268b00000000000000000000000000000000000000000000000000000000";
        assert_eq!(encoding, expected_encoding);
        let id = c.id().unwrap();
        let expected_id =
            EthHash::from_str("0x45430ea5312431d78303dd4f9a95cb73f66795d4f1427804d24dbc985e7206c8").unwrap();
        assert_eq!(id, expected_id);
    }
}
