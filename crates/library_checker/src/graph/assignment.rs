use competitive::graph::minimum_assignment;
use competitive::prelude::*;

#[verify::library_checker("assignment")]
pub fn assignment(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, a: [[i32; n]; n]);
    let (cost, assignment) = minimum_assignment(&a);
    pp!(cost);
    pp!(@it assignment);
}
