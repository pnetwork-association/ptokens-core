use crate::Batch;

#[derive(Debug, Clone)]
pub enum BroadcastMessages {
    Shutdown,
    ProcessHost(Batch),
    ProcessNative(Batch),
}
