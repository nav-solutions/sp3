//! Line by line (streaming) SP3 parsing.
//!
//! [SP3Stream] parses the SP3 header when created, then iterates the file
//! one data line at a time. Each line is decoded into a lightweight [SP3Line]
//! (no heap allocation per line) and handed to you, nothing is retained in memory.
//!
//! This is the building block of the historical, fully allocating
//! [SP3::from_file] / [SP3::from_reader] methods, which simply
//! [SP3Stream::collect_sp3] the stream.
//!
//! ```
//! use sp3::prelude::*;
//!
//! let mut stream = SP3Stream::from_file("data/SP3/C/co108870.sp3")
//!     .unwrap();
//!
//! // the header is fully parsed and available right away
//! assert_eq!(stream.header().version, Version::C);
//! assert_eq!(stream.header().timescale, TimeScale::GPST);
//!
//! let mut nb_positions = 0;
//!
//! for line in stream.by_ref() {
//!     match line.unwrap() {
//!         SP3Line::Position(position) => {
//!             // satellite + ECEF coordinates (+ clock) at this epoch
//!             let (_x_km, _y_km, _z_km) = position.position_km;
//!             let _ = (position.epoch, position.sv, position.clock_us);
//!             nb_positions += 1;
//!         },
//!         SP3Line::Velocity(_velocity) => {},
//!     }
//! }
//!
//! assert!(nb_positions > 0);
//! ```

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    header::{
        line1::{is_header_line1, Line1},
        line2::{is_header_line2, Line2},
    },
    position::{position_entry, PositionEntry},
    prelude::{
        Constellation, Epoch, Error, Header, ParsingError, ProductionAttributes, SP3Entry, SP3Key,
        TimeScale, Version, SP3, SV,
    },
    velocity::{velocity_entry, VelocityEntry},
    Vector3D,
};

#[cfg(doc)]
use std::io::Read;

fn file_descriptor(content: &str) -> bool {
    content.starts_with("%c")
}

fn sp3_comment(content: &str) -> bool {
    content.starts_with("/*")
}

fn end_of_file(content: &str) -> bool {
    content.eq("EOF")
}

fn new_epoch(content: &str) -> bool {
    content.starts_with("*  ")
}

/// Parses [Epoch] from standard SP3 format
pub(crate) fn parse_epoch(content: &str, timescale: TimeScale) -> Result<Epoch, ParsingError> {
    if content.len() < 27 {
        return Err(ParsingError::EpochParsing);
    }

    let y = u32::from_str(content[0..4].trim()).or(Err(ParsingError::EpochParsing))?;
    let m = u32::from_str(content[4..7].trim()).or(Err(ParsingError::EpochParsing))?;
    let d = u32::from_str(content[7..10].trim()).or(Err(ParsingError::EpochParsing))?;
    let hh = u32::from_str(content[10..13].trim()).or(Err(ParsingError::EpochParsing))?;
    let mm = u32::from_str(content[13..16].trim()).or(Err(ParsingError::EpochParsing))?;
    let ss = u32::from_str(content[16..19].trim()).or(Err(ParsingError::EpochParsing))?;
    let _ss_fract = f64::from_str(content[20..27].trim()).or(Err(ParsingError::EpochParsing))?;

    Epoch::from_str(&format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
        y, m, d, hh, mm, ss, timescale,
    ))
    .or(Err(ParsingError::Epoch))
}

/// [PositionLine] is one `P` line of an SP3 file: the state of one satellite
/// (ECEF position and optional clock offset) at one [Epoch], expressed
/// in the [TimeScale] defined in the [Header].
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PositionLine {
    /// [Epoch] of this state, in the file [TimeScale].
    pub epoch: Epoch,

    /// Satellite as [SV].
    pub sv: SV,

    /// ECEF position (x, y, z) in kilometers.
    /// SP3 uses (0, 0, 0) to describe a missing (bad or absent) position,
    /// see [PositionLine::has_valid_position].
    pub position_km: Vector3D,

    /// Clock offset in microseconds, when provided.
    pub clock_us: Option<f64>,

    /// Discontinuity in the satellite clock correction.
    pub clock_event: bool,

    /// True when the clock state is predicted.
    pub predicted_clock: bool,

    /// True when the vehicle is being maneuvered.
    pub maneuver: bool,

    /// True when the orbital state is predicted.
    pub predicted_orbit: bool,
}

