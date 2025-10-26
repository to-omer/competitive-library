use super::{One, Zero};
use std::{
    mem::swap,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

#[derive(Debug, Clone)]
pub struct NetworkSimplex<F, C> {
    n: usize,
    edges: Vec<Edge<F, C>>,
    lowers: Vec<F>,
    dss: Vec<F>,
    bucket_size: Option<usize>,
    minor_limit: Option<usize>,
}

impl<F, C> NetworkSimplex<F, C>
where
    F: Copy + PartialOrd + Zero + Add<Output = F> + Sub<Output = F> + AddAssign + SubAssign,
    C: Copy
        + PartialOrd
        + Zero
        + One
        + Add<Output = C>
        + Sub<Output = C>
        + AddAssign
        + SubAssign
        + Neg<Output = C>
        + Mul<Output = C>
        + From<F>,
{
    pub fn new(n: usize) -> Self {
        Self {
            n,
            edges: vec![],
            lowers: vec![],
            dss: vec![F::zero(); n],
            bucket_size: None,
            minor_limit: None,
        }
    }

    /// Add demand
    pub fn add_demand(&mut self, vid: usize, demand: F) {
        assert!(vid < self.n);
        assert!(demand >= F::zero());
        self.dss[vid] -= demand;
    }

    /// Add supply
    pub fn add_supply(&mut self, vid: usize, supply: F) {
        assert!(vid < self.n);
        assert!(supply >= F::zero());
        self.dss[vid] += supply;
    }

    /// Add demand/supply (positive for supply, negative for demand)
    pub fn add_demand_supply(&mut self, vid: usize, ds: F) {
        assert!(vid < self.n);
        self.dss[vid] += ds;
    }

    /// Add edge with lower/upper capacity and cost
    pub fn add_edge(&mut self, from: usize, to: usize, lower: F, upper: F, cost: C) {
        assert!(from < self.n);
        assert!(to < self.n);
        assert!(lower <= upper);
        self.edges.push(Edge {
            to,
            cap: upper - lower,
            cost,
        });
        self.edges.push(Edge {
            to: from,
            cap: F::zero(),
            cost: -cost,
        });
        self.lowers.push(lower);
        self.dss[from] -= lower;
        self.dss[to] += lower;
    }

    pub fn set_bucket_size(&mut self, size: usize) {
        self.bucket_size = Some(size);
    }

    pub fn set_minor_limit(&mut self, limit: usize) {
        self.minor_limit = Some(limit);
    }

    pub fn solve_minimize(self) -> Option<NetworkSimplexSolution<F, C>> {
        NetworkSimplexSolver::build(self).solve()
    }
}

#[derive(Debug, Clone)]
struct Edge<F, C> {
    to: usize,
    cap: F,
    cost: C,
}

#[derive(Debug, Clone)]
struct Parent<F> {
    par: usize,
    eid: usize,
    up: F,
    down: F,
}

#[derive(Debug)]
pub struct NetworkSimplexSolution<F, C> {
    pub cost: C,
    pub flows: Vec<F>,
    pub potentials: Vec<C>,
}

#[derive(Debug)]
struct NetworkSimplexSolver<F, C> {
    n: usize,
    m: usize,
    edges: Vec<Edge<F, C>>, // forward/backward edges interleaved
    lowers: Vec<F>,         // for each original edge (forward index/2)
    dss: Vec<F>,            // demand/supply for each node

    bucket_size: usize,
    minor_limit: usize,
    potentials: Vec<C>,      // potentials for nodes (size n)
    parents: Vec<Parent<F>>, // size n (exclude super node)
    depth: Vec<usize>,       // size n+1, depth[super]=0
    next: Vec<usize>,        // euler-tour linked list for dynamic tree
    prev: Vec<usize>,
    candidates: Vec<usize>,
}

impl<F, C> NetworkSimplexSolver<F, C>
where
    F: Copy + PartialOrd + Zero + Add<Output = F> + Sub<Output = F> + AddAssign + SubAssign,
    C: Copy
        + PartialOrd
        + Zero
        + One
        + Add<Output = C>
        + Sub<Output = C>
        + AddAssign
        + SubAssign
        + Neg<Output = C>
        + Mul<Output = C>
        + From<F>,
{
    fn connect(&mut self, a: usize, b: usize) {
        self.next[a] = b;
        self.prev[b] = a;
    }

    fn build(ns: NetworkSimplex<F, C>) -> Self {
        let NetworkSimplex {
            n,
            mut edges,
            lowers,
            dss,
            bucket_size,
            minor_limit,
        } = ns;

        let m = edges.len();
        let bucket_size =
            bucket_size.unwrap_or_else(|| (((m as f64).sqrt() * 0.2) as usize).max(10));
        let minor_limit =
            minor_limit.unwrap_or_else(|| (((bucket_size as f64) * 0.1) as usize).max(3));

        let mut potentials = vec![C::zero(); n + 1];
        let mut parents = vec![
            Parent {
                par: 0,
                eid: 0,
                up: F::zero(),
                down: F::zero(),
            };
            n
        ];
        let inf_cost = edges.iter().step_by(2).fold(C::one(), |acc, edge| {
            acc + if edge.cost >= C::zero() {
                edge.cost
            } else {
                -edge.cost
            }
        });
        edges.reserve(m + n * 2);
        let super_node = n;
        for v in 0..n {
            let eid = edges.len();
            if dss[v] >= F::zero() {
                edges.push(Edge {
                    to: super_node,
                    cap: F::zero(),
                    cost: inf_cost,
                });
                edges.push(Edge {
                    to: v,
                    cap: dss[v],
                    cost: -inf_cost,
                });
                potentials[v] = -inf_cost;
            } else {
                edges.push(Edge {
                    to: super_node,
                    cap: F::zero() - dss[v],
                    cost: -inf_cost,
                });
                edges.push(Edge {
                    to: v,
                    cap: F::zero(),
                    cost: inf_cost,
                });
                potentials[v] = inf_cost;
            }
            parents[v] = Parent {
                par: super_node,
                eid,
                up: edges[eid].cap,
                down: edges[eid ^ 1].cap,
            };
        }

        let mut depth = vec![1; n + 1];
        depth[super_node] = 0;

        let mut this = NetworkSimplexSolver {
            n,
            m,
            edges,
            lowers,
            dss,
            bucket_size,
            minor_limit,
            parents,
            depth,
            next: vec![0; (n + 1) * 2],
            prev: vec![0; (n + 1) * 2],
            candidates: Vec::with_capacity(bucket_size),
            potentials,
        };
        for v in 0..=n {
            this.connect(v * 2, v * 2 + 1);
        }
        for v in 0..n {
            this.connect(v * 2 + 1, this.next[super_node * 2]);
            this.connect(super_node * 2, v * 2);
        }
        this
    }

    fn solve(mut self) -> Option<NetworkSimplexSolution<F, C>> {
        let mut eid = 0usize;
        loop {
            for _ in 0..self.minor_limit {
                if !self.minor() {
                    break;
                }
            }
            let mut best = C::zero();
            let mut best_eid: Option<usize> = None;
            self.candidates.clear();
            for _ in 0..self.edges.len() {
                if !self.edges[eid].cap.is_zero() {
                    let clen = self.edges[eid].cost + self.potentials[self.edges[eid ^ 1].to]
                        - self.potentials[self.edges[eid].to];
                    if clen < C::zero() {
                        if best_eid.is_none() || clen < best {
                            best = clen;
                            best_eid = Some(eid);
                        }
                        self.candidates.push(eid);
                        if self.candidates.len() == self.bucket_size {
                            break;
                        }
                    }
                }
                eid += 1;
                if eid == self.edges.len() {
                    eid = 0;
                }
            }
            if self.candidates.is_empty() {
                break;
            }
            if let Some(be) = best_eid {
                self.push_flow(be);
            }
        }
        for i in 0..self.n {
            let eid = self.parents[i].eid;
            self.edges[eid].cap = self.parents[i].up;
            self.edges[eid ^ 1].cap = self.parents[i].down;
        }
        self.generate_solution()
    }

    fn minor(&mut self) -> bool {
        if self.candidates.is_empty() {
            return false;
        }
        let mut best = C::zero();
        let mut best_eid: Option<usize> = None;
        let mut i = 0usize;
        while i < self.candidates.len() {
            let eid = self.candidates[i];
            if self.edges[eid].cap.is_zero() {
                self.candidates.swap_remove(i);
                continue;
            }
            let clen = self.edges[eid].cost + self.potentials[self.edges[eid ^ 1].to]
                - self.potentials[self.edges[eid].to];
            if clen >= C::zero() {
                self.candidates.swap_remove(i);
                continue;
            }
            if best_eid.is_none() || clen < best {
                best = clen;
                best_eid = Some(eid);
            }
            i += 1;
        }
        if let Some(best_eid) = best_eid {
            self.push_flow(best_eid);
            true
        } else {
            false
        }
    }

    fn get_lca(
        &self,
        mut u: usize,
        mut v: usize,
        flow: &mut F,
        del_u_side: &mut bool,
        del_u: &mut usize,
    ) -> usize {
        if self.depth[u] >= self.depth[v] {
            for _ in 0..self.depth[u] - self.depth[v] {
                if self.parents[u].down < *flow {
                    *flow = self.parents[u].down;
                    *del_u = u;
                    *del_u_side = true;
                }
                u = self.parents[u].par;
            }
        } else {
            for _ in 0..self.depth[v] - self.depth[u] {
                if self.parents[v].up <= *flow {
                    *flow = self.parents[v].up;
                    *del_u = v;
                    *del_u_side = false;
                }
                v = self.parents[v].par;
            }
        }
        while u != v {
            if self.parents[u].down < *flow {
                *flow = self.parents[u].down;
                *del_u = u;
                *del_u_side = true;
            }
            u = self.parents[u].par;
            if self.parents[v].up <= *flow {
                *flow = self.parents[v].up;
                *del_u = v;
                *del_u_side = false;
            }
            v = self.parents[v].par;
        }
        u
    }

    fn push_flow(&mut self, eid: usize) {
        let u0 = self.edges[eid ^ 1].to;
        let v0 = self.edges[eid].to;
        let mut del_u = v0;
        let mut flow = self.edges[eid].cap;
        let mut del_u_side = true;
        let lca = self.get_lca(u0, v0, &mut flow, &mut del_u_side, &mut del_u);
        if !flow.is_zero() {
            let mut u = u0;
            let mut v = v0;
            while u != lca {
                self.parents[u].up += flow;
                self.parents[u].down -= flow;
                u = self.parents[u].par;
            }
            while v != lca {
                self.parents[v].up -= flow;
                self.parents[v].down += flow;
                v = self.parents[v].par;
            }
        }
        let mut u = u0;
        let mut par = v0;
        let mut p_caps = (self.edges[eid].cap - flow, self.edges[eid ^ 1].cap + flow);
        let mut p_diff = -(self.edges[eid].cost + self.potentials[u0] - self.potentials[v0]);
        if !del_u_side {
            swap(&mut u, &mut par);
            swap(&mut p_caps.0, &mut p_caps.1);
            p_diff = -p_diff;
        }
        let mut par_eid = eid ^ if del_u_side { 0 } else { 1 };
        while par != del_u {
            let mut d = self.depth[par];
            let mut idx = u * 2;
            while idx != u * 2 + 1 {
                if idx % 2 == 0 {
                    d += 1;
                    self.potentials[idx / 2] += p_diff;
                    self.depth[idx / 2] = d;
                } else {
                    d -= 1;
                }
                idx = self.next[idx];
            }
            self.connect(self.prev[u * 2], self.next[u * 2 + 1]);
            self.connect(u * 2 + 1, self.next[par * 2]);
            self.connect(par * 2, u * 2);
            swap(&mut self.parents[u].eid, &mut par_eid);
            par_eid ^= 1;
            swap(&mut self.parents[u].up, &mut p_caps.0);
            swap(&mut self.parents[u].down, &mut p_caps.1);
            swap(&mut p_caps.0, &mut p_caps.1);
            let next_u = self.parents[u].par;
            self.parents[u].par = par;
            par = u;
            u = next_u;
        }
        self.edges[par_eid].cap = p_caps.0;
        self.edges[par_eid ^ 1].cap = p_caps.1;
    }

    fn generate_solution(mut self) -> Option<NetworkSimplexSolution<F, C>> {
        for v in 0..self.n {
            if self.dss[v] >= F::zero() {
                if !self.edges[self.m + v * 2 + 1].cap.is_zero() {
                    return None;
                }
            } else if !self.edges[self.m + v * 2].cap.is_zero() {
                return None;
            }
        }
        let mut cost = C::zero();
        let mut flows = Vec::with_capacity(self.m / 2);
        for eid in (0..self.m).step_by(2) {
            let f = self.lowers[eid / 2] + self.edges[eid ^ 1].cap;
            flows.push(f);
            cost += C::from(f) * self.edges[eid].cost;
        }
        self.potentials.pop();
        Some(NetworkSimplexSolution {
            cost,
            flows,
            potentials: self.potentials,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feasible_b_flow() {
        let mut ns = NetworkSimplex::<i32, i32>::new(3);
        ns.add_supply(0, 5);
        ns.add_demand(1, 5);
        ns.add_edge(0, 2, 0, 10, -1);
        ns.add_edge(2, 1, 0, 10, -1);
        ns.add_edge(0, 1, 0, 10, 10);
        let sol = ns.solve_minimize();
        assert!(sol.is_some(), "should be feasible");
        let sol = sol.unwrap();
        assert_eq!(sol.cost, -10);
        assert_eq!(sol.flows, vec![5, 5, 0]);
    }

    #[test]
    fn test_infeasible_b_flow() {
        let mut ns = NetworkSimplex::<i32, i32>::new(2);
        ns.add_demand(0, 5);
        ns.add_supply(1, 5);
        ns.add_edge(0, 1, 0, 3, 2);
        let sol = ns.solve_minimize();
        assert!(sol.is_none(), "should be infeasible");
    }
}
