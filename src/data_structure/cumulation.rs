#[derive(Clone, Debug)]
pub struct Cumulation<G: Group> {
    acc: Vec<G::T>,
    group: G,
}
impl<G: Group> Cumulation<G> {
    pub fn new<I: IntoIterator<Item = G::T>>(iter: I, group: G) -> Self {
        let mut acc = vec![group.unit()];
        for t in iter.into_iter() {
            let x = group.operate(acc.last().unwrap(), &t);
            acc.push(x);
        }
        Cumulation {
            acc: acc,
            group: group,
        }
    }
    pub fn fold(&self, l: usize, r: usize) -> G::T {
        self.group
            .operate(&self.acc[l], &self.group.inverse(&self.acc[r]))
    }
}
