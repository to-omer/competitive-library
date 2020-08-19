#![cfg_attr(feature = "verify_doc", feature(external_doc))]
#![cfg_attr(feature = "verify_doc", feature(doc_alias))]
// #![warn(missing_docs)]
//! [github](https://github.com/to-omer/competitive-library)
//!
//! [verification summary](index.html?search=verify)

#[macro_use]
pub mod algebra;
pub mod algorithm;
pub mod data_structure;
pub mod geometry;
pub mod graph;
pub mod math;
pub mod num;
pub mod string;
#[macro_use]
pub mod tools;
pub(crate) mod prelude;
pub mod tree;

pub mod aizu_online_judge;
pub mod library_checker;
#[cfg(test)]
pub(crate) mod verify;
