use competitive::{algorithm, prelude::*};

#[verify::library_checker("chromatic_number")]
pub fn chromatic_number(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, uv: [(usize, usize); m]);
    pp!(algorithm::chromatic_number(n, &uv));
}
