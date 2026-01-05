const NEG_INF: i64 = i64::MIN / 2;

#[derive(Debug, Clone)]
pub struct GeneralWeightedMatching {
    size: usize,
    weight: Vec<Vec<i64>>,
    mate: Vec<usize>,
    matching_weight: i64,
}

impl GeneralWeightedMatching {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            weight: vec![vec![NEG_INF; size]; size],
            mate: vec![!0; size],
            matching_weight: NEG_INF,
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize, w: i64) {
        assert!(u < self.size);
        assert!(v < self.size);
        if u == v {
            return;
        }
        if w > self.weight[u][v] {
            self.weight[u][v] = w;
            self.weight[v][u] = w;
            self.matching_weight = NEG_INF;
        }
    }
    pub fn from_edges(size: usize, edges: &[(usize, usize, i64)]) -> Self {
        let mut gm = Self::new(size);
        for &(u, v, w) in edges {
            gm.add_edge(u, v, w);
        }
        gm
    }
    pub fn maximum_weight_matching(&mut self) -> (i64, Vec<(usize, usize)>) {
        self.compute();
        let mut res = Vec::with_capacity(self.size / 2);
        for v in 0..self.size {
            let u = self.mate[v];
            if u != !0 && v < u {
                res.push((v, u));
            }
        }
        (self.matching_weight, res)
    }
    fn compute(&mut self) {
        if self.matching_weight != NEG_INF {
            return;
        }
        let n = self.size;
        if n == 0 {
            self.matching_weight = 0;
            return;
        }
        let mut edges = Vec::new();
        let mut weights = Vec::new();
        let mut neighbend = vec![Vec::new(); n];
        for i in 0..n {
            for j in i + 1..n {
                let w = self.weight[i][j];
                if w == NEG_INF {
                    continue;
                }
                let k = weights.len();
                weights.push(w);
                edges.push(i);
                edges.push(j);
                neighbend[i].push(2 * k + 1);
                neighbend[j].push(2 * k);
            }
        }
        let m = weights.len();
        if m == 0 {
            self.mate.fill(!0);
            self.matching_weight = 0;
            return;
        }
        let mut blossom = BlossomMatching::new(n, edges, weights, neighbend);
        blossom.solve(false);
        self.mate.fill(!0);
        let mut total = 0i64;
        for v in 0..n {
            let u = blossom.mate[v];
            if u != !0 && v < u {
                self.mate[v] = u;
                self.mate[u] = v;
                total += self.weight[v][u];
            }
        }
        self.matching_weight = total;
    }
}

struct BlossomMatching {
    n: usize,
    endpoint: Vec<usize>,
    weight: Vec<i64>,
    neighbend: Vec<Vec<usize>>,
    mate: Vec<usize>,
    label: Vec<i32>,
    labelend: Vec<usize>,
    inblossom: Vec<usize>,
    blossomparent: Vec<usize>,
    blossomchilds: Vec<Vec<usize>>,
    blossombase: Vec<usize>,
    blossomendps: Vec<Vec<usize>>,
    bestedge: Vec<usize>,
    blossombestedges: Vec<Vec<usize>>,
    unusedblossoms: Vec<usize>,
    dualvar: Vec<i64>,
    allowedge: Vec<bool>,
    queue: Vec<usize>,
}

impl BlossomMatching {
    fn new(n: usize, endpoint: Vec<usize>, weight: Vec<i64>, neighbend: Vec<Vec<usize>>) -> Self {
        let mut maxweight = 0i64;
        for &w in &weight {
            maxweight = maxweight.max(w);
        }
        let mate = vec![!0; n];
        let label = vec![0i32; 2 * n];
        let labelend = vec![!0; 2 * n];
        let mut inblossom = vec![0usize; n];
        let blossomparent = vec![!0; 2 * n];
        let blossomchilds = vec![Vec::new(); 2 * n];
        let mut blossombase = vec![!0; 2 * n];
        let blossomendps = vec![Vec::new(); 2 * n];
        let bestedge = vec![!0; 2 * n];
        let blossombestedges = vec![Vec::new(); 2 * n];
        let mut unusedblossoms = Vec::new();
        for v in 0..n {
            inblossom[v] = v;
            blossombase[v] = v;
        }
        for b in n..2 * n {
            unusedblossoms.push(b);
        }
        let mut dualvar = vec![0i64; 2 * n];
        for dv in dualvar.iter_mut().take(n) {
            *dv = maxweight;
        }
        let allowedge = vec![false; weight.len()];
        let queue = Vec::new();
        Self {
            n,
            endpoint,
            weight,
            neighbend,
            mate,
            label,
            labelend,
            inblossom,
            blossomparent,
            blossomchilds,
            blossombase,
            blossomendps,
            bestedge,
            blossombestedges,
            unusedblossoms,
            dualvar,
            allowedge,
            queue,
        }
    }

