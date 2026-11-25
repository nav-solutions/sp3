use crate::{prelude::*, tests::legacy::legacy_from_reader};

use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

/// All SP3 test files of the pool
fn test_pool() -> Vec<PathBuf> {
    let mut files = Vec::new();

    for revision in ["A", "C", "D"] {
        let dir = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/SP3")
            .join(revision);

        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();

            if is_gzip(&path) && !cfg!(feature = "flate2") {
                continue;
            }

            // Not supported yet, see test_pool.rs (TODO: omitted seconds)
            let filename = path.file_name().unwrap().to_string_lossy();
            if filename == "emr08874.sp3" || filename == "sio06492.sp3" {
                continue;
            }

            files.push(path);
        }
    }

    files.sort();
    files
}

fn is_gzip(path: &Path) -> bool {
    path.extension().map(|ext| ext == "gz").unwrap_or(false)
}

/// Opens the file as a dynamic [BufRead]
fn open(path: &Path) -> BufReader<Box<dyn Read>> {
    let fd = File::open(path).unwrap();

    #[cfg(feature = "flate2")]
    if is_gzip(path) {
        return BufReader::new(Box::new(GzDecoder::new(fd)));
    }

    BufReader::new(Box::new(fd))
}

#[test]
fn stream_collect_equals_legacy_parser() {
    let pool = test_pool();
    assert!(pool.len() > 10, "test pool is too small");

    for path in pool {
        let mut reader = open(&path);
        let legacy = legacy_from_reader(&mut reader)
            .unwrap_or_else(|e| panic!("legacy parser failed on {}: {}", path.display(), e));

        let collected = SP3Stream::new(open(&path))
            .unwrap()
            .collect_sp3()
            .unwrap_or_else(|e| panic!("collection failed on {}: {}", path.display(), e));

        assert_eq!(legacy.header, collected.header, "{}", path.display());
        assert_eq!(legacy.comments, collected.comments, "{}", path.display());
        assert_eq!(
            legacy.data.len(),
            collected.data.len(),
            "{}",
            path.display()
        );
        assert_eq!(legacy, collected, "{}", path.display());

        // historical API (which now collects the stream)
        let mut reader = open(&path);
        let from_reader = SP3::from_reader(&mut reader).unwrap();
        assert_eq!(legacy, from_reader, "{}", path.display());
    }
}

#[test]
fn stream_from_file_equals_from_file() {
    for path in test_pool() {
        let (streamed, parsed) = if is_gzip(&path) {
            #[cfg(feature = "flate2")]
            {
                (
                    SP3Stream::from_gzip_file(&path)
                        .unwrap()
                        .collect_sp3()
                        .unwrap(),
                    SP3::from_gzip_file(&path).unwrap(),
                )
            }
            #[cfg(not(feature = "flate2"))]
            unreachable!()
        } else {
            (
                SP3Stream::from_file(&path).unwrap().collect_sp3().unwrap(),
                SP3::from_file(&path).unwrap(),
            )
        };

        assert_eq!(streamed, parsed, "{}", path.display());

        // production attributes are deduced from the file name
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        if let Ok(attributes) = filename.parse::<ProductionAttributes>() {
            assert_eq!(streamed.prod_attributes, Some(attributes));
        }
    }
}

