use competitive::prelude::*;
use competitive::{
    math::Fps998244353,
    num::{One, montgomery::MInt998244353 as M},
};

#[verify::library_checker("kth_term_of_linearly_recurrent_sequence")]
pub fn kth_term_of_linearly_recurrent_sequence(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(d, k, a: [M; d], c: [M; d]);
    let q = Fps998244353::one() - (Fps998244353::from_vec(c) << 1);
    pp!(q.kth_term_of_linearly_recurrence(a, k));
}
