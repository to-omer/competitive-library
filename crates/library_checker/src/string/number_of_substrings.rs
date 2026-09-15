use competitive::prelude::*;
use competitive::string::{StringSearch, SuffixAutomaton};

#[verify::library_checker("number_of_substrings")]
pub fn number_of_substrings(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(s: Chars);
    let n = s.len();
    let search = StringSearch::new(s);
    let mut ans = n * (n + 1) / 2;
    for &x in search.lcp_array() {
        ans -= x;
    }
    pp!(ans);
}

#[verify::library_checker("number_of_substrings")]
pub fn number_of_substrings_suffix_automaton(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(s: Bytes);
    let sa = SuffixAutomaton::from_iter(s.iter().map(|&c| c as usize));
    pp!(sa.number_of_substrings());
}
