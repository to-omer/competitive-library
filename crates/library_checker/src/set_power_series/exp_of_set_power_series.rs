use competitive::prelude::*;
use competitive::{
    algebra::AddMulOperation, math::SubsetConvolve, num::mint_basic::MInt998244353 as M,
};

#[verify::library_checker("exp_of_set_power_series", include("SubsetConvolve", "MInt"))]
pub fn exp_of_set_power_series(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [M; 1 << n]);
    pp!(@it SubsetConvolve::<AddMulOperation<_>>::exp(&a));
}
