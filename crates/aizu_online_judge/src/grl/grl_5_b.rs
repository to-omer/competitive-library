use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::MaxOperation, graph::TreeGraphScanner, tree::ReRooting};

#[verify::aizu_online_judge("GRL_5_B")]
pub fn grl_5_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, (graph, w): @TreeGraphScanner::<usize, u64>::new(n));
    let re = ReRooting::<MaxOperation<u64>, _>::new(&graph, |d, _vid, eid_opt| {
        d + eid_opt.map_or(0, |eid| w[eid])
    });
    iter_print!(writer, @sep '\n', @it re.dp);
}
