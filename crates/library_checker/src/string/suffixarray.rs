use competitive::prelude::*;
use competitive::string::SuffixArray;

#[verify::library_checker("suffixarray")]
pub fn suffixarray(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(s: Chars);
    let n = s.len();
    let sa = SuffixArray::new(&s);
    pp!(@it (1..=n).map(|i| sa[i]));
}
