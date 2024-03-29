#[doc(no_inline)]
pub use competitive::graph::DirectedGraphScanner;
use competitive::prelude::*;

#[verify::aizu_online_judge("GRL_4_B", judge = "judge_grl_4_b")]
pub fn grl_4_b(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, vs, es, (graph, _): @DirectedGraphScanner::<usize, ()>::new(vs, es));
    for u in graph.topological_sort().into_iter() {
        writeln!(writer, "{}", u).ok();
    }
}

pub fn judge_grl_4_b(input: impl Read, _output: impl Read, result: impl Read) -> bool {
    let (s_in, s_res) = (read_all_unchecked(input), read_all_unchecked(result));
    let (mut scanner_in, mut scanner_res) = (Scanner::new(&s_in), Scanner::new(&s_res));
    scan!(scanner_in, vs, es, edges: [(usize, usize)]);
    let mut ord = vec![!0usize; vs];
    let mut is_ac = true;
    for (i, u) in scanner_res.iter::<usize>().take(vs).enumerate() {
        is_ac &= ord[u] == !0usize;
        ord[u] = i;
    }
    for (u, v) in edges.take(es) {
        is_ac &= ord[u] < ord[v];
    }
    is_ac
}
