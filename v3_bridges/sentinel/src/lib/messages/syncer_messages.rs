#[derive(Debug, Clone)]
pub enum SyncerMessages {
    Pause,
    Resume,
    Restart(u64),
    ProcessingError(u64),
}
