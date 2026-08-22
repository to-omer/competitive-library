use competitive::graph::minimum_assignment;
use competitive::prelude::*;

#[verify::library_checker("assignment")]
pub fn assignment(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [[i32; n]; n]);
    let (cost, assignment) = minimum_assignment(&a);
    iter_print!(writer, cost);
    iter_print!(writer, @it assignment);
}
