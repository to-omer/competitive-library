#[snippet::entry]
pub trait MoSolver {
    type Answer;
    fn insert(&mut self, i: usize);
    fn remove(&mut self, i: usize);
    fn answer(&mut self) -> Self::Answer;
    fn mo_solve(&mut self, sqn: usize, ans: &mut [Self::Answer], lr: &[(usize, usize)]) {
        let q = lr.len();
        let mut idx: Vec<usize> = (0..q).collect();
        idx.sort_by_key(|&i| (lr[i].0 / sqn, lr[i].1));
        let (mut nl, mut nr) = (0, 0);
        for &i in idx.iter() {
            let (l, r) = lr[i];
            while nl > l {
                nl -= 1;
                self.insert(nl);
            }
            while nr < r {
                self.insert(nr);
                nr += 1;
            }
            while nl < l {
                self.remove(nl);
                nl += 1;
            }
            while nr > r {
                nr -= 1;
                self.remove(nr);
            }
            ans[i] = self.answer();
        }
    }
}
