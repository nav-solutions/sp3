use crate::{
    prelude::{Duration, Epoch},
    ParsingError,
};

use hifitime::HifitimeError;

#[cfg(doc)]
use crate::prelude::SP3;

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

/// [SP3] [ProductionAttributes] come with files that
/// follow standard naming conventions.
/// See <https://files.igs.org/pub/resource/guidelines/Guidelines_for_Long_Product_Filenames_in_the_IGS_v2.2_EN.pdf>
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProductionAttributes {
    /// 3-Letter code
    pub agency: String,

    /// ID# in case this file is part of a batch (starting at 0).
    pub batch_id: u8,

    /// [ReleaseDate]
    pub release_date: ReleaseDate,

    /// [ReleasePeriod]
    pub release_period: ReleasePeriod,

    /// SP3 fit availability
    pub availability: Availability,

    /// Steady sampling period as [Duration] contained in this file
    pub sampling_period: Duration,

    /// True if this file was gzip compressed
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub gzip_compressed: bool,
}

impl std::fmt::Display for ProductionAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sampling_interval_mins = (self.sampling_period.to_seconds() / 60.0).round() as u16;

        write!(
            f,
            "{}{}OPS{}_{:04}{:03}0000_{}_{:02}M_ORB.SP3",
            &self.agency[..3],
            self.batch_id,
            self.availability,
            self.release_date.year,
            self.release_date.doy,
            self.release_period,
            sampling_interval_mins,
        )?;

        #[cfg(feature = "flate2")]
        if self.gzip_compressed {
            write!(f, ".gz")?;
        }

        Ok(())
    }
}

impl std::str::FromStr for ProductionAttributes {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let size = s.len();

        if size < 38 {
            return Err(ParsingError::InvalidFilename);
        }

        let agency = s[0..3].to_string();

        let batch_id = s[3..4]
            .parse::<u8>()
            .or(Err(ParsingError::InvalidFilename))?;

        let availability =
            Availability::from_str(&s[7..10]).or(Err(ParsingError::InvalidFilename))?;

        let release_year = s[11..15]
            .parse::<u16>()
            .or(Err(ParsingError::InvalidFilename))?;

        let release_doy = s[15..18]
            .parse::<u16>()
            .or(Err(ParsingError::InvalidFilename))?;

        let release_period = ReleasePeriod::from_str(&s[23..26])?;

        let sampling = s[27..29]
            .parse::<u8>()
            .or(Err(ParsingError::InvalidFilename))?;

        let scaling = match &s[29..30] {
            "S" => 1.0,
            "M" => 60.0,
            "H" => 3600.0,
            "D" => 24.0 * 3600.0,
            "W" => 7.0 * 24.0 * 3600.0,
            "L" => 30.0 * 7.0 * 24.0 * 3600.0,
            _ => 365.0 * 7.0 * 24.0 * 3600.0,
        };

        let sampling_period = Duration::from_seconds((sampling as f64) * scaling);

        Ok(Self {
            agency,
            batch_id,
            availability,
            release_date: ReleaseDate {
                year: release_year,
                doy: release_doy,
            },
            release_period,
            sampling_period,
            #[cfg(feature = "flate2")]
            gzip_compressed: s.ends_with(".gz"),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn release_epoch() {
        let release_date = ReleaseDate {
            year: 2023,
            doy: 239,
        };

        let release_epoch = release_date.to_epoch().unwrap();

        assert_eq!(
            release_epoch,
            Epoch::from_str("2023-08-27T00:00:00 UTC").unwrap()
        );
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn production_attributes_parsing() {
        for (expected, filename) in [
            (
                ProductionAttributes {
                    agency: "ESA".to_string(),
                    batch_id: 0,
                    release_date: ReleaseDate {
                        year: 2023,
                        doy: 239,
                    },
                    availability: Availability::Rapid,
                    release_period: ReleasePeriod::Daily,
                    sampling_period: Duration::from_hours(0.25),
                    gzip_compressed: true,
                },
                "ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz",
            ),
            (
                ProductionAttributes {
                    agency: "GRS".to_string(),
                    batch_id: 0,
                    release_date: ReleaseDate { year: 2019, doy: 1 },
                    availability: Availability::Final,
                    release_period: ReleasePeriod::Hourly,
                    sampling_period: Duration::from_hours(0.25),
                    gzip_compressed: true,
                },
                "GRS0OPSFIN_20190010000_01H_15M_ORB.SP3.gz",
            ),
            (
                ProductionAttributes {
                    agency: "GRS".to_string(),
                    batch_id: 5,
                    release_date: ReleaseDate { year: 2019, doy: 1 },
                    availability: Availability::Final,
                    release_period: ReleasePeriod::Hourly,
                    sampling_period: Duration::from_seconds(5.0 * 60.0),
                    gzip_compressed: true,
                },
                "GRS5OPSFIN_20190010000_01H_05M_ORB.SP3.gz",
            ),
        ] {
            let parsed = ProductionAttributes::from_str(filename).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse production attributes from \"{}\": {}",
                    filename, e
                );
            });

            assert_eq!(parsed, expected);

            let formatted = expected.to_string();
            assert_eq!(formatted, filename);
        }
    }
}