#[test]
fn stream_lines_are_consistent_with_collection() {
    for path in test_pool() {
        let stream = SP3Stream::new(open(&path)).unwrap();

        let header = stream.header().clone();
        let collected = SP3Stream::new(open(&path)).unwrap().collect_sp3().unwrap();

        // header is fully known before any data line is consumed
        assert_eq!(header, collected.header, "{}", path.display());

        let mut keys = HashSet::new();
        let mut last_epoch = None::<Epoch>;
        let mut nb_velocities = 0;

        for line in stream {
            let line = line.unwrap();

            // one line = one satellite at one epoch, in the file timescale
            assert_eq!(line.epoch().time_scale, header.timescale);

            // the stream preserves the chronological order of the file
            if let Some(last) = last_epoch {
                assert!(line.epoch() >= last, "{}", path.display());
            }
            last_epoch = Some(line.epoch());

            match line {
                SP3Line::Position(position) => {
                    if position.has_valid_position() {
                        keys.insert(position.key());

                        // the collected entry exists and matches this line
                        let entry = collected.data.get(&position.key()).unwrap();
                        assert_eq!(entry.position_km, position.position_km);
                        assert_eq!(entry.maneuver, position.maneuver);
                        assert_eq!(entry.predicted_orbit, position.predicted_orbit);
                    } else {
                        assert!(
                            !collected.data.contains_key(&position.key()),
                            "null position should not be collected"
                        );
                    }
                },
                SP3Line::Velocity(velocity) => {
                    nb_velocities += 1;

                    if velocity.has_valid_velocity() {
                        keys.insert(velocity.key());

                        let entry = collected.data.get(&velocity.key()).unwrap();
                        assert_eq!(entry.velocity_km_s, Some(velocity.velocity_km_s));
                    }
                },
            }
        }

        assert_eq!(keys.len(), collected.data.len(), "{}", path.display());

        if header.data_type == DataType::Velocity {
            assert!(nb_velocities > 0, "{}: missing velocities", path.display());
        } else {
            assert_eq!(nb_velocities, 0, "{}", path.display());
        }
    }
}

