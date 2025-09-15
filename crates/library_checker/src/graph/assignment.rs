#[doc(no_inline)]
pub use competitive::graph::NetworkSimplex;
use competitive::prelude::*;

#[verify::library_checker("assignment")]
pub fn assignment(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, a: [[i64; n]; n]);
    let mut ns = NetworkSimplex::<i64, i64>::new(n * 2);
    for (i, a) in a.iter().enumerate() {
        ns.add_supply(i, 1);
        ns.add_demand(i + n, 1);
        for (j, &a) in a.iter().enumerate() {
            ns.add_edge(i, j + n, 0, 1, a);
        }
    }
    let sol = ns.solve_minimize();
    if let Some(sol) = sol {
        iter_print!(writer, sol.cost);
        let p = (0..n * n)
            .filter(|&eid| sol.flows[eid] != 0)
            .map(|eid| eid % n);
        iter_print!(writer, @it p);
    } else {
        iter_print!(writer, "infeasible");
    }
}
