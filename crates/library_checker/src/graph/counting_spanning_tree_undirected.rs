use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353 as M};

#[verify::library_checker("counting_spanning_tree_undirected")]
pub fn counting_spanning_tree_undirected(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, edges: [(usize, usize); iter m]);
    let mut a = Matrix::<AddMulOperation<M>>::zeros((n - 1, n - 1));
    for (u, v) in edges {
        if u < n - 1 {
            a[u][u] += M::from(1);
        }
        if v < n - 1 {
            a[v][v] += M::from(1);
        }
        if u < n - 1 && v < n - 1 {
            a[u][v] -= M::from(1);
            a[v][u] -= M::from(1);
        }
    }
    pp!(a.determinant());
}
