use crate::tools::Xorshift;

#[codesnip::entry("beam_search")]
pub use self::beam_search::{ModifiableState, beam_search};
#[codesnip::entry("SimulatedAnnealing")]
pub use self::simulated_annealing::SimulatedAnnealing;

#[codesnip::entry("beam_search")]
mod beam_search;
#[codesnip::entry("SimulatedAnnealing", include("Xorshift"))]
mod simulated_annealing;
