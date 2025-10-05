use super::Dinic;
use std::{cmp::Ordering, collections::HashMap};

#[derive(Debug, Default, Clone)]
pub struct ProjectSelectionProblem {
    n_values: Vec<usize>,
    start: Vec<usize>,
    cost1: Vec<Vec<i64>>,
    cost2: HashMap<(usize, usize), u64>,
    totalcost: i64,
}
impl ProjectSelectionProblem {
    pub fn new(n_project: usize, n_value: usize) -> Self {
        Self {
            n_values: vec![n_value; n_project],
            start: (0..=n_project * (n_value - 1))
                .step_by(n_value - 1)
                .collect(),
            cost1: vec![vec![0i64; n_value]; n_project],
            cost2: Default::default(),
            totalcost: 0i64,
        }
    }
    pub fn with_n_values(n_values: Vec<usize>) -> Self {
        let mut start = Vec::with_capacity(n_values.len() + 1);
        start.push(0usize);
        for nv in n_values.iter() {
            start.push(start.last().unwrap() + nv - 1);
        }
        let cost1 = n_values.iter().map(|&n| vec![0i64; n]).collect();
        Self {
            n_values,
            start,
            cost1,
            cost2: Default::default(),
            totalcost: 0i64,
        }
    }
    pub fn add_cost1(&mut self, p: usize, v: usize, c: i64) {
        self.cost1[p][v] += c;
    }
    /// x1 >= v1 && x2 < v2 (0 < v1 < nv1, 0 < v2 < nv2)
    pub fn add_cost2_01(&mut self, p1: usize, p2: usize, v1: usize, v2: usize, c: u64) {
        debug_assert!(0 < v1 && v1 < self.n_values[p1]);
        debug_assert!(0 < v2 && v2 < self.n_values[p2]);
        let key = (self.start[p1] + v1 - 1, self.start[p2] + v2 - 1);
        if c > 0 {
            *self.cost2.entry(key).or_default() += c;
        }
    }
    /// x1 < v1 && x2 >= v2 (0 < v1 < nv1, 0 < v2 < nv2)
    pub fn add_cost2_10(&mut self, p1: usize, p2: usize, v1: usize, v2: usize, c: u64) {
        self.add_cost2_01(p2, p1, v2, v1, c);
    }
    /// cost is monge: cost(v1-1, v2) + cost(v1, v2-1) >= cost(v1, v2) + cost(v1-1, v2-1)
    pub fn add_cost2<F>(&mut self, p1: usize, p2: usize, mut cost: F)
    where
        F: FnMut(usize, usize) -> i64,
    {
        debug_assert_ne!(p1, p2);
        let nv1 = self.n_values[p1];
        let nv2 = self.n_values[p2];
        debug_assert_ne!(nv1, 0);
        debug_assert_ne!(nv2, 0);
        let c00 = cost(0, 0);
        self.totalcost += c00;
        for v1 in 1usize..nv1 {
            self.add_cost1(p1, v1, cost(v1, 0) - c00);
        }
        for v2 in 1usize..nv2 {
            self.add_cost1(p2, v2, cost(0, v2) - c00);
        }
        let mut acc = 0i64;
        for v1 in 1usize..nv1 {
            for v2 in 1usize..nv2 {
                let c = cost(v1 - 1, v2) + cost(v1, v2 - 1) - cost(v1, v2) - cost(v1 - 1, v2 - 1);
                debug_assert!(c >= 0, "cost is not monge");
                let key = (self.start[p1] + v1 - 1, self.start[p2] + v2 - 1);
                if c > 0 {
                    *self.cost2.entry(key).or_default() += c as u64;
                }
                acc -= c;
            }
            self.add_cost1(p1, v1, acc);
        }
    }
    pub fn solve(&self) -> (i64, Vec<usize>) {
        let vsize = *self.start.last().unwrap();
        let esize_expect = vsize * 2 + self.cost2.len();
        let mut builder = Dinic::builder(vsize + 2, esize_expect);
        let mut totalcost = self.totalcost;
        let s = vsize;
        let t = s + 1;
        for (p, c) in self.cost1.iter().enumerate() {
            let nv = self.n_values[p];
            totalcost += c[nv - 1];
            for v in 1usize..nv {
                let u = self.start[p] + v - 1;
                let d = c[v] - c[v - 1];
                match d.cmp(&0) {
                    Ordering::Less => {
                        builder.add_edge(s, u, (-d) as u64);
                    }
                    Ordering::Greater => {
                        builder.add_edge(u, t, d as u64);
                        totalcost -= d;
                    }
                    Ordering::Equal => {}
                }
                if v >= 2 {
                    builder.add_edge(u, u - 1, u64::MAX);
                }
            }
        }
        for (&(x, y), &c) in self.cost2.iter() {
            builder.add_edge(x, y, c);
        }
        let dgraph = builder.gen_graph();
        let mut dinic = builder.build(&dgraph);
        let res = dinic.maximum_flow(s, t) as i64 + totalcost;
        let visited = dinic.minimum_cut(s);
        let mut values = vec![0usize; self.n_values.len()];
        for (p, &nv) in self.n_values.iter().enumerate() {
            for v in 1usize..nv {
                values[p] += visited[self.start[p] + v - 1] as usize;
            }
        }
        (res, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Xorshift;

    fn brute_force<F>(n_values: &[usize], mut evaluate: F) -> (i64, Vec<Vec<usize>>)
    where
        F: FnMut(&[usize]) -> i64,
    {
        let mut best_cost = None;
        let mut best_assignments = Vec::new();
        let mut current = vec![0; n_values.len()];

        fn dfs<F>(
            n_values: &[usize],
            idx: usize,
            current: &mut [usize],
            best_cost: &mut Option<i64>,
            best_assignments: &mut Vec<Vec<usize>>,
            evaluate: &mut F,
        ) where
            F: FnMut(&[usize]) -> i64,
        {
            if idx == current.len() {
                let cost = evaluate(current);
                match best_cost {
                    None => {
                        *best_cost = Some(cost);
                        best_assignments.push(current.to_vec());
                    }
                    Some(bc) if cost < *bc => {
                        *best_cost = Some(cost);
                        best_assignments.clear();
                        best_assignments.push(current.to_vec());
                    }
                    Some(bc) if cost == *bc => {
                        best_assignments.push(current.to_vec());
                    }
                    _ => {}
                }
                return;
            }
            for value in 0..n_values[idx] {
                current[idx] = value;
                dfs(
                    n_values,
                    idx + 1,
                    current,
                    best_cost,
                    best_assignments,
                    evaluate,
                );
            }
        }

        dfs(
            n_values,
            0,
            &mut current,
            &mut best_cost,
            &mut best_assignments,
            &mut evaluate,
        );
        (best_cost.unwrap(), best_assignments)
    }

    #[test]
    fn test_project_selection_problem() {
        #[derive(Clone)]
        struct Penalty {
            p1: usize,
            p2: usize,
            v1: usize,
            v2: usize,
            dir_01: bool,
            cost: u64,
        }

        #[derive(Clone)]
        struct MongePair {
            p1: usize,
            p2: usize,
            costs: Vec<Vec<i64>>,
        }

        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let n_projects = rng.random(1..=5);
            let mut n_values = Vec::with_capacity(n_projects);
            for _ in 0..n_projects {
                n_values.push(rng.random(2..=5));
            }

            let mut unary_costs = Vec::with_capacity(n_projects);
            for &nv in &n_values {
                let mut costs = Vec::with_capacity(nv);
                for _ in 0..nv {
                    costs.push(rng.random(-50..=50));
                }
                unary_costs.push(costs);
            }

            let mut penalties = Vec::new();
            let mut monge_pairs = Vec::new();
            for p1 in 0..n_projects {
                for p2 in 0..n_projects {
                    if p1 == p2 {
                        continue;
                    }
                    if rng.random(0..=2) == 0 {
                        let v1 = rng.random(1..n_values[p1]);
                        let v2 = rng.random(1..n_values[p2]);
                        let cost = rng.random(0..=50);
                        if cost == 0 {
                            continue;
                        }
                        penalties.push(Penalty {
                            p1,
                            p2,
                            v1,
                            v2,
                            dir_01: rng.random(0..=1) == 0,
                            cost,
                        });
                    }
                    if p1 < p2 && rng.random(0..=3) == 0 {
                        let nv1 = n_values[p1];
                        let nv2 = n_values[p2];
                        let mut costs = vec![vec![0i64; nv2]; nv1];
                        costs[0][0] = rng.random(-30..=30);
                        for y in 1..nv2 {
                            let delta = rng.random(-20i64..=20);
                            costs[0][y] = costs[0][y - 1] + delta;
                        }
                        for x in 1..nv1 {
                            let delta = rng.random(-20i64..=20);
                            costs[x][0] = costs[x - 1][0] + delta;
                        }
                        for x in 1..nv1 {
                            for y in 1..nv2 {
                                let upper = costs[x - 1][y] + costs[x][y - 1] - costs[x - 1][y - 1];
                                let reduction = rng.random(0i64..=40);
                                costs[x][y] = upper - reduction;
                            }
                        }
                        monge_pairs.push(MongePair { p1, p2, costs });
                    }
                }
            }

            let (expected_cost, expected_assignments) = brute_force(&n_values, |assignment| {
                let mut total = 0i64;
                for (p, &value) in assignment.iter().enumerate() {
                    total += unary_costs[p][value];
                }
                for penalty in &penalties {
                    let applies = if penalty.dir_01 {
                        assignment[penalty.p1] >= penalty.v1 && assignment[penalty.p2] < penalty.v2
                    } else {
                        assignment[penalty.p1] < penalty.v1 && assignment[penalty.p2] >= penalty.v2
                    };
                    if applies {
                        total += penalty.cost as i64;
                    }
                }
                for monge in &monge_pairs {
                    let x = assignment[monge.p1];
                    let y = assignment[monge.p2];
                    total += monge.costs[x][y];
                }
                total
            });

            let mut psp = if n_values.iter().all(|&nv| nv == n_values[0]) {
                ProjectSelectionProblem::new(n_values.len(), n_values[0])
            } else {
                ProjectSelectionProblem::with_n_values(n_values.clone())
            };
            for (p, costs) in unary_costs.iter().enumerate() {
                for (value, &cost) in costs.iter().enumerate() {
                    psp.add_cost1(p, value, cost);
                }
            }
            for penalty in &penalties {
                if penalty.dir_01 {
                    psp.add_cost2_01(penalty.p1, penalty.p2, penalty.v1, penalty.v2, penalty.cost);
                } else {
                    psp.add_cost2_10(penalty.p1, penalty.p2, penalty.v1, penalty.v2, penalty.cost);
                }
            }
            for monge in &monge_pairs {
                let costs = monge.costs.clone();
                psp.add_cost2(monge.p1, monge.p2, move |x, y| costs[x][y]);
            }

            let (cost, values) = psp.solve();
            assert_eq!(cost, expected_cost);
            assert!(expected_assignments.contains(&values));
        }
    }
}