    fn slack(&self, k: usize) -> i64 {
        let i = self.endpoint[2 * k];
        let j = self.endpoint[2 * k + 1];
        self.dualvar[i] + self.dualvar[j] - 2 * self.weight[k]
    }

    fn blossom_leaves(&self, b: usize, out: &mut Vec<usize>) {
        if b < self.n {
            out.push(b);
        } else {
            for &t in &self.blossomchilds[b] {
                if t < self.n {
                    out.push(t);
                } else {
                    self.blossom_leaves(t, out);
                }
            }
        }
    }

    fn assign_label(&mut self, w: usize, t: i32, p: usize) {
        let b = self.inblossom[w];
        self.label[w] = t;
        self.label[b] = t;
        self.labelend[w] = p;
        self.labelend[b] = p;
        self.bestedge[w] = !0;
        self.bestedge[b] = !0;
        if t == 1 {
            let mut leaves = Vec::new();
            self.blossom_leaves(b, &mut leaves);
            for v in leaves {
                self.queue.push(v);
            }
        } else {
            let base = self.blossombase[b];
            let m = self.mate[base];
            if m == !0 {
                return;
            }
            let u = self.endpoint[m];
            self.assign_label(u, 1, m ^ 1);
        }
    }

    fn scan_blossom(&mut self, mut v: usize, mut w: usize) -> usize {
        let mut path: Vec<usize> = Vec::new();
        let mut base = !0;
        while v != !0 || w != !0 {
            let b = self.inblossom[v];
            if (self.label[b] & 4) != 0 {
                base = self.blossombase[b];
                break;
            }
            path.push(b);
            self.label[b] = 5;
            if self.labelend[b] == !0 {
                v = !0;
            } else {
                v = self.endpoint[self.labelend[b]];
                let b2 = self.inblossom[v];
                v = self.endpoint[self.labelend[b2]];
            }
            if w != !0 {
                std::mem::swap(&mut v, &mut w);
            }
        }
        for b in path {
            self.label[b] = 1;
        }
        base
    }

