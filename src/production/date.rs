use crate::prelude::Epoch;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use hifitime::HifitimeError;

/// [SP3] [ReleaseDate]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReleaseDate {
    /// Release Year
    pub year: u16,

    /// Day of year (starting at 1).
    pub doy: u16,
}

impl From<Epoch> for ReleaseDate {
    fn from(value: Epoch) -> Self {
        Self {
            year: value.year() as u16,
            doy: value.day_of_year().floor() as u16,
        }
    }
}

impl ReleaseDate {
    /// Converts [ReleaseDate] to [Epoch]
    pub fn to_epoch(&self) -> Result<Epoch, HifitimeError> {
        Epoch::from_format_str(&format!("{} {}", self.year, self.doy), "%Y %j")
    }
}
