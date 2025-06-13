use thiserror::Error;

use gnss_rs::constellation::ParsingError as ConstellationParsingError;
use hifitime::errors::ParsingError as EpochParsingError;
use std::io::Error as IoError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingError),

    #[error("Epoch parsing error: {0}")]
    HifitimeParsingError(#[from] EpochParsingError),

    #[error("Constellation parsing error: {0}")]
    ConstellationParsing(#[from] ConstellationParsingError),

    #[error("File i/o error: {0}")]
    FileIo(#[from] IoError),
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Non supported SP3 revision")]
    NonSupportedRevision,

    #[error("Unknown SP3 orbit type")]
    UnknownOrbitType,

    #[error("Unknown SP3 data type")]
    UnknownDataType,

    #[error("malformed header line #1")]
    MalformedH1,

    #[error("malformed header line #2")]
    MalformedH2,

    #[error("malformed %c line \"{0}\"")]
    MalformedDescriptor(String),

    #[error("failed to parse Epoch")]
    EpochParsing,

    #[error("failed to parse number of epochs \"{0}\"")]
    NumberEpoch(String),

    #[error("failed to parse week counter")]
    WeekCounter,

    #[error("failed to parse seconds of week")]
    WeekSeconds,

    #[error("failed to parse Epoch")]
    Epoch,

    #[error("failed to parse sampling period")]
    SamplingPeriod,

    #[error("failed to parse MJD")]
    Mjd,

    #[error("failed to parse sv from \"{0}\"")]
    SV(String),

    #[error("failed to parse (x, y, or z) coordinates from \"{0}\"")]
    Coordinates(String),

    #[error("failed to parse clock data from \"{0}\"")]
    Clock(String),

    #[error("Not a standardized filename")]
    InvalidFilename,

    #[error("Invalid file availability")]
    InvalidFileAvailability,

    #[error("not a valid IGS campaign name")]
    InvalidCampaignName,
}

/// Errors that may rise in Formatting process
#[derive(Error, Debug)]
pub enum FormattingError {
    #[error("i/o: output error")]
    OutputError(#[from] IoError),
}
