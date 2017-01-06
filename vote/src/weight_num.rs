use num_rational::BigRational;
use std::str::FromStr;

use traits::Weight;

impl Weight for BigRational {
    type FromStrErr = <BigRational as FromStr>::Err;

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
