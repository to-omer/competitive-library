use super::{BidirectionalSparseGraph, DirectedSparseGraph, UndirectedSparseGraph};
use std::{fmt::Display, fmt::Write};

impl DirectedSparseGraph {
    pub fn to_graphvis<N, NA, E, EA>(&self, node_attr: N, edge_attr: E) -> String
    where
        N: Fn(usize) -> NA,
        E: Fn(usize) -> EA,
        NA: Display,
        EA: Display,
    {
        let mut s = String::new();
        s.push_str("digraph G {\n    graph [ splines=false, layout=neato ];\n");
        for u in self.vertices() {
            writeln!(s, "    {} [{}];", u, node_attr(u)).ok();
        }
        for u in self.vertices() {
            for a in self.adjacencies(u) {
                writeln!(s, "    {} -> {} [{}];", u, a.to, edge_attr(a.id)).ok();
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
        NA: Display,
        EA: Display,
    {
        let mut s = String::new();
        s.push_str("graph G {\n    graph [ splines=false, layout=neato ];\n");
        for u in self.vertices() {
            writeln!(s, "    {} [{}];", u, node_attr(u)).ok();
        }
        for (i, (u, v)) in self.edges.iter().cloned().enumerate() {
            writeln!(s, "    {} -- {} [{}];", u, v, edge_attr(i)).ok();
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
        NA: Display,
        EA: Display,
    {
        let mut s = String::new();
        s.push_str("digraph G {\n    graph [ splines=false, layout=neato ];\n");
        for u in self.vertices() {
            writeln!(s, "    {} [{}];", u, node_attr(u)).ok();
        }
        for u in self.vertices() {
            for a in self.adjacencies(u) {
                writeln!(s, "    {} -> {} [{}];", u, a.to, edge_attr(a.id)).ok();
            }
        }
        s.push('}');
        s
    }
}
