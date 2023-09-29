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
            Self::Stop => "stop",
            Self::Start => "start",
            Self::CoreConnected => "core connected",
            Self::CoreDisconnected => "core disconnected",
        };
        write!(f, "{prefix} {s}")
    }
}
