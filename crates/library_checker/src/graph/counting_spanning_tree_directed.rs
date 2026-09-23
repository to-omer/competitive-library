use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353 as M};

#[verify::library_checker("counting_spanning_tree_directed")]
pub fn counting_spanning_tree_directed(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, r: usize, edges: [(usize, usize); iter m]);
    let mut a = Matrix::<AddMulOperation<M>>::zeros((n - 1, n - 1));
    for (u, v) in edges {
        if v != r {
            let v = v - usize::from(v > r);
            a[v][v] += M::from(1);
            if u != r {
                a[u - usize::from(u > r)][v] -= M::from(1);
            }
        }
    }
    pp!(a.determinant());
}
