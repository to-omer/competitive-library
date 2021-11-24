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
                    builder.add_edge(u, u - 1, std::u64::MAX);
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