impl PositionLine {
    /// Returns the [SP3Key] this line is indexed by, in a collected [SP3].
    pub fn key(&self) -> SP3Key {
        SP3Key {
            epoch: self.epoch,
            sv: self.sv,
        }
    }

    /// Returns true if all three coordinates are non zero.
    /// Lines that do not pass this test are not stored in a collected [SP3].
    pub fn has_valid_position(&self) -> bool {
        let (x, y, z) = self.position_km;
        x != 0.0 && y != 0.0 && z != 0.0
    }

    /// Returns clock offset in seconds, when provided.
    pub fn clock_offset_s(&self) -> Option<f64> {
        self.clock_us.map(|us| us * 1.0E-6)
    }

    /// Converts this [PositionLine] to [SP3Entry] (as stored in a collected [SP3]).
    pub fn to_entry(&self) -> SP3Entry {
        let value = if self.predicted_orbit {
            SP3Entry::from_predicted_position_km(self.position_km)
        } else {
            SP3Entry::from_position_km(self.position_km)
        };

        let mut value = match self.clock_us {
            Some(clk_us) if self.predicted_clock => value.with_predicted_clock_offset_us(clk_us),
            Some(clk_us) => value.with_clock_offset_us(clk_us),
            None => value,
        };

        value.maneuver = self.maneuver;
        value.clock_event = self.clock_event;
        value
    }
}

/// [VelocityLine] is one `V` line of an SP3 file: the velocity vector
/// (and optional clock drift) of one satellite at one [Epoch].
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VelocityLine {
    /// [Epoch] of this state, in the file [TimeScale].
    pub epoch: Epoch,

    /// Satellite as [SV].
    pub sv: SV,

    /// ECEF velocity (x, y, z) in km/s.
    /// (0, 0, 0) describes a missing velocity, see [VelocityLine::has_valid_velocity].
    pub velocity_km_s: Vector3D,

    /// Clock drift in nanoseconds per second, when provided.
    pub clock_drift_ns: Option<f64>,
}

impl VelocityLine {
    /// Returns the [SP3Key] this line is indexed by, in a collected [SP3].
    pub fn key(&self) -> SP3Key {
        SP3Key {
            epoch: self.epoch,
            sv: self.sv,
        }
    }

    /// Returns true if all three components are non zero.
    /// Lines that do not pass this test are not stored in a collected [SP3].
    pub fn has_valid_velocity(&self) -> bool {
        let (x, y, z) = self.velocity_km_s;
        x != 0.0 && y != 0.0 && z != 0.0
    }
}

/// [SP3Line] is one data line of an SP3 file, as produced by [SP3Stream].
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SP3Line {
    /// Satellite position (and clock) state.
    Position(PositionLine),

    /// Satellite velocity (and clock drift) state.
    Velocity(VelocityLine),
}

impl SP3Line {
    /// [Epoch] of this line.
    pub fn epoch(&self) -> Epoch {
        match self {
            Self::Position(p) => p.epoch,
            Self::Velocity(v) => v.epoch,
        }
    }

    /// Satellite described by this line.
    pub fn sv(&self) -> SV {
        match self {
            Self::Position(p) => p.sv,
            Self::Velocity(v) => v.sv,
        }
    }

    /// [SP3Key] of this line.
    pub fn key(&self) -> SP3Key {
        SP3Key {
            epoch: self.epoch(),
            sv: self.sv(),
        }
    }

    /// Returns [PositionLine] if this is a position line.
    pub fn as_position(&self) -> Option<&PositionLine> {
        match self {
            Self::Position(p) => Some(p),
            _ => None,
        }
    }

    /// Returns [VelocityLine] if this is a velocity line.
    pub fn as_velocity(&self) -> Option<&VelocityLine> {
        match self {
            Self::Velocity(v) => Some(v),
            _ => None,
        }
    }
}

