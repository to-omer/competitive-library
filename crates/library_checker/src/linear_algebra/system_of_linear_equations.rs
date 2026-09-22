use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353 as M};

#[verify::library_checker("system_of_linear_equations")]
pub fn system_of_linear_equations(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [[M; m]; n], b: [M; n]);
    let a = Matrix::<AddMulOperation<M>>::from_vec(a);
    if let Some(sol) = a.solve_system_of_linear_equations(&b) {
        pp!(sol.basis.len(); @it2d std::iter::once(sol.particular).chain(sol.basis));
    } else {
        pp!(-1);
    }
}
