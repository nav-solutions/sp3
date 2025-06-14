use crate::errors::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Campaign {
    /// Part of a demonstration campaign.
    Demo,

    /// Multi-GNSS project product
    MGX,

    /// Operational IGS product
    #[default]
    OPS,

    /// Reprocessing campaign (with number of campaign iteration)
    Reprocessing(u8),

    /// Tide Gauche Benchmark Monitoring (TIGA)
    TGA,

    /// Part of a test campaign
    Test,
}

impl std::str::FromStr for Campaign {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEM" => Ok(Self::Demo),
            "TST" => Ok(Self::Test),
            "TGA" => Ok(Self::TGA),
            "OPS" => Ok(Self::OPS),
            "MGX" => Ok(Self::MGX),
            value => {
                if value.starts_with('R') {
                    let rnn = value[1..]
                        .parse::<u8>()
                        .or(Err(ParsingError::InvalidCampaignName))?;

                    Ok(Self::Reprocessing(rnn))
                } else {
                    Err(ParsingError::InvalidCampaignName)
                }
            },
        }
    }
}

impl std::fmt::Display for Campaign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Demo => write!(f, "DEM"),
            Self::MGX => write!(f, "MGX"),
            Self::OPS => write!(f, "OPS"),
            Self::TGA => write!(f, "TGA"),
            Self::Test => write!(f, "TST"),
            Self::Reprocessing(value) => write!(f, "R{:02}", value),
        }
    }
}
