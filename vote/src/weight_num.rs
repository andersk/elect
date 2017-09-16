use num_integer::Integer;
use num_rational::Ratio;
use std::fmt::Debug;

use traits::Weight;

impl<T> Weight for Ratio<T>
where
    T: Clone + Integer + Debug,
    i64: Into<T>,
{
    #[inline]
    fn from_i64(n: i64) -> Ratio<T> {
        Ratio::from_integer(n.into())
    }
}
