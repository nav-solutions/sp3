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

            let sp3 = SP3::from_gzip_file(&file_path);
            assert!(
                sp3.is_ok(),
                "failed to parse data/{}, error: {:?}",
                file,
                sp3.err()
            );
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
            let sp3 = SP3::from_file(&file_path);
            assert!(
                sp3.is_ok(),
                "failed to parse data/{}, error: {:?}",
                file,
                sp3.err()
            );
        }
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn rev_d_gzip() {
        let prefix = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/SP3")
            .join("D");

        for file in [
            "COD0MGXFIN_20230500000_01D_05M_ORB.SP3.gz",
            "Sta21114.sp3.gz",
        ] {
            let file_path = prefix.clone().join(file);
            println!("Parsing file \"{}\"", file_path.to_string_lossy());

            let sp3 = SP3::from_gzip_file(&file_path);
            assert!(
                sp3.is_ok(),
                "failed to parse data/{}, error: {:?}",
                file,
                sp3.err()
            );
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
            let sp3 = SP3::from_file(&file_path);
            assert!(
                sp3.is_ok(),
                "failed to parse data/{}, error: {:?}",
                file,
                sp3.err()
            );
        }
    }
}
