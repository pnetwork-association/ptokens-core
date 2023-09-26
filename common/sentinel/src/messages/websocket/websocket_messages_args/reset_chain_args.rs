use std::str::FromStr;

use common::BridgeSide;
use common_eth::EthSubmissionMaterial;
use common_metadata::MetadataChainId;
use derive_getters::{Dissolve, Getters};
use serde::{Deserialize, Serialize};

use crate::WebSocketMessagesError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Dissolve)]
pub struct WebSocketMessagesResetChainArgs {
    confs: u64,
    validate: bool,
    mcid: MetadataChainId,
    use_latest_block: bool,
    block_num: Option<u64>,
    side: Option<BridgeSide>,
    block: Option<EthSubmissionMaterial>,
}

impl WebSocketMessagesResetChainArgs {
    pub fn add_sub_mat(&mut self, m: EthSubmissionMaterial) {
        self.block = Some(m)
    }

    pub fn add_side(&mut self, s: BridgeSide) {
        self.side = Some(s)
    }
}

// NOTE: Because these args are passed in via an RPC call
impl TryFrom<Vec<String>> for WebSocketMessagesResetChainArgs {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        // NOTE: Example: ["EthereumMainnet", "latest", "10", "false"]

        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };

        let expected_num_args = 4;
        if args.len() != expected_num_args {
            return Err(WebSocketMessagesError::NotEnoughArgs {
                got: args.len(),
                expected: expected_num_args,
                args,
            });
        }

        let mut arg = args[0].clone();

        let mcid = MetadataChainId::from_str(&arg).map_err(|_| WebSocketMessagesError::UnrecognizedChainId(arg))?;

        arg = args[1].clone();
        let use_latest_block = matches!(arg.to_lowercase().as_ref(), "latest");

        let block_num = if use_latest_block {
            None
        } else {
            let n = arg.parse::<u64>().map_err(|_| WebSocketMessagesError::ParseInt(arg))?;
            Some(n)
        };

        arg = args[2].clone();
        let confs = arg.parse::<u64>().map_err(|_| WebSocketMessagesError::ParseInt(arg))?;

        let validate = matches!(args[3].as_ref(), "true");

        let block = None;
        let side = None;

        Ok(Self {
            mcid,
            use_latest_block,
            block_num,
            block,
            side,
            confs,
            validate,
        })
    }
}
