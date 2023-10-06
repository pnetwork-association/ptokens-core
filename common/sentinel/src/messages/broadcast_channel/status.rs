use std::fmt;

#[derive(Debug, Clone)]
pub enum StatusBroadcastChannelMessages {
    Stop,
    Start,
    CoreConnected,
    CoreDisconnected,
}

impl fmt::Display for StatusBroadcastChannelMessages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "status broadcast channel message:";
        let s = match self {
            Self::Stop => "stop".to_string(),
            Self::Start => "start".to_string(),
            Self::CoreConnected => "core connected".to_string(),
            Self::CoreDisconnected => "core disconnected".to_string(),
        };
        write!(f, "{prefix} {s}")
    }
}
