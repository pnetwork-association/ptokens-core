use common_eth::EthSubmissionMaterials;

#[derive(Debug, Clone)]
pub enum BroadcastMessages {
    Shutdown,
    ProcessHost(EthSubmissionMaterials),
    ProcessNative(EthSubmissionMaterials),
}