    fn add_blossom(&mut self, base: usize, k: usize) {
        let mut v = self.endpoint[2 * k];
        let mut w = self.endpoint[2 * k + 1];
        let bb = self.inblossom[base];
        let mut bv = self.inblossom[v];
        let mut bw = self.inblossom[w];
        let b = self.unusedblossoms.pop().unwrap();
        self.blossombase[b] = base;
        self.blossomparent[b] = !0;
        self.blossomparent[bb] = b;
        self.blossomchilds[b].clear();
        self.blossomendps[b].clear();
        let mut path: Vec<usize> = Vec::new();
        let mut endps: Vec<usize> = Vec::new();
        while bv != bb {
            self.blossomparent[bv] = b;
            path.push(bv);
            endps.push(self.labelend[bv]);
            v = self.endpoint[self.labelend[bv]];
            bv = self.inblossom[v];
        }
        path.push(bb);
        path.reverse();
        self.blossomchilds[b] = path.clone();
        endps.reverse();
        endps.push(2 * k);
        self.blossomendps[b] = endps.clone();
        while bw != bb {
            self.blossomparent[bw] = b;
            path.push(bw);
            self.blossomchilds[b] = path.clone();
            endps.push(self.labelend[bw] ^ 1);
            self.blossomendps[b] = endps.clone();
            w = self.endpoint[self.labelend[bw]];
            bw = self.inblossom[w];
        }
        self.label[b] = 1;
        self.labelend[b] = self.labelend[bb];
        self.dualvar[b] = 0;
        let mut leaves = Vec::new();
        self.blossom_leaves(b, &mut leaves);
        for v in leaves {
            if self.label[self.inblossom[v]] == 2 {
                self.queue.push(v);
            }
            self.inblossom[v] = b;
        }
        let mut bestedgeto = vec![!0; 2 * self.n];
        for &bv in &path {
            let nblists: Vec<Vec<usize>> = if self.blossombestedges[bv].is_empty() {
                let mut lists = Vec::new();
                let mut leaves = Vec::new();
                self.blossom_leaves(bv, &mut leaves);
                for v in leaves {
                    let mut list = Vec::new();
                    let neigh = self.neighbend[v].clone();
                    for p in neigh {
                        list.push(p / 2);
                    }
                    lists.push(list);
                }
                lists
            } else {
                vec![self.blossombestedges[bv].clone()]
            };
            for nblist in nblists {
                for k in nblist {
                    let mut i = self.endpoint[2 * k];
                    let mut j = self.endpoint[2 * k + 1];
                    if self.inblossom[j] == b {
                        std::mem::swap(&mut i, &mut j);
                    }
                    let bj = self.inblossom[j];
                    if bj != b
                        && self.label[bj] == 1
                        && (bestedgeto[bj] == !0 || self.slack(k) < self.slack(bestedgeto[bj]))
                    {
                        bestedgeto[bj] = k;
                    }
                }
            }
            self.blossombestedges[bv].clear();
            self.bestedge[bv] = !0;
        }
        self.blossombestedges[b].clear();
        for k in bestedgeto {
            if k != !0 {
                self.blossombestedges[b].push(k);
            }
        }
        self.bestedge[b] = !0;
        for &k in &self.blossombestedges[b] {
            if self.bestedge[b] == !0 || self.slack(k) < self.slack(self.bestedge[b]) {
                self.bestedge[b] = k;
            }
        }
    }

    fn step_index(len: usize, j: usize, forward: bool) -> usize {
        if forward {
            let next = j + 1;
            if next == len { 0 } else { next }
        } else if j == 0 {
            len - 1
        } else {
            j - 1
        }
    }

    fn dec_index(len: usize, j: usize) -> usize {
        if j == 0 { len - 1 } else { j - 1 }
    }

