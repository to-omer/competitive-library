use competitive::math::prime_factors_flatten;
use competitive::prelude::*;

#[verify::library_checker("factorize")]
pub fn factorize(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(q);
    for a in sv!([u64]).take(q) {
        let x = prime_factors_flatten(a);
        pp!(x.len(), @it x);
    }
}
