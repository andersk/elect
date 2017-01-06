#[cfg(any(feature = "use-gmp", test))]
use gmp::mpq::Mpq;
use gmp::mpz::Mpz;
use std::error::Error;
use std::fmt;

use traits::Weight;

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
