#[allow(clippy::module_inception)]
mod processor;

pub use processor::{process_batch, process_single};
