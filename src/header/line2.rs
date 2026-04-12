//! header line #2 helpers
use std::io::{BufWriter, Write};

use crate::{formatting::CoordsFormatter, prelude::Duration, FormattingError, ParsingError};

pub(crate) fn is_header_line2(content: &str) -> bool {
    content.starts_with("##")
}

pub(crate) struct Line2 {
    pub week: u32,
    pub week_nanos: u64,

    /// MJD and MJD fract
    pub mjd_fract: (u32, f64),

    pub sampling_period: Duration,
}

impl std::str::FromStr for Line2 {
    type Err = ParsingError;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.len() != 60 {
            return Err(ParsingError::MalformedH2);
        }

        let week = line[2..7]
            .trim()
            .parse::<u32>()
            .or(Err(ParsingError::WeekCounter))?;

        let week_seconds = line[7..14]
            .trim()
            .parse::<u64>()
            .or(Err(ParsingError::WeekSeconds))?;

        let mut week_nanos = line[15..23]
            .trim()
            .parse::<u64>()
            .or(Err(ParsingError::WeekSeconds))?;

        week_nanos *= 10;
        week_nanos += week_seconds * 1_000_000_000;

        let (dt_s, dt_nanos) = (&line[24..29].trim(), &line[30..38].trim());

        let dt_s = dt_s.parse::<u32>().or(Err(ParsingError::SamplingPeriod))? as i128;

        let dt_nanos = dt_nanos
            .parse::<u32>()
            .or(Err(ParsingError::SamplingPeriod))? as i128;

        let mjd = line[38..44]
            .trim()
            .parse::<u32>()
            .or(Err(ParsingError::Mjd))?;

        let mjd_fraction = line[44..]
            .trim()
            .parse::<f64>()
            .or(Err(ParsingError::Mjd))?;

        Ok(Self {
            week,
            week_nanos,
            mjd_fract: (mjd, mjd_fraction),
            sampling_period: Duration::from_total_nanoseconds(dt_s * 1_000_000_000 + dt_nanos * 10),
        })
    }
}

impl Line2 {
    pub fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let dt_seconds = self.sampling_period.to_seconds().floor() as u32;

        let dt_nanos =
            self.sampling_period.total_nanoseconds() - (dt_seconds as i128) * 1_000_000_000;

        let week_seconds = self.week_nanos / 1_000_000_000;
        let week_nanos = self.week_nanos - week_seconds * 1_000_000_000;

        write!(
            w,
            "##{:5} {:6}.{:08} {:5}.{:08} {:05} {}",
            self.week,
            week_seconds,
            week_nanos / 10,
            dt_seconds,
            dt_nanos / 10,
            self.mjd_fract.0,
            CoordsFormatter::fractional_mjd(self.mjd_fract.1),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{io::BufWriter, str::FromStr};

    use crate::{header::line2::Line2, tests::formatting::Utf8Buffer};

    #[test]
    fn test_line2_parsing() {
        for (line, week, sow, week_nanos, epoch_interval, mjd, mjd_fract) in [
            (
                "##  887      0.00000000   900.00000000 50453 0.0000000000000",
                887,
                0,
                0,
                900.0,
                50453,
                0.0,
            ),
            (
                "##    7     10.12345678    10.55000000 50453 0.0000000000000",
                7,
                10,
                123456780,
                10.55,
                50453,
                0.0,
            ),
            (
                "##    7     10.12345678   900.00000000 50453 0.0000000000000",
                7,
                10,
                123456780,
                900.00,
                50453,
                0.0,
            ),
            (
                "##   87     10.12300000   900.00000000 50453 0.0000000000000",
                87,
                10,
                123000000,
                900.0,
                50453,
                0.0,
            ),
            (
                "##   87     10.12345678   900.00000000 50453 0.0000000000000",
                87,
                10,
                123456780,
                900.0,
                50453,
                0.0,
            ),
            (
                "##    7     10.12345678    10.00000000 50453 0.0000000000000",
                7,
                10,
                123456780,
                10.0,
                50453,
                0.0,
            ),
            (
                "## 2276  21600.00000000   900.00000000 60176 0.2500000000000",
                2276,
                21600,
                0,
                900.0,
                60176,
                0.25,
            ),
            (
                "##  887      0.00000000   900.00000000 50453 0.0000000000000",
                887,
                0,
                0,
                900.0,
                50453,
                0.0,
            ),
            (
                "##  887  86400.00000000   900.00000000 50454 0.0000000000000",
                887,
                86400,
                0,
                900.0,
                50454,
                0.0,
            ),
            (
                "## 2277  64800.00000000   900.00000000 60183 0.7500000000000",
                2277,
                64800,
                0,
                900.0,
                60183,
                0.75,
            ),
        ] {
            let line2 = Line2::from_str(line).unwrap();

            assert_eq!(line2.week, week);
            assert_eq!(line2.week_nanos, sow * 1_000_000_000 + week_nanos);
            assert_eq!(line2.mjd_fract.0, mjd);
            assert_eq!(line2.mjd_fract.1, mjd_fract);

            assert_eq!(line2.sampling_period.to_seconds(), epoch_interval);

            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            line2.format(&mut buf).unwrap_or_else(|e| {
                panic!("Header/Line#2 formatting error: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, line);
        }
    }
}
