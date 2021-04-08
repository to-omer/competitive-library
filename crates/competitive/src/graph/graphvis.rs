#[codesnip::skip]
use super::{BidirectionalSparseGraph, DirectedSparseGraph, UndirectedSparseGraph};

impl DirectedSparseGraph {
    pub fn to_graphvis<N, NA, E, EA>(&self, node_attr: N, edge_attr: E) -> String
    where
        N: Fn(usize) -> NA,
        E: Fn(usize) -> EA,
        NA: std::fmt::Display,
        EA: std::fmt::Display,
    {
        let mut s = String::new();
        s.push_str("digraph G {\n    graph [ splines=false, layout=neato ];\n");
        for u in self.vertices() {
            s.push_str(&format!("    {} [{}];\n", u, node_attr(u)));
        }
        for u in self.vertices() {
            for a in self.adjacencies(u) {
                s.push_str(&format!("    {} -> {} [{}];\n", u, a.to, edge_attr(a.id)));
            }
        }
        s.push('}');
        s
    }
}

impl UndirectedSparseGraph {
    pub fn to_graphvis<N, NA, E, EA>(&self, node_attr: N, edge_attr: E) -> String
    where
        N: Fn(usize) -> NA,
        E: Fn(usize) -> EA,
        NA: std::fmt::Display,
        EA: std::fmt::Display,
    {
        let mut s = String::new();
        s.push_str("graph G {\n    graph [ splines=false, layout=neato ];\n");
        for u in self.vertices() {
            s.push_str(&format!("    {} [{}];\n", u, node_attr(u)));
        }
        for (i, (u, v)) in self.edges.iter().cloned().enumerate() {
            s.push_str(&format!("    {} -- {} [{}];\n", u, v, edge_attr(i)));
        }
        s.push('}');
        s
    }
}

impl BidirectionalSparseGraph {
    pub fn to_graphvis<N, NA, E, EA>(&self, node_attr: N, edge_attr: E) -> String
    where
        N: Fn(usize) -> NA,
        E: Fn(usize) -> EA,
        NA: std::fmt::Display,
        EA: std::fmt::Display,
    {
        let mut s = String::new();
        s.push_str("digraph G {\n    graph [ splines=false, layout=neato ];\n");
        for u in self.vertices() {
            s.push_str(&format!("    {} [{}];\n", u, node_attr(u)));
        }
        for u in self.vertices() {
            for a in self.adjacencies(u) {
                s.push_str(&format!("    {} -> {} [{}];\n", u, a.to, edge_attr(a.id)));
            }
        }
        s.push('}');
        s
    }
}
