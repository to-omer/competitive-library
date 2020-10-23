#[macro_use]
mod iterable;
#[macro_use]
#[cfg_attr(nightly, codesnip::entry("minmax", inline))]
mod minmax;
#[cfg_attr(nightly, codesnip::entry("Counter", inline))]
mod counter;
#[cfg_attr(nightly, codesnip::entry("SimuratedAnnealing", inline))]
mod heuristics;
mod main;
mod output;
#[cfg_attr(nightly, codesnip::entry("Xorshift", inline))]
mod random;
mod scanner;
#[cfg_attr(nightly, codesnip::entry("TotalOrd", inline))]
mod totalord;

pub use counter::Counter;
pub use heuristics::SimuratedAnnealing;
pub use output::echo;
pub use random::Xorshift;
pub use scanner::*;
pub use totalord::TotalOrd;
