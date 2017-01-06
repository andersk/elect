use gmp::mpq::Mpq;

use traits::Weight;

impl Weight for Mpq {
    #[inline]
    fn from_i64(n: i64) -> Mpq {
        Mpq::from(n)
    }

    #[inline]
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
