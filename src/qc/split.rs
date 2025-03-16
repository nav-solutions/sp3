use crate::prelude::{Epoch, Header, SP3};
use qc_traits::QcSplit;

impl QcSplit for Header {
    fn split_mut(&mut self, t: Epoch) -> Self {
        let t0 = Epoch::from_mjd_in_time_scale(self.mjd, self.timescale);
        let mut rhs = self.clone();

        if t0 > t {
            // TODO: leap second error here
            self.mjd = t.to_mjd_utc_days();
        } else {
            // TODO: leap second error here
            rhs.mjd = t.to_mjd_utc_days();
        }

        let (weekn, sow) = t.to_time_of_week();

        if self.week_counter > weekn {
            self.week_counter = weekn;
            self.week_sow = sow as f64;
        } else {
            rhs.week_counter = weekn;
            rhs.week_sow = sow as f64;
        }

        rhs
    }
}

impl QcSplit for SP3 {
    fn split_mut(&mut self, t: Epoch) -> Self {
        let rhs_header = self.header.split_mut(t);
        let mut rhs = self.clone().with_header(rhs_header);
        self.data.retain(|k, _| k.epoch <= t);
        rhs.data.retain(|k, _| k.epoch > t);
        rhs
    }
}
