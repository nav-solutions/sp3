use crate::prelude::{Epoch, Header, TimeScale, SP3};
use qc_traits::{GnssAbsoluteTime, Timeshift};

impl Timeshift for Header {
    fn timeshift(&self, target: TimeScale, solver: &GnssAbsoluteTime) -> Self {
        let mut s = self.clone();
        s.time_shift_mut(target, &solver);
        s
    }

    fn timeshift_mut(&mut self, target: TimeScale, solver: &GnssAbsoluteTime) {
        self.timescale = target;

        let t = Epoch::from_time_of_week(self.week_counter, self.week_sow * 1_000_000_000);
        let converted = solver.epoch_time_correction(t);

        let (week, sow) = converted.to_time_of_week();
        self.week_counter = week;
        self.week_sow = sow / 1_000_000_000;

        self.mjd = converted.to_mjd();
    }
}

impl Timeshift for SP3 {
    fn timeshift(&self, target: TimeScale, solver: &GnssAbsoluteTime) -> Self {
        let mut s = self.clone();
        s.timeshift_mut(target, &solver);
        s
    }

    fn timeshift_mut(&mut self, target: TimeScale, solver: &GnssAbsoluteTime) {
        self.header.timeshift_mut(target, solver);

        for (k, _) in self.data.iter_mut() {
            *k.epoch = solver.epoch_time_correction(k.epoch, target);
        }
    }
}
