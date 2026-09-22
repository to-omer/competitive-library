use competitive::prelude::*;
use competitive::{
    math::{Fps998244353, MemorizedFactorial},
    num::{One, Zero, montgomery::MInt998244353 as M},
};

#[verify::library_checker("sharp_p_subset_sum")]
pub fn sharp_p_subset_sum(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, t, s: [usize; n]);
    let f = MemorizedFactorial::new(t);
    let mut c = vec![M::zero(); t + 1];
    for s in s {
        c[s] += M::one();
    }
    let a = Fps998244353::from_vec(c).count_subset_sum(t + 1, |x| f.inv(x));
    pp!(@it a.data[1..]);
}
