// #![warn(missing_docs)]
#![allow(unknown_lints)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::manual_div_ceil)] // FIXME: Remove this (supported since 1.73.0)
#![allow(clippy::unnecessary_map_or)] // FIXME: Remove this (supported since 1.82.0)
#![allow(clippy::manual_repeat_n)] // FIXME: Remove this (supported since 1.82.0)
#![allow(clippy::manual_is_multiple_of)] // FIXME: Remove this (supported since 1.87.0)
#![allow(clippy::collapsible_if)] // FIXME: Remove this (supported since 1.88.0)

//! [github]
//!
//! [verification summary]
//!
//! [benchmarks]
//!
//! [coverage]
//!
//! [github]: https://github.com/to-omer/competitive-library
//! [verification summary]: ?search=verify
//! [benchmarks]: ../benchmarks/report/index.html
//! [coverage]: ../coverage/index.html

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
