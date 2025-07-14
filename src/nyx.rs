/*
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/rtk-rs/sp3/graphs/contributors)
 * This framework is shipped under Mozilla Public V2 license.
 *
 * Documentation: https://github.com/rtk-rs/sp3
 *
 * The Nyx feature is released under AGPLv3
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * Documentation: https://nyxspace.com/
 */
use log::{debug, error};
use thiserror::Error;

use crate::prelude::{Duration, Epoch, SP3Entry, SP3Key, SP3, SV};

#[cfg(doc)]
use crate::prelude::DataType;

use anise::{
    constants::{
        celestial_objects::{MOON, SUN},
        frames::EARTH_J2000,
    },
    prelude::{Almanac, Frame},
};

use std::sync::Arc;

use hifitime::{TimeSeries, Unit};

use nyx_space::{
    cosmic::GuidanceMode,
    dynamics::{
        DynamicsError as NyxDynamicsError, OrbitalDynamics, SolarPressure, SpacecraftDynamics,
    },
    md::trajectory::Traj,
    propagators::Propagator,
    Spacecraft,
};

/// [SP3] prediction specific errors.
#[derive(Debug, Error)]
pub enum PredictionError {
    #[error("dynamics must be resolved for a prediction")]
    UnresolvedDynamics,

    #[error("undetermined initial state")]
    UndeterminedInitialState,

    #[error("dynamics error: {0}")]
    NyxDynamics(#[from] NyxDynamicsError),
}

#[derive(Copy, Clone)]
pub struct SpacecraftModel {
    /// Satellite identity as [SV]
    pub satellite: SV,

    /// True when this spacecraft is undergoing maneuver
    pub maneuver: bool,

    /// [Spacecraft] model
    pub model: Spacecraft,
}

pub struct SpacecraftTrajectory {
    /// Satellite identity as [SV]
    pub satellite: SV,

