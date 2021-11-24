use super::Dinic;
use std::{cmp::Ordering, collections::HashMap};

#[derive(Debug, Default, Clone)]
pub struct ProjectSelectionProblem {
    n_values: Vec<usize>,
    cost1: Vec<Vec<i64>>,
    cost2: HashMap<(usize, usize), Vec<Vec<u64>>>,
    totalcost: i64,
}
impl ProjectSelectionProblem {
    pub fn new(n_project: usize, n_value: usize) -> Self {
        Self {
            n_values: vec![n_value; n_project],
            cost1: vec![vec![0i64; n_value]; n_project],
            cost2: Default::default(),
            totalcost: 0i64,
        }
    }
    pub fn with_n_values(n_values: Vec<usize>) -> Self {
        let cost1 = n_values.iter().map(|&n| vec![0i64; n]).collect();
        Self {
            n_values,
            cost1,
            cost2: Default::default(),
            totalcost: 0i64,
        }
    }
    pub fn add_cost1(&mut self, p: usize, v: usize, c: i64) {
        self.cost1[p][v] += c;
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
            self.cost1[p1][v1] += cost(v1, 0) - c00;
        }
        for v2 in 1usize..nv2 {
            self.cost1[p2][v2] += cost(0, v2) - c00;
        }
        let cost2 = self
            .cost2
            .entry((p1, p2))
            .or_insert_with(|| vec![vec![0u64; nv2 - 1]; nv1 - 1]);
        let mut acc = 0i64;
        for v1 in 1usize..nv1 {
            for v2 in 1usize..nv2 {
                let c = cost(v1 - 1, v2) + cost(v1, v2 - 1) - cost(v1, v2) - cost(v1 - 1, v2 - 1);
                debug_assert!(c >= 0, "cost is not monge");
                cost2[v1 - 1][v2 - 1] += c as u64;
                acc -= c;
            }
            self.cost1[p1][v1] += acc;
        }
    }
    pub fn solve(&self) -> (i64, Vec<usize>) {
        let mut nvacc = Vec::with_capacity(self.n_values.len() + 1);
        nvacc.push(0usize);
        for nv in self.n_values.iter() {
            nvacc.push(nvacc.last().unwrap() + nv - 1);
        }
        let vsize = *nvacc.last().unwrap();
        let esize_expect = vsize * 2
            + self
                .cost2
                .values()
                .map(|c| c.len() * c[0].len())
                .sum::<usize>();
        let mut builder = Dinic::builder(vsize + 2, esize_expect);
        let mut totalcost = self.totalcost;
        let s = vsize;
        let t = s + 1;
        for (p, c) in self.cost1.iter().enumerate() {
            let nv = self.n_values[p];
            totalcost += c[nv - 1];
            for v in 1usize..nv {
                let u = nvacc[p] + v - 1;
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
        for (&(p1, p2), cost2) in self.cost2.iter() {
            for (v1, cost2) in cost2.iter().enumerate() {
                for (v2, &c) in cost2.iter().enumerate() {
                    if c > 0 {
                        builder.add_edge(nvacc[p1] + v1, nvacc[p2] + v2, c);
                    }
                }
            }
        }
        let dgraph = builder.gen_graph();
        let mut dinic = builder.build(&dgraph);
        let res = dinic.maximum_flow(s, t) as i64 + totalcost;
        let visited = dinic.minimum_cut(s);
        let mut values = vec![0usize; self.n_values.len()];
        for (p, &nv) in self.n_values.iter().enumerate() {
            for v in 1usize..nv {
                values[p] += visited[nvacc[p] + v - 1] as usize;
            }
        }
        (res, values)
    }
}
