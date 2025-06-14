#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(doc)]
use crate::prelude::SP3;

use crate::ParsingError;

/// [SP3] [ReleasePeriod]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ReleasePeriod {
    #[default]
    /// [ReleasePeriod::Daily] files
    Daily,

    /// [ReleasePeriod::Hourly] files
    Hourly,

    /// [ReleasePeriod::HalfDay] 12H files
    HalfDay,

    /// [ReleasePeriod::Weekly] files
    Weekly,

    /// [ReleasePeriod::Monthly] files
    Monthly,

    /// [ReleasePeriod::Yearly] files
    Yearly,
}

impl std::fmt::Display for ReleasePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hourly => write!(f, "01H"),
            Self::HalfDay => write!(f, "12H"),
            Self::Daily => write!(f, "01D"),
            Self::Weekly => write!(f, "01W"),
            Self::Monthly => write!(f, "01L"),
            Self::Yearly => write!(f, "01Y"),
        }
    }
}

impl std::str::FromStr for ReleasePeriod {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "01H" => Ok(Self::Hourly),
            "12H" => Ok(Self::HalfDay),
            "01D" => Ok(Self::Daily),
            "01W" => Ok(Self::Weekly),
            "01L" => Ok(Self::Monthly),
            "01Y" => Ok(Self::Yearly),
            _ => Err(ParsingError::InvalidFilename),
        }
    }
}
