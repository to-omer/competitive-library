use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::string::{StringSearch, SuffixAutomaton};

#[verify::library_checker("number_of_substrings")]
pub fn number_of_substrings(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Chars);
    let n = s.len();
    let search = StringSearch::new(s);
    let mut ans = n * (n + 1) / 2;
    for &x in search.lcp_array() {
        ans -= x;
    }
    writeln!(writer, "{}", ans).ok();
}

#[verify::library_checker("number_of_substrings")]
pub fn number_of_substrings_suffix_automaton(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, s: Bytes);
    let sa = SuffixAutomaton::from_iter(s.iter().map(|&c| c as usize));
    writeln!(writer, "{}", sa.number_of_substrings()).ok();
}
