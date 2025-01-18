#[codesnip::entry]
pub fn slide_minimum<T: Clone + Ord>(v: &[T], k: usize) -> Vec<usize> {
    let mut deq = std::collections::VecDeque::new();
    let mut res = vec![];
    for i in 0..v.len() {
        while deq.back().map(|&j| v[j] >= v[i]).unwrap_or(false) {
            deq.pop_back();
        }
        deq.push_back(i);
        if i + 1 >= k {
            let f = *deq.front().unwrap();
            res.push(f);
            if f == i + 1 - k {
                deq.pop_front();
            }
        }
    }
    res
}

#[codesnip::entry("SlideMinimum")]
pub struct SlideMinimum<'a> {
    deq: std::collections::VecDeque<usize>,
    width: usize,
    left: usize,
    right: usize,
    seq: &'a [i64],
}
#[codesnip::entry("SlideMinimum")]
impl<'a> SlideMinimum<'a> {
    pub fn new(width: usize, seq: &'a [i64]) -> Self {
        let mut self_ = Self {
            deq: std::collections::VecDeque::new(),
            width,
            left: 0,
            right: 0,
            seq,
        };
        self_.build();
        self_
    }
    fn build(&mut self) {
        while self.right + 1 < self.width {
            self.rsucc();
        }
    }
    fn rsucc(&mut self) {
        while self
            .deq
            .back()
            .is_some_and(|&v| self.seq[v] >= self.seq[self.right])
        {
            self.deq.pop_back();
        }
        self.deq.push_back(self.right);
        self.right += 1;
    }
    fn lsucc(&mut self) {
        if *self.deq.front().unwrap() < self.left {
            self.deq.pop_front();
        }
        self.left += 1;
    }
    pub fn next_minimum(&mut self) -> i64 {
        self.rsucc();
        self.lsucc();
        self.seq[*self.deq.front().unwrap()]
    }
}
