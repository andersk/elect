use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use std::str::FromStr;

use traits::Weight;

#[derive(Clone, Debug)]
pub struct HwFloat(f64);

impl PartialEq for HwFloat {
    fn eq(&self, other: &HwFloat) -> bool {
        &self.0 == &other.0
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
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
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

impl Weight for HwFloat {
    type FromStrErr = <f64 as FromStr>::Err;

    #[inline]
    fn zero() -> HwFloat {
        HwFloat(0.0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    #[inline]
    fn one() -> HwFloat {
        HwFloat(1.0)
    }

    #[inline]
    fn from_i64(n: i64) -> HwFloat {
        HwFloat(n as f64)
    }

    #[inline]
    fn from_str(s: &str) -> Result<HwFloat, <f64 as FromStr>::Err> {
        FromStr::from_str(s).map(HwFloat)
    }

    #[inline]
    fn to_string(&self) -> String {
        ToString::to_string(&self.0)
    }

    #[inline]
    fn fuzzy_eq(&self, other: &HwFloat) -> bool {
        (self.0 - other.0).abs() < 1.0e-8
    }
}
