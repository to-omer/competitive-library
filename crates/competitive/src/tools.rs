#[macro_use]
mod iterable;
#[macro_use]
#[cfg_attr(nightly, codesnip::entry("minmax", inline))]
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
    codesnip::entry("main", inline, include("scanner", "minmax", "echo"))
)]
mod main;
mod output;
#[cfg_attr(nightly, codesnip::entry("Xorshift", inline))]
mod random;
#[cfg_attr(nightly, codesnip::entry("scanner", inline))]
mod scanner;
mod slice;
#[cfg_attr(nightly, codesnip::entry("TotalOrd", inline))]
mod totalord;

pub use counter::Counter;
pub use heuristics::SimuratedAnnealing;
pub use output::echo;
pub use random::Xorshift;
pub use scanner::*;
pub use slice::GetDistinctMut;
pub use totalord::TotalOrd;
