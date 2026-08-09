use competitive::graph::TreeGraphScanner;
use competitive::prelude::*;

#[verify::library_checker("jump_on_tree")]
pub fn jump_on_tree(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, (g, _): @TreeGraphScanner::<usize>::new(n));
    let hld = g.hld(0);
    for _ in 0..q {
        scan!(scanner, s, t, i);
        writeln!(writer, "{}", hld.jump(s, t, i).unwrap_or(!0) as isize).ok();
    }
}

#[verify::library_checker("jump_on_tree")]
pub fn jump_on_tree_level_ancestor(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, (g, _): @TreeGraphScanner::<usize>::new(n));
    let la = g.level_ancestor(0);
    let lca = g.lca(0);
    for _ in 0..q {
        scan!(scanner, s, t, i);
        let l = lca.lca(s, t);
        let dl = la.depth(l);
        let ds = la.depth(s) - dl;
        let dt = la.depth(t) - dl;
        let ans = if i <= ds {
            la.la(s, i)
        } else if i <= ds + dt {
            la.la(t, ds + dt - i)
        } else {
            None
        };
        writeln!(writer, "{}", ans.unwrap_or(!0) as isize).ok();
    }
}

#[verify::library_checker("jump_on_tree")]
pub fn jump_on_tree_level_ancestor_batch(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, q, (g, _): @TreeGraphScanner::<usize>::new(n), queries: [(usize, usize, usize)]);
    let lca = g.lca(0);
    let results = g.level_ancestor_batch(
        0,
        queries.take(q).map(|(s, t, i)| {
            let l = lca.lca(s, t);
            let dl = lca.depth(l);
            let ds = lca.depth(s) - dl;
            let dt = lca.depth(t) - dl;
            if i <= ds {
                (s, i)
            } else if i <= ds + dt {
                (t, ds + dt - i)
            } else {
                (0, n)
            }
        }),
    );
    iter_print!(writer, @lf @it results.iter().map(|&v| v.unwrap_or(!0) as isize));
}
