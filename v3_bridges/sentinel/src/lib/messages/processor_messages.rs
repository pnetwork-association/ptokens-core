use common_eth::EthSubmissionMaterials;
use derive_more::Constructor;

use crate::Responder;

#[derive(Debug, Constructor)]
pub struct ProcessArgs {
    pub batch: EthSubmissionMaterials,
    pub responder: Responder<()>,
}

#[derive(Debug)]
pub enum ProcessorMessages {
    //PauseHost, // TODO
    //PauseNative, // TODO
    ProcessHost(ProcessArgs),
    ProcessNative(ProcessArgs),
}
