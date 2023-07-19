use common::BridgeSide;
use common_eth::EthSubmissionMaterials;
use derive_more::Constructor;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{Responder, SentinelError};

#[derive(Debug, Constructor)]
pub struct ProcessArgs {
    side: BridgeSide,
    pub responder: Responder<()>,
    pub batch: EthSubmissionMaterials,
}

impl ProcessArgs {
    pub fn side(&self) -> BridgeSide {
        self.side
    }
}

#[derive(Debug)]
pub enum ProcessorMessages {
    Process(ProcessArgs),
}

impl ProcessorMessages {
    pub fn get_process_msg(
        side: BridgeSide,
        sub_mat: EthSubmissionMaterials,
    ) -> (Self, Receiver<Result<(), SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        let args = ProcessArgs::new(side, tx, sub_mat);
        (ProcessorMessages::Process(args), rx)
    }
}
