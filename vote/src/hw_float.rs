use num_traits::{One, Zero};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

use traits::Weight;

#[derive(Clone, Debug)]
pub struct HwFloat(f64);

impl PartialEq for HwFloat {
    fn eq(&self, other: &HwFloat) -> bool {
        self.0 == other.0
    }
}

impl Eq for HwFloat {}

impl PartialOrd for HwFloat {
    fn partial_cmp(&self, other: &HwFloat) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for HwFloat {
    fn cmp(&self, other: &HwFloat) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

macro_rules! derive_ops {
    ($tr: ident, $meth: ident) => {
        impl $tr<HwFloat> for HwFloat {
            type Output = HwFloat;

            #[inline]
            fn $meth(self, other: HwFloat) -> HwFloat {
                HwFloat((self.0).$meth(other.0))
            }
        }

        impl<'a> $tr<&'a HwFloat> for HwFloat {
            type Output = HwFloat;

            #[inline]
            fn $meth(self, other: &HwFloat) -> HwFloat {
                HwFloat((self.0).$meth(&other.0))
            }
        }

        impl<'a> $tr<HwFloat> for &'a HwFloat {
            type Output = HwFloat;

            #[inline]
            fn $meth(self, other: HwFloat) -> HwFloat {
                HwFloat((self.0).$meth(other.0))
            }
        }

        impl<'a, 'b> $tr<&'b HwFloat> for &'a HwFloat {
            type Output = HwFloat;

            #[inline]
            fn $meth(self, other: &HwFloat) -> HwFloat {
                HwFloat((self.0).$meth(&other.0))
            }
        }
    }
}

derive_ops!(Add, add);
derive_ops!(Sub, sub);
derive_ops!(Mul, mul);
derive_ops!(Div, div);

impl Zero for HwFloat {
    #[inline]
    fn zero() -> HwFloat {
        HwFloat(0.0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl One for HwFloat {
    #[inline]
    fn one() -> HwFloat {
        HwFloat(1.0)
    }
}

impl FromStr for HwFloat {
    type Err = <f64 as FromStr>::Err;

    #[inline]
    fn from_str(s: &str) -> Result<HwFloat, <f64 as FromStr>::Err> {
        FromStr::from_str(s).map(HwFloat)
    }
}

impl fmt::Display for HwFloat {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Weight for HwFloat {
    #[inline]
    fn from_i64(n: i64) -> HwFloat {
        HwFloat(n as f64)
    }

    #[inline]
    fn fuzzy_eq(&self, other: &HwFloat) -> bool {
        (self.0 - other.0).abs() < 1.0e-8
    }
}
