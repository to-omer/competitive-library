#[doc(no_inline)]
pub use competitive::graph::NetworkSimplex;
use competitive::prelude::*;

#[verify::library_checker("min_cost_b_flow")]
pub fn min_cost_b_flow(reader: impl Read, mut writer: impl Write) {
    let s = read_all_unchecked(reader);
    let mut scanner = Scanner::new(&s);
    scan!(scanner, n, m, b: [i64; n], edges: [(usize, usize, i64, i64, i128); m]);
    let mut ns = NetworkSimplex::<i64, i128>::new(n);
    for (i, b) in b.into_iter().enumerate() {
        ns.add_demand_supply(i, b);
    }
    for (s, t, l, u, c) in edges {
        ns.add_edge(s, t, l, u, c);
    }
    let sol = ns.solve_minimize();
    if let Some(sol) = sol {
        iter_print!(writer, sol.cost);
        for i in 0..n {
            iter_print!(writer, sol.potentials[i]);
        }
        for i in 0..m {
            iter_print!(writer, sol.flows[i]);
        }
    } else {
        iter_print!(writer, "infeasible");
    }
}
