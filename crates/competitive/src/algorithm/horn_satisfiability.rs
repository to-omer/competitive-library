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

    #[test]
    fn test_horn_satisfiability() {
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

                let check = |lits: &Vec<bool>| -> bool {
                    for (p, q, f) in &clauses {
                        if p.iter().all(|&p| lits[p]) && lits[*q] != *f {
                            return false;
                        }
                    }
                    true
                };
                let res = hs.solve();
                let satisfiable = (0..1 << n)
                    .map(|bits| (0..n).map(|i| (bits & (1 << i)) != 0).collect())
                    .any(|lits| check(&lits));
                assert_eq!(res.is_some(), satisfiable);
                if let Some(lits) = res {
                    assert!(check(&lits));
                }
            }
        }
    }
}
