use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::string::wildcard_pattern_matching as wildcard_pattern_matching_library;

#[verify::library_checker("wildcard_pattern_matching")]
pub fn wildcard_pattern_matching(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, mut s: Bytes, mut t: Bytes);
    for c in s.iter_mut().chain(t.iter_mut()) {
        if *c == b'*' {
            *c = b'?';
        }
    }
    let ans = wildcard_pattern_matching_library(&t, &s);
    for ok in ans {
        write!(writer, "{}", ok as u8).ok();
    }
    writeln!(writer).ok();
}
