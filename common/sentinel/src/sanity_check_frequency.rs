use crate::{
    constants::{MAX_FREQUENCY, MIN_FREQUENCY},
    SentinelError,
};

pub fn sanity_check_frequency(f: u64) -> Result<u64, SentinelError> {
    debug!("sanity checking frequency of {f}...");
    if !(MIN_FREQUENCY..=MAX_FREQUENCY).contains(&f) {
        Err(SentinelError::InvalidFrequency {
            frequency: f,
            min: MIN_FREQUENCY,
            max: MAX_FREQUENCY,
        })
    } else {
        Ok(f)
    }
}
