#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BridgeSide {
    Host,
    Native,
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
            _ => Err("Error converting '{s}' into `BridgeSide`".into()),
        }
    }
}
