use num_rational::BigRational;

use traits::Weight;

impl Weight for BigRational {
    #[inline]
    fn from_i64(n: i64) -> BigRational {
        BigRational::from_integer(n.into())
    }
}
