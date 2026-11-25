//! Fully allocating SP3 parsing, built on top of [SP3Stream].
use std::{
    io::{BufReader, Read},
    path::Path,
};

use crate::prelude::{Error, SP3Stream, SP3};

impl SP3 {
    /// Parse [SP3] data from local file, entirely allocated in memory.
    ///
    /// This collects an [SP3Stream]. If you do not need the complete
    /// structure, iterate [SP3Stream::from_file] instead.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        SP3Stream::from_file(path)?.collect_sp3()
    }

    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    /// Parse [SP3] data from gzip encoded local file, entirely allocated in memory.
    ///
    /// This collects an [SP3Stream]. If you do not need the complete
    /// structure, iterate [SP3Stream::from_gzip_file] instead.
    pub fn from_gzip_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        SP3Stream::from_gzip_file(path)?.collect_sp3()
    }

    /// Parse [SP3] data from [Read]able I/O, entirely allocated in memory.
    ///
    /// This collects an [SP3Stream]. If you do not need the complete
    /// structure, iterate [SP3Stream::new] instead.
    pub fn from_reader<R: Read>(reader: &mut BufReader<R>) -> Result<Self, Error> {
        SP3Stream::new(reader)?.collect_sp3()
    }
}
