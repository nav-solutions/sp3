use crate::prelude::{Epoch, SP3Entry, SP3Key, SP3};
use qc_traits::{GnssAbsoluteTime, Timeshift};

use std::collections::BTreeMap;

impl Timeshift for SP3 {
    fn timeshift(&self, solver: &GnssAbsoluteTime, target: hifitime::TimeScale) -> Self
    where
        Self: Sized,
    {
        let mut s = self.clone();
        s.timeshift_mut(solver, target);
        s
    }

    fn timeshift_mut(&mut self, solver: &GnssAbsoluteTime, target: hifitime::TimeScale) {
        let epoch = Epoch::from_time_of_week(
            self.header.week,
            self.header.week_nanos,
            self.header.timescale,
        );

        // convert
        if let Some(converted) = solver.precise_epoch_correction(epoch, target) {
            let mjd = converted.to_mjd_utc_days();
            let (week, tow) = converted.to_time_of_week();

            self.header.mjd = mjd.floor() as u32;
            self.header.mjd_fraction = mjd.fract();

            self.header.week = week;
            self.header.week_nanos = tow;

            // timeshift
            let mut rec = BTreeMap::<SP3Key, SP3Entry>::new();

            for (k, value) in self.data.iter() {
                let mut key = k.clone();

                key.epoch = solver
                    .precise_epoch_correction(k.epoch, target)
                    .expect("internal error: SP3 with inconsisten time system");

                rec.insert(key, value.clone());
            }

            self.data = rec;

            // confirm & exit
            self.header.timescale = target;
        }
    }
}
