use competitive::prelude::*;
use competitive::{
    data_structure::RangeParallelUnionFind,
    num::{Zero, mint_basic::MInt998244353 as M},
};

#[verify::library_checker("range_parallel_unionfind", include("RangeParallelUnionFind", "MInt"))]
pub fn range_parallel_unionfind(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, q, mut sums: [M; n]);
    let mut uf = RangeParallelUnionFind::new(n);
    let mut answer = M::zero();
    for _ in 0..q {
        sc!(k, a, b);
        uf.unite(a, b, k, |to, from| {
            let absorbed = sums[from];
            answer += sums[to] * absorbed;
            sums[to] += absorbed;
        });
        pp!(answer);
    }
}
