use crate::SP3;

use qc_traits::QcRepair;

impl QcRepair for SP3 {
    fn zero_repair_mut(&mut self) {
        for (k, v) in self.data.iter_mut() {
            if let Some(0.0) = v.clock_us {
                v.clock_us = None;
            }
            if let Some(0.0) = v.clock_drift_ns {
                v.clock_drift_ns = None;
            }
        }
    }
}
