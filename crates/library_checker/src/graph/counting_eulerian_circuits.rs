use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation,
    math::{Matrix, MemorizedFactorial},
    num::{One, mint_basic::MInt998244353 as M},
};

#[verify::library_checker("counting_eulerian_circuits")]
pub fn counting_eulerian_circuits(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, edges: [(usize, usize); iter m]);
    let mut a = Matrix::<AddMulOperation<M>>::zeros((n, n));
    let mut indegree = vec![0; n];
    let mut outdegree = vec![0; n];
    for (u, v) in edges {
        a[u][v] -= M::from(1);
        a[v][v] += M::from(1);
        outdegree[u] += 1;
        indegree[v] += 1;
    }
    if indegree != outdegree {
        pp!(0);
        return;
    }
    let root = outdegree.iter().position(|&d| d != 0).unwrap();
    for i in 0..n {
        a[root][i] = M::from(0);
        a[i][root] = M::from(0);
        if outdegree[i] == 0 {
            a[i][i] = M::one();
        }
    }
    a[root][root] = M::one();
    let factorial = MemorizedFactorial::new(*outdegree.iter().max().unwrap() - 1);
    let mut ans = a.determinant();
    for d in outdegree {
        if d != 0 {
            ans *= factorial.fact[d - 1];
        }
    }
    pp!(ans);
}
