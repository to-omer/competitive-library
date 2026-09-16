use competitive::graph::TwoSatisfiability;
use competitive::prelude::*;

#[verify::library_checker("two_sat")]
pub fn two_sat(reader: impl Read, writer: impl Write) {
    prepare_io!(reader, writer);
    sc!(_p: String,
        _cnf: String,
        n,
        m,
        ab: [(isize, isize, isize)]);
    let mut two_sat = TwoSatisfiability::new(n);
    for (a, b, _) in ab.take(m) {
        two_sat.add_clause(a.unsigned_abs() - 1, a >= 0, b.unsigned_abs() - 1, b >= 0);
    }
    if let Some(v) = two_sat.two_satisfiability() {
        let ans = v
            .into_iter()
            .enumerate()
            .map(|(i, v)| if v { i as i32 + 1 } else { -(i as i32 + 1) });
        pp!("s SATISFIABLE"; "v", @it ans, 0, !);
    } else {
        pp!("s UNSATISFIABLE");
    }
}
