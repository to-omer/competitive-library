use competitive::prelude::*;
use competitive::{algebra::AddMulOperation, math::Matrix, num::montgomery::MInt998244353};

#[verify::library_checker("system_of_linear_equations")]
pub fn system_of_linear_equations(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, a: [[MInt998244353; m]; n], b: [MInt998244353; n]);
    let a = Matrix::<AddMulOperation<MInt998244353>>::from_vec(a);
    if let Some(sol) = a.solve_system_of_linear_equations(&b) {
        pp!(sol.basis.len(); @it &sol.particular);
        for b in sol.basis {
            pp!(@it &b);
        }
    } else {
        pp!(-1);
    }
}
