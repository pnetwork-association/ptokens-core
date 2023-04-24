use common::BridgeSide;
use common_eth::EthSubmissionMaterials;
use derive_more::Constructor;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{get_utc_timestamp, Responder, SentinelError};

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

    #[allow(unused)]
    fn is_in_sync(&self) -> Result<bool, SentinelError> {
        // NOTE: We define the core as being in sync if the about-to-be-submitted batch's last
        // block timestamp is within ~ an hour of now.
        let last_block_timestamp = self.batch.get_last_block_timestamp()?.as_secs();
        let one_hour_in_seconds = 1000 * 60 * 60;
        let now = get_utc_timestamp()?;
        let target = now - one_hour_in_seconds;
        let delta = if now > last_block_timestamp {
            now - last_block_timestamp
        } else {
            0
        };
        Ok(delta <= target)
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
