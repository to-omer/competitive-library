use crate::tools::Xorshift;

#[codesnip::entry("beam_search")]
pub use self::beam_search::{beam_search, ModifiableState};
#[codesnip::entry("SimuratedAnnealing")]
pub use self::simurated_annealing::SimuratedAnnealing;

#[cfg_attr(nightly, codesnip::entry("beam_search"))]
mod beam_search;
#[cfg_attr(nightly, codesnip::entry("SimuratedAnnealing", include("Xorshift")))]
mod simurated_annealing;
