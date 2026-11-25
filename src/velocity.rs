//! Velocity entry parsing
use crate::{
    errors::ParsingError,
    prelude::{Constellation, Version, SV},
};

use std::str::FromStr;

pub fn velocity_entry(content: &str) -> bool {
    content.starts_with('V')
}

pub struct VelocityEntry {
    sv: SV,
    velocity: (f64, f64, f64),
    clock: Option<f64>,
}

impl VelocityEntry {
    pub fn parse(line: &str, revision: Version) -> Result<Self, ParsingError> {
        let mut clock: Option<f64> = None;

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

        let x_km = f64::from_str(line[4..18].trim())
            .or(Err(ParsingError::Coordinates(line[4..18].to_string())))?
            * 1.0E-4;

        let y_km = f64::from_str(line[18..32].trim())
            .or(Err(ParsingError::Coordinates(line[18..32].to_string())))?
            * 1.0E-4;

        let z_km = f64::from_str(line[32..46].trim())
            .or(Err(ParsingError::Coordinates(line[32..46].to_string())))?
            * 1.0E-4;

        if !line[45..52].trim().eq("999999.") {
            /*
             * Clock data present
             */
            let clk_data = f64::from_str(line[46..60].trim())
                .or(Err(ParsingError::Clock(line[46..60].to_string())))?;

            clock = Some(clk_data);
        }

        Ok(Self {
            sv,
            velocity: (x_km, y_km, z_km),
            clock,
        })
    }
}

impl VelocityEntry {
    pub fn to_parts(&self) -> (SV, (f64, f64, f64), Option<f64>) {
        (self.sv, self.velocity, self.clock)
    }
}

#[cfg(test)]
mod test {
    use super::VelocityEntry;
    use crate::prelude::{Version, SV};
    use std::str::FromStr;

    #[test]
    fn velocity_entry_parsing() {
        let content =
            "V  1  -8880.949046 -23142.274905 -14050.679881      0.089376                    ";

        let entry = VelocityEntry::parse(content, Version::A).unwrap();
        let (sv, (vx, vy, vz), clk) = entry.to_parts();

        assert_eq!(sv, SV::from_str("G01").unwrap());

        // dm/s => km/s
        assert!((vx - -0.8880949046).abs() < 1.0E-12);
        assert!((vy - -2.3142274905).abs() < 1.0E-12);
        assert!((vz - -1.4050679881).abs() < 1.0E-12);
        assert_eq!(clk, Some(0.089376));
    }
}