const CONTENT: &str = "#dP2020  6 24  0  0  0.00000000       2   u+U IGS14 FIT ESOC
## 2111 259200.00000000   900.00000000 59024 0.0000000000000
+    2   G01G02  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0
++         0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0
%c M  cc GPS ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc
%c cc cc ccc ccc cccc cccc cccc cccc ccccc ccccc ccccc ccccc
%f  1.2500000  1.025000000  0.00000000000  0.000000000000000
%f  0.0000000  0.000000000  0.00000000000  0.000000000000000
%i    0    0    0    0      0      0      0      0         0
%i    0    0    0    0      0      0      0      0         0
/* first comment
/* second comment
*  2020  6 24  0  0  0.00000000
PG01 -22335.782004 -14656.280389  -1218.238499   -176.397152 10  9 11 102 EP  MP
PG02      0.000000      0.000000      0.000000 999999.999999
*  2020  6 24  0 15  0.00000000
PG01 -22161.769422 -14419.212734   2853.227432   -176.395897
PG02 -15019.394530   1955.418434  21668.960108    114.548431
EOF
";

#[test]
fn in_memory_stream() {
    let mut stream = SP3Stream::new(CONTENT.as_bytes()).unwrap();

    assert_eq!(stream.header().version, Version::D);
    assert_eq!(stream.header().timescale, TimeScale::GPST);
    assert_eq!(stream.header().constellation, Constellation::Mixed);
    assert_eq!(stream.header().agency, "ESOC");
    assert_eq!(stream.comments(), &["first comment", "second comment"]);

    let t0 = Epoch::from_gregorian(2020, 6, 24, 0, 0, 0, 0, TimeScale::GPST);
    let t1 = t0 + Duration::from_seconds(900.0);

    let g01 = SV::new(Constellation::GPS, 1);
    let g02 = SV::new(Constellation::GPS, 2);

    // first line: fully flagged G01 state
    let first = stream.next().unwrap().unwrap();
    let first = first.as_position().unwrap();
    assert_eq!(first.epoch, t0);
    assert_eq!(first.sv, g01);
    assert_eq!(
        first.position_km,
        (-22335.782004, -14656.280389, -1218.238499)
    );
    assert_eq!(first.clock_us, Some(-176.397152));
    assert!(first.clock_event && first.predicted_clock);
    assert!(first.maneuver && first.predicted_orbit);
    assert!(first.has_valid_position());

    // null G02 is streamed, but marked invalid
    let second = stream.next().unwrap().unwrap();
    let second = second.as_position().unwrap();
    assert_eq!(second.sv, g02);
    assert!(!second.has_valid_position());

    let third = stream.next().unwrap().unwrap();
    assert_eq!(third.key(), SP3Key { epoch: t1, sv: g01 });
    assert_eq!(stream.current_epoch(), t1);

    let fourth = stream.next().unwrap().unwrap();
    assert_eq!(fourth.key(), SP3Key { epoch: t1, sv: g02 });

    // EOF terminates the stream (fused)
    assert!(stream.next().is_none());
    assert!(stream.next().is_none());

    // null state is not collected
    let sp3 = SP3Stream::new(CONTENT.as_bytes())
        .unwrap()
        .collect_sp3()
        .unwrap();

    assert_eq!(sp3.data.len(), 3);
    assert!(!sp3.data.contains_key(&SP3Key { epoch: t0, sv: g02 }));
}

#[test]
fn stream_reports_faulty_lines_and_carries_on() {
    let faulty = CONTENT.replace(
        "PG01 -22161.769422 -14419.212734   2853.227432   -176.395897",
        "PG01 -22161.7694xx -14419.212734   2853.227432   -176.395897",
    );

    let lines = SP3Stream::new(faulty.as_bytes())
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(lines.len(), 4);
    assert!(lines[0].is_ok());
    assert!(lines[1].is_ok());
    assert!(lines[2].is_err(), "faulty line should be reported");
    assert!(lines[3].is_ok(), "stream should carry on");

    // collecting stops on the first error, like the historical parser
    assert!(SP3Stream::new(faulty.as_bytes())
        .unwrap()
        .collect_sp3()
        .is_err());

    assert!(SP3::from_reader(&mut BufReader::new(faulty.as_bytes())).is_err());
}

#[test]
fn stream_without_data() {
    let header_only = CONTENT.split("*  2020").next().unwrap();

    let mut stream = SP3Stream::new(header_only.as_bytes()).unwrap();
    assert_eq!(stream.header().agency, "ESOC");
    assert!(stream.next().is_none());

    let sp3 = SP3Stream::new(header_only.as_bytes())
        .unwrap()
        .collect_sp3()
        .unwrap();

    assert!(sp3.data.is_empty());
}

#[test]
fn stream_with_unterminated_last_line() {
    // no EOF marker and no final \n
    let content = CONTENT.trim_end().trim_end_matches("EOF").trim_end();

    let n = SP3Stream::new(content.as_bytes())
        .unwrap()
        .filter_map(|line| line.ok())
        .count();

    assert_eq!(n, 4);
}

#[test]
fn stream_any_bufread() {
    // works with &mut references too (the historical API relies on this)
    let mut reader = BufReader::new(CONTENT.as_bytes());

    {
        let stream = SP3Stream::new(&mut reader).unwrap();
        assert_eq!(stream.count(), 4);
    }

    // reader fully consumed
    let mut rem = String::new();
    reader.read_line(&mut rem).unwrap();
    assert!(rem.is_empty());
}

#[test]
fn stream_file_open_error() {
    assert!(SP3Stream::from_file("this/file/does/not/exist.sp3").is_err());
    assert!(SP3::from_file("this/file/does/not/exist.sp3").is_err());
}

#[test]
#[cfg(feature = "flate2")]
fn stream_rev_a_velocities() {
    let mut stream =
        SP3Stream::from_gzip_file("data/SP3/A/NGA0OPSRAP_20251850000_01D_15M_ORB.SP3.gz").unwrap();

    assert_eq!(stream.header().version, Version::A);
    assert_eq!(stream.header().data_type, DataType::Velocity);

    let position = stream.next().unwrap().unwrap();
    let velocity = stream.next().unwrap().unwrap();

    assert_eq!(position.key(), velocity.key());

    let position = position.as_position().unwrap();
    let velocity = velocity.as_velocity().unwrap();

    assert_eq!(position.sv, SV::new(Constellation::GPS, 1));
    assert_eq!(position.epoch.time_scale, TimeScale::GPST);
    assert_eq!(position.clock_us, Some(307.266012));
    assert!((position.clock_offset_s().unwrap() - 307.266012E-6).abs() < 1.0E-15);

    let (vx, vy, vz) = velocity.velocity_km_s;
    assert!((vx - -0.8880949046).abs() < 1.0E-9);
    assert!((vy - -2.3142274905).abs() < 1.0E-9);
    assert!((vz - -1.4050679881).abs() < 1.0E-9);

    // 0.089376 (10⁻⁴ µs/s) = 0.0089376 ns/s
    assert!((velocity.clock_drift_ns.unwrap() - 0.0089376).abs() < 1.0E-12);
}
