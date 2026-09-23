use competitive::prelude::*;
use competitive::{
    data_structure::UndoableUnionFind,
    graph::{DirectedSparseGraph, Graph},
};

#[verify::library_checker("persistent_unionfind")]
pub fn persistent_unionfind(reader: impl Read, writer: impl Write) {
    prepare_io!(buffered; reader, writer);
    sc!(n, q, queries: [(u8, i32, u32, u32); q]);
    let children = DirectedSparseGraph::from_edges(
        q + 1,
        queries
            .iter()
            .enumerate()
            .map(|(i, &(_, k, _, _))| ((k + 1) as usize, i + 1))
            .collect(),
    );
    let mut uf = UndoableUnionFind::new(n);
    let mut ans = vec![false; q];
    let mut stack: Vec<_> = children
        .neighbors(0)
        .map(|edge| (edge.to as u32 - 1, false))
        .collect();
    while let Some((i, undo)) = stack.pop() {
        let i = i as usize;
        if undo {
            uf.undo();
            continue;
        }
        let (t, _, u, v) = queries[i];
        if t == 0 {
            if uf.unite(u as usize, v as usize) {
                stack.push((i as u32, true));
            }
            stack.extend(
                children
                    .neighbors(i + 1)
                    .map(|edge| (edge.to as u32 - 1, false)),
            );
        } else {
            ans[i] = uf.same(u as usize, v as usize);
        }
    }
    for (i, &(t, _, _, _)) in queries.iter().enumerate() {
        if t == 1 {
            pp!(ans[i] as u8);
        }
    }
}
