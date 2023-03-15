mod core_accessor;
mod mongo_accessor;
mod processor;
mod start_sentinel;
mod syncer;

pub(crate) use self::start_sentinel::start_sentinel;
use self::{
    core_accessor::core_accessor_loop,
    mongo_accessor::mongo_accessor_loop,
    processor::processor_loop,
    syncer::syncer_loop,
};
