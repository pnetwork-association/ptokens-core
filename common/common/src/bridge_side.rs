#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub enum BridgeSide {
    Native = 1,
    Host   = 2,
}

impl Default for BridgeSide {
    fn default() -> Self {
        Self::Native
    }
}

impl std::fmt::Display for BridgeSide {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Host => write!(f, "host"),
            Self::Native => write!(f, "native"),
        }
    }
}

impl std::str::FromStr for BridgeSide {
    type Err = crate::AppError;

    fn from_str(s: &str) -> std::result::Result<Self, crate::AppError> {
        match s.to_lowercase().as_ref() {
            "host" => Ok(Self::Host),
            "native" => Ok(Self::Native),
            _ => Err(format!("Error converting '{s}' into `BridgeSide`, expected `native` or `host`!").into()),
        }
    }
}

impl BridgeSide {
    pub fn is_native(&self) -> bool {
        self == &Self::Native
    }

    pub fn is_host(&self) -> bool {
        self == &Self::Host
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_bridge_side_should_be_native() {
        let b = BridgeSide::Native;
        assert!(b.is_native());
        assert!(!b.is_host());
    }

    #[test]
    fn host_bridge_side_should_be_host() {
        let b = BridgeSide::Host;
        assert!(b.is_host());
        assert!(!b.is_native());
    }

    #[test]
    fn native_should_be_less_than_host() {
        assert!(BridgeSide::Native < BridgeSide::Host)
    }
}
