use competitive::prelude::*;
use competitive::{data_structure::BitSet, math::BitMatrix};

#[verify::library_checker("matrix_product_mod_2")]
pub fn matrix_product_mod_2(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, _k, a: [@BitSet::from_binary; n], b: [@BitSet::from_binary; m]);
    let a = BitMatrix::from_vec(a);
    let b = BitMatrix::from_vec(b);
    let c = &a * &b;
    for row in c.data {
        pp!(row.to_binary());
    }
}
