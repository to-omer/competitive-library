#[codesnip::entry("AssociatedValue")]
pub use self::associated_value::AssociatedValue;
#[codesnip::entry("SimuratedAnnealing")]
pub use self::heuristics::SimuratedAnnealing;
#[codesnip::entry("ord_tools")]
pub use self::ord_tools::PartialOrdExt;
pub use self::random::*;
#[codesnip::entry("scanner")]
pub use self::scanner::*;
pub use self::slice::GetDistinctMut;
#[codesnip::entry("TotalOrd")]
pub use self::totalord::TotalOrd;

#[cfg_attr(nightly, codesnip::entry("AssociatedValue"))]
mod associated_value;
#[cfg_attr(nightly, codesnip::entry)]
mod capture;
#[cfg_attr(nightly, codesnip::entry("SimuratedAnnealing", include("Xorshift")))]
mod heuristics;
#[cfg_attr(nightly, codesnip::entry("_iter_print"))]
mod iter_print;
#[cfg_attr(nightly, codesnip::entry("comprehension"))]
mod iterable;
#[cfg_attr(
    nightly,
    codesnip::entry("main", inline, include("scanner", "_iter_print"))
)]
mod main;
mod map;
#[cfg_attr(nightly, codesnip::entry("ord_tools"))]
mod ord_tools;
mod random;
#[cfg_attr(nightly, codesnip::entry("scanner"))]
mod scanner;
mod slice;
#[cfg_attr(nightly, codesnip::entry("TotalOrd"))]
mod totalord;
