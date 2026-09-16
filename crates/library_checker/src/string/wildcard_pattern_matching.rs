use competitive::prelude::*;
use competitive::string::wildcard_pattern_matching as wildcard_pattern_matching_library;

#[verify::library_checker("wildcard_pattern_matching")]
pub fn wildcard_pattern_matching(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(mut s: Bytes, mut t: Bytes);
    for c in s.iter_mut().chain(t.iter_mut()) {
        if *c == b'*' {
            *c = b'?';
        }
    }
    let ans = wildcard_pattern_matching_library(&t, &s);
    pp!(@ns @bw (b'0' ans));
}
