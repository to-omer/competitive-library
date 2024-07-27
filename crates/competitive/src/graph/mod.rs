//! graph structures and algorithms

use crate::{
    algebra::{Monoid, SemiRing},
    algorithm::BitDp,
    num::Bounded,
    tools::{IterScan, MarkedIterScan, PartialIgnoredOrd},
};

#[codesnip::entry("AdjacencyListGraph")]
pub use self::adjacency_list::{AdjacencyListGraph, AdjacencyListGraphScanner};
#[codesnip::entry("BipartiteMatching")]
pub use self::bipartite_matching::BipartiteMatching;
#[codesnip::entry("ClosureGraph")]
pub use self::closure::{ClosureGraph, UsizeGraph};
#[codesnip::entry("dulmage_mendelsohn_decomposition")]
pub use self::dulmage_mendelsohn_decomposition::dulmage_mendelsohn_decomposition;
#[codesnip::entry("EdgeListGraph")]
pub use self::edge_list::{EdgeListGraph, EdgeListGraphScanner};
#[codesnip::entry("GraphBase")]
pub use self::graph_base::*;
#[codesnip::entry("GridGraph")]
pub use self::grid::GridGraph;
#[codesnip::entry("LowLink")]
pub use self::low_link::LowLink;
#[codesnip::entry("Dinic")]
pub use self::maximum_flow::{Dinic, DinicBuilder};
#[codesnip::entry("PrimalDual")]
pub use self::minimum_cost_flow::{PrimalDual, PrimalDualBuilder};
#[codesnip::entry("ProjectSelectionProblem")]
pub use self::project_selection_problem::ProjectSelectionProblem;
#[codesnip::entry("shortest_path")]
pub use self::shortest_path::*;
#[codesnip::entry("SparseGraph")]
pub use self::sparse_graph::*;
#[codesnip::entry("steiner_tree")]
pub use self::steiner_tree::{SteinerTreeExt, SteinerTreeOutput};
#[codesnip::entry("StronglyConnectedComponent")]
pub use self::strongly_connected_component::StronglyConnectedComponent;
#[codesnip::entry("TwoSatisfiability")]
pub use self::two_satisfiability::TwoSatisfiability;

#[cfg_attr(nightly, codesnip::entry("AdjacencyListGraph", include("scanner")))]
mod adjacency_list;
#[cfg_attr(nightly, codesnip::entry("BipartiteMatching"))]
mod bipartite_matching;
#[cfg_attr(nightly, codesnip::entry("ClosureGraph", include("GraphBase")))]
mod closure;
#[cfg_attr(
    nightly,
    codesnip::entry(
        "dulmage_mendelsohn_decomposition",
        include("BipartiteMatching", "StronglyConnectedComponent")
    )
)]
mod dulmage_mendelsohn_decomposition;
#[cfg_attr(nightly, codesnip::entry("EdgeListGraph", include("scanner")))]
mod edge_list;
#[cfg_attr(nightly, codesnip::entry("GraphBase"))]
mod graph_base;
#[cfg_attr(nightly, codesnip::entry("graphvis", include("SparseGraph")))]
mod graphvis;
#[cfg_attr(nightly, codesnip::entry("GridGraph", include("GraphBase")))]
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
#[cfg_attr(
    nightly,
    codesnip::entry(
        "shortest_path",
        include("GraphBase", "ring", "PartialIgnoredOrd", "bounded")
    )
)]
mod shortest_path;
#[cfg_attr(
    nightly,
    codesnip::entry("SparseGraph", include("scanner", "GraphBase"))
)]
mod sparse_graph;
#[cfg_attr(
    nightly,
    codesnip::entry("steiner_tree", include("shortest_path", "BitDp"))
)]
mod steiner_tree;
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
