#[cfg(doc)]
use crate::prelude::SP3Key;

use crate::{formatting::CoordsFormatter, prelude::SV, FormattingError, Vector3D};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::io::{BufWriter, Write};

/// SP3 record content are [SP3Entry] indexed by [SP3Key].
#[derive(Debug, Copy, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SP3Entry {
    /// ECEF position in kilometers with 10⁻³ precision.
    pub position_km: Vector3D,

    /// ECEF velocity vectori in km.s⁻¹.
    pub velocity_km_s: Option<Vector3D>,

    /// True if the state vector is predicted
    pub orbit_prediction: bool,

    /// True if vehicle being maneuvered (rocket truster)
    /// since last state.
    pub maneuver: bool,

    /// Discontinuity in the satellite clock correction
    /// (for example: internal clock swap)
    pub clock_event: bool,

    /// True when the clock state is actually predicted
    pub clock_prediction: bool,

    /// Clock offset correction, in microsecond with 10⁻¹² precision.
    pub clock_us: Option<f64>,

    /// Clock drift in nanoseconds with 10⁻¹⁶ precision.
    pub clock_drift_ns: Option<f64>,
}

impl std::ops::Sub for SP3Entry {
    type Output = SP3Entry;

    fn sub(self, rhs: Self) -> Self {
        Self {
            position_km: (
                self.position_km.0 - rhs.position_km.0,
                self.position_km.1 - self.position_km.1,
                self.position_km.2 - rhs.position_km.2,
            ),
            velocity_km_s: if let Some(velocity_km_s) = self.velocity_km_s {
                rhs.velocity_km_s.map(|rhs| (
                        velocity_km_s.0 - rhs.0,
                        velocity_km_s.1 - rhs.1,
                        velocity_km_s.2 - rhs.2,
                    ))
            } else {
                None
            },
            maneuver: self.maneuver,
            clock_event: self.clock_event,
            clock_prediction: self.clock_prediction,
            orbit_prediction: self.orbit_prediction,
            clock_us: if let Some(clock_us) = self.clock_us {
                rhs.clock_us.map(|rhs| clock_us - rhs)
            } else {
                None
            },
            clock_drift_ns: if let Some(clock_drift_ns) = self.clock_drift_ns {
                rhs.clock_drift_ns.map(|rhs| clock_drift_ns - rhs)
            } else {
                None
            },
        }
    }
}

impl std::ops::SubAssign for SP3Entry {
    fn sub_assign(&mut self, rhs: Self) {
        self.position_km.0 -= rhs.position_km.0;
        self.position_km.1 -= rhs.position_km.1;
        self.position_km.2 -= rhs.position_km.2;

        if let Some(velocity_km_s) = &mut self.velocity_km_s {
            if let Some(rhs) = rhs.velocity_km_s {
                velocity_km_s.0 -= rhs.0;
                velocity_km_s.1 -= rhs.1;
                velocity_km_s.2 -= rhs.2;
            }
        }

        if let Some(clock_us) = &mut self.clock_us {
            if let Some(rhs) = rhs.clock_us {
                *clock_us -= rhs;
            }
        }

        if let Some(clock_drift_ns) = &mut self.clock_drift_ns {
            if let Some(rhs) = rhs.clock_drift_ns {
                *clock_drift_ns -= rhs;
            }
        }
    }
}

