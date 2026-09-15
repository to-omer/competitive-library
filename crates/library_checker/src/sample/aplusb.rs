use competitive::prelude::*;

#[verify::library_checker("aplusb")]
pub fn aplusb(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(a, b);
    pp!(a + b);
}