/// [SP3Stream] is a line by line SP3 parser, generic over any [BufRead]able
/// interface (local file, gzip decoder, socket, in memory slice..).
///
/// - the [Header] (and header comments) are parsed when the stream is created
/// - [SP3Stream] is an [Iterator] of `Result<SP3Line, Error>`: each item is
///   one data line (one satellite at one epoch). Lines are not retained.
/// - an internal line buffer is reused, so iterating does not allocate per line.
/// - an I/O error terminates the stream. A line that fails to parse is reported
///   as an `Err` item, and the stream carries on with the next line: you decide
///   whether to stop or tolerate it.
/// - [SP3Stream::collect_sp3] collects everything into a fully allocated [SP3],
///   exactly like [SP3::from_reader] (which is implemented that way).
pub struct SP3Stream<R: BufRead> {
    reader: R,
    buf: String,
    header: Header,
    comments: Vec<String>,
    timescale: TimeScale,
    epoch: Epoch,
    pc_count: u8,
    /// a data line was read while parsing the header, and is to be processed first
    pending: bool,
    done: bool,
    prod_attributes: Option<ProductionAttributes>,
}

impl<R: BufRead> SP3Stream<R> {
    /// Creates a new [SP3Stream] from any [BufRead]able interface.
    /// The [Header] section is parsed immediately: the stream is positioned
    /// on the first data line when this returns.
    ///
    /// ```
    /// use sp3::prelude::*;
    /// use std::{fs::File, io::BufReader};
    ///
    /// let fd = File::open("data/SP3/D/example.txt").unwrap();
    /// let stream = SP3Stream::new(BufReader::new(fd)).unwrap();
    ///
    /// assert_eq!(stream.header().version, Version::D);
    ///
    /// // Iterator adapters work as usual. For example, positions
    /// // of a single satellite, without retaining anything else.
    /// let g01 = SV::new(Constellation::GPS, 1);
    ///
    /// let g01_positions = stream
    ///     .filter_map(|line| line.ok())
    ///     .filter_map(|line| line.as_position().copied())
    ///     .filter(|position| position.sv == g01)
    ///     .count();
    /// ```
    pub fn new(reader: R) -> Result<Self, Error> {
        let mut s = Self {
            reader,
            buf: String::with_capacity(128),
            header: Header::default(),
            comments: Vec::new(),
            timescale: TimeScale::default(),
            epoch: Epoch::default(),
            pc_count: 0,
            pending: false,
            done: false,
            prod_attributes: None,
        };

        s.parse_header()?;
        Ok(s)
    }

    /// Defines [ProductionAttributes] (usually deduced from the file name).
    pub fn with_production_attributes(mut self, attributes: ProductionAttributes) -> Self {
        self.prod_attributes = Some(attributes);
        self
    }

    /// Returns the [Header], parsed when this stream was created.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Returns all comments found so far (header comments
    /// are all known once the stream has been created).
    pub fn comments(&self) -> &[String] {
        &self.comments
    }

    /// Returns [ProductionAttributes], if they could be determined.
    pub fn production_attributes(&self) -> Option<&ProductionAttributes> {
        self.prod_attributes.as_ref()
    }

    /// Returns the latest [Epoch] encountered.
    pub fn current_epoch(&self) -> Epoch {
        self.epoch
    }

    /// Consumes the header section, stops on the first epoch descriptor
    /// (included) or the first data line (left pending).
    fn parse_header(&mut self) -> Result<(), Error> {
        loop {
            self.buf.clear();

            if self.reader.read_line(&mut self.buf)? == 0 {
                self.done = true;
                return Ok(());
            }

            let line = self.buf.trim();

            if position_entry(line) || velocity_entry(line) {
                self.pending = true;
                return Ok(());
            }

            let is_epoch = new_epoch(line);

            // header lines never produce data
            let _ = self.process_line()?;

            if is_epoch || self.done {
                return Ok(());
            }
        }
    }

