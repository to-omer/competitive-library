use competitive::algorithm::FloorQuotientIndex;
use competitive::prelude::*;

#[verify::library_checker("enumerate_quotients")]
pub fn enumerate_quotients(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n);
    let qi = FloorQuotientIndex::new(n);
    pp!(qi.len(); @it qi.values());
}
