#![cfg_attr(feature = "verify_doc", feature(external_doc))]

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
