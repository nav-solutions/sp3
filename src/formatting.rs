use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{errors::FormattingError, prelude::SP3};

#[cfg(feature = "flate2")]
use flate2::{write::GzEncoder, Compression as GzCompression};

impl SP3 {
    /// Formats [SP3] into writable I/O using efficient buffered writer
    /// and following standard specifications.
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), FormattingError> {
        self.header.format(writer)?;
        writer.flush()?;
        Ok(())
    }

    /// Dumps [SP3] into writable local file (as readable ASCII UTF-8),
    /// using efficient buffered formatting.
    /// This is the mirror operation of [Self::to_file]
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

        // assert_eq!(parsed, sp3);
    }

    #[test]
    fn sp3_d_formatting() {
        let sp3 = SP3::from_file("data/SP3/D/example.txt").unwrap();

        sp3.to_file("test-d.sp3").unwrap_or_else(|e| {
            panic!("SP3/formatting issue: {}", e);
        });

        let parsed = SP3::from_file("test-d.sp3").unwrap_or_else(|e| {
            panic!("SP3/failed to parse back: {}", e);
        });

        assert_eq!(parsed, sp3);
    }
}
