use competitive::prelude::*;
use competitive::{math::BinomialPrefixSum, num::montgomery::MInt998244353 as M};

#[verify::library_checker("prefix_sum_of_binomial_coefficient")]
pub fn prefix_sum_of_binomial_coefficient(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t, nk: [(usize, usize); iter t]);
    let mut sum = BinomialPrefixSum::with_capacity(t);
    for (n, k) in nk {
        sum.push(n, k);
    }
    let ans: Vec<M> = sum.solve();
    pp!(@lf @it ans);
}
