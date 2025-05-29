//! SP3 - SP3 test
use crate::prelude::*;
use std::path::PathBuf;

#[test]
#[cfg(feature = "flate2")]
fn substract_null() {
    let test_pool = PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("data/SP3")
        .join("C");

    let path_a = test_pool
        .clone()
        .join("EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz");

    let sp3_a = SP3::from_gzip_file(&path_a).unwrap();

    let null_sp3 = sp3_a.substract(&sp3_a);

    let mut passed = 0;

    for (_, v) in null_sp3.data.iter() {
        assert_eq!(v.position_km, (0.0, 0.0, 0.0));
        let clock_us = v.clock_us.unwrap();
        assert_eq!(clock_us, 0.0);
        passed += 1;
    }

    assert_eq!(passed, 10_176);
}
