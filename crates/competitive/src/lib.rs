// #![warn(missing_docs)]
#![cfg_attr(verus_keep_ghost, feature(proc_macro_hygiene))]
#![allow(unknown_lints)]
#![allow(clippy::missing_safety_doc)]
#![allow(deprecated)] // FIXME: Remove this (supported since 1.95.0)
#![allow(clippy::approx_constant)] // FIXME: Remove this (supported since 1.94.0)
#![allow(clippy::manual_isolate_lowest_one)] // FIXME: Remove this (supported since 1.98.0)

//! [github]
//!
//! [snippets]
//!
//! [verification summary]
//!
//! [benchmarks]
//!
//! [coverage]
//!
//! [github]: https://github.com/to-omer/competitive-library
//! [snippets]: ../snippets/index.html
//! [verification summary]: ?search=verify
//! [benchmarks]: ../benchmarks/report/index.html
//! [coverage]: ../coverage/index.html
#![cfg_attr(feature = "verify_doc", doc = "\n[formal verification](formal)\n")]

pub mod algebra;
pub mod algorithm;
pub mod combinatorial_optimization;
pub mod data_structure;
pub mod geometry;
pub mod graph;
pub mod heuristic;
pub mod math;
pub mod num;
pub mod prelude;
pub mod string;
pub mod tools;
pub mod tree;

#[cfg(all(doc, feature = "verify_doc"))]
#[doc = include_str!("formal.md")]
pub mod formal {}
