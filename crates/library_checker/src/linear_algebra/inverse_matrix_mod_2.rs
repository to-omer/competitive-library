use competitive::prelude::*;
use competitive::{data_structure::BitSet, math::BitMatrix};

#[verify::library_checker("inverse_matrix_mod_2")]
pub fn inverse_matrix_mod_2(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [@BitSet::from_binary; n]);
    let a = BitMatrix::from_vec(a);
    if let Some(b) = a.inverse() {
        for row in b.data {
            pp!(row.to_binary());
        }
    } else {
        pp!(-1);
    }
}
