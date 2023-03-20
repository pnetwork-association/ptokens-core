use common_eth::EthSubmissionMaterials;
use derive_more::Constructor;

use crate::{Responder, SentinelError};

#[derive(Debug, Constructor)]
pub struct ProcessArgs {
    pub confs: u64,
    pub batch_size: u64,
    pub latest_block_number: u64,
    pub responder: Responder<()>,
    pub batch: EthSubmissionMaterials,
}

impl ProcessArgs {
    pub fn is_in_sync(&self) -> Result<bool, SentinelError> {
        // NOTE: We define the core as being in sync if the about-to-be-submitted batch's last
        // block number is within x blocks of the chain's tip, where x is whichever is the
        // smaller of the batch size OR the number of confs.
        let batch_last_block_num = self.batch.get_last_block_num()?;
        let delta = if self.latest_block_number > batch_last_block_num {
            self.latest_block_number - batch_last_block_num
        } else {
            0
        };
        let required_delta = self.confs.min(self.batch_size);
        Ok(delta <= required_delta)
    }
}

#[derive(Debug)]
pub enum ProcessorMessages {
    //PauseHost, // TODO
    //PauseNative, // TODO
    ProcessHost(ProcessArgs),
    ProcessNative(ProcessArgs),
}
