use competitive::algebra::AddMulOperation;
use competitive::math::DirichletPrefixSums;
use competitive::num::montgomery::MInt998244353 as M;
use competitive::prelude::*;

#[verify::library_checker(
    "dirichlet_convolution_and_prefix_sums",
    include("DirichletPrefixSums", "montgomery")
)]
pub fn dirichlet_convolution_and_prefix_sums(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t);
    for _ in 0..t {
        sc!(n: u64);
        let f = DirichletPrefixSums::from_fn(n, |_| sv!(M));
        let g = DirichletPrefixSums::from_fn(n, |_| sv!(M));
        let h = f.dirichlet_convolve::<AddMulOperation<_>>(&g);
        pp!(@it h.as_slice());
    }
}
