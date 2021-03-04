#[macro_use]
mod iterable;
#[macro_use]
#[cfg_attr(nightly, codesnip::entry("_minmax", inline))]
mod minmax;
#[cfg_attr(nightly, codesnip::entry("Counter", inline))]
mod counter;
#[cfg_attr(
    nightly,
    codesnip::entry("SimuratedAnnealing", inline, include("Xorshift"))
)]
mod heuristics;
#[macro_use]
#[cfg_attr(
    nightly,
    codesnip::entry("main", inline, include("scanner", "_minmax", "_echo"))
)]
mod main;
mod output;
mod random;
#[cfg_attr(nightly, codesnip::entry("scanner", inline))]
mod scanner;
mod slice;
#[cfg_attr(nightly, codesnip::entry("TotalOrd", inline))]
mod totalord;

pub use counter::Counter;
pub use heuristics::SimuratedAnnealing;
pub use output::echo;
pub use random::*;
pub use scanner::*;
pub use slice::GetDistinctMut;
pub use totalord::TotalOrd;
