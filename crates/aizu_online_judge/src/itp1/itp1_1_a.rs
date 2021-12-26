use competitive::prelude::*;

#[cfg_attr(nightly, verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/lesson/2/ITP1/1/ITP1_1_A"))]
pub fn itp1_1_a(_reader: impl Read, mut writer: impl Write) {
    writeln!(writer, "Hello World").ok();
}
