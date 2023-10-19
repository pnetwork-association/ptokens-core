#[derive(Debug, Clone)]
pub enum SyncerMessages {
    Restart(u64),
    ProcessingError(u64),
}
