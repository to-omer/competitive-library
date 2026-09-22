use competitive::math::min_of_mod_of_linear as min_of_mod_of_linear_library;
use competitive::prelude::*;

#[verify::library_checker("min_of_mod_of_linear")]
pub fn min_of_mod_of_linear(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(t, query: [(u64, u64, u64, u64); iter t]);
    for (n, m, a, b) in query {
        pp!(min_of_mod_of_linear_library(n, a, b, m));
    }
}
