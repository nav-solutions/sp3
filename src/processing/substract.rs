use crate::prelude::SP3;

#[cfg(feature = "processing")]
#[cfg_attr(docsrs, doc(cfg(feature = "processing")))]
impl SP3 {
    /// Substract rhs [SP3] to this [SP3], returning a new "residual" [SP3].
    /// All common components are modified and replaced by residual value.
    /// That applies to both coordinates and temporal values.
    /// This may be used in SP3 residual analysis, where one compares [SP3] from
    /// one laboratory to another. Use with care, because the resulting values
    /// are obviously not what a standard [SP3] should physically represent.
    /// ```
    /// use sp3::prelude::SP3;
    ///
    /// // this dataset comes with position vectors and clock states
    /// let sp3 = SP3::from_gzip_file("data/SP3/C/EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz")
    ///     .unwrap();
    ///
    /// let null_sp3 = sp3.substract(&sp3);
    ///
    /// // Verify that self-self is null residual
    /// for (_, v) in null_sp3.data.iter() {
    ///     // residual position is null
    ///     assert_eq!(v.position_km, (0.0, 0.0, 0.0));
    ///
    ///     // residual clock exists
    ///     let clock_us = v.clock_us.unwrap();
    ///     assert_eq!(clock_us, 0.0);
    /// }
    /// ```
    pub fn substract(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.substract_mut(rhs);
        s
    }

    /// Substract rhs [SP3] to Self, with mutable access. Refer to [Self::substract].
    pub fn substract_mut(&mut self, rhs: &Self) {
        for (k, v) in self.data.iter_mut() {
            if let Some(v_rhs) = rhs.data.get(&k) {
                *v -= *v_rhs;
            }
        }
    }
}
