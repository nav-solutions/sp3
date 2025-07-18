//! Position & Clock data parsing
use crate::{
    errors::ParsingError,
    prelude::{Constellation, Version, SV},
};

use std::str::FromStr;

pub fn position_entry(content: &str) -> bool {
    content.starts_with('P')
}

pub struct PositionEntry {
    pub sv: SV,
    pub x_km: f64,
    pub y_km: f64,
    pub z_km: f64,
    pub clock_us: Option<f64>,
    pub clock_event: bool,
    pub clock_prediction: bool,
    pub maneuver: bool,
    pub orbit_prediction: bool,
}

impl PositionEntry {
    pub fn parse(line: &str, revision: Version) -> Result<Self, ParsingError> {
        let line_len = line.len();

        let mut clock_event = false;
        let mut clock_prediction = false;
        let mut maneuver = false;
        let mut orbit_prediction = false;

        let mut clock_us: Option<f64> = None;

        let sv = match revision {
            Version::A => {
                // GPS-Only: constellation might be omitted
                let prn = line[2..4].trim().parse::<u8>().or(Err(ParsingError::SV))?;

                SV::new(Constellation::GPS, prn)
            },
            _ => {
                // parsing needs to pass
                SV::from_str(line[1..4].trim()).or(Err(ParsingError::SV))?
            },
        };

        let x = f64::from_str(line[4..18].trim())
            .or(Err(ParsingError::Coordinates(line[4..18].to_string())))?;

        let y = f64::from_str(line[18..32].trim())
            .or(Err(ParsingError::Coordinates(line[18..32].to_string())))?;

        let z = f64::from_str(line[32..46].trim())
            .or(Err(ParsingError::Coordinates(line[32..46].to_string())))?;

        if line_len > 51 && !line[45..52].trim().eq("999999.") {
            // clock data present
            let clk_data = f64::from_str(line[46..60].trim())
                .or(Err(ParsingError::Clock(line[46..60].to_string())))?;
            clock_us = Some(clk_data);
        }

        if line_len > 74 && line[74..75].eq("E") {
            clock_event = true;
        }

        if line_len > 75 && line[75..76].eq("P") {
            clock_prediction = true;
        }

        if line_len > 78 && line[78..79].eq("M") {
            maneuver = true;
        }

        if line_len > 79 && line[79..80].eq("P") {
            orbit_prediction = true;
        }

        Ok(Self {
            sv,
            clock_us,
            clock_event,
            clock_prediction,
            orbit_prediction,
            maneuver,
            x_km: x,
            y_km: y,
            z_km: z,
        })
    }
}

#[cfg(test)]
mod test {
    use super::PositionEntry;
    use crate::prelude::{Version, SV};
    use std::str::FromStr;

    #[test]
    fn position_entry_parsing() {
        for (
            content,
            sv,
            x_km,
            y_km,
            z_km,
            clock_offset_us,
            clock_event,
            clock_prediction,
            maneuver,
            orbit_prediction,
        ) in [
            (
                "PC01 -32312.652253  27060.656563    205.195454     63.035497",
                "C01",
                -32312.652253,
                27060.656563,
                205.195454,
                Some(63.035497),
                false,
                false,
                false,
                false,
            ),
            (
                "PG01 -22335.782004 -14656.280389  -1218.238499   -176.397152 10  9 11 102      P",
                "G01",
                -22335.782004,
                -14656.280389,
                -1218.238499,
                Some(-176.397152),
                false,
                false,
                false,
                true,
            ),
            (
                "PG01 -22335.782004 -14656.280389  -1218.238499   -176.397152 10  9 11 102     MP",
                "G01",
                -22335.782004,
                -14656.280389,
                -1218.238499,
                Some(-176.397152),
                false,
                false,
                true,
                true,
            ),
            (
                "PG01 -22335.782004 -14656.280389  -1218.238499   -176.397152 10  9 11 102 E",
                "G01",
                -22335.782004,
                -14656.280389,
                -1218.238499,
                Some(-176.397152),
                true,
                false,
                false,
                false,
            ),
            (
                "PG01 -22335.782004 -14656.280389  -1218.238499   -176.397152 10  9 11 102  P",
                "G01",
                -22335.782004,
                -14656.280389,
                -1218.238499,
                Some(-176.397152),
                false,
                true,
                false,
                false,
            ),
            (
                "PG23      0.000000      0.000000      0.000000 999999.999999                  M",
                "G23",
                0.000000,
                0.000000,
                0.000000,
                Some(999999.999999),
                false,
                false,
                true,
                false,
            ),
        ] {
            let sv = SV::from_str(sv).unwrap();
            let entry = PositionEntry::parse(content, Version::C).unwrap();
            assert_eq!(entry.sv, sv);
            assert_eq!(entry.x_km, x_km);
            assert_eq!(entry.y_km, y_km);
            assert_eq!(entry.z_km, z_km);
            assert_eq!(entry.clock_us, clock_offset_us);
            assert_eq!(entry.clock_event, clock_event);
            assert_eq!(entry.clock_prediction, clock_prediction);
            assert_eq!(entry.maneuver, maneuver);
            assert_eq!(entry.orbit_prediction, orbit_prediction);
        }
    }

    #[test]
    fn sp3_d_predicted_position() {
        let g01 = SV::from_str("G01").unwrap();

        let content =
            "PG01 -22335.782004 -14656.280389  -1218.238499   -176.397152 10  9 11 102 EP  MP";

        let position = PositionEntry::parse(content, Version::C).unwrap_or_else(|e| {
            panic!("Failed to parse predicted state \"{}\": {}", content, e);
        });

        assert_eq!(position.sv, g01);
        assert_eq!(position.x_km, -22335.782004);
        assert_eq!(position.y_km, -14656.280389);
        assert_eq!(position.z_km, -1218.238499);
        assert!(position.clock_event);
        assert!(position.clock_prediction);
        assert!(position.maneuver);
        assert!(position.orbit_prediction);
    }
}
