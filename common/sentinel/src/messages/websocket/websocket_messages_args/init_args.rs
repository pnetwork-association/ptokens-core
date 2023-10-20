use common_eth::{convert_hex_to_eth_address, EthSubmissionMaterial};
use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{NetworkId, WebSocketMessagesError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct WebSocketMessagesInitArgs {
    validate: bool,
    hub: EthAddress,
    tail_length: u64,
    confirmations: u64,
    network_id: NetworkId,
    #[getter(skip)]
    sub_mat: Option<EthSubmissionMaterial>,
}

impl WebSocketMessagesInitArgs {
    fn name(&self) -> String {
        "WebSocketMessagesInitArgs".into()
    }

    pub fn add_sub_mat(&mut self, m: EthSubmissionMaterial) {
        self.sub_mat = Some(m);
    }

    pub fn sub_mat(&self) -> Result<EthSubmissionMaterial, WebSocketMessagesError> {
        match self.sub_mat {
            Some(ref b) => Ok(b.clone()),
            None => Err(WebSocketMessagesError::NoBlock {
                struct_name: self.name(),
                network_id: self.network_id,
            }),
        }
    }
}

// NOTE: Because these args are passed in via an RPC call
impl TryFrom<Vec<String>> for WebSocketMessagesInitArgs {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };

        let expected_num_args = 5;
        if args.len() != expected_num_args {
            return Err(WebSocketMessagesError::NotEnoughArgs {
                got: args.len(),
                expected: expected_num_args,
                args,
            });
        }
        let tail_length_arg = args[2].clone();
        let tail_length = tail_length_arg
            .parse::<u64>()
            .map_err(|_| WebSocketMessagesError::ParseInt(tail_length_arg))?;

        let confirmations_arg = args[3].clone();
        let confirmations = confirmations_arg
            .parse::<u64>()
            .map_err(|_| WebSocketMessagesError::ParseInt(confirmations_arg))?;

        let network_id_arg = args[4].clone();
        let network_id = NetworkId::try_from(&network_id_arg).map_err(|e| {
            error!("{e}");
            WebSocketMessagesError::ParseNetworkId(network_id_arg)
        })?;

        Ok(Self {
            validate: matches!(args[0].as_ref(), "true"),
            hub: convert_hex_to_eth_address(&args[1])?,
            tail_length,
            confirmations,
            network_id,
            sub_mat: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::WebSocketMessagesEncodable;

    #[test]
    fn should_get_init_message_from_string_of_args() {
        let args = vec![
            "init",
            "true",
            "0x4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97",
            "50",
            "10",
            "EthereumMainnet",
        ];
        let r = WebSocketMessagesEncodable::try_from(args);
        assert!(r.is_ok());
    }
}
