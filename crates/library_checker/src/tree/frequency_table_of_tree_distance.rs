#[doc(no_inline)]
pub use competitive::graph::TreeGraphScanner;
use competitive::prelude::*;

#[verify::library_checker("frequency_table_of_tree_distance")]
pub fn frequency_table_of_tree_distance(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, (g, _): @TreeGraphScanner::<usize>::new(n));
    let freqs = g.distance_frequencies();
    iter_print!(writer, @it freqs[1..].iter().map(|&f| f / 2));
}
