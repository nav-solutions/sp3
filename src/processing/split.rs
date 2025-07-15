use crate::prelude::{Duration, Epoch, Header, SP3};
use qc_traits::Split;

impl Split for Header {
    fn split(&self, epoch: Epoch) -> (Self, Self)
    where
        Self: Sized,
    {
        let mut lhs = self.clone();
        let rhs = lhs.split_mut(epoch);
        (lhs, rhs)
    }

    fn split_mut(&mut self, _: Epoch) -> Self {
        self.clone()
    }

    fn split_even_dt(&self, _: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        Default::default()
    }
}

impl Split for SP3 {
    fn split(&self, epoch: Epoch) -> (Self, Self)
    where
        Self: Sized,
    {
        let mut lhs = self.clone();
        let rhs = lhs.split_mut(epoch);
        (lhs, rhs)
    }

    fn split_mut(&mut self, epoch: Epoch) -> Self {
        let mut rhs = self.clone();
        let rhs_header = self.header.split_mut(epoch);

        rhs.data.retain(|k, _| k.epoch > epoch);
        rhs.header = rhs_header; // overwrite header

        self.data.retain(|k, _| k.epoch <= epoch);

        rhs
    }

    fn split_even_dt(&self, _dt: Duration) -> Vec<Self>
    where
        Self: Sized,
    {
        Default::default()
    }
}
