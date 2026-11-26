use super::StronglyConnectedComponent;

#[derive(Debug, Clone)]
pub struct TwoSatisfiability {
    vsize: usize,
    edges: Vec<(usize, usize)>,
}

impl TwoSatisfiability {
    pub fn new(vsize: usize) -> Self {
        Self {
            vsize,
            edges: Vec::new(),
        }
    }
    /// (p_x = f) | (p_y = g)
    pub fn add_clause(&mut self, x: usize, f: bool, y: usize, g: bool) {
        self.edges.push((2 * x + f as usize, 2 * y + !g as usize));
        self.edges.push((2 * y + g as usize, 2 * x + !f as usize));
    }
    pub fn add_or(&mut self, x: usize, y: usize) {
        self.add_clause(x, true, y, true);
    }
    pub fn add_nand(&mut self, x: usize, y: usize) {
        self.add_clause(x, false, y, false);
    }
    pub fn set_true(&mut self, x: usize) {
        self.edges.push((2 * x + 1, 2 * x));
    }
    pub fn set_false(&mut self, x: usize) {
        self.edges.push((2 * x, 2 * x + 1));
    }
    pub fn two_satisfiability(self) -> Option<Vec<bool>> {
        let ids =
            StronglyConnectedComponent::from_edges(self.vsize * 2, self.edges.iter().copied())
                .component_ids();
        let mut res = vec![false; self.vsize];
        for i in 0..self.vsize {
            if ids[i * 2] == ids[i * 2 + 1] {
                return None;
            }
            res[i] = ids[i * 2] > ids[i * 2 + 1];
        }
        Some(res)
    }
}
