use competitive::math::euler_phi_sum;
use competitive::prelude::*;

#[verify::library_checker("sum_of_totient_function", include("euler_phi_sum"))]
pub fn sum_of_totient_function(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n: u64);
    pp!(euler_phi_sum(n) % 998244353);
}
