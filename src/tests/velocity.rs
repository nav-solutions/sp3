use crate::prelude::*;
use std::str::FromStr;

#[test]
#[cfg(feature = "flate2")]
fn rev_a_velocity_vectors() {
    let sp3 = SP3::from_gzip_file("data/SP3/A/NGA0OPSRAP_20251850000_01D_15M_ORB.SP3.gz")
        .unwrap_or_else(|e| panic!("failed to parse SP3/A file: {}", e));

    assert!(sp3.has_satellite_velocity());

    let g01 = SV::from_str("G01").unwrap();
    let t0 = Epoch::from_str("2025-07-04T00:00:00 GPST").unwrap();

    let (_, entry) = sp3
        .data
        .iter()
        .find(|(k, _)| k.sv == g01 && k.epoch == t0)
        .expect("missing G01 state at t0");

    let (vx, vy, vz) = entry.velocity_km_s.expect("missing velocity vector");

    // V  1  -8880.949046 -23142.274905 -14050.679881 (dm/s)
    assert!((vx - -0.8880949046).abs() < 1.0E-9, "vx={}", vx);
    assert!((vy - -2.3142274905).abs() < 1.0E-9, "vy={}", vy);
    assert!((vz - -1.4050679881).abs() < 1.0E-9, "vz={}", vz);

    // GPS orbital speed is about 3.9 km/s in inertial frame,
    // ECEF speed is smaller but within [1; 5] km/s.
    let norm = (vx * vx + vy * vy + vz * vz).sqrt();
    assert!(
        norm > 1.0 && norm < 5.0,
        "unrealistic velocity norm {}",
        norm
    );
}

#[test]
#[cfg(feature = "flate2")]
fn rev_a_predicted_states_with_velocity() {
    let sp3 = SP3::from_gzip_file("data/SP3/A/NGA0OPSRAP_20251850000_01D_15M_ORB.SP3.gz")
        .unwrap_or_else(|e| panic!("failed to parse SP3/A file: {}", e));

    let g01 = SV::from_str("G01").unwrap();

    // P  1  18204.177455   7476.601076  17846.619585    307.658902               P   P
    // V  1   9269.408044  20744.927110 -18113.301198      0.089424
    let t = Epoch::from_str("2025-07-04T12:15:00 GPST").unwrap();

    let entry = sp3
        .data
        .get(&SP3Key { sv: g01, epoch: t })
        .expect("missing G01 state");

    assert!(entry.velocity_km_s.is_some());
    assert!(entry.predicted_clock);
    assert!(
        entry.predicted_orbit,
        "velocity vector should not erase the orbit prediction flag"
    );

    // the first states of this file are not predicted
    let t0 = Epoch::from_str("2025-07-04T00:00:00 GPST").unwrap();
    let entry = sp3.data.get(&SP3Key { sv: g01, epoch: t0 }).unwrap();
    assert!(!entry.predicted_orbit);
}
