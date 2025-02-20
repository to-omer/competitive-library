use super::{binary_search, MInt, MIntBase, One, Zero};

#[derive(Debug, Clone)]
pub struct IndependentSubSet<M: MIntBase> {
    pub n: usize,
    pub ind: Vec<MInt<M>>,
}
impl<M: MIntBase> IndependentSubSet<M> {
    pub fn from_adj_graph(g: &[usize]) -> Self {
        let n = g.len();
        let mut ind = Vec::with_capacity(1 << n);
        ind.push(MInt::one());
        for s in 1usize..1 << n {
            let v = s.trailing_zeros() as usize;
            ind.push(ind[s - (1 << v)] + ind[s & !(g[v] | (1 << v))]);
        }
        Self { n, ind }
    }
    pub fn k_colorable(&self, k: usize) -> bool {
        !self
            .ind
            .iter()
            .map(|d| d.pow(k))
            .enumerate()
            .map(|(s, d)| if s.count_ones() % 2 == 0 { d } else { -d })
            .sum::<MInt<M>>()
            .is_zero()
    }
    /// The smallest number of colors needed to color a graph.
    pub fn chromatic_number(&self) -> usize {
        binary_search(|&k| self.k_colorable(k), self.n, 0)
    }
}
