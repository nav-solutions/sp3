use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    str::FromStr,
};

use itertools::Itertools;

use crate::{errors::FormattingError, prelude::SP3};

#[cfg(feature = "flate2")]
use flate2::{write::GzEncoder, Compression as GzCompression};

use hifitime::efmt::{Format, Formatter};

pub(crate) struct CoordsFormatter {
    value: f64,
    width: usize,
    precision: usize,
}

impl CoordsFormatter {
    pub fn coordinates(value: f64) -> Self {
        Self {
            value,
            width: 13,
            precision: 6,
        }
    }

    pub fn fractional_mjd(value: f64) -> Self {
        Self {
            value,
            width: 15,
            precision: 13,
        }
    }
}

impl std::fmt::Display for CoordsFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = self.value;
        let sign_str = if self.precision == 13 {
            ""
        } else {
            if value.is_sign_positive() {
                " "
            } else {
                ""
            }
        };

        let formatted = if value.is_sign_positive() {
            format!(
                "{:width$.precision$}",
                value,
                width = self.width,
                precision = self.precision
            )
        } else {
            format!(
                "{:width$.precision$}",
                value,
                width = self.width + 1,
                precision = self.precision
            )
        };

        write!(f, "{}{}", sign_str, formatted)
    }
}

impl SP3 {
    /// Formats [SP3] into writable I/O using efficient buffered writer
    /// and following standard specifications.
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let efmt = Format::from_str("%Y %m %d %H %M %S.%f").unwrap();

        self.header.format(writer)?;

        for comment in self.comments.iter() {
            writeln!(writer, "/* {}", comment)?;
        }

        for epoch in self.data.keys().map(|k| k.epoch).unique().sorted() {
            let formatter = Formatter::new(epoch, efmt);

            // let (y, m, d, hh, mm, ss, nanos) = epoch.to_gregorian_utc();

            // writeln!(
            //     writer,
            //     "*  {:04} {:2} {:2} {:2} {:2} {:2}.{:08}",
            //     y,
            //     m,
            //     d,
            //     hh,
            //     mm,
            //     ss,
            //     nanos / 10
            // )?;
            writeln!(writer, "*  {}", formatter)?;

            for key in self
                .data
                .keys()
                .filter_map(|k| if k.epoch == epoch { Some(k) } else { None })
                .unique()
                .sorted()
            {
                if let Some(entry) = self.data.get(key) {
                    entry.format(key.sv, writer)?;
                }
            }
        }

        writeln!(writer, "EOF")?;
        writer.flush()?;

        Ok(())
    }

    /// Dumps [SP3] into writable local file (as readable ASCII UTF-8),
    /// using efficient buffered formatting.
    /// This is the mirror operation of [Self::from_file]
    /// ```
    /// use sp3::prelude::*;
    ///
    /// let sp3 = SP3::from_file("data/SP3/C/co108870.sp3").unwrap();
    ///
    /// assert!(sp3.to_file("output.sp3").is_ok());
    /// ```
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), FormattingError> {
        let fd = File::create(path)?;
        let mut writer = BufWriter::new(fd);
        self.format(&mut writer)?;
        Ok(())
    }

    /// Dumps [SP3] into Gzip compressed file.
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn to_gzip_file<P: AsRef<Path>>(&self, path: P) -> Result<(), FormattingError> {
        let fd = File::create(path)?;
        let compression = GzCompression::new(5);
        let mut writer = BufWriter::new(GzEncoder::new(fd, compression));
        self.format(&mut writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::SP3;

    #[test]
    fn sp3_c_formatting() {
        let sp3 = SP3::from_file("data/SP3/C/co108870.sp3").unwrap();

        sp3.to_file("test-c.sp3").unwrap_or_else(|e| {
            panic!("SP3/formatting issue: {}", e);
        });

        let parsed = SP3::from_file("test-c.sp3").unwrap_or_else(|e| {
            panic!("SP3/failed to parse back: {}", e);
        });

        assert_eq!(parsed, sp3);
    }

    #[test]
    fn sp3_d_formatting() {
        let sp3 = SP3::from_file("data/SP3/D/example.txt").unwrap();

        sp3.to_file("test-d.sp3").unwrap_or_else(|e| {
            panic!("SP3/formatting issue: {}", e);
        });

        let _ = SP3::from_file("test-d.sp3").unwrap_or_else(|e| {
            panic!("SP3/failed to parse back: {}", e);
        });

        // TODO: achieve this equality
        // assert_eq!(parsed, sp3);
    }

    #[test]
    fn sp3_c_predicted_orbits() {
        let sp3 =
            SP3::from_gzip_file("data/SP3/C/ESA0OPSULT_20232320600_02D_15M_ORB.SP3.gz").unwrap();

        sp3.to_file("test-c.sp3").unwrap_or_else(|e| {
            panic!("SP3/formatting issue: {}", e);
        });

        let _ = SP3::from_file("test-c.sp3").unwrap_or_else(|e| {
            panic!("SP3/failed to parse back: {}", e);
        });

        // assert_eq!(parsed, sp3);
    }
}
