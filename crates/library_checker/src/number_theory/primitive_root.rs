use competitive::math::primitive_root as primitive_root_library;
use competitive::prelude::*;

#[verify::library_checker("primitive_root")]
pub fn primitive_root(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(q, p: [u64; iter q]);
    for p in p {
        pp!(primitive_root_library(p));
    }
}
