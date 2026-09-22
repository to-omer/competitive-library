use competitive::math::discrete_logarithm;
use competitive::prelude::*;

#[verify::library_checker("discrete_logarithm_mod")]
pub fn discrete_logarithm_mod(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t, query: [(u64, u64, u64); iter t]);
    for (x, y, m) in query {
        let ans = discrete_logarithm(x, y, m).map(|k| k as i64).unwrap_or(-1);
        pp!(ans);
    }
}
