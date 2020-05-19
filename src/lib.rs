#![cfg_attr(feature = "verify_doc", feature(external_doc))]
#![cfg_attr(feature = "verify_doc", feature(doc_alias))]
// #![warn(missing_docs)]
//! [github](https://github.com/to-omer/competitive-library)
//!
//! You can see all verifications by searching `verify` in the search bar above.
//!

#[macro_use]
pub mod algebra;
pub mod algorithm;
pub mod data_structure;
pub mod geometry;
pub mod graph;
pub mod math;
pub mod string;
#[macro_use]
pub mod tools;
pub mod tree;

pub mod aizu_online_judge;
pub mod library_checker;
#[cfg(test)]
pub(crate) mod verify;
