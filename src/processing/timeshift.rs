use crate::prelude::{Epoch, SP3Entry, SP3Key, TimeScale, SP3};
use qc_traits::{TimeCorrectionError, TimeCorrectionsDB, Timeshift};

use std::collections::BTreeMap;

impl Timeshift for SP3 {
    fn timeshift(&self, timescale: TimeScale) -> Self
    where
        Self: Sized,
    {
        let mut s = self.clone();
        s.timeshift_mut(timescale);
        s
    }

    fn timeshift_mut(&mut self, timescale: TimeScale) {
        // transpose week counter
        let epoch = Epoch::from_time_of_week(
            self.header.week,
            self.header.week_nanos,
            self.header.timescale,
        )
        .to_time_scale(timescale);

        (self.header.week, self.header.week_nanos) = epoch.to_time_of_week();

        // transpose MJD
        let mut days = self.header.mjd as f64;
        days += self.header.mjd_fraction;

        let mjd_epoch = Epoch::from_mjd_in_time_scale(days, self.header.timescale);

        let days = match self.header.timescale {
            TimeScale::TAI | TimeScale::UTC => mjd_epoch.to_mjd_utc_days(),
            ts => (mjd_epoch
                - ts.reference_epoch()
                    .to_duration_in_time_scale(TimeScale::TAI))
            .to_mjd_utc_days(),
        };

        self.header.mjd = days.floor() as u32;
        self.header.mjd_fraction = days.fract();

        // finally: overwrite
        self.header.timescale = timescale;

        let mut new = BTreeMap::<SP3Key, SP3Entry>::new();

        for (k, v) in self.data.iter() {
            let key = SP3Key {
                sv: k.sv,
                epoch: k.epoch.to_time_scale(timescale),
            };

            new.insert(key, v.clone());
        }

        self.data = new.clone();
    }

    fn precise_correction(
        &self,
        db: &TimeCorrectionsDB,
        timescale: TimeScale,
    ) -> Result<Self, TimeCorrectionError>
    where
        Self: Sized,
    {
        let mut s = self.clone();
        s.precise_correction_mut(db, timescale)?;
        Ok(s)
    }

    fn precise_correction_mut(
        &mut self,
        db: &TimeCorrectionsDB,
        timescale: TimeScale,
    ) -> Result<(), TimeCorrectionError> {
        let (lhs, rhs) = (self.header.timescale, timescale);

        // transpose week counter
        let epoch = Epoch::from_time_of_week(self.header.week, self.header.week_nanos, lhs);

        let transposed = db
            .precise_epoch_correction(epoch, rhs)
            .ok_or(TimeCorrectionError::NoCorrectionAvailable(lhs, rhs))?;

        (self.header.week, self.header.week_nanos) = transposed.to_time_of_week();

        // transpose MJD
        let mut days = self.header.mjd as f64;
        days += self.header.mjd_fraction;

        let mjd_epoch = Epoch::from_mjd_in_time_scale(days, lhs);

        let days = match self.header.timescale {
            TimeScale::TAI | TimeScale::UTC => mjd_epoch.to_mjd_utc_days(),
            ts => (mjd_epoch
                - ts.reference_epoch()
                    .to_duration_in_time_scale(TimeScale::TAI))
            .to_mjd_utc_days(),
        };

        self.header.mjd = days.floor() as u32;
        self.header.mjd_fraction = days.fract();

        // finally: overwrite
        self.header.timescale = rhs;

        let mut new = BTreeMap::<SP3Key, SP3Entry>::new();

        for (k, v) in self.data.iter() {
            let epoch = db
                .precise_epoch_correction(k.epoch, rhs)
                .ok_or(TimeCorrectionError::NoCorrectionAvailable(lhs, rhs))?;

            let key = SP3Key { epoch, sv: k.sv };

            new.insert(key, v.clone());
        }

        self.data = new.clone();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Duration, Epoch, TimeScale, SP3};
    use hifitime::Polynomial;
    use qc_traits::{TimeCorrection, TimeCorrectionsDB, Timeshift};
    use std::str::FromStr;

    #[test]
    fn gpst2utc_transposition() {
        let sp3 = SP3::from_file("data/SP3/D/example.txt").unwrap();

        let transposed = sp3.timeshift(TimeScale::UTC);

        assert!(sp3 != transposed);

        transposed.to_file("test-utc.sp3").unwrap();

        let parsed_back = SP3::from_file("test-utc.sp3").unwrap();

        for (k, _) in parsed_back.data.iter() {
            assert_eq!(k.epoch.time_scale, TimeScale::UTC);
        }
    }

    #[test]
    fn gpst2gst_transposition() {
        let sp3 = SP3::from_file("data/SP3/D/example.txt").unwrap();

        let transposed = sp3.timeshift(TimeScale::GST);

        assert!(sp3 != transposed);

        transposed.to_file("test-gst.sp3").unwrap();

        let parsed_back = SP3::from_file("test-gst.sp3").unwrap();

        for (k, _) in parsed_back.data.iter() {
            assert_eq!(k.epoch.time_scale, TimeScale::GST);
        }
    }

    #[test]
    fn gpst2utc_precise_correction_transposition() {
        let mut db = TimeCorrectionsDB::default();

        let t0 = Epoch::from_str("2019-10-27T00:00:00 GPST").unwrap();
        let validity = Duration::from_hours(24.0);

        let correction = TimeCorrection::from_reference_epoch(
            t0,
            validity,
            TimeScale::UTC,
            Polynomial::from_constant_offset(Duration::from_nanoseconds(1.0)),
        );

        db.add(correction);

        let sp3 = SP3::from_file("data/SP3/D/example.txt").unwrap();

        let transposed = sp3
            .precise_correction(&db, TimeScale::UTC)
            .unwrap_or_else(|e| {
                panic!("Failed to transpose to UTC: {}", e);
            });

        assert!(sp3 != transposed);

        transposed.to_file("test-precise-utc.sp3").unwrap();

        let parsed_back = SP3::from_file("test-precise-utc.sp3").unwrap();

        for (k, _) in parsed_back.data.iter() {
            assert_eq!(k.epoch.time_scale, TimeScale::UTC);
        }

        // let _ = std::fs::remove_file("test-precise-utc.sp3");
    }

    #[test]
    fn gpst2gst_precise_correction_transposition() {
        let mut db = TimeCorrectionsDB::default();

        let t0 = Epoch::from_str("2019-10-27T00:00:00 GPST").unwrap();
        let validity = Duration::from_hours(1000000.0);

        let correction = TimeCorrection::from_reference_epoch(
            t0,
            validity,
            TimeScale::GST,
            Polynomial::from_constant_offset(Duration::from_nanoseconds(1.0)),
        );

        db.add(correction);

        let sp3 = SP3::from_file("data/SP3/D/example.txt").unwrap();

        let transposed = sp3
            .precise_correction(&db, TimeScale::GST)
            .unwrap_or_else(|e| {
                panic!("Failed to transpose to GST: {}", e);
            });

        assert!(sp3 != transposed);

        transposed.to_file("test-precise-gst.sp3").unwrap();

        let parsed_back = SP3::from_file("test-precise-gst.sp3").unwrap();

        for (k, _) in parsed_back.data.iter() {
            assert_eq!(k.epoch.time_scale, TimeScale::GST);
        }

        // let _ = std::fs::remove_file("test-precise-gst.sp3");
    }
}
