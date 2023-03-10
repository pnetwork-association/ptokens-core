use common_eth::EthSubmissionMaterials;

#[derive(Clone, Debug)]
pub enum ProcessorMessages {
    //PauseHost, // TODO
    //PauseNative, // TODO
    ProcessHost(EthSubmissionMaterials),
    ProcessNative(EthSubmissionMaterials),
}