impl SP3Entry {
    /// Builds new [SP3Entry] with "true" position and all other
    /// fields are unknown.
    pub fn from_position_km(position_km: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            velocity_km_s: None,
            clock_drift_ns: None,
            clock_prediction: false,
            orbit_prediction: false,
            clock_event: false,
        }
    }

    /// Builds new [SP3Entry] with position prediction, in kilometers.
    pub fn from_predicted_position_km(position_km: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            velocity_km_s: None,
            clock_drift_ns: None,
            clock_prediction: false,
            orbit_prediction: true,
            clock_event: false,
        }
    }

    /// Builds new [SP3Entry] with "true" position and velocity vector,
    /// any other fields are unknown.
    pub fn from_position_velocity_km_km_s(position_km: Vector3D, velocity_km_s: Vector3D) -> Self {
        Self {
            position_km,
            velocity_km_s: Some(velocity_km_s),
            clock_us: None,
            maneuver: false,
            clock_drift_ns: None,
            clock_prediction: false,
            orbit_prediction: false,
            clock_event: false,
        }
    }

    /// Builds new [SP3Entry] with predicted position and velocity vectors,
    /// all other fields are unknown.
    pub fn from_predicted_position_velocity_km_km_s(
        position_km: Vector3D,
        velocity_km_s: Vector3D,
    ) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            clock_drift_ns: None,
            velocity_km_s: Some(velocity_km_s),
            clock_prediction: false,
            orbit_prediction: true,
            clock_event: false,
        }
    }

    /// Formats this [SP3Entry] according to SP3 standards
    pub fn format<W: Write>(&self, sv: SV, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let mut formatted = format!(
            "P{}{}{}{}",
            sv,
            CoordsFormatter::new(self.position_km.0),
            CoordsFormatter::new(self.position_km.1),
            CoordsFormatter::new(self.position_km.2),
        );

        if let Some(value) = self.clock_us {
            formatted.push_str(&format!("{}", CoordsFormatter::new(value)));
        } else {
            formatted.push_str("                            ");
        }

        if self.clock_event {
            formatted.push('E');
        } else {
            formatted.push(' ');
        }

        if self.clock_prediction {
            formatted.push_str("P  ");
        } else {
            formatted.push_str("   ");
        }

        //10  9 11 102 EP  MP
        if self.maneuver {
            formatted.push('M');
        } else {
            formatted.push(' ');
        }

        if self.orbit_prediction {
            formatted.push('P');
        }

        formatted = formatted.trim_end().to_string();

        if let Some((vel_x_km, vel_y_km, vel_z_km)) = self.velocity_km_s {
            formatted.push_str(&format!(
                "\nV{}{}{}{}",
                sv,
                CoordsFormatter::new(vel_x_km),
                CoordsFormatter::new(vel_y_km),
                CoordsFormatter::new(vel_z_km),
            ));

            if let Some(drift_ns) = self.clock_drift_ns {
                formatted.push_str(&format!("{}", CoordsFormatter::new(drift_ns)));
            }
        }

        writeln!(w, "{}", formatted)?;
        Ok(())
    }

    /// Copies and returns [SP3Entry] with "true" position vector.
    pub fn with_position_km(&self, position_km: Vector3D) -> Self {
        let mut s = *self;
        s.position_km = position_km;
        s.orbit_prediction = false;
        s
    }

    /// Copies and returns [SP3Entry] with predicted position vector.
    pub fn with_predicted_position_km(&self, position_km: Vector3D) -> Self {
        let mut s = *self;
        s.position_km = position_km;
        s.orbit_prediction = true;
        s
    }

    /// Copies and returns [SP3Entry] with "true" velocity vector
    pub fn with_velocity_km_s(&self, velocity_km_s: Vector3D) -> Self {
        let mut s = *self;
        s.velocity_km_s = Some(velocity_km_s);
        s.orbit_prediction = false;
        s
    }

    /// Copies and returns [SP3Entry] with predicted velocity vector
    pub fn with_predicted_velocity_km_s(&self, velocity_km_s: Vector3D) -> Self {
        let mut s = *self;
        s.velocity_km_s = Some(velocity_km_s);
        s.orbit_prediction = true;
        s
    }

    /// Copies and returns [Self] with "true" clock offset in seconds
    pub fn with_clock_offset_s(&self, offset_s: f64) -> Self {
        let mut s = *self;
        s.clock_us = Some(offset_s * 1.0E6);
        s.clock_prediction = false;
        s
    }

    /// Copies and returns [Self] with predicted clock offset in seconds
    pub fn with_predicted_clock_offset_s(&self, offset_s: f64) -> Self {
        let mut s = *self;
        s.clock_us = Some(offset_s * 1.0E6);
        s.clock_prediction = true;
        s
    }

    /// Copies and returns [Self] with "true" clock offset in microseconds
    pub fn with_clock_offset_us(&self, offset_us: f64) -> Self {
        let mut s = *self;
        s.clock_us = Some(offset_us);
        s.clock_prediction = false;
        s
    }

    /// Copies and returns [Self] with predicted clock offset in microseconds
    pub fn with_predicted_clock_offset_us(&self, offset_us: f64) -> Self {
        let mut s = *self;
        s.clock_us = Some(offset_us);
        s.clock_prediction = true;
        s
    }

    /// Copies and returns [Self] with clock drift in seconds
    pub fn with_clock_drift_s(&self, drift_s: f64) -> Self {
        let mut s = *self;
        s.clock_drift_ns = Some(drift_s * 1.0E9);
        s
    }

    /// Copies and returns [Self] with clock drift in nanoseconds
    pub fn with_clock_drift_ns(&self, drift_ns: f64) -> Self {
        let mut s = *self;
        s.clock_drift_ns = Some(drift_ns);
        s
    }
}

