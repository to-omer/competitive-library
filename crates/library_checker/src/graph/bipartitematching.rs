#[doc(no_inline)]
pub use competitive::graph::DinicBuilder;
use competitive::prelude::*;

#[verify::verify("https://judge.yosupo.jp/problem/bipartitematching")]
pub fn bipartitematching(reader: &mut impl Read, writer: &mut impl Write) {
    let s = read_all(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, l, r, m, ab: [(usize, usize); m]);
    let mut builder = DinicBuilder::new(l + r + 2, m + l + r);
    let s = l + r;
    let t = s + 1;
    for (a, b) in ab.iter().cloned() {
        builder.add_edge(a, b + l, 1);
    }
    for a in 0..l {
        builder.add_edge(s, a, 1);
    }
    for b in 0..r {
        builder.add_edge(b + l, t, 1);
    }
    let graph = builder.gen_graph();
    let mut dinic = builder.build(&graph);
    let f = dinic.maximum_flow(s, t);
    writeln!(writer, "{}", f).ok();
    for (i, (a, b)) in ab.iter().enumerate() {
        if dinic.get_flow(i) > 0 {
            writeln!(writer, "{} {}", a, b).ok();
        }
    }
}
