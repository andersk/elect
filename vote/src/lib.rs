#[cfg(any(feature = "use-gmp", test))]
extern crate gmp;
#[cfg(feature = "use-num-rational")]
extern crate num_rational;
extern crate num_traits;

mod combination;
pub mod hw_float;
pub mod traits;
mod proportional_completion;
pub mod schulze;
pub mod schulze_stv;
pub mod schwartz_set;
mod util;
mod vote_management;
#[cfg(feature = "use-num-rational")]
mod weight_num;
#[cfg(any(feature = "use-gmp", test))]
mod weight_mpq;
