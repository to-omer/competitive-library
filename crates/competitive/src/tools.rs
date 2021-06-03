#[cfg_attr(nightly, codesnip::entry("AssociatedValue", inline))]
mod associated_value;
#[cfg_attr(nightly, codesnip::entry)]
mod capture;
#[cfg_attr(
    nightly,
    codesnip::entry("SimuratedAnnealing", inline, include("Xorshift"))
)]
mod heuristics;
mod iterable;
#[cfg_attr(nightly, codesnip::entry("main", inline, include("scanner", "_echo")))]
mod main;
mod map;
#[cfg_attr(nightly, codesnip::entry("ord_tools", inline))]
mod ord_tools;
mod output;
mod random;
#[cfg_attr(nightly, codesnip::entry("scanner", inline))]
mod scanner;
mod slice;
#[cfg_attr(nightly, codesnip::entry("TotalOrd", inline))]
mod totalord;

pub use associated_value::AssociatedValue;
pub use heuristics::SimuratedAnnealing;
pub use ord_tools::*;
pub use output::{echo, Echo};
pub use random::*;
pub use scanner::*;
pub use slice::GetDistinctMut;
pub use totalord::TotalOrd;
