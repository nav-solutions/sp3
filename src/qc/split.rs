use std::collections::BTreeMap;

use crate::prelude::{Duration, Epoch, Header, QcSplit, SP3Entry, SP3Key, SP3};

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

    fn split_even_dt(&self, _: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        vec![self.clone()]
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

    fn split_even_dt(&self, dt: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        let mut t = self.first_epoch();

        let mut data_sets = Vec::<BTreeMap<SP3Key, SP3Entry>>::new();
        let mut map = BTreeMap::<SP3Key, SP3Entry>::new();

        for (k, v) in self.data.iter() {
            if k.epoch > t + dt {
                data_sets.push(map.clone());
                t = k.epoch;
                map.clear();
            }

            map.insert(k.clone(), v.clone());
        }

        data_sets
            .iter()
            .map(|set| SP3 {
                header: self.header.clone(),
                comments: self.comments.clone(),
                data: set.clone(),
            })
            .collect()
    }
}
