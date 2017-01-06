use gmp::mpq::Mpq;

use traits::Weight;

impl Weight for Mpq {
    #[inline]
    fn from_i64(n: i64) -> Mpq {
        Mpq::from(n)
    }
}
