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
            self.header.week_counter,
            (self.header.week_sow * 1.0E9).round() as u64,
            self.header.timescale,
        );

        // convert
        if let Some(converted) = solver.epoch_time_correction(epoch, target) {
            let mjd = converted.to_mjd_utc_days();
            let (week, tow) = converted.to_time_of_week();

            self.header.mjd = mjd;
            self.header.week_counter = week;
            self.header.week_sow = (tow / 1_000_000_000) as f64;

            // timeshift
            let mut rec = BTreeMap::<SP3Key, SP3Entry>::new();

            for (k, value) in self.data.iter() {
                let mut key = k.clone();

                key.epoch = solver
                    .epoch_time_correction(k.epoch, target)
                    .expect("internal error: SP3 with inconsisten time system");

                rec.insert(key, value.clone());
            }

            self.data = rec;

            // confirm & exit
            self.header.timescale = target;
        }
    }
}
