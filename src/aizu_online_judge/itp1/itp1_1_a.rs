use std::io::{Read, Write};

#[verify_attr::verify("https://onlinejudge.u-aizu.ac.jp/courses/lesson/2/ITP1/1/ITP1_1_A")]
pub fn itp1_1_a(_reader: &mut impl Read, writer: &mut impl Write) {
    writeln!(writer, "Hello World").ok();
}
