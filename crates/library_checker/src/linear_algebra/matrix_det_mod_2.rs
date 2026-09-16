use competitive::prelude::*;
use competitive::{data_structure::BitSet, math::BitMatrix};

#[verify::library_checker("matrix_det_mod_2")]
pub fn matrix_det_mod_2(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [@BitSet::from_binary; n]);
    let mut a = BitMatrix::from_vec(a);
    pp!(u8::from(a.determinant()));
}