    /// [Traj]ectory
    pub trajectory: Traj<Spacecraft>,
}

impl SP3 {
    /// Obtain a [SpacecraftModel] at desired [Epoch] for desired [SV].
    ///
    /// NB: Only the satellites for which the dynamics are fully resolved can be
    /// converted into a [SpacecraftModel]. Because we use this structure for
    /// Orbit Determination (OD) processes, which requires the 6 dimension vector to be fully
    /// resolved. When coming from a [DataType::Position] file, you can use
    /// [SP3::resolve_dynamics_mut] or [SP3::resolve_velocities_mut] to manually resolve
    /// the dynamics first.
    ///
    /// ## Inputs
    /// - epoch: [Epoch] of model initialization, that must exist in this file
    /// and have spatial dynamics resolved.
    /// - sv: [SV] that must exist in this file.
    /// - frame: ECEF [Frame] model with gravitational constant.
    pub fn spacecraft_model(&self, epoch: Epoch, sv: SV, frame: Frame) -> Option<SpacecraftModel> {
        self.spacecraft_model_iter(epoch, frame)
            .filter_map(|model| {
                if model.satellite == sv {
                    Some(model)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }

    /// Iterate over each satellite, converted into a [SpacecraftModel] initialized
    /// at desired [Epoch].
    ///
    /// NB: Only the satellites for which the dynamics are fully resolved can be
    /// converted into a [SpacecraftModel]. Because we use this structure for
    /// Orbit Determination (OD) processes, which requires the 6 dimension vector to be fully
    /// resolved. When coming from a [DataType::Position] file, you can use
    /// [SP3::resolve_dynamics_mut] or [SP3::resolve_velocities_mut] to manually resolve
    /// the dynamics first.
    ///
    /// ## Inputs
    /// - epoch: [Epoch] of model initialization, that must exist in this file
    /// and have spatial dynamics resolved.
    /// - frame: ECEF [Frame] model with gravitational constant.
    pub fn spacecraft_model_iter(
        &self,
        epoch: Epoch,
        frame: Frame,
    ) -> Box<dyn Iterator<Item = SpacecraftModel> + '_> {
        Box::new(self.satellites_orbit_iter(frame).filter_map(move |state| {
            if state.orbit.has_velocity_dynamics() && state.epoch == epoch {
                // Spacecraft model:
                //  Dry mass: 1_500 kg
                //  SRP   Cr: 1.3
                //  SRP area: 25 (m^2)
                // Drag   Cd: 2.2
                // Drag area: 25  (m^2)
                let sc_model = Spacecraft::builder()
                    .orbit(state.orbit)
                    .build()
                    .with_dry_mass(1_500.0)
                    .with_srp(25.0, 1.3)
                    .with_drag(25.0, 2.2)
                    .with_guidance_mode(GuidanceMode::Coast)
                    .with_prop_mass(0.0);

                Some(SpacecraftModel {
                    model: sc_model,
                    maneuver: state.maneuver,
                    satellite: state.satellite,
                })
            } else {
                None
            }
        }))
    }

    /// Obtain a predicted [Traj]ectory for each satellite.
    ///
    /// ## Input
    /// - almanac: [Almanac] definition
    /// - frame: ECEF [Frame] model with gravitational constant.
    /// - initial_epoch: possible initial [Epoch].
    /// When undefine
    /// When undefined, we simply use the latest state in time.
    /// - duration: total [Duration] of the prediction.
    /// NB: you can use a negative [Duration] here to predict in the past
    /// (backwards) from `initial_epoch`.
    ///
    /// ## Returns
    /// - ([SV], [Trajectory]) for each satellite
    /// - [PredictionError]
    pub fn trajectory_predictions_iter(
        &self,
        almanac: Arc<Almanac>,
        frame: Frame,
        initial_epoch: Option<Epoch>,
        duration: Duration,
    ) -> Result<Box<dyn Iterator<Item = SpacecraftTrajectory> + '_>, PredictionError> {
        let orbital_model = OrbitalDynamics::point_masses(vec![MOON, SUN]);
        let srp_model = SolarPressure::new(vec![EARTH_J2000], almanac.clone())?;

        let dynamics = SpacecraftDynamics::from_model(orbital_model, srp_model);

        let initial_epoch = match initial_epoch {
            Some(initial_epoch) => initial_epoch,
            None => self
                .last_epoch()
                .ok_or(PredictionError::UndeterminedInitialState)?,
        };

        let last_epoch = initial_epoch + duration;

        // create a propagator for each satellite
        // using the same solar pressure and dynamics model
        let iter = self
            .spacecraft_model_iter(initial_epoch, frame)
            .filter_map(move |spacecraft| {
                debug!("spacecraft: {} {}", spacecraft.satellite, spacecraft.model);

                let dynamics = dynamics.clone();

                match Propagator::default(dynamics)
                    .with(spacecraft.model, almanac.clone())
                    .until_epoch_with_traj(last_epoch)
                {
                    Ok((_final_state, trajectory)) => Some(SpacecraftTrajectory {
                        trajectory,
                        satellite: spacecraft.satellite,
                    }),
                    Err(e) => {
                        // prediction error (should not happen)
                        // simply catch it, and the "future" states will be missing
                        // for those satellites
                        error!(
                            "{}({}) - prediction error: {}",
                            spacecraft.satellite, initial_epoch, e
                        );
                        None
                    },
                }
            });

        Ok(Box::new(iter))
    }

    /// Predict spatial coordinates for each satellite, for desired duration, returning a
    /// new [SP3] expanded in time. Refer to [SP3::spatial_prediction_mut] for more information.
    pub fn spatial_prediction(
        &self,
        almanac: Arc<Almanac>,
        frame: Frame,
        initial_epoch: Option<Epoch>,
        duration: Duration,
    ) -> Result<Self, PredictionError> {
        let mut s = self.clone();
        s.spatial_prediction_mut(almanac, frame, initial_epoch, duration)?;
        Ok(s)
    }

    /// Predict spatial coordinates for each satellite, for desired duration, with mutable access,
    /// expanding this [SP3] in the future. The new states are marked with the prediction flag.
    ///
    /// ## Input
    /// - almanac: [Almanac]
    /// - frame: ECEF [Frame] model with gravitational constant.
    /// - initial_epoch: Possible custom [Epoch] offset
    /// used to determine the initial state. When set to None,
    /// we will use the latest state described by this [SP3].
    /// - duration: [Duration] of the prediction.
    /// NB: you can use a negative [Duration] here to predict in the past
    /// (backwards) from `initial_epoch`.
    pub fn spatial_prediction_mut(
        &mut self,
        almanac: Arc<Almanac>,
        frame: Frame,
        initial_epoch: Option<Epoch>,
        duration: Duration,
    ) -> Result<(), PredictionError> {
        let sampling_period = self.header.sampling_period;

        // Determine initial state
        let initial_epoch = match initial_epoch {
            Some(initial_epoch) => initial_epoch,
            None => self
                .last_epoch()
                .ok_or(PredictionError::UndeterminedInitialState)?,
        };

        let last_epoch = initial_epoch + duration;

        let new_epochs = ((last_epoch - initial_epoch).to_unit(Unit::Second)
            / sampling_period.to_unit(Unit::Second))
        .round() as u64;

        // obtain a predicted trajectory for each satellite
        let satellite_trajectories = self
            .trajectory_predictions_iter(almanac, frame, Some(initial_epoch), duration)?
            .collect::<Vec<_>>();

        // iterate each trajectories and expand self
        for sat_trajectory in satellite_trajectories.iter() {
            let timeserie = if duration.is_negative() {
                TimeSeries::inclusive(last_epoch, initial_epoch, sampling_period)
            } else {
                TimeSeries::inclusive(initial_epoch, last_epoch, sampling_period)
            };

            for epoch in timeserie.into_iter() {
                match sat_trajectory.trajectory.at(epoch) {
                    Ok(state) => {
                        let pos_vel_km = state.orbit.to_cartesian_pos_vel();

                        let key = SP3Key {
                            epoch,
                            sv: sat_trajectory.satellite,
                        };

                        let value = SP3Entry::from_predicted_position_km((
                            pos_vel_km[0],
                            pos_vel_km[1],
                            pos_vel_km[2],
                        ))
                        .with_velocity_km_s((
                            pos_vel_km[3],
                            pos_vel_km[4],
                            pos_vel_km[5],
                        ));

                        // push new content
                        self.data.insert(key, value);
                    },
                    Err(e) => {
                        error!(
                            "{}({}) - prediction failed with {}",
                            epoch, sat_trajectory.satellite, e
                        );
                    },
                }
            }
        }

        // update self
        self.header.num_epochs += new_epochs; //TODO: incorrect in case of failure(s)

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        prelude::{Duration, Epoch, Split, SP3},
        tests::init_logger,
    };

    use anise::{constants::frames::EARTH_J2000, prelude::Almanac};

    use log::info;
    use std::str::FromStr;
    use std::sync::Arc;

    #[test]
    fn forward_spatial_propagation() {
        init_logger();
        let almanac = Arc::new(Almanac::until_2035().unwrap());

        let earth_cef = almanac.frame_from_uid(EARTH_J2000).unwrap();

        let mut parsed =
            SP3::from_gzip_file("data/SP3/C/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz").unwrap();

        // resolve dynamics
        parsed.resolve_dynamics_mut();

        let midnight = Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap();
        let noon = midnight + Duration::from_hours(12.0);
        let j_1 = noon + Duration::from_hours(12.0);

        let (morning, _) = parsed.split(noon);

        let predicted = morning
            .spatial_prediction(almanac, earth_cef, Some(noon), Duration::from_hours(12.0))
            .unwrap_or_else(|e| {
                panic!("SP3 (spatial) prediction failed with: {}", e);
            });

        assert_eq!(
            predicted.first_epoch(),
            Some(midnight),
            "first epoch should have been preserved",
        );

        assert_eq!(
            predicted.last_epoch(),
            Some(j_1),
            "forward prediction did not extend correctly",
        );

        // obtain residuals
        let residuals = predicted.substract(&parsed);

        // Dump residuals as fake SP3
        residuals.to_file("od-residuals.sp3").unwrap_or_else(|e| {
            panic!("failed to dump OD residuals: {}", e);
        });

        // Run testbench
        for (k, v) in residuals.data.iter() {
            let (x_err_m, y_err_m, z_err_m) = (
                v.position_km.0.abs() * 1.0E3,
                v.position_km.1.abs() * 1.0E3,
                v.position_km.2.abs() * 1.0E3,
            );

            info!(
                "{}({}) - x_err={:.3}m y_err={:.3}m z_err={:.3}m",
                k.epoch, k.sv, x_err_m, y_err_m, z_err_m
            );

            // assert!(x_err_m < 5.0, "{}({}) - x_err={} too large", k.epoch, k.sv, x_err_m);
            // assert!(y_err_m < 5.0, "{}({}) - y_err={} too large", k.epoch, k.sv, y_err_m);
            // assert!(z_err_m < 5.0, "{}({}) - z_err={} too large", k.epoch, k.sv, z_err_m);
        }
    }

    #[test]
    fn backwards_spatial_propagation() {
        init_logger();
        let almanac = Arc::new(Almanac::until_2035().unwrap());

        let earth_cef = almanac.frame_from_uid(EARTH_J2000).unwrap();

        let mut parsed =
            SP3::from_gzip_file("data/SP3/C/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz").unwrap();

        // resolve dynamics
        parsed.resolve_dynamics_mut();

        let midnight = Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap();

        let predicted = parsed
            .spatial_prediction(
                almanac,
                earth_cef,
                Some(midnight),
                -Duration::from_hours(24.0),
            )
            .unwrap_or_else(|e| {
                panic!("SP3 (spatial) prediction failed with: {}", e);
            });

        // parse actual data for that day
        let model =
            SP3::from_gzip_file("data/SP3/C/GRG0MGXFIN_20201760000_01D_15M_ORB.SP3.gz").unwrap();

        assert_eq!(
            predicted.first_epoch(),
            model.first_epoch(),
            "backwards prediction did not extend correctly"
        );

        // compute residuals
        let residuals = predicted.substract(&model);

        // Dump residuals as fake SP3
        residuals
            .to_file("backwards-od-residuals.sp3")
            .unwrap_or_else(|e| {
                panic!("failed to dump OD residuals: {}", e);
            });
    }

    #[test]
    fn spatial_propagation_without_dynamics() {
        init_logger();
        let almanac = Arc::new(Almanac::until_2035().unwrap());

        let earth_cef = almanac.frame_from_uid(EARTH_J2000).unwrap();

        let parsed =
            SP3::from_gzip_file("data/SP3/C/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz").unwrap();

        let midnight = Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap();
        let noon_offset = midnight + Duration::from_hours(12.0);

        let predicted = parsed
            .spatial_prediction(
                almanac,
                earth_cef,
                Some(noon_offset),
                Duration::from_hours(12.0),
            )
            .unwrap_or_else(|e| {
                panic!("SP3 (spatial) prediction failed with: {}", e);
            });

        // obtain residuals
        let residuals = predicted.substract(&parsed);

        // Dump residuals as fake SP3
        residuals
            .to_file("od-nodyn-residuals.sp3")
            .unwrap_or_else(|e| {
                panic!("failed to dump OD residuals: {}", e);
            });
    }
}
