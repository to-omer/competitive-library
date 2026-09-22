use competitive::graph::NetworkSimplex;
use competitive::prelude::*;

#[verify::library_checker("min_cost_b_flow")]
pub fn min_cost_b_flow(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(n, m, b: [i64; iter n]);
    let mut ns = NetworkSimplex::<i64, i128>::new(n);
    for (i, b) in b.enumerate() {
        ns.add_demand_supply(i, b);
    }
    sc!(edges: [(usize, usize, i64, i64, i128); iter m]);
    for (s, t, l, u, c) in edges {
        ns.add_edge(s, t, l, u, c);
    }
    let sol = ns.solve_minimize();
    if let Some(sol) = sol {
        pp!(@lf sol.cost, @it sol.potentials, @it sol.flows);
    } else {
        pp!("infeasible");
    }
}
