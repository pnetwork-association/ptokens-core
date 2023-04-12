#[allow(clippy::module_inception)]
mod processor;
mod processor_loop;

pub(crate) use self::processor::{process_batch, process_single};
pub(in crate::sentinel) use self::processor_loop::processor_loop;
