#[derive(Debug, Clone)]
pub struct SlopeTrick {
    left: std::collections::BinaryHeap<i64>,
    right: std::collections::BinaryHeap<i64>,
    addl: i64,
    addr: i64,
    minval: i64,
}
impl Default for SlopeTrick {
    fn default() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            addl: 0,
            addr: 0,
            minval: 0,
        }
    }
}
impl SlopeTrick {
    /// Create empty
    pub fn new() -> Self {
        Default::default()
    }
    /// Create valley
    ///
    /// f(x) = max(n(x-a), n(a-x))
    pub fn valley(a: i64, n: usize) -> Self {
        let mut self_: Self = Default::default();
        self_.left.extend(std::iter::repeat(a).take(n));
        self_.right.extend(std::iter::repeat(a).take(n));
        self_
    }
    fn push_left(&mut self, x: i64) {
        self.left.push(x - self.addl);
    }
    fn push_right(&mut self, x: i64) {
        self.right.push(-(x - self.addr));
    }
    fn peek_left(&self) -> Option<i64> {
        self.left.peek().map(|x| x + self.addl)
    }
    fn peek_right(&self) -> Option<i64> {
        self.right.peek().map(|x| -x + self.addr)
    }
    fn pop_left(&mut self) -> Option<i64> {
        self.left.pop().map(|x| x + self.addl)
    }
    fn pop_right(&mut self) -> Option<i64> {
        self.right.pop().map(|x| -x + self.addr)
    }
    /// min f(x)
    pub fn minimum(&self) -> i64 {
        self.minval
    }
    /// argmin_x f(x)
    pub fn min_range(&self) -> (Option<i64>, Option<i64>) {
        (self.peek_left(), self.peek_right())
    }
    /// f(x) += a
    pub fn add_const(&mut self, a: i64) {
        self.minval += a;
    }
    /// f(x) += max(0, (x-a))
    pub fn add_ramp(&mut self, a: i64) {
        if let Some(x) = self.peek_left() {
            self.minval += (x - a).max(0);
        }
        self.push_left(a);
        let x = self.pop_left().unwrap();
        self.push_right(x);
    }
    /// f(x) += max(0, (a-x))
    pub fn add_pmar(&mut self, a: i64) {
        if let Some(x) = self.peek_right() {
            self.minval += (a - x).max(0);
        }
        self.push_right(a);
        let x = self.pop_right().unwrap();
        self.push_left(x);
    }
    /// f(x) += |x-a|
    pub fn add_abs(&mut self, a: i64) {
        self.add_ramp(a);
        self.add_pmar(a);
    }
    /// right to left accumulated minimum
    ///
    /// f'(x) := min f(y) (y >= x)
    pub fn clear_left(&mut self) {
        self.left.clear();
        self.addl = 0;
    }
    /// left to right accumulated minimum
    ///
    /// f'(x) := min f(y) (y <= x)
    pub fn clear_right(&mut self) {
        self.right.clear();
        self.addr = 0;
    }
    /// f'(x) := f(x-a)
    pub fn shift(&mut self, a: i64) {
        self.slide_minimum(a, a);
    }
    /// f'(x) := min f(y) (x-a <= y <= x-b)
    pub fn slide_minimum(&mut self, a: i64, b: i64) {
        assert!(a <= b);
        self.addl += a;
        self.addr += b;
    }
}
