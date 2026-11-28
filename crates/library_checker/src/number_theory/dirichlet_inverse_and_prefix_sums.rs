use competitive::algebra::AddMulOperation;
use competitive::math::DirichletPrefixSums;
use competitive::num::montgomery::MInt998244353 as M;
use competitive::prelude::*;

#[verify::library_checker(
    "dirichlet_inverse_and_prefix_sums",
    include("DirichletPrefixSums", "montgomery")
)]
pub fn dirichlet_inverse_and_prefix_sums(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t);
    for _ in 0..t {
        sc!(n: u64);
        let f = DirichletPrefixSums::from_fn(n, |_| sv!(M));
        let g = f.dirichlet_inverse::<AddMulOperation<_>>(f[1].inv());
        pp!(@it g.as_slice());
    }
}
