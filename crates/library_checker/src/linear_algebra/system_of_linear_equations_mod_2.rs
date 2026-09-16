use competitive::prelude::*;
use competitive::{data_structure::BitSet, math::BitMatrix};

#[verify::library_checker("system_of_linear_equations_mod_2")]
pub fn system_of_linear_equations_mod_2(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, _m, a: [@BitSet::from_binary; n], b: @BitSet::from_binary);
    let a = BitMatrix::from_vec(a);
    if let Some(sol) = a.solve_system_of_linear_equations(&b) {
        pp!(sol.basis.len());
        for row in std::iter::once(sol.particular).chain(sol.basis) {
            pp!(row.to_binary());
        }
    } else {
        pp!(-1);
    }
}
