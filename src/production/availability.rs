use crate::errors::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Availability {
    /// [Availability::UltraRapid]: SP3-fit released _hours_ after observation.
    UltraRapid,

    #[default]
    /// [Availability::Rapid]: SP3-fit released _days_ after observation.
    Rapid,

    /// [Availability::Final]: SP3-fit released _weeks_ after observation.
    Final,
}

impl std::str::FromStr for Availability {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RAP" => Ok(Self::Rapid),
            "FIN" => Ok(Self::Final),
            "ULT" => Ok(Self::UltraRapid),
            _ => Err(ParsingError::InvalidFileAvailability),
        }
    }
}

impl std::fmt::Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rapid => write!(f, "RAP"),
            Self::Final => write!(f, "FIN"),
            Self::UltraRapid => write!(f, "ULT"),
        }
    }
}