    /// Processes the line currently held in the internal buffer.
    fn process_line(&mut self) -> Result<Option<SP3Line>, Error> {
        let line = self.buf.trim();

        if sp3_comment(line) {
            if line.len() > 4 {
                self.comments.push(line[3..].to_string());
            }
            return Ok(None);
        }

        if end_of_file(line) {
            self.done = true;
            return Ok(None);
        }

        if is_header_line1(line) && !is_header_line2(line) {
            let l1 = Line1::from_str(line)?;

            self.header.version = l1.version;
            self.header.data_type = l1.data_type;
            self.header.coord_system = l1.coord_system;
            self.header.orbit_type = l1.orbit_type;
            self.header.agency = l1.agency.to_string();
            self.header.num_epochs = l1.num_epochs;
            self.header.observables = l1.observables.to_string();
            self.header.release_epoch = l1.epoch;
            return Ok(None);
        }

        if is_header_line2(line) {
            let l2 = Line2::from_str(line)?;
            self.header.week = l2.week;
            self.header.week_nanos = l2.week_nanos;
            self.header.sampling_period = l2.sampling_period;
            self.header.mjd = l2.mjd_fract.0;
            self.header.mjd_fraction = l2.mjd_fract.1;
            return Ok(None);
        }

        if file_descriptor(line) {
            if line.len() < 60 {
                return Err(Error::ParsingError(ParsingError::MalformedDescriptor(
                    line.to_string(),
                )));
            }

            if self.header.version == Version::A {
                // Rev-A is limited to GPS: no need to parse this line
                self.header.constellation = Constellation::GPS;
                self.header.timescale = TimeScale::GPST;
                self.timescale = TimeScale::GPST;
            } else if self.pc_count == 0 {
                // Constellation identification needs to pass
                self.header.constellation = Constellation::from_str(line[3..5].trim())?;
                self.timescale = TimeScale::from_str(line[9..12].trim())?;
                self.header.timescale = self.timescale;
            }

            self.pc_count = self.pc_count.saturating_add(1);
            return Ok(None);
        }

        if new_epoch(line) {
            self.epoch = parse_epoch(&line[3..], self.timescale)?;
            return Ok(None);
        }

        if position_entry(line) {
            if line.len() < 60 {
                // tolerates malformed position vectors
                return Ok(None);
            }

            let entry = PositionEntry::parse(line, self.header.version)?;

            return Ok(Some(SP3Line::Position(PositionLine {
                epoch: self.epoch,
                sv: entry.sv,
                position_km: (entry.x_km, entry.y_km, entry.z_km),
                clock_us: entry.clock_us,
                clock_event: entry.clock_event,
                predicted_clock: entry.clock_prediction,
                maneuver: entry.maneuver,
                predicted_orbit: entry.orbit_prediction,
            })));
        }

        if velocity_entry(line) {
            if line.len() < 60 {
                // tolerates malformed velocity vectors
                return Ok(None);
            }

            let entry = VelocityEntry::parse(line, self.header.version)?;
            let (sv, velocity_km_s, clk_sub_ns) = entry.to_parts();

            return Ok(Some(SP3Line::Velocity(VelocityLine {
                epoch: self.epoch,
                sv,
                velocity_km_s,
                // 10⁻⁴ µs/s => ns/s
                clock_drift_ns: clk_sub_ns.map(|clk| clk * 0.1),
            })));
        }

        Ok(None)
    }

    /// Collects this [SP3Stream] into a fully allocated [SP3] structure.
    /// Stops on (and returns) the first error.
    ///
    /// ```
    /// use sp3::prelude::*;
    ///
    /// let sp3 = SP3Stream::from_file("data/SP3/D/example.txt")
    ///     .unwrap()
    ///     .collect_sp3()
    ///     .unwrap();
    ///
    /// // strictly identical to the historical method
    /// assert_eq!(sp3, SP3::from_file("data/SP3/D/example.txt").unwrap());
    /// ```
    pub fn collect_sp3(mut self) -> Result<SP3, Error> {
        let mut sp3 = SP3 {
            header: Header::default(),
            comments: Vec::new(),
            prod_attributes: None,
            data: Default::default(),
        };

        for line in self.by_ref() {
            sp3.insert_line(line?);
        }

        sp3.header = self.header;
        sp3.comments = self.comments;
        sp3.prod_attributes = self.prod_attributes;
        Ok(sp3)
    }
}

impl SP3Stream<BufReader<File>> {
    /// Creates a new [SP3Stream] from local file.
    /// [ProductionAttributes] are deduced from file names that follow
    /// standard conventions.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let fd = File::open(&path)?;
        let stream = Self::new(BufReader::new(fd))?;
        Ok(stream.with_file_name(path.as_ref()))
    }
}

