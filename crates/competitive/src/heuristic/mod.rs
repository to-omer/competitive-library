use crate::tools::Xorshift;

#[codesnip::entry("beam_search")]
pub use self::beam_search::{beam_search, ModifiableState};
#[codesnip::entry("SimulatedAnnealing")]
pub use self::simulated_annealing::SimulatedAnnealing;

#[cfg_attr(nightly, codesnip::entry("beam_search"))]
mod beam_search;
#[cfg_attr(nightly, codesnip::entry("SimulatedAnnealing", include("Xorshift")))]
mod simulated_annealing;
