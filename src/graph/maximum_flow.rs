#[cargo_snippet::snippet("MaximumFlow")]
#[derive(Debug, Clone)]
pub struct RevEdge {
    pub to: usize,
    pub rev: usize,
    pub cap: u64,
}
#[cargo_snippet::snippet("MaximumFlow")]
impl RevEdge {
    pub fn new(to: usize, rev: usize, cap: u64) -> RevEdge {
        RevEdge {
            to: to,
            rev: rev,
            cap: cap,
        }
    }
}

#[derive(Debug)]
pub struct FordFulkerson {
    graph: Vec<Vec<RevEdge>>,
    used: Vec<bool>,
}
impl FordFulkerson {
    pub fn new(n: usize) -> FordFulkerson {
        FordFulkerson {
            graph: vec![vec![]; n],
            used: vec![],
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize, cap: u64) {
        let e1 = RevEdge::new(to, self.graph[to].len(), cap);
        let e2 = RevEdge::new(from, self.graph[from].len(), 0);
        self.graph[from].push(e1);
        self.graph[to].push(e2);
    }
    pub fn dfs(&mut self, u: usize, t: usize, f: u64) -> u64 {
        if u == t {
            return f;
        }
        self.used[u] = true;
        for i in 0..self.graph[u].len() {
            let RevEdge { to, rev, cap } = self.graph[u][i];
            if !self.used[to] && cap > 0 {
                let d = self.dfs(to, t, std::cmp::min(f, cap));
                if d > 0 {
                    self.graph[u][i].cap -= d;
                    self.graph[to][rev].cap += d;
                    return d;
                }
            }
        }
        0
    }
    pub fn maximum_flow(&mut self, s: usize, t: usize) -> u64 {
        let mut flow = 0;
        loop {
            self.used = vec![false; self.graph.len()];
            let f = self.dfs(s, t, std::u64::MAX);
            if f == 0 {
                break;
            }
            flow += f;
        }
        flow
    }
}

#[cargo_snippet::snippet("MaximumFlow")]
#[derive(Debug)]
pub struct Dinic {
    pub graph: Vec<Vec<RevEdge>>,
    iter: Vec<usize>,
    level: Vec<usize>,
}
#[cargo_snippet::snippet("MaximumFlow")]
impl Dinic {
    pub fn new(n: usize) -> Dinic {
        Dinic {
            graph: vec![vec![]; n],
            iter: vec![],
            level: vec![],
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize, cap: u64) {
        let e1 = RevEdge::new(to, self.graph[to].len(), cap);
        let e2 = RevEdge::new(from, self.graph[from].len(), 0);
        self.graph[from].push(e1);
        self.graph[to].push(e2);
    }
    fn bfs(&mut self, s: usize) {
        self.level = vec![std::usize::MAX; self.graph.len()];
        let mut deq = std::collections::VecDeque::new();
        self.level[s] = 0;
        deq.push_back(s);
        while let Some(u) = deq.pop_front() {
            for e in &self.graph[u] {
                if e.cap > 0 && self.level[e.to] == std::usize::MAX {
                    self.level[e.to] = self.level[u] + 1;
                    deq.push_back(e.to);
                }
            }
        }
    }
    fn dfs(&mut self, u: usize, t: usize, f: u64) -> u64 {
        if u == t {
            return f;
        }
        for i in self.iter[u]..self.graph[u].len() {
            self.iter[u] = i;
            let RevEdge { to, rev, cap } = self.graph[u][i];
            if cap > 0 && self.level[u] < self.level[to] {
                let d = self.dfs(to, t, std::cmp::min(f, cap));
                if d > 0 {
                    self.graph[u][i].cap -= d;
                    self.graph[to][rev].cap += d;
                    return d;
                }
            }
        }
        0
    }
    pub fn maximum_flow(&mut self, s: usize, t: usize) -> u64 {
        let mut flow = 0;
        loop {
            self.bfs(s);
            if self.level[t] == std::usize::MAX {
                return flow;
            }
            self.iter = vec![0; self.graph.len()];
            loop {
                let f = self.dfs(s, t, std::u64::MAX);
                if f == 0 {
                    break;
                }
                flow += f;
            }
        }
    }
}
