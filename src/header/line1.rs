//! Header line #1 helpers

use crate::{
    header::{DataType, OrbitType, Version},
    prelude::Epoch,
    FormattingError, ParsingError,
};

use std::io::{BufWriter, Write};

pub(crate) fn is_header_line1(content: &str) -> bool {
    content.starts_with('#')
}

pub(crate) struct Line1 {
    pub version: Version,
    pub data_type: DataType,
    pub epoch: Epoch,
    pub fit_type: String,
    pub num_epochs: u64,
    pub coord_system: String,
    pub orbit_type: OrbitType,
    pub agency: String,
}

impl std::str::FromStr for Line1 {
    type Err = ParsingError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.len() < 59 {
            return Err(ParsingError::MalformedH1);
        }

        let (y, m, d, hh, mm, ss, nanos) = (
            &line[3..7].trim(),
            &line[8..11].trim(),
            &line[11..13].trim(),
            &line[14..16].trim(),
            &line[17..19].trim(),
            &line[20..22].trim(),
            &line[23..31].trim(),
        );

        let y = y.parse::<i32>().or(Err(ParsingError::Epoch))?;
        let m = m.parse::<u8>().or(Err(ParsingError::Epoch))?;
        let d = d.parse::<u8>().or(Err(ParsingError::Epoch))?;
        let hh = hh.parse::<u8>().or(Err(ParsingError::Epoch))?;
        let mm = mm.parse::<u8>().or(Err(ParsingError::Epoch))?;
        let ss = ss.parse::<u8>().or(Err(ParsingError::Epoch))?;
        let nanos = nanos.parse::<u32>().or(Err(ParsingError::Epoch))?;

        let epoch = Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, nanos * 10);

        let num_epochs = &line[32..40].trim();
        let num_epochs = num_epochs.parse::<u64>().or(Err(ParsingError::Epoch))?;

        Ok(Self {
            epoch,
            num_epochs,
            fit_type: line[40..45].trim().to_string(),
            version: Version::from_str(&line[1..2])?,
            data_type: DataType::from_str(&line[2..3])?,
            coord_system: line[45..51].trim().to_string(),
            orbit_type: OrbitType::from_str(line[51..55].trim())?,
            agency: line[55..].trim().to_string(),
        })
    }
}

impl Line1 {
    pub fn to_parts(&self) -> (Version, DataType, String, OrbitType, String) {
        (
            self.version,
            self.data_type,
            self.coord_system.clone(),
            self.orbit_type,
            self.agency.clone(),
        )
    }

    pub fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let (y, m, d, hh, mm, ss, nanos) = self.epoch.to_gregorian_utc();

        write!(
            w,
            "#{}{}{:04} {:2} {:2} {:2} {:2} {:2}.{:08} {:7} {} {} {}  {}",
            self.version,
            self.data_type,
            y,
            m,
            d,
            hh,
            mm,
            ss,
            nanos / 10,
            self.num_epochs,
            self.fit_type,
            self.coord_system,
            self.orbit_type,
            self.agency,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        header::{line1::Line1, DataType, OrbitType, Version},
        prelude::Epoch,
        tests::formatting::Utf8Buffer,
    };

    use std::{io::BufWriter, str::FromStr};

    #[test]
    fn test_line1() {
        for (line, version, dtype, epoch_str, num_epochs, fit_type, coord_system, orbit_type) in [
            (
                "#dP2020  6 24  1  3  4.12345678      97 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Position,
                "2020-06-24T01:03:04.12345678 UTC",
                97,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
            (
                "#dV2020  6 24  1  3 54.12345678      97 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Velocity,
                "2020-06-24T01:03:54.12345678 UTC",
                97,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
            (
                "#dV2020 12 24 21  3 54.12345678      97 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Velocity,
                "2020-12-24T21:03:54.12345678 UTC",
                97,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
            (
                "#dV2020 12 24 21 43 54.12345678      97 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Velocity,
                "2020-12-24T21:43:54.12345678 UTC",
                97,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
            (
                "#dV2020 12 24 21 43 54.12345678       7 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Velocity,
                "2020-12-24T21:43:54.12345678 UTC",
                7,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
            (
                "#dV2020 12 24 21 43 54.12345678    1000 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Velocity,
                "2020-12-24T21:43:54.12345678 UTC",
                1000,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
            (
                "#dV2020 12 24 21 43 54.12345678  100022 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Velocity,
                "2020-12-24T21:43:54.12345678 UTC",
                100022,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
            (
                "#dV2020 12 24 21 43 54.12345678 9100022 __u+U IGS14 FIT  IAC",
                Version::D,
                DataType::Velocity,
                "2020-12-24T21:43:54.12345678 UTC",
                9100022,
                "__u+U",
                "IGS14",
                OrbitType::FIT,
            ),
        ] {
            let line1 = Line1::from_str(&line).unwrap();
            let epoch = Epoch::from_str(epoch_str).unwrap();

            assert_eq!(line1.version, version);
            assert_eq!(line1.coord_system, coord_system);
            assert_eq!(line1.orbit_type, orbit_type);
            assert_eq!(line1.epoch, epoch);
            assert_eq!(line1.data_type, dtype);

            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            line1.format(&mut buf).unwrap_or_else(|e| {
                panic!("Header/Line#1 formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();
            assert_eq!(formatted, line);
        }
    }
}
