#[codesnip::entry("AssociatedValue")]
pub use self::associated_value::AssociatedValue;
#[codesnip::entry("char_convert")]
pub use self::char_convert::{CharConvertTryFrom, CharConvertTryInto};
#[codesnip::entry("coding")]
pub use self::coding::{SerdeByteStr, unescape};
#[codesnip::entry("Comparator")]
pub use self::comparator::Comparator;
#[codesnip::entry("digit_sequence")]
pub use self::digit_sequence::ToDigitSequence;
#[codesnip::entry("fastio")]
pub use self::fastio::{FastInput, FastOutput};
#[codesnip::entry("IdGenerator")]
pub use self::id_generator::IdGenerator;
#[codesnip::entry("_iter_print")]
pub use self::iter_print::IterPrint;
#[codesnip::entry("IteratorExt")]
pub use self::iterator_ext::IteratorExt;
#[codesnip::entry("ord_tools")]
pub use self::ord_tools::PartialOrdExt;
#[codesnip::entry("PartialIgnoredOrd")]
pub use self::partial_ignored_ord::PartialIgnoredOrd;
#[codesnip::entry("random_generator")]
pub use self::random_generator::{
    NotEmptySegment, RandIter, RandRange, RandomSpec, WeightedSampler, WithEmptySegment,
};
#[codesnip::entry("scanner")]
pub use self::scanner::*;
#[codesnip::entry("TotalOrd")]
pub use self::totalord::{AsTotalOrd, TotalOrd};
#[codesnip::entry("Xorshift")]
pub use self::xorshift::Xorshift;

#[cfg_attr(nightly, codesnip::entry)]
mod array;
#[cfg_attr(nightly, codesnip::entry)]
mod assign_ops;
#[cfg_attr(nightly, codesnip::entry("AssociatedValue"))]
mod associated_value;
#[cfg_attr(nightly, codesnip::entry("avx_helper"))]
mod avx_helper;
#[cfg_attr(nightly, codesnip::entry)]
mod capture;
#[cfg_attr(nightly, codesnip::entry("char_convert"))]
mod char_convert;
#[cfg_attr(nightly, codesnip::entry("coding"))]
mod coding;
#[cfg_attr(nightly, codesnip::entry("Comparator"))]
pub mod comparator;
#[cfg_attr(nightly, codesnip::entry("digit_sequence"))]
mod digit_sequence;
#[cfg_attr(nightly, codesnip::entry("fastio"))]
mod fastio;
#[cfg_attr(nightly, codesnip::entry("IdGenerator"))]
mod id_generator;
#[cfg_attr(nightly, codesnip::entry)]
mod invariant;
#[cfg_attr(nightly, codesnip::entry("_iter_print"))]
mod iter_print;
#[cfg_attr(nightly, codesnip::entry("comprehension"))]
mod iterable;
#[cfg_attr(nightly, codesnip::entry("IteratorExt"))]
mod iterator_ext;
#[cfg_attr(
    nightly,
    codesnip::entry("main", inline, include("scanner", "_iter_print"))
)]
mod main;
#[cfg_attr(nightly, codesnip::entry)]
mod mlambda;
#[cfg_attr(nightly, codesnip::entry("ord_tools"))]
mod ord_tools;
#[cfg_attr(nightly, codesnip::entry("PartialIgnoredOrd"))]
mod partial_ignored_ord;
#[cfg_attr(nightly, codesnip::entry("random_generator", include("Xorshift")))]
mod random_generator;
#[cfg_attr(nightly, codesnip::entry("scanner", include("array")))]
mod scanner;
#[cfg_attr(nightly, codesnip::entry("TotalOrd"))]
mod totalord;
#[cfg_attr(nightly, codesnip::entry("Xorshift"))]
mod xorshift;
