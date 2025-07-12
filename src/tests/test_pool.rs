#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::path::PathBuf;

    #[test]
    #[cfg(feature = "flate2")]
    fn rev_c_gzip() {
        let prefix = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/SP3")
            .join("C");

        for file in [
            "EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz",
            "ESA0OPSULT_20232320600_02D_15M_ORB.SP3.gz",
            "ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz",
            "GRG0MGXFIN_20201760000_01D_15M_ORB.SP3.gz",
            "GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz",
            "ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz",
        ] {
            let file_path = prefix.clone().join(file);
            println!("Parsing file \"{}\"", file_path.to_string_lossy());

            let sp3 = SP3::from_gzip_file(&file_path).unwrap_or_else(|e| {
                panic!("failed to parse data/C/{}: {}", file, e);
            });

            // dump
            sp3.to_file("test1.txt").unwrap_or_else(|e| {
                panic!("Failed to dump data/C/{}: {}", file, e);
            });

            // parse back
            let _ = SP3::from_file("test1.txt").unwrap_or_else(|e| {
                panic!("Failed to parse dumped data/C/{}: {}", file, e);
            });

            // assert_eq!(parsed_back, sp3); // TODO
        }
    }

    #[test]
    fn rev_c_folder() {
        let prefix = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/SP3")
            .join("C");

        for file in ["co108870.sp3", "em108871.sp3"] {
            let file_path = prefix.clone().join(file);
            println!("Parsing file \"{}\"", file_path.to_string_lossy());

            let sp3 = SP3::from_file(&file_path).unwrap_or_else(|e| {
                panic!("failed to parse data/C/{}: {}", file, e);
            });

            // dump
            sp3.to_file("test2.txt").unwrap_or_else(|e| {
                panic!("Failed to dump data/C/{}: {}", file, e);
            });

            // parse back
            let parsed_back = SP3::from_file("test2.txt").unwrap_or_else(|e| {
                panic!("Failed to parse dumped data/C/{}: {}", file, e);
            });

            assert_eq!(parsed_back, sp3);
        }
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn rev_d_gzip() {
        let prefix = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/SP3")
            .join("D");

        for (file, expected_name) in [
            (
                "COD0MGXFIN_20230500000_01D_05M_ORB.SP3.gz",
                "COD0MGXFIN_202305000000_01D_05M_ORB.SP3.gz",
            ),
            ("Sta21114.sp3.gz", "IAC0OPSRAP_20201770000_01D_15M_ORB.SP3"),
        ] {
            let file_path = prefix.clone().join(file);
            println!("Parsing file \"{}\"", file_path.to_string_lossy());

            let sp3 = SP3::from_gzip_file(&file_path).unwrap_or_else(|e| {
                panic!("failed to parse data/D/{}: {}", file, e);
            });

            assert_eq!(sp3.standardized_filename(), expected_name);

            // dump
            sp3.to_file("test3.txt").unwrap_or_else(|e| {
                panic!("Failed to dump data/D/{}: {}", file, e);
            });

            // parse back
            let _ = SP3::from_file("test3.txt").unwrap_or_else(|e| {
                panic!("Failed to parse dumped data/D/{}: {}", file, e);
            });

            // assert_eq!(parsed_back, sp3); // TODO
        }
    }

    #[test]
    fn rev_d_folder() {
        let prefix = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/SP3")
            .join("D");

        {
            let file = "example.txt";
            let file_path = prefix.clone().join(file);
            println!("Parsing file \"{}\"", file_path.to_string_lossy());

            let sp3 = SP3::from_file(&file_path).unwrap_or_else(|e| {
                panic!("failed to parse data/D/{}: {}", file, e);
            });

            assert_eq!(
                sp3.standardized_filename(),
                "IGS0OPSRAP_20193000000_01D_05M_ORB.SP3"
            );

            sp3.to_file("test4.txt").unwrap_or_else(|e| {
                panic!("Failed to dump data/D/{}: {}", file, e);
            });

            // parse back
            let parsed_back = SP3::from_file("test4.txt").unwrap_or_else(|e| {
                panic!("Failed to parse dumped data/D/{}: {}", file, e);
            });

            assert_eq!(parsed_back, sp3); // TODO
        }
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn rev_a() {
        let prefix = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/SP3")
            .join("A");

        for file in [
            // "emr08874.sp3", // TODO ? THH:MM:SS with omitted seconds..
            // "sio06492.sp3", // TODO? THH:MM:SS with omitted seconds..
            "NGA0OPSRAP_20251850000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251860000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251870000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251880000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251890000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251900000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251910000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251920000_01D_15M_ORB.SP3.gz",
            "NGA0OPSRAP_20251930000_01D_15M_ORB.SP3.gz",
        ] {
            let file_path = prefix.clone().join(file);
            println!("Parsing file \"{}\"", file_path.to_string_lossy());

            let sp3 = SP3::from_gzip_file(&file_path).unwrap_or_else(|e| {
                panic!("failed to parse data/A/{}: {}", file, e);
            });

            // dump
            sp3.to_file("test1.txt").unwrap_or_else(|e| {
                panic!("Failed to dump data/A/{}: {}", file, e);
            });

            // parse back
            let _ = SP3::from_file("test1.txt").unwrap_or_else(|e| {
                panic!("Failed to parse dumped data/C/{}: {}", file, e);
            });

            // assert_eq!(parsed_back, sp3); // TODO
        }
    }
}
