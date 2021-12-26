use crate::algebra::Group;

#[codesnip::entry("BabyStepGiantStep", include("algebra"))]
/// $\min\{1\le i \le n | x^i=a\}$
#[derive(Debug)]
pub struct BabyStepGiantStep<G>
where
    G: Group,
    G::T: Eq + std::hash::Hash,
{
    block_size: usize,
    baby: std::collections::HashMap<G::T, usize>,
    xi: G::T,
}
impl<G> BabyStepGiantStep<G>
where
    G: Group,
    G::T: Eq + std::hash::Hash,
{
    pub fn new(n: usize, x: G::T) -> Self {
        let block_size = (n as f64).sqrt().ceil() as usize;
        let mut baby = std::collections::HashMap::<G::T, usize>::new();
        let mut xj = x.clone();
        for j in 1..block_size {
            let nxj = G::operate(&xj, &x);
            baby.entry(xj).or_insert(j);
            xj = nxj;
        }
        let xi = G::inverse(&xj);
        baby.entry(xj).or_insert(block_size);
        Self {
            block_size,
            baby,
            xi,
        }
    }
    pub fn solve(&self, mut a: G::T) -> Option<usize> {
        let bs = self.block_size;
        for i in (0..bs * bs).step_by(bs) {
            if let Some(j) = self.baby.get(&a) {
                return Some(i + j);
            }
            a = G::operate(&self.xi, &a);
        }
        None
    }
}
