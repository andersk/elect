use num_traits::{One, Zero};
use std::error::Error;
use std::fmt;
use std::ops::{Add, Sub, Mul, Div};

pub trait WeightValOps<RHS = Self, Output = Self>
    where Self: Sized,
          Self: Add<RHS, Output = Output>,
          Self: Sub<RHS, Output = Output>,
          Self: Mul<RHS, Output = Output>,
          Self: Div<RHS, Output = Output>
{
}

impl<RHS, Output, T> WeightValOps<RHS, Output> for T
    where T: Sized,
          T: Add<RHS, Output = Output>,
          T: Sub<RHS, Output = Output>,
          T: Mul<RHS, Output = Output>,
          T: Div<RHS, Output = Output>
{
}

pub trait WeightOps<Base>
    where Self: WeightValOps<Base, Base> + for<'a> WeightValOps<&'a Base, Base>
{
}

impl<Base, T> WeightOps<Base> for T
    where T: WeightValOps<Base, Base> + for<'a> WeightValOps<&'a Base, Base>
{
}

pub trait Weight: Clone + Ord + WeightOps<Self> + Zero + One + fmt::Debug {
    type FromStrErr: Error;

    fn from_i64(i64) -> Self;
    fn from_str(s: &str) -> Result<Self, Self::FromStrErr>;
    fn to_string(&self) -> String;

    #[inline]
    fn fuzzy_eq(&self, other: &Self) -> bool {
        self == other
    }
}
