use competitive::prelude::*;
#[doc(no_inline)]
pub use competitive::{algebra::AdditiveOperation, graph::DinicBuilder};

#[cfg_attr(
    nightly,
    verify::verify("https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/7/GRL_7_A")
)]
pub fn grl_7_a(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, xs, ys, es, edges: [(usize, usize)]);
    let mut builder = DinicBuilder::new(xs + ys + 2, xs + ys + es);
    let s = xs + ys;
    let t = s + 1;
    for x in 0..xs {
        builder.add_edge(s, x, 1);
    }
    for y in 0..ys {
        builder.add_edge(y + xs, t, 1);
    }
    for (x, y) in edges.take(es) {
        builder.add_edge(x, y + xs, 1);
    }
    let graph = builder.gen_graph();
    let mut dinic = builder.build(&graph);
    writeln!(writer, "{}", dinic.maximum_flow(s, t)).ok();
}