#[cfg(test)]
mod test {
    use super::SP3Entry;
    use crate::prelude::SV;
    use crate::tests::formatting::Utf8Buffer;

    use std::io::BufWriter;
    use std::str::FromStr;

    #[test]
    fn basic_data_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (15402.861499, 21607.418873, -992.500669),
                velocity_km_s: None,
                orbit_prediction: false,
                maneuver: false,
                clock_drift_ns: None,
                clock_event: false,
                clock_prediction: false,
                clock_us: Some(10.571484),
            },
            "PG01  15402.861499  21607.418873   -992.500669     10.571484\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn basic_noclock_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (15402.861499, 21607.418873, -992.500669),
                velocity_km_s: None,
                orbit_prediction: false,
                maneuver: false,
                clock_drift_ns: None,
                clock_event: false,
                clock_prediction: false,
                clock_us: None,
            },
            "PG01  15402.861499  21607.418873   -992.500669\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn basic_maneuver_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (-12593.593500, 10170.327650, -20354.534400),
                velocity_km_s: None,
                orbit_prediction: false,
                maneuver: true,
                clock_drift_ns: None,
                clock_event: false,
                clock_prediction: false,
                clock_us: None,
            },
            "PG01 -12593.593500  10170.327650 -20354.534400                                M\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn clock_prediction_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (-11044.805800, -10475.672350, 21929.418200),
                velocity_km_s: None,
                orbit_prediction: false,
                maneuver: false,
                clock_drift_ns: None,
                clock_event: false,
                clock_prediction: true,
                clock_us: None,
            },
            "PG01 -11044.805800 -10475.672350  21929.418200                             P\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn orbit_prediction_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (-11044.805800, -10475.672350, 21929.418200),
                velocity_km_s: None,
                orbit_prediction: true,
                maneuver: false,
                clock_drift_ns: None,
                clock_event: false,
                clock_prediction: false,
                clock_us: None,
            },
            "PG01 -11044.805800 -10475.672350  21929.418200                                 P\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn dual_clock_orbit_prediction_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (-11044.805800, -10475.672350, 21929.418200),
                velocity_km_s: None,
                orbit_prediction: true,
                maneuver: false,
                clock_drift_ns: None,
                clock_event: false,
                clock_prediction: true,
                clock_us: None,
            },
            "PG01 -11044.805800 -10475.672350  21929.418200                             P   P\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn basic_velocity_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (15402.861499, 21607.418873, -992.500669),
                velocity_km_s: Some((-22859.768469, -8524.538983, -15063.229095)),
                orbit_prediction: false,
                maneuver: false,
                clock_drift_ns: None,
                clock_event: false,
                clock_prediction: false,
                clock_us: None,
            },
            "PG01  15402.861499  21607.418873   -992.500669
VG01 -22859.768469  -8524.538983 -15063.229095\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn basic_velocity_drift_formatting() {
        let g01 = SV::from_str("G01").unwrap();

        {
            let (data, expected) = (
            SP3Entry {
                position_km: (15402.861499, 21607.418873, -992.500669),
                velocity_km_s: Some((-22859.768469, -8524.538983, -15063.229095)),
                orbit_prediction: false,
                maneuver: false,
                clock_drift_ns: Some(-3.292980),
                clock_event: false,
                clock_prediction: false,
                clock_us: None,
            },
            "PG01  15402.861499  21607.418873   -992.500669
VG01 -22859.768469  -8524.538983 -15063.229095     -3.292980\n",
        );
            let mut buf = BufWriter::new(Utf8Buffer::new(1024));

            data.format(g01, &mut buf).unwrap_or_else(|e| {
                panic!("SP3/data formatting issue: {}", e);
            });

            let formatted = buf.into_inner().unwrap();
            let formatted = formatted.to_ascii_utf8();

            assert_eq!(formatted, expected);
        }
    }
}
