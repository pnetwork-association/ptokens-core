use std::fmt;

#[derive(Debug, Clone)]
pub enum StatusMessages {
    SendStatusUpdate,
    SetStatusPublishingFreqency(u64),
}

impl fmt::Display for StatusMessages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "status broadcast channel message:";
        let s = match self {
            Self::SendStatusUpdate => "send status update".to_string(),
            Self::SetStatusPublishingFreqency(n) => format!("set status publishing frequency to {n}"),
        };
        write!(f, "{prefix} {s}")
    }
}
