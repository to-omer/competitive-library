//! graph structures and algorithms

use crate::{
    algebra::{
        AddMulOperation, AdditiveOperation, Associative, DotProduct, Field, Group, Invertible,
        Magma, Monoid, MonoidAct, Ring, SemiRing, Unital,
    },
    algorithm::BitDpExt,
    data_structure::{
        MergingUnionFind, PairingHeap, RangeMinimumQuery, SegmentTree, UnionFind, WaveletMatrix,
    },
    math::Matrix,
    num::{Bounded, One, Zero},
    tools::{MarkedScan, PartialIgnoredOrd, Scan, ScanSource, comparator},
    tree::XorLinkedRootedTree,
};

#[codesnip::entry("AdjacencyListGraph")]
pub use self::adjacency_list::{AdjacencyListGraph, AdjacencyListGraphScanner};
#[codesnip::entry("minimum_assignment")]
pub use self::assignment::minimum_assignment;
#[codesnip::entry("BipartiteMatching")]
pub use self::bipartite_matching::BipartiteMatching;
#[codesnip::entry("ClosureGraph")]
pub use self::closure::{ClosureGraph, UsizeGraph};
#[codesnip::entry("dulmage_mendelsohn_decomposition")]
pub use self::dulmage_mendelsohn_decomposition::dulmage_mendelsohn_decomposition;
#[codesnip::entry("EdgeListGraph")]
pub use self::edge_list::{EdgeListGraph, EdgeListGraphScanner};
#[codesnip::entry("GeneralMatching")]
pub use self::general_matching::GeneralMatching;
#[codesnip::entry("GeneralWeightedMatching")]
pub use self::general_weighted_matching::GeneralWeightedMatching;
#[codesnip::entry("Graph")]
pub use self::graph_base::*;
#[codesnip::entry("GridGraph")]
pub use self::grid::{Adj4, Adj8, GridAdjacency, GridDirection, GridGraph};
#[codesnip::entry("LowLink")]
pub use self::low_link::LowLink;
#[codesnip::entry("Dinic")]
pub use self::maximum_flow::{Dinic, DinicBuilder};
#[codesnip::entry("PrimalDual")]
pub use self::minimum_cost_flow::{PrimalDual, PrimalDualBuilder};
#[codesnip::entry("NetworkSimplex")]
pub use self::network_simplex::NetworkSimplex;
#[codesnip::entry("graph_order")]
pub use self::order::GraphOrderExt;
#[codesnip::entry("PathWeightedMatching")]
pub use self::path_weighted_matching::{PathWeightedMatching, path_weighted_matching_queries};
#[codesnip::entry("ProjectSelectionProblem")]
pub use self::project_selection_problem::ProjectSelectionProblem;
#[codesnip::entry("regular_bipartite_edge_coloring")]
pub use self::regular_bipartite_edge_coloring::regular_bipartite_edge_coloring;
#[codesnip::entry("shortest_path")]
pub use self::shortest_path::{ShortestPathExt, ShortestPathSemiRing};
#[codesnip::entry("SparseGraph")]
pub use self::sparse_graph::*;
#[codesnip::entry("steiner_tree")]
pub use self::steiner_tree::{
    SteinerTreeBuilder, SteinerTreeExt, SteinerTreeOutput, SteinerTreeParent,
    SteinerTreeParentPolicy,
};
#[codesnip::entry("StronglyConnectedComponent")]
pub use self::strongly_connected_component::StronglyConnectedComponent;
#[codesnip::entry("topological_sort")]
pub use self::topological_sort::TopologicalSortExt;
#[codesnip::entry("TwoSatisfiability")]
pub use self::two_satisfiability::TwoSatisfiability;

#[codesnip::entry("AdjacencyListGraph", include("scanner", "Graph"))]
mod adjacency_list;
#[codesnip::entry("minimum_assignment")]
mod assignment;
#[codesnip::entry("bipartite_coloring", include("SparseGraph"))]
mod bipartite_coloring;
#[codesnip::entry("BipartiteMatching")]
mod bipartite_matching;
#[codesnip::entry("ClosureGraph", include("Graph"))]
mod closure;
#[codesnip::entry("count_spanning_tree", include("EdgeListGraph", "Matrix"))]
mod count_spanning_tree;
#[codesnip::entry(
    "dulmage_mendelsohn_decomposition",
    include("BipartiteMatching", "StronglyConnectedComponent")
)]
mod dulmage_mendelsohn_decomposition;
#[codesnip::entry("EdgeListGraph", include("scanner"))]
mod edge_list;
#[codesnip::entry("GeneralMatching")]
mod general_matching;
#[codesnip::entry("GeneralWeightedMatching")]
mod general_weighted_matching;
#[codesnip::entry("Graph")]
mod graph_base;
#[codesnip::entry("graphvis", include("SparseGraph"))]
mod graphvis;
#[codesnip::entry("GridGraph", include("Graph"))]
mod grid;
#[codesnip::entry("LowLink", include("SparseGraph"))]
mod low_link;
#[codesnip::entry("Dinic", include("SparseGraph", "bounded", "zero_one"))]
mod maximum_flow;
#[codesnip::entry("PrimalDual", include("SparseGraph"))]
mod minimum_cost_flow;
#[codesnip::entry(
    "minimum_spanning_arborescence",
    include("EdgeListGraph", "PairingHeap", "UnionFind")
)]
mod minimum_spanning_arborescence;
#[codesnip::entry("minimum_spanning_tree", include("EdgeListGraph", "UnionFind"))]
mod minimum_spanning_tree;
#[codesnip::entry("NetworkSimplex", include("zero_one"))]
mod network_simplex;
#[codesnip::entry("graph_order", include("Graph"))]
mod order;
#[codesnip::entry(
    "PathWeightedMatching",
    include("zero_one", "SegmentTree", "RangeMinimumQuery", "WaveletMatrix")
)]
mod path_weighted_matching;
#[codesnip::entry("ProjectSelectionProblem", include("Dinic"))]
mod project_selection_problem;
#[codesnip::entry("regular_bipartite_edge_coloring", include("BipartiteMatching"))]
mod regular_bipartite_edge_coloring;
#[codesnip::entry(
    "shortest_path",
    include("Graph", "ring", "PartialIgnoredOrd", "bounded")
)]
pub mod shortest_path;
#[codesnip::entry("SparseGraph", include("scanner", "Graph"))]
mod sparse_graph;
#[codesnip::entry("steiner_tree", include("shortest_path", "BitDp", "UnionFind"))]
mod steiner_tree;
#[codesnip::entry("StronglyConnectedComponent", include("SparseGraph"))]
mod strongly_connected_component;
#[codesnip::entry("topological_sort", include("Graph"))]
mod topological_sort;
#[codesnip::entry(
    "tree_diameter",
    include("EdgeListGraph", "XorLinkedRootedTree", "zero_one")
)]
mod tree_diameter;
#[codesnip::entry("TwoSatisfiability", include("StronglyConnectedComponent"))]
mod two_satisfiability;
