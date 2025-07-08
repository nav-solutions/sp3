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
use log::error;
use thiserror::Error;

use crate::prelude::{Duration, Epoch, SP3Entry, SP3Key, SP3, SV};

use anise::{
    constants::{
        celestial_objects::{MOON, SUN},
        frames::{EARTH_J2000, MOON_J2000},
    },
    prelude::Almanac,
};

use std::sync::Arc;

use hifitime::{TimeSeries, Unit};

use nyx_space::{
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
    #[error("undetermined initial state")]
    UndeterminedInitialState,

    #[error("dynamics error: {0}")]
    NyxDynamics(#[from] NyxDynamicsError),
}

impl SP3 {
    /// Obtain a [Spacecraft] model, at desired [Epoch] for desired [SV]
    pub fn spacecraft_model(&self, epoch: Epoch, sv: SV) -> Option<Spacecraft> {
        self.spacecraft_model_iter(epoch)
            .filter_map(|(sv_i, sc_i)| if sv_i == sv { Some(sc_i) } else { None })
            .reduce(|k, _| k)
    }

    /// Obtain a [Spacecraft] model at desired [Epoch], for each satellite
    pub fn spacecraft_model_iter(
        &self,
        epoch: Epoch,
    ) -> Box<dyn Iterator<Item = (SV, Spacecraft)> + '_> {
        Box::new(self.satellites_orbit_iter(EARTH_J2000).filter_map(
            move |(orbit_t, orbit_sv, orbit)| {
                if orbit_t == epoch {
                    let sc_model = Spacecraft::builder().orbit(orbit).build();
                    Some((orbit_sv, sc_model))
                } else {
                    None
                }
            },
        ))
    }

    /// Obtain a predicted [Traj]ectory for each satellite.
    ///
    /// ## Input
    /// - almanac: [Almanac] definition
    /// - initial_epoch: possible initial [Epoch].
    /// When undefined, we simply use the latest state in time.
    /// - prediction_duration: [Duration] of the predicted [Trajectory]
    ///
    /// ## Returns
    /// - ([SV], [Trajectory]) for each satellite
    /// - [PredictionError]
    pub fn trajectory_predictions_iter(
        &self,
        almanac: Arc<Almanac>,
        initial_epoch: Option<Epoch>,
        prediction_duration: Duration,
    ) -> Result<Box<dyn Iterator<Item = (SV, Traj<Spacecraft>)> + '_>, PredictionError> {
        let orbital_model = OrbitalDynamics::point_masses(vec![MOON, SUN]);

        let srp_model = SolarPressure::new(vec![EARTH_J2000], almanac.clone())?;

        let dynamics = SpacecraftDynamics::from_model(orbital_model, srp_model);

        let initial_epoch = match initial_epoch {
            Some(initial_epoch) => initial_epoch,
            None => self
                .last_epoch()
                .ok_or(PredictionError::UndeterminedInitialState)?,
        };

        let last_epoch = initial_epoch + prediction_duration;

        // create a propagator for each satellite
        // using the same solar pressure and dynamics model
        let iter = self
            .spacecraft_model_iter(initial_epoch)
            .filter_map(move |(sv, sc_model)| {
                let dynamics = dynamics.clone();

                match Propagator::default(dynamics)
                    .with(sc_model, almanac.clone())
                    .until_epoch_with_traj(last_epoch)
                {
                    Ok((_final_state, traj)) => Some((sv, traj)),
                    Err(e) => {
                        // prediction error (should not happen)
                        // simply catch it, and the "future" states will be missing
                        // for those satellites
                        error!("{}({}) - prediction error: {}", sv, initial_epoch, e);
                        None
                    },
                }
            });

        Ok(Box::new(iter))
    }

    /// Predict spatial coordinates for each satellite, for desired duration, returning a
    /// new [SP3] expanded in time. Refer to [Self::spatial_prediction_mut] for more information.
    pub fn spatial_prediction(
        &self,
        almanac: Arc<Almanac>,
        initial_epoch: Option<Epoch>,
        prediction_duration: Duration,
    ) -> Result<Self, PredictionError> {
        let mut s = self.clone();
        s.spatial_prediction_mut(almanac, initial_epoch, prediction_duration)?;
        Ok(s)
    }

    /// Predict spatial coordinates for each satellite, for desired duration, with mutable access,
    /// expanding this [SP3] in the future.
    ///
    /// ## Input
    /// - almanac: [Almanac]
    /// - initial_epoch: Possible custom [Epoch] offset
    /// used to determine the initial state. When set to None,
    /// we will use the latest state described by this [SP3].
    /// - prediction_duration: [Duration] of the prediction
    pub fn spatial_prediction_mut(
        &mut self,
        almanac: Arc<Almanac>,
        initial_epoch: Option<Epoch>,
        prediction_duration: Duration,
    ) -> Result<(), PredictionError> {
        // Determine initial state
        let initial_epoch = match initial_epoch {
            Some(initial_epoch) => initial_epoch,
            None => self
                .last_epoch()
                .ok_or(PredictionError::UndeterminedInitialState)?,
        };

        let last_epoch = initial_epoch + prediction_duration;

        let new_epochs = ((last_epoch - initial_epoch).to_unit(Unit::Second)
            / self.header.sampling_period.to_unit(Unit::Second))
        .round() as u64;

        // obtain a predicted trajectory for each satellite
        let trajectories = self
            .trajectory_predictions_iter(almanac, Some(initial_epoch), prediction_duration)?
            .collect::<Vec<_>>();

        let timeserie =
            TimeSeries::inclusive(initial_epoch, last_epoch, self.header.sampling_period);

        // iterate each trajectories and expand self
        for (sv, traj) in trajectories.iter() {
            for epoch in timeserie.clone().into_iter() {
                match traj.at(epoch) {
                    Ok(state) => {
                        let pos_vel_km = state.orbit.to_cartesian_pos_vel();

                        let key = SP3Key { sv: *sv, epoch };

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
                        error!("{}({}) - prediction failed with {}", epoch, sv, e);
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
    use crate::prelude::{Duration, Epoch, SP3};
    use anise::prelude::Almanac;
    use std::str::FromStr;
    use std::sync::Arc;

    #[test]
    fn spatial_propagation() {
        let almanac = Arc::new(Almanac::until_2035().unwrap());

        // entire setup
        let parsed =
            SP3::from_gzip_file("data/SP3/A/NGA0OPSRAP_20251850000_01D_15M_ORB.SP3.gz").unwrap();

        let midnight = Epoch::from_str("2020-06-25T00:00:00 GPST").unwrap();
        let noon_offset = midnight + Duration::from_hours(12.0);

        let predicted = parsed
            .spatial_prediction(almanac, Some(noon_offset), Duration::from_hours(12.0))
            .unwrap_or_else(|e| {
                panic!("SP3 (spatial) prediction failed with: {}", e);
            });

        // obtain residuals
        let residuals = predicted.substract(&parsed);

        // Dump residuals as fake SP3
        residuals.to_file("od-residuals.txt").unwrap_or_else(|e| {
            panic!("failed to dump OD residuals.txt: {}", e);
        });

        // run testbench
    }
}
