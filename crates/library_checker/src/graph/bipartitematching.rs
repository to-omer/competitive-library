#[doc(no_inline)]
pub use competitive::graph::{BipartiteMatching, DinicBuilder};
use competitive::prelude::*;

#[verify::library_checker("bipartitematching")]
pub fn bipartitematching_dinic(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
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

#[verify::library_checker("bipartitematching")]
pub fn bipartitematching(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, l, r, m, ab: [(usize, usize); m]);
    let mut bm = BipartiteMatching::from_edges(l, r, &ab);
    let matching = bm.maximum_matching();
    writeln!(writer, "{}", matching.len()).ok();
    for (x, y) in matching {
        writeln!(writer, "{} {}", x, y).ok();
    }
}
