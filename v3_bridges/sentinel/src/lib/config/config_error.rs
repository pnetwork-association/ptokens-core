#[derive(Debug, PartialEq)]
pub enum Error {
    /// No endpoint error
    NoEndpoints(bool),

    /// Batching size is out of bounds
    BatchSizeError { size: u64, min: u64, max: u64 },

    /// Batch duration is out of bounds
    BatchDurationError { size: u64, max: u64 },

    /// Log number is out of bounds
    LogNumError { size: usize, min: usize, max: usize },

    /// Log size is out of bounds
    LogSizeError { size: u64, min: u64, max: u64 },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::BatchSizeError {
                size: ref a,
                min: ref b,
                max: ref c,
            } => write!(f, "batch size of {a} is not between min of {b} and max of {c}"),
            Self::BatchDurationError {
                size: ref a,
                max: ref b,
            } => write!(f, "batch duration of {a} is greater than max of {b}"),
            Self::LogNumError {
                size: ref a,
                min: ref b,
                max: ref c,
            } => write!(f, "number of logs of {a} is not between min of {b} and max of {c}"),
            Self::LogSizeError {
                size: ref a,
                min: ref b,
                max: ref c,
            } => write!(f, "logs of size {a}b is not between min of {b}b and max of {c}b"),
            Self::NoEndpoints(ref is_native) => write!(
                f,
                "Cannot create {} sub mat batch - there are  no endpoints",
                if is_native == &true { "native" } else { "host" },
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;

        match self {
            LogSizeError { .. }
            | LogNumError { .. }
            | BatchDurationError { .. }
            | BatchSizeError { .. }
            | NoEndpoints(_) => None,
        }
    }
}
