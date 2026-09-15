use competitive::num::mint_basic::DynMIntU32;
use competitive::prelude::*;

#[verify::library_checker("sqrt_mod")]
pub fn sqrt_mod(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(q, yp: [(u32, u32)]);
    for (y, p) in yp.take(q) {
        DynMIntU32::set_mod(p);
        if let Some(x) = DynMIntU32::from(y).sqrt() {
            pp!(x);
        } else {
            pp!("-1");
        }
    }
}
