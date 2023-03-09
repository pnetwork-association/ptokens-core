mod host;
mod native;
mod processor_loop;

pub(in crate::sentinel::processor) use host::process_host_batch;
pub(in crate::sentinel::processor) use native::process_native_batch;

pub(crate) use self::processor_loop::processor_loop;
