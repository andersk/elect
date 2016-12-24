#[cfg(any(feature = "rust-gmp", test))]
extern crate gmp;
#[cfg(feature = "num-rational")]
extern crate num_rational;
#[cfg(feature = "num-rational")]
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
