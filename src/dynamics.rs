use crate::{
    prelude::{DataType, Epoch, SP3Entry, SP3, SV},
    Vector3D,
};

use std::collections::HashMap;

impl SP3 {
    /// Resolve all satellite dynamics for each [Epoch]
    /// where they were not originally defined (both spatial and temporal).
    /// Modifies the file type to [DataType::Velocity] on success.
    pub fn resolve_dynamics_mut(&mut self) {
        let mut success = false;
        let mut past_states = HashMap::<SV, (Epoch, SP3Entry)>::with_capacity(8);

        for (k, v) in self.data.iter_mut() {
            if let Some((past_t, past_state)) = past_states.get(&k.sv) {
                let dt = (k.epoch - *past_t).to_seconds();

                if v.velocity_km_s.is_none() {
                    let (x_km, y_km, z_km) = (
                        (v.position_km.0 - past_state.position_km.0) / dt,
                        (v.position_km.1 - past_state.position_km.1) / dt,
                        (v.position_km.2 - past_state.position_km.2) / dt,
                    );

                    v.velocity_km_s = Some((x_km, y_km, z_km));
                    success = true;
                }

                if v.clock_drift_ns.is_none() {}
            }

            past_states.insert(k.sv, (k.epoch, *v));
        }

        if success {
            self.header.data_type = DataType::Velocity;
        }
    }

    /// See [Self::resolve_dynamics_mut].
    pub fn resolve_dynamics(&self) -> Self {
        let mut s = self.clone();
        s.resolve_dynamics_mut();
        s
    }

    /// Resolve the clock drift for each satellite
    /// and each [Epoch] where it was not originally defined.
    pub fn resolve_clock_drift_mut(&mut self) {
        let mut past_states = HashMap::<SV, (Epoch, f64)>::with_capacity(8);

        for (k, v) in self.data.iter_mut() {
            if let Some(clock_us) = v.clock_us {
                if v.clock_drift_ns.is_none() {
                    if let Some((past_t, past_state)) = past_states.get(&k.sv) {
                        let dt = (k.epoch - *past_t).to_seconds();
                        let ddt = (clock_us - past_state) / dt / 1.0E3;
                        v.clock_drift_ns = Some(ddt);
                    }
                }
                past_states.insert(k.sv, (k.epoch, clock_us));
            }
        }
    }

    /// See [Self::resolve_clock_drift_mut].
    pub fn resolve_clock_drift(&self) -> Self {
        let mut s = self.clone();
        s.resolve_clock_drift_mut();
        s
    }

    /// Resolve the velocity vector for each satellite
    /// for each [Epoch] where it was not originally defined.
    /// Modifies the file type to [DataType::Velocity] on success.
    pub fn resolve_velocities_mut(&mut self) {
        let mut success = false;
        let mut past_states = HashMap::<SV, (Epoch, Vector3D)>::with_capacity(8);

        for (k, v) in self.data.iter_mut() {
            if v.velocity_km_s.is_none() {
                if let Some((past_t, past_state)) = past_states.get(&k.sv) {
                    let dt = (k.epoch - *past_t).to_seconds();

                    let (dx_km, dy_km, dz_km) = (
                        (v.position_km.0 - past_state.0) / dt,
                        (v.position_km.1 - past_state.1) / dt,
                        (v.position_km.2 - past_state.2) / dt,
                    );

                    v.velocity_km_s = Some((dx_km, dy_km, dz_km));
                    success = true;
                }
            }

            past_states.insert(k.sv, (k.epoch, v.position_km));
        }

        if success {
            self.header.data_type = DataType::Velocity;
        }
    }

