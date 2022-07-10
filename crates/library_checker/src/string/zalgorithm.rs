use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::string::Zarray;

#[verify::library_checker("zalgorithm")]
pub fn zalgorithm(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let z = Zarray::new(&s);
    iter_print!(writer, @it (0..s.len()).map(|i| z[i]));
}
