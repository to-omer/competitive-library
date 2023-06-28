#[codesnip::entry("AssociatedValue")]
pub use self::associated_value::AssociatedValue;
#[codesnip::entry("char_tools")]
pub use self::char_tools::CharTools;
#[codesnip::entry("coding")]
pub use self::coding::{unescape, SerdeByteStr};
#[codesnip::entry("_iter_print")]
pub use self::iter_print::IterPrint;
#[codesnip::entry("ord_tools")]
pub use self::ord_tools::PartialOrdExt;
#[codesnip::entry("PartialIgnoredOrd")]
pub use self::partial_ignored_ord::PartialIgnoredOrd;
pub use self::random::*;
#[codesnip::entry("scanner")]
pub use self::scanner::*;
pub use self::slice::GetDistinctMut;
#[codesnip::entry("TotalOrd")]
pub use self::totalord::{AsTotalOrd, TotalOrd};

#[cfg_attr(nightly, codesnip::entry)]
mod array;
#[cfg_attr(nightly, codesnip::entry)]
mod assign_ops;
#[cfg_attr(nightly, codesnip::entry("AssociatedValue"))]
mod associated_value;
#[cfg_attr(nightly, codesnip::entry)]
mod capture;
#[cfg_attr(nightly, codesnip::entry("char_tools"))]
mod char_tools;
#[cfg_attr(nightly, codesnip::entry("coding"))]
mod coding;
#[cfg_attr(nightly, codesnip::entry)]
mod invariant;
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
#[cfg_attr(nightly, codesnip::entry)]
mod mlambda;
#[cfg_attr(nightly, codesnip::entry("ord_tools"))]
mod ord_tools;
#[cfg_attr(nightly, codesnip::entry("PartialIgnoredOrd"))]
mod partial_ignored_ord;
mod random;
#[cfg_attr(nightly, codesnip::entry("scanner", include("array")))]
mod scanner;
mod slice;
#[cfg_attr(nightly, codesnip::entry("TotalOrd"))]
mod totalord;
