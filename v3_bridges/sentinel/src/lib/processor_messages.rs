use std::result::Result;

use common_eth::EthSubmissionMaterials;
use derive_more::Constructor;
use tokio::sync::oneshot;

use crate::SentinelError;

// TODO maybe move to own mod?
pub type Responder<T> = oneshot::Sender<Result<T, SentinelError>>;

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
