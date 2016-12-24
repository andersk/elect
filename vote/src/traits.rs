use gmp::mpq::Mpq;
use gmp::mpz::Mpz;
use num_rational::BigRational;
use num_traits::{Zero, One};
use std::error::Error;
use std::fmt;
use std::ops::{Add, Sub, Mul, Div};
use std::str::FromStr;

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

pub trait Weight: Clone + Ord + WeightOps<Self> + fmt::Debug {
    // TODO: Replace these with num-traits when rust-gmp supports it.

    type FromStrErr: Error;

    fn zero() -> Self;
    fn is_zero(&self) -> bool;
    fn one() -> Self;
    fn from_i64(i64) -> Self;
    fn from_str(s: &str) -> Result<Self, Self::FromStrErr>;
    fn to_string(&self) -> String;

    #[inline]
    fn fuzzy_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug)]
pub struct ParseMpqError(());

impl fmt::Display for ParseMpqError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for ParseMpqError {
    fn description(&self) -> &'static str {
        "invalid rational number"
    }

    fn cause(&self) -> Option<&'static Error> {
        None
    }
}

impl Weight for Mpq {
    type FromStrErr = ParseMpqError;

    #[inline]
    fn zero() -> Mpq {
        Mpq::zero()
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.is_zero()
    }

    #[inline]
    fn one() -> Mpq {
        Mpq::one()
    }

    #[inline]
    fn from_i64(n: i64) -> Mpq {
        Mpq::from(n)
    }

    #[inline]
    fn from_str(s: &str) -> Result<Mpq, ParseMpqError> {
        match s.find('/') {
            Some(i) => {
                let n = Mpz::from_str_radix(&s[..i], 10).map_err(|_| ParseMpqError(()))?;
                let d = Mpz::from_str_radix(&s[i + 1..], 10).map_err(|_| ParseMpqError(()))?;
                Ok(Mpq::ratio(&n, &d))
            }
            None => {
                let n = Mpz::from_str_radix(s, 10).map_err(|_| ParseMpqError(()))?;
                Ok(Mpq::ratio(&n, &Mpz::one()))
            }
        }
    }

    #[inline]
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl Weight for BigRational {
    type FromStrErr = <BigRational as FromStr>::Err;

    #[inline]
    fn zero() -> BigRational {
        Zero::zero()
    }

    #[inline]
    fn is_zero(&self) -> bool {
        Zero::is_zero(self)
    }

    #[inline]
    fn one() -> BigRational {
        One::one()
    }

    #[inline]
    fn from_i64(n: i64) -> BigRational {
        BigRational::from_integer(n.into())
    }

    #[inline]
    fn from_str(s: &str) -> Result<BigRational, <BigRational as FromStr>::Err> {
        FromStr::from_str(s)
    }

    #[inline]
    fn to_string(&self) -> String {
        ToString::to_string(self)
    }
}
