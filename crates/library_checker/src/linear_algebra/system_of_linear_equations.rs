use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AddMulOperation, math::Matrix, num::mint_basic::MInt998244353};

#[verify::library_checker("system_of_linear_equations")]
pub fn system_of_linear_equations(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, a: [[MInt998244353; m]; n], b: [MInt998244353; n]);
    let a = Matrix::<AddMulOperation<MInt998244353>>::from_vec(a);
    if let Some(sol) = a.solve_system_of_linear_equations(&b) {
        iter_print!(writer, sol.basis.len(); @it &sol.particular);
        for b in sol.basis {
            iter_print!(writer, @it &b);
        }
    } else {
        iter_print!(writer, -1);
    }
}
