use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::U256;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::NetworkId;

#[derive(Clone, Debug, Serialize, Deserialize, Deref, DerefMut, Constructor)]
pub struct Balances(Vec<Balance>);

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    // NOTE: Because default display for U256 is hex
    balance: String,
    #[serde_as(as = "DisplayFromStr")]
    network_id: NetworkId,
}

impl Balance {
    pub fn new(balance: U256, network_id: NetworkId) -> Self {
        Self {
            balance: balance.to_string(),
            network_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn balances_should_contain_dec_str_representation_of_balance_instead_of_hex() {
        let b = "1337000000000000000000";
        let x = Balance::new(U256::from_dec_str(b).unwrap(), NetworkId::default());
        let r = Balances::new(vec![x]);
        let j = json!(r).to_string();
        assert!(j.contains(b));
        assert!(!j.contains("0x"));
    }
}
