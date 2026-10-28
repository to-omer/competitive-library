/// Horn Satisfiability
///
/// $\wedge(\wedge_i v_{p_i}\rightarrow v_q = f)$
pub struct HornSatisfiability {
    literals: Vec<Option<bool>>,
    clauses: Vec<(usize, usize, bool)>,
    idx: Vec<Vec<usize>>,
    satisfiable: bool,
    stack: Vec<usize>,
}

impl HornSatisfiability {
    pub fn new(size: usize) -> Self {
        Self {
            literals: vec![None; size],
            clauses: vec![],
            idx: vec![vec![]; size],
            satisfiable: true,
            stack: vec![],
        }
    }

    pub fn add_clause(&mut self, p: impl IntoIterator<Item = usize>, q: usize, f: bool) {
        let mut count = 0;
        let id = self.clauses.len();
        for p_i in p.into_iter() {
            if self.literals[p_i] != Some(true) {
                self.idx[p_i].push(id);
                count += 1;
            }
        }
        self.clauses.push((count, q, f));
        if count == 0 {
            self.stack.push(id);
            self.resolve();
        }
    }

    pub fn is_satisfiable(&self) -> bool {
        self.satisfiable
    }

    pub fn solve(&self) -> Option<Vec<bool>> {
        if !self.satisfiable {
            return None;
        }
        Some(self.literals.iter().map(|&x| x.unwrap_or(false)).collect())
    }

    fn resolve(&mut self) {
        while let Some(id) = self.stack.pop() {
            let (_count, q, f) = self.clauses[id];
            self.satisfiable &= self.literals[q] != Some(!f);
            self.literals[q] = Some(f);
            if f {
                while let Some(i) = self.idx[q].pop() {
                    let (ref mut count, _, _) = self.clauses[i];
                    *count -= 1;
                    if *count == 0 {
                        self.stack.push(i);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    fn check(solver: &HornSatisfiability, n: usize, clauses: &[(Vec<usize>, usize, bool)]) {
        let mut least = None;
        for bits in 0..1usize << n {
            if clauses.iter().all(|(p, q, f)| {
                !p.iter().all(|&v| bits & (1 << v) != 0) || (bits & (1 << q) != 0) == *f
            }) {
                least = Some(least.unwrap_or(bits) & bits);
            }
        }
        assert_eq!(solver.is_satisfiable(), least.is_some());
        let expected = least.map(|bits| (0..n).map(|v| bits & (1 << v) != 0).collect());
        assert_eq!(solver.solve(), expected);
    }

    #[test]
    fn test_horn_satisfiability() {
        for n in 0..=4 {
            let mut candidates = Vec::new();
            for q in 0..n {
                for f in [false, true] {
                    candidates.push((vec![], q, f));
                    for a in 0..n {
                        candidates.push((vec![a], q, f));
                        for b in 0..n {
                            candidates.push((vec![a, b], q, f));
                        }
                    }
                }
            }
            check(&HornSatisfiability::new(n), n, &[]);
            for first in &candidates {
                for second in &candidates {
                    let clauses = [first.clone(), second.clone()];
                    let mut solver = HornSatisfiability::new(n);
                    for (i, (p, q, f)) in clauses.iter().enumerate() {
                        solver.add_clause(p.iter().copied(), *q, *f);
                        check(&solver, n, &clauses[..=i]);
                    }
                }
            }
        }
        let cycle = [
            (vec![0, 0], 1, true),
            (vec![1, 1], 2, true),
            (vec![2, 2], 0, true),
            (vec![], 0, true),
            (vec![0, 1, 2], 2, false),
        ];
        let mut solver = HornSatisfiability::new(3);
        for (i, (p, q, f)) in cycle.iter().enumerate() {
            solver.add_clause(p.iter().copied(), *q, *f);
            check(&solver, 3, &cycle[..=i]);
        }
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let n = rng.random(1..=10);
            let mut clauses = vec![];
            let mut hs = HornSatisfiability::new(n);
            for _ in 0..100 {
                let prob = rng.randf();
                let mut clause: Vec<_> = (0..n).filter(|_| rng.randf() < prob).collect();
                let cand_q: Vec<_> = (0..n).filter(|x| !clause.contains(x)).collect();
                let q = if clause.len() == n {
                    clause.remove(rng.random(0..n))
                } else {
                    cand_q[rng.random(0..cand_q.len())]
                };
                let f = rng.gen_bool(0.5);
                hs.add_clause(clause.iter().cloned(), q, f);
                clauses.push((clause, q, f));

                check(&hs, n, &clauses);
            }
        }
    }
}