    fn expand_blossom(&mut self, b: usize, endstage: bool) {
        let childs = self.blossomchilds[b].clone();
        for s in childs {
            self.blossomparent[s] = !0;
            if s < self.n {
                self.inblossom[s] = s;
            } else if endstage && self.dualvar[s] == 0 {
                self.expand_blossom(s, endstage);
            } else {
                let mut leaves = Vec::new();
                self.blossom_leaves(s, &mut leaves);
                for v in leaves {
                    self.inblossom[v] = s;
                }
            }
        }
        if !endstage && self.label[b] == 2 {
            let entrychild = {
                let p = self.labelend[b];
                let u = self.endpoint[p ^ 1];
                self.inblossom[u]
            };
            let len = self.blossomchilds[b].len();
            let mut j = self.blossomchilds[b]
                .iter()
                .position(|&x| x == entrychild)
                .unwrap();
            let forward = j % 2 == 1;
            let endptrick = if forward { 0usize } else { 1usize };
            let mut p = self.labelend[b];
            while j != 0 {
                let v1 = self.endpoint[p ^ 1];
                self.label[v1] = 0;
                let idx = if endptrick == 0 {
                    j
                } else {
                    Self::dec_index(len, j)
                };
                let tmp = self.blossomendps[b][idx];
                let v2 = self.endpoint[(tmp ^ endptrick) ^ 1];
                self.label[v2] = 0;
                self.assign_label(v1, 2, p);
                let tmp = self.blossomendps[b][idx];
                self.allowedge[tmp / 2] = true;
                j = Self::step_index(len, j, forward);
                let idx2 = if endptrick == 0 {
                    j
                } else {
                    Self::dec_index(len, j)
                };
                let tmp2 = self.blossomendps[b][idx2];
                p = tmp2 ^ endptrick;
                self.allowedge[p / 2] = true;
                j = Self::step_index(len, j, forward);
            }
            let bv = self.blossomchilds[b][j];
            let v1 = self.endpoint[p ^ 1];
            self.label[v1] = 2;
            self.label[bv] = 2;
            self.labelend[v1] = p;
            self.labelend[bv] = p;
            self.bestedge[bv] = !0;
            j = Self::step_index(len, j, forward);
            loop {
                let tmpb = self.blossomchilds[b][j];
                if tmpb == entrychild {
                    break;
                }
                let bv = tmpb;
                if self.label[bv] == 1 {
                    j = Self::step_index(len, j, forward);
                    continue;
                }
                let mut reached: Option<usize> = None;
                let mut leaves = Vec::new();
                self.blossom_leaves(bv, &mut leaves);
                for v in leaves {
                    if self.label[v] != 0 {
                        reached = Some(v);
                        break;
                    }
                }
                if let Some(v) = reached {
                    self.label[v] = 0;
                    let base = self.blossombase[bv];
                    let m = self.mate[base];
                    self.label[self.endpoint[m]] = 0;
                    self.assign_label(v, 2, self.labelend[v]);
                }
                j = Self::step_index(len, j, forward);
            }
        }
        self.label[b] = -1;
        self.labelend[b] = !0;
        self.blossomchilds[b].clear();
        self.blossomendps[b].clear();
        self.blossombase[b] = !0;
        self.blossombestedges[b].clear();
        self.bestedge[b] = !0;
        self.unusedblossoms.push(b);
    }

    fn augment_blossom(&mut self, b: usize, v: usize) {
        let mut t = v;
        while self.blossomparent[t] != b {
            t = self.blossomparent[t];
        }
        if t >= self.n {
            self.augment_blossom(t, v);
        }
        let len = self.blossomchilds[b].len();
        let i = self.blossomchilds[b].iter().position(|&x| x == t).unwrap();
        let mut j = i;
        let forward = i % 2 == 1;
        let endptrick = if forward { 0usize } else { 1usize };
        while j != 0 {
            j = Self::step_index(len, j, forward);
            let t = self.blossomchilds[b][j];
            let pidx = if endptrick == 0 {
                j
            } else {
                Self::dec_index(len, j)
            };
            let p = self.blossomendps[b][pidx] ^ endptrick;
            if t >= self.n {
                self.augment_blossom(t, self.endpoint[p]);
            }
            j = Self::step_index(len, j, forward);
            let t2 = self.blossomchilds[b][j];
            if t2 >= self.n {
                self.augment_blossom(t2, self.endpoint[p ^ 1]);
            }
            self.mate[self.endpoint[p]] = p ^ 1;
            self.mate[self.endpoint[p ^ 1]] = p;
        }
        let mut new_child = vec![0usize; len];
        for (idx, &val) in self.blossomchilds[b].iter().enumerate() {
            let pos = (idx + len - i) % len;
            new_child[pos] = val;
        }
        self.blossomchilds[b] = new_child;
        let mut new_endps = vec![0usize; len];
        for (idx, &val) in self.blossomendps[b].iter().enumerate() {
            let pos = (idx + len - i) % len;
            new_endps[pos] = val;
        }
        self.blossomendps[b] = new_endps;
        self.blossombase[b] = self.blossombase[self.blossomchilds[b][0]];
    }

    fn augment_matching(&mut self, k: usize) {
        let v = self.endpoint[2 * k];
        let w = self.endpoint[2 * k + 1];
        for (s, p) in [(v, 2 * k + 1), (w, 2 * k)] {
            let mut s = s;
            let mut p = p;
            loop {
                let bs = self.inblossom[s];
                if bs >= self.n {
                    self.augment_blossom(bs, s);
                }
                self.mate[s] = p;
                if self.labelend[bs] == !0 {
                    break;
                }
                let t = self.endpoint[self.labelend[bs]];
                let bt = self.inblossom[t];
                let s2 = self.endpoint[self.labelend[bt]];
                let j = self.endpoint[self.labelend[bt] ^ 1];
                if bt >= self.n {
                    self.augment_blossom(bt, j);
                }
                self.mate[j] = self.labelend[bt];
                p = self.labelend[bt] ^ 1;
                s = s2;
            }
        }
    }

