use competitive::prelude::*;
use competitive::{math::BinomialPrefixSum, num::mint_basic::MInt998244353 as M};

#[verify::library_checker(
    "prefix_sum_of_binomial_coefficient",
    include("BinomialPrefixSum", "MInt")
)]
pub fn prefix_sum_of_binomial_coefficient(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t, nk: [(u32, u32); iter t]);
    let mut sum = BinomialPrefixSum::with_capacity(t);
    for (n, k) in nk {
        sum.push(n as usize, k as usize);
    }
    let ans: Vec<M> = sum.solve();
    pp!(@lf @it ans);
}
