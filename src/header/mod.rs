//! header parsing utilities
pub(crate) mod line1;
pub(crate) mod line2;

use std::io::{BufWriter, Write};

pub mod version;

use crate::{
    errors::FormattingError,
    header::version::Version,
    prelude::{Constellation, Duration, Epoch, ParsingError, TimeScale, SV},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use line1::Line1;
use line2::Line2;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    /// [DataType::Position] means this file provides state vectors only
    /// (spatial is mandatory, clock state is optional).
    #[default]
    Position,

    /// [DataType::Velocity] means this file provides both state vectors
    /// and velocity vectors. The clock drift (clock state derivative) is once
    /// again optional.
    Velocity,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Position => f.write_str("P"),
            Self::Velocity => f.write_str("V"),
        }
    }
}

impl std::str::FromStr for DataType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("P") {
            Ok(Self::Position)
        } else if s.eq("V") {
            Ok(Self::Velocity)
        } else {
            Err(ParsingError::UnknownDataType)
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrbitType {
    #[default]
    FIT,
    EXT,
    BCT,
    BHN,
    HLM,
}

impl std::fmt::Display for OrbitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FIT => f.write_str("FIT"),
            Self::EXT => f.write_str("EXT"),
            Self::BCT => f.write_str("BCT"),
            Self::BHN => f.write_str("BHN"),
            Self::HLM => f.write_str("HLM"),
        }
    }
}

impl std::str::FromStr for OrbitType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("FIT") {
            Ok(Self::FIT)
        } else if s.eq("EXT") {
            Ok(Self::EXT)
        } else if s.eq("BCT") {
            Ok(Self::BCT)
        } else if s.eq("BHN") {
            Ok(Self::BHN)
        } else if s.eq("HLM") {
            Ok(Self::HLM)
        } else {
            Err(ParsingError::UnknownOrbitType)
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header {
    /// File revision as [Version]
    pub version: Version,

    /// File publication [Epoch], expressed in [Self.timescale] or
    /// [TimeScale::GPST] for older revisions
    pub release_epoch: Epoch,

    /// [DataType] used in this file.
    /// [DataType::Velocity] means velocity vector will be provided in following record.
    pub data_type: DataType,

    /// Coordinates system description.
    pub coord_system: String,

    /// [OrbitType] used in the fitting process prior publication.
    pub orbit_type: OrbitType,

    /// "Observables" used for this fit, we parse "as is".
    /// Explanations on typical values:
    /// - `u`  undifferenced carrier phase
    /// - `du` change in u with time
    /// - `s`  2-receiver/1-satellite carrier phase
    /// - `ds` change on s with time
    /// - `d` 2-receiver/2-satellite carrier phase
    /// - `dd` change in d with time
    /// - `U` undifferenced code phase
    /// - `dU` change in U with time
    /// - `S` 2-receiver/1-satellite code phase
    /// - `dS` change in S with time
    /// - `D` 2-receiver/2-satellite code phase
    /// - `dD` change in D with time
    /// - `+` used as separator
    pub observables: String,

    /// Total number of epochs
    pub num_epochs: u64,

    /// Agency providing this record.
    pub agency: String,

    /// Type of [Constellation] found in this record.
    /// For example [Constellation::GPS] means you will only find GPS satellite vehicles.
    pub constellation: Constellation,

    /// [TimeScale] that applies to all following [Epoch]s.
    pub timescale: TimeScale,

    /// Total elapsed weeks in [TimeScale].
    pub week: u32,

    /// Total number of nanoseconds in current week.
    pub week_nanos: u64,

    /// Datetime as MJD (in [TimeScale])
    pub mjd: u32,

    /// MJD fraction of day (>=0, <1.0)
    pub mjd_fraction: f64,

    /// Sampling period, as [Duration].
    pub sampling_period: Duration,

    /// [SV] to be found in this record.
    pub satellites: Vec<SV>,
}

impl Header {
    /// Format this SP3 [Header] according to standard specifications.
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let line1 = Line1 {
            version: self.version,
            data_type: self.data_type,
            epoch: self.release_epoch,
            num_epochs: self.num_epochs,
            orbit_type: self.orbit_type,
            agency: self.agency.to_string(),
            observables: self.observables.to_string(),
            coord_system: self.coord_system.to_string(),
        };

        let line2 = Line2 {
            week: self.week,
            week_nanos: self.week_nanos,
            mjd_fract: (self.mjd, self.mjd_fraction),
            sampling_period: self.sampling_period,
        };

        line1.format(writer)?;
        writeln!(writer)?;

        line2.format(writer)?;
        writeln!(writer)?;

        // file descriptor support is incomplete
        let gnss_timescale = match self.timescale {
            TimeScale::GPST => "GPS",
            TimeScale::GST => "GAL",
            TimeScale::QZSST => "QZS",
            TimeScale::UTC => "UTC",
            _ => "TAI",
        };

        // TODO: `L` exists here in case only LEO vehicles are to be found
        writeln!(
            writer,
            "%c {:x}  cc {} ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc",
            self.constellation, gnss_timescale,
        )?;

        writeln!(
            writer,
            "%c cc cc ccc ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc"
        )?;

        // other fields are not supported
        // %i
        // %f

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{
        Constellation, DataType, Duration, Epoch, Header, OrbitType, TimeScale, Version, SV,
    };
    use crate::tests::formatting::Utf8Buffer;

    use std::io::BufWriter;
    use std::str::FromStr;

    #[test]
    #[ignore]
    fn header_formatting() {
        let header = Header {
            version: Version::C,
            observables: "__u+U".to_string(),
            release_epoch: Epoch::from_str("2020-01-01T00:00:00 GPST").unwrap(),
            data_type: DataType::Position,
            coord_system: "ITRF93".to_string(),
            orbit_type: OrbitType::FIT,
            num_epochs: 10,
            agency: "GRGS".to_string(),
            constellation: Constellation::GPS,
            timescale: TimeScale::GPST,
            week: 1234,
            week_nanos: 5678,
            mjd: 12,
            mjd_fraction: 0.123,
            sampling_period: Duration::from_seconds(900.0),
            satellites: "G01,G02,G03,G04,G05"
                .split(',')
                .map(|s| SV::from_str(s).unwrap())
                .collect(),
        };

        let mut buffer = BufWriter::new(Utf8Buffer::new(8192));

        header.format(&mut buffer).unwrap_or_else(|e| {
            panic!("Header formatting issue: {}", e);
        });

        let formatted = buffer.into_inner().unwrap();
        let formatted = formatted.to_ascii_utf8();

        assert_eq!(
            formatted,
            "#cP2019 12 31 23 59 42.00000000      10 __u+U ITRF93 FIT  GRGS
## 1234      0.00000567   900.00000000 00012 33999999.0000000999999
%c G  cc GPS ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc
%c cc cc ccc ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc\n"
        );
    }
}
