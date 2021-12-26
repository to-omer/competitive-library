//! graph structures and algorithms

use crate::tools::{IterScan, MarkedIterScan};

#[cfg_attr(nightly, codesnip::entry("AdjacencyListGraph"))]
pub use self::adjacency_list::{AdjacencyListGraph, AdjacencyListGraphScanner};
#[cfg_attr(nightly, codesnip::entry("dulmage_mendelsohn_decomposition"))]
pub use self::dulmage_mendelsohn_decomposition::dulmage_mendelsohn_decomposition;
#[cfg_attr(nightly, codesnip::entry("EdgeListGraph"))]
pub use self::edge_list::{EdgeListGraph, EdgeListGraphScanner};
#[cfg_attr(nightly, codesnip::entry("GridGraph"))]
pub use self::grid::GridGraph;
#[cfg_attr(nightly, codesnip::entry("LowLink"))]
pub use self::low_link::LowLink;
#[cfg_attr(nightly, codesnip::entry("Dinic"))]
pub use self::maximum_flow::{Dinic, DinicBuilder};
#[cfg_attr(nightly, codesnip::entry("PrimalDual"))]
pub use self::minimum_cost_flow::{PrimalDual, PrimalDualBuilder};
#[cfg_attr(nightly, codesnip::entry("ProjectSelectionProblem"))]
pub use self::project_selection_problem::ProjectSelectionProblem;
#[cfg_attr(nightly, codesnip::entry("SparseGraph"))]
pub use self::sparse_graph::*;
#[cfg_attr(nightly, codesnip::entry("StronglyConnectedComponent"))]
pub use self::strongly_connected_component::StronglyConnectedComponent;
#[cfg_attr(nightly, codesnip::entry("TwoSatisfiability"))]
pub use self::two_satisfiability::TwoSatisfiability;

#[cfg_attr(nightly, codesnip::entry("AdjacencyListGraph", include("scanner")))]
mod adjacency_list;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "dulmage_mendelsohn_decomposition",
        include("Dinic", "StronglyConnectedComponent")
    )
)]
mod dulmage_mendelsohn_decomposition;
#[cfg_attr(nightly, codesnip::entry("EdgeListGraph", include("scanner")))]
mod edge_list;
#[cfg_attr(nightly, codesnip::entry("graphvis", include("SparseGraph")))]
mod graphvis;
#[cfg_attr(nightly, codesnip::entry("GridGraph"))]
mod grid;
#[cfg_attr(nightly, codesnip::entry("LowLink", include("SparseGraph")))]
mod low_link;
#[cfg_attr(nightly, codesnip::entry("Dinic", include("SparseGraph")))]
mod maximum_flow;
#[cfg_attr(nightly, codesnip::entry("PrimalDual", include("SparseGraph")))]
mod minimum_cost_flow;
mod minimum_spanning_tree;
mod order;
#[cfg_attr(nightly, codesnip::entry("ProjectSelectionProblem", include("Dinic")))]
mod project_selection_problem;
mod shortest_path;
#[cfg_attr(nightly, codesnip::entry("SparseGraph", include("scanner")))]
mod sparse_graph;
#[cfg_attr(
    nightly,
    codesnip::entry("StronglyConnectedComponent", include("SparseGraph"))
)]
mod strongly_connected_component;
#[cfg_attr(nightly, codesnip::entry("topological_sort", include("SparseGraph")))]
mod topological_sort;
#[cfg_attr(
    nightly,
    codesnip::entry("TwoSatisfiability", include("StronglyConnectedComponent"))
)]
mod two_satisfiability;
