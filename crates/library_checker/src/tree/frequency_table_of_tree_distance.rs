use competitive::graph::TreeGraphScanner;
use competitive::prelude::*;

#[verify::library_checker("frequency_table_of_tree_distance")]
pub fn frequency_table_of_tree_distance(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, (g, _): @TreeGraphScanner::<usize>::new(n));
    let freqs = g.distance_frequencies();
    pp!(@it freqs[1..].iter().map(|&f| f / 2));
}
