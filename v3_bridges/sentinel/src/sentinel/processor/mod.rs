mod host;
mod native;
mod processor_loop;

pub(in crate::sentinel) use self::processor_loop::processor_loop;
pub(crate) use self::{host::process_host_batch, native::process_native_batch};