#[cfg(feature = "flate2")]
#[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
impl SP3Stream<BufReader<GzDecoder<File>>> {
    /// Creates a new [SP3Stream] from gzip compressed local file.
    /// Decompression happens on the fly, the file is never fully decompressed in memory.
    ///
    /// ```
    /// use sp3::prelude::*;
    ///
    /// let stream = SP3Stream::from_gzip_file(
    ///     "data/SP3/C/ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz",
    /// ).unwrap();
    ///
    /// assert_eq!(stream.header().agency, "ESOC");
    /// assert!(stream.production_attributes().is_some());
    ///
    /// // find the latest epoch, with constant memory usage
    /// let last = stream
    ///     .filter_map(|line| line.ok())
    ///     .map(|line| line.epoch())
    ///     .max()
    ///     .unwrap();
    /// ```
    pub fn from_gzip_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let fd = File::open(&path)?;
        let stream = Self::new(BufReader::new(GzDecoder::new(fd)))?;
        Ok(stream.with_file_name(path.as_ref()))
    }
}

impl<R: BufRead> SP3Stream<R> {
    fn with_file_name(mut self, path: &Path) -> Self {
        if let Some(filename) = path.file_name() {
            if let Ok(attributes) = ProductionAttributes::from_str(&filename.to_string_lossy()) {
                self.prod_attributes = Some(attributes);
            }
        }
        self
    }
}

impl<R: BufRead> Iterator for SP3Stream<R> {
    type Item = Result<SP3Line, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pending {
            self.pending = false;

            match self.process_line() {
                Ok(Some(line)) => return Some(Ok(line)),
                Ok(None) => {},
                Err(e) => return Some(Err(e)),
            }
        }

        loop {
            if self.done {
                return None;
            }

            self.buf.clear();

            match self.reader.read_line(&mut self.buf) {
                Ok(0) => {
                    self.done = true;
                    return None;
                },
                Ok(_) => match self.process_line() {
                    Ok(Some(line)) => return Some(Ok(line)),
                    Ok(None) => continue,
                    Err(e) => return Some(Err(e)),
                },
                Err(e) => {
                    self.done = true;
                    return Some(Err(e.into()));
                },
            }
        }
    }
}

impl<R: BufRead> std::iter::FusedIterator for SP3Stream<R> {}

impl SP3 {
    /// Inserts one [SP3Line] (for example, obtained with [SP3Stream])
    /// into this [SP3]. This is how [SP3Stream::collect_sp3] builds the
    /// fully allocated structure:
    /// - lines with null vectors are not stored
    /// - a velocity line completes the position line of the same [SP3Key],
    ///   the prediction flag of the position line is preserved
    ///   (if the velocity comes first, a null position is introduced).
    pub fn insert_line(&mut self, line: SP3Line) {
        match line {
            SP3Line::Position(position) => {
                if !position.has_valid_position() {
                    return;
                }

                let key = position.key();

                if let Some(e) = self.data.get_mut(&key) {
                    e.position_km = position.position_km;
                    e.maneuver = position.maneuver;
                    e.predicted_orbit = position.predicted_orbit;
                } else {
                    self.data.insert(key, position.to_entry());
                }
            },
            SP3Line::Velocity(velocity) => {
                if !velocity.has_valid_velocity() {
                    return;
                }

                let key = velocity.key();

                if let Some(e) = self.data.get_mut(&key) {
                    // V lines do not have a prediction flag:
                    // preserve the flag of the P line
                    *e = if e.predicted_orbit {
                        e.with_predicted_velocity_km_s(velocity.velocity_km_s)
                    } else {
                        e.with_velocity_km_s(velocity.velocity_km_s)
                    };

                    if let Some(drift_ns) = velocity.clock_drift_ns {
                        *e = e.with_clock_drift_ns(drift_ns);
                    }
                } else {
                    // Entry does not exist (velocity prior position)
                    // Should not exist, but we tolerate
                    let mut entry = SP3Entry::from_position_km((0.0, 0.0, 0.0))
                        .with_velocity_km_s(velocity.velocity_km_s);

                    if let Some(drift_ns) = velocity.clock_drift_ns {
                        entry = entry.with_clock_drift_ns(drift_ns);
                    }

                    self.data.insert(key, entry);
                }
            },
        }
    }
}