    fn solve(&mut self, maxcardinality: bool) {
        for _ in 0..self.n {
            self.label.fill(0);
            self.bestedge.fill(!0);
            for b in self.n..2 * self.n {
                self.blossombestedges[b].clear();
            }
            self.allowedge.fill(false);
            self.queue.clear();
            for v in 0..self.n {
                if self.mate[v] == !0 && self.label[self.inblossom[v]] == 0 {
                    self.assign_label(v, 1, !0);
                }
            }
            let mut augmented = false;
            loop {
                while let Some(v) = self.queue.pop() {
                    if self.label[self.inblossom[v]] != 1 {
                        continue;
                    }
                    let neigh = self.neighbend[v].clone();
                    for p in neigh {
                        let k = p / 2;
                        let w = self.endpoint[p];
                        if self.inblossom[v] == self.inblossom[w] {
                            continue;
                        }
                        let mut kslack = 0i64;
                        if !self.allowedge[k] {
                            kslack = self.slack(k);
                            if kslack <= 0 {
                                self.allowedge[k] = true;
                            }
                        }
                        if self.allowedge[k] {
                            if self.label[self.inblossom[w]] == 0 {
                                self.assign_label(w, 2, p ^ 1);
                            } else if self.label[self.inblossom[w]] == 1 {
                                let base = self.scan_blossom(v, w);
                                if base != !0 {
                                    self.add_blossom(base, k);
                                } else {
                                    self.augment_matching(k);
                                    augmented = true;
                                    break;
                                }
                            } else if self.label[w] == 0 {
                                self.label[w] = 2;
                                self.labelend[w] = p ^ 1;
                            }
                        } else if self.label[self.inblossom[w]] == 1 {
                            let b = self.inblossom[v];
                            if self.bestedge[b] == !0 || kslack < self.slack(self.bestedge[b]) {
                                self.bestedge[b] = k;
                            }
                        } else if self.label[w] == 0
                            && (self.bestedge[w] == !0 || kslack < self.slack(self.bestedge[w]))
                        {
                            self.bestedge[w] = k;
                        }
                    }
                    if augmented {
                        break;
                    }
                }
                if augmented {
                    break;
                }
                let mut deltatype = -1;
                let mut delta = 0i64;
                let mut deltaedge = !0;
                let mut deltablossom = !0;
                if !maxcardinality {
                    deltatype = 1;
                    delta = self.dualvar[0..self.n].iter().cloned().min().unwrap_or(0);
                }
                for v in 0..self.n {
                    if self.label[self.inblossom[v]] == 0 && self.bestedge[v] != !0 {
                        let d = self.slack(self.bestedge[v]);
                        if deltatype == -1 || d < delta {
                            delta = d;
                            deltatype = 2;
                            deltaedge = self.bestedge[v];
                        }
                    }
                }
                for b in 0..2 * self.n {
                    if self.blossomparent[b] == !0 && self.label[b] == 1 && self.bestedge[b] != !0 {
                        let kslack = self.slack(self.bestedge[b]);
                        let d = kslack / 2;
                        if deltatype == -1 || d < delta {
                            delta = d;
                            deltatype = 3;
                            deltaedge = self.bestedge[b];
                        }
                    }
                }
                for b in self.n..2 * self.n {
                    if self.blossombase[b] != !0
                        && self.blossomparent[b] == !0
                        && self.label[b] == 2
                        && (deltatype == -1 || self.dualvar[b] < delta)
                    {
                        delta = self.dualvar[b];
                        deltatype = 4;
                        deltablossom = b;
                    }
                }
                if deltatype == -1 {
                    if maxcardinality {
                        deltatype = 1;
                        delta = 0.max(*self.dualvar[0..self.n].iter().min().unwrap_or(&0));
                    } else {
                        break;
                    }
                }
                for v in 0..self.n {
                    if self.label[self.inblossom[v]] == 1 {
                        self.dualvar[v] -= delta;
                    } else if self.label[self.inblossom[v]] == 2 {
                        self.dualvar[v] += delta;
                    }
                }
                for b in self.n..2 * self.n {
                    if self.blossombase[b] != !0 && self.blossomparent[b] == !0 {
                        if self.label[b] == 1 {
                            self.dualvar[b] += delta;
                        } else if self.label[b] == 2 {
                            self.dualvar[b] -= delta;
                        }
                    }
                }
                if deltatype == 1 {
                    break;
                } else if deltatype == 2 {
                    let k = deltaedge;
                    self.allowedge[k] = true;
                    let mut i = self.endpoint[2 * k];
                    let mut j = self.endpoint[2 * k + 1];
                    if self.label[self.inblossom[i]] == 0 {
                        std::mem::swap(&mut i, &mut j);
                    }
                    self.queue.push(i);
                } else if deltatype == 3 {
                    let k = deltaedge;
                    self.allowedge[k] = true;
                    let i = self.endpoint[2 * k];
                    self.queue.push(i);
                } else if deltatype == 4 {
                    self.expand_blossom(deltablossom, false);
                }
            }
            if !augmented {
                break;
            }
            for b in self.n..2 * self.n {
                if self.blossomparent[b] == !0
                    && self.blossombase[b] != !0
                    && self.label[b] == 1
                    && self.dualvar[b] == 0
                {
                    self.expand_blossom(b, true);
                }
            }
        }
        for v in 0..self.n {
            if self.mate[v] != !0 {
                self.mate[v] = self.endpoint[self.mate[v]];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    fn brute(n: usize, edges: &[(usize, usize, i64)]) -> i64 {
        let mut w = vec![vec![NEG_INF; n]; n];
        for &(u, v, c) in edges {
            w[u][v] = w[u][v].max(c);
            w[v][u] = w[v][u].max(c);
        }
        let mut dp = vec![NEG_INF; 1 << n];
        dp[0] = 0;
        for mask in 1usize..1 << n {
            let i = mask.trailing_zeros() as usize;
            let mask_without_i = mask & !(1 << i);
            let mut best = dp[mask_without_i];
            let mut m = mask_without_i;
            while m != 0 {
                let j = m.trailing_zeros() as usize;
                if w[i][j] != NEG_INF {
                    let val = w[i][j] + dp[mask_without_i & !(1 << j)];
                    if val > best {
                        best = val;
                    }
                }
                m &= m - 1;
            }
            dp[mask] = best;
        }
        dp[(1 << n) - 1]
    }

    #[test]
    fn test_general_weighted_matching() {
        const Q: usize = 200;
        const N: usize = 10;
        let mut rng = Xorshift::default();
        for _ in 0..Q {
            rand!(rng, n: 1..=N);
            let mut edges = vec![];
            for i in 0..n {
                for j in i + 1..n {
                    rand!(rng, b: 0..3usize);
                    if b != 0 {
                        rand!(rng, w: 1..=100i64);
                        edges.push((i, j, w));
                    }
                }
            }
            let mut gm = GeneralWeightedMatching::from_edges(n, &edges);
            let (ans, matching) = gm.maximum_weight_matching();
            let mut used = vec![false; n];
            let mut sum = 0i64;
            let mut adj = vec![vec![NEG_INF; n]; n];
            for &(u, v, w) in &edges {
                adj[u][v] = adj[u][v].max(w);
                adj[v][u] = adj[v][u].max(w);
            }
            for &(u, v) in &matching {
                assert!(u < v);
                assert!(adj[u][v] != NEG_INF);
                assert!(!used[u]);
                assert!(!used[v]);
                used[u] = true;
                used[v] = true;
                sum += adj[u][v];
            }
            assert_eq!(ans, sum);
            assert_eq!(ans, brute(n, &edges));
        }
    }
}
