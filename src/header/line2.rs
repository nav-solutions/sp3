//! header line #2 helpers
use std::io::{BufWriter, Write};

use crate::{prelude::Duration, FormattingError, ParsingError};

pub(crate) fn is_header_line2(content: &str) -> bool {
    content.starts_with("##")
}

pub(crate) struct Line2 {
    pub week: u32,
    pub sow_nanos: (u32, u64),
    pub mjd: (u32, f64),
    pub sampling_period: Duration,
}

impl std::str::FromStr for Line2 {
    type Err = ParsingError;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.len() != 60 {
            return Err(ParsingError::MalformedH2);
        }

        let week;
        let mut sow_nanos = (0_u32, 0_u64);
        let mut mjd = (0_u32, 0.0_f64);

        week = line[2..7]
            .trim()
            .parse::<u32>()
            .or(Err(ParsingError::WeekCounter))?;

        sow_nanos.0 = line[7..14]
            .trim()
            .parse::<u32>()
            .or(Err(ParsingError::WeekSeconds))?;

        sow_nanos.1 = line[15..23]
            .trim()
            .parse::<u64>()
            .or(Err(ParsingError::WeekSeconds))?;

        sow_nanos.1 *= 10;

        let (dt_s, dt_nanos) = (&line[24..29].trim(), &line[30..38].trim());

        let dt_s = dt_s.parse::<u32>().or(Err(ParsingError::SamplingPeriod))? as i128;
        let dt_nanos = dt_nanos
            .parse::<u32>()
            .or(Err(ParsingError::SamplingPeriod))? as i128;

        mjd.0 = u32::from_str(line[38..44].trim())
            .or(Err(ParsingError::Mjd(line[38..44].to_string())))?;

        mjd.1 =
            f64::from_str(line[44..].trim()).or(Err(ParsingError::Mjd(line[44..].to_string())))?;

        Ok(Self {
            mjd,
            week,
            sow_nanos,
            sampling_period: Duration::from_total_nanoseconds(dt_s * 1_000_000_000 + dt_nanos * 10),
        })
    }
}

impl Line2 {
    pub fn to_parts(&self) -> (u32, (u32, u64), Duration, (u32, f64)) {
        (self.week, self.sow_nanos, self.sampling_period, self.mjd)
    }

    pub fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let dt_integer = self.sampling_period.to_seconds().floor() as u32;

        let dt_fract =
            self.sampling_period.total_nanoseconds() - (dt_integer as i128) * 1_000_000_000;

        let (mjd_sod_integer, mjd_sod_fract) = (
            self.mjd.1.floor() as u32,
            (self.mjd.1.fract() * 10.0E5) as u32,
        );

        write!(
            w,
            "##{:5} {:6}.{:08} {:5}.{:08} {:05} {}.{:013}",
            self.week,
            self.sow_nanos.0,
            self.sow_nanos.1 / 10,
            dt_integer,
            dt_fract / 10,
            self.mjd.0,
            mjd_sod_integer,
            mjd_sod_fract,
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
        ] {
            let line2 = Line2::from_str(&line).unwrap();

            assert_eq!(line2.week, week);
            assert_eq!(line2.sow_nanos.0, sow);
            assert_eq!(line2.sow_nanos.1, week_nanos);

            assert_eq!(line2.mjd.0, mjd);
            assert_eq!(line2.mjd.1, mjd_fract);

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
