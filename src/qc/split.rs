use crate::prelude::{Duration, Epoch, SP3};
use qc_traits::QcSplit;

impl QcSplit for SP3 {

    fn split_mut(&mut self, _t: Epoch) -> Self {
        Self::default()
    }

    fn split_even_dt_ref<'a>(&'a self, dt: Duration) -> &'a [&'a Self] {
        &[self]
    }

    fn split_even_dt_vec(&self, dt: Duration) -> Vec<Self> {
        vec![self.clone()]
    }
}