    /// Refer to [Self::resolve_dynamics_mut].
    pub fn resolve_velocities(&self) -> Self {
        let mut s = self.clone();
        s.resolve_velocities_mut();
        s
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{DataType, Duration, Epoch, SP3, SV};

    use std::str::FromStr;

    #[test]
    fn dynamics_velocity() {
        let mut tests = 0;

        let sp3 =
            SP3::from_gzip_file("data/SP3/C/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz").unwrap();

        assert_eq!(sp3.header.data_type, DataType::Position);

        let dt = Duration::from_hours(0.25).to_seconds();

        let e01 = SV::from_str("E01").unwrap();
        let g03 = SV::from_str("G03").unwrap();

        let t0_gpst = Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap();
        let t1_gpst = Epoch::from_str("2020-06-25T00:15:00 GPST").unwrap();
        let t2_gpst = Epoch::from_str("2020-06-25T00:30:00 GPST").unwrap();
        let tlast_gpst = Epoch::from_str("2020-06-25T23:45:00 GPST").unwrap();

        let velocities = sp3.resolve_velocities();
        let dynamics = sp3.resolve_dynamics();

        for (k, v) in velocities.data.iter() {
            if k.epoch == t0_gpst {
                if k.sv == e01 {
                    assert!(v.velocity_km_s.is_none(), "not feasible on 1st epoch");
                    tests += 1;
                } else if k.sv == g03 {
                    assert!(v.velocity_km_s.is_none(), "not feasible on 1st epoch");
                    tests += 1;
                }
            } else {
                assert!(v.velocity_km_s.is_some(), "should have been resolved");
            }

            if k.epoch == t1_gpst {
                if k.sv == e01 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (-13618.625154 - -11562.163582) / dt,
                            (13865.251337 - 14053.114306) / dt,
                            (22325.739925 - 23345.128269) / dt,
                        ))
                    );

                    tests += 1;
                } else if k.sv == g03 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (-3800.388902 - -1490.224168) / dt,
                            (14678.852973 - 15550.044531) / dt,
                            (-21871.749478 - -21555.137342) / dt,
                        ))
                    );

                    tests += 1;
                }
            } else if k.epoch == t2_gpst {
                if k.sv == e01 {
                } else if k.sv == g03 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (-6168.491275 - -3800.388902) / dt,
                            (13926.922736 - 14678.852973) / dt,
                            (-21814.280798 - -21871.749478) / dt,
                        ))
                    );

                    tests += 1;
                }
            } else if k.epoch == tlast_gpst {
                if k.sv == e01 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (29210.355389 - 28912.007696) / dt,
                            (4476.465587 - 4519.886717) / dt,
                            (-1746.625183 - -4470.989394) / dt,
                        )),
                    );
                    tests += 1;
                } else if k.sv == g03 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (103.973571 - 2213.818352) / dt,
                            (16239.419381 - 17262.742254) / dt,
                            (-21098.565457 - -20155.414780) / dt,
                        ))
                    );
                    tests += 1;
                }
            }
        }

        assert_eq!(tests, 7);

        let mut tests = 0;

        for (k, v) in dynamics.data.iter() {
            if k.epoch == t0_gpst {
                if k.sv == e01 {
                    assert!(v.velocity_km_s.is_none(), "not feasible on 1st epoch");
                    tests += 1;
                } else if k.sv == g03 {
                    assert!(v.velocity_km_s.is_none(), "not feasible on 1st epoch");
                    tests += 1;
                }
            } else {
                assert!(v.velocity_km_s.is_some(), "should have been resolved");
            }

            if k.epoch == t1_gpst {
                if k.sv == e01 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (-13618.625154 - -11562.163582) / dt,
                            (13865.251337 - 14053.114306) / dt,
                            (22325.739925 - 23345.128269) / dt,
                        ))
                    );

                    tests += 1;
                } else if k.sv == g03 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (-3800.388902 - -1490.224168) / dt,
                            (14678.852973 - 15550.044531) / dt,
                            (-21871.749478 - -21555.137342) / dt,
                        ))
                    );

                    tests += 1;
                }
            } else if k.epoch == t2_gpst {
                if k.sv == e01 {
                } else if k.sv == g03 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (-6168.491275 - -3800.388902) / dt,
                            (13926.922736 - 14678.852973) / dt,
                            (-21814.280798 - -21871.749478) / dt,
                        ))
                    );

                    tests += 1;
                }
            } else if k.epoch == tlast_gpst {
                if k.sv == e01 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (29210.355389 - 28912.007696) / dt,
                            (4476.465587 - 4519.886717) / dt,
                            (-1746.625183 - -4470.989394) / dt,
                        )),
                    );
                    tests += 1;
                } else if k.sv == g03 {
                    assert_eq!(
                        v.velocity_km_s,
                        Some((
                            (103.973571 - 2213.818352) / dt,
                            (16239.419381 - 17262.742254) / dt,
                            (-21098.565457 - -20155.414780) / dt,
                        ))
                    );
                    tests += 1;
                }
            }
        }

        assert_eq!(tests, 7);

        // test specific iterators

        let mut tests = 0;

        for (epoch, sv, state_km) in velocities.satellites_velocity_km_s_iter() {
            assert!(epoch != t0_gpst, "1st epoch should not exist");

            if epoch == t1_gpst {
                if sv == e01 {
                    assert_eq!(
                        state_km,
                        (
                            (-13618.625154 - -11562.163582) / dt,
                            (13865.251337 - 14053.114306) / dt,
                            (22325.739925 - 23345.128269) / dt,
                        )
                    );
                    tests += 1;
                } else if sv == g03 {
                    assert_eq!(
                        state_km,
                        (
                            (-3800.388902 - -1490.224168) / dt,
                            (14678.852973 - 15550.044531) / dt,
                            (-21871.749478 - -21555.137342) / dt,
                        )
                    );
                    tests += 1;
                }
            } else if epoch == tlast_gpst {
                if sv == e01 {
                    assert_eq!(
                        state_km,
                        (
                            (29210.355389 - 28912.007696) / dt,
                            (4476.465587 - 4519.886717) / dt,
                            (-1746.625183 - -4470.989394) / dt,
                        ),
                    );
                    tests += 1;
                } else if sv == g03 {
                    assert_eq!(
                        state_km,
                        (
                            (103.973571 - 2213.818352) / dt,
                            (16239.419381 - 17262.742254) / dt,
                            (-21098.565457 - -20155.414780) / dt,
                        )
                    );
                    tests += 1;
                }
            }
        }

        assert_eq!(tests, 4);
    }
}
