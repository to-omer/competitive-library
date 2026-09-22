use competitive::algorithm::chromatic_number as chromatic_number_library;
use competitive::prelude::*;

#[verify::library_checker("chromatic_number")]
pub fn chromatic_number(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, uv: [(usize, usize); m]);
    pp!(chromatic_number_library(n, &uv));
}
