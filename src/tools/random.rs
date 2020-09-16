#[cargo_snippet::snippet("Xorshift")]
#[derive(Clone, Debug)]
pub struct Xorshift {
    y: u64,
}
#[cargo_snippet::snippet("Xorshift")]
impl Xorshift {
    pub fn new(seed: u64) -> Self {
        Xorshift { y: seed }
    }
    pub fn time() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .unwrap_or_default()
            .as_secs() as u64;
        Xorshift::new(seed)
    }
    #[inline]
    pub fn rand64(&mut self) -> u64 {
        self.y ^= self.y << 5;
        self.y ^= self.y >> 17;
        self.y ^= self.y << 11;
        self.y
    }
    #[inline]
    pub fn rand(&mut self, k: u64) -> u64 {
        self.rand64() % k
    }
    #[inline]
    pub fn rands(&mut self, k: u64, n: usize) -> Vec<u64> {
        (0..n).map(|_| self.rand(k)).collect::<Vec<_>>()
    }
    #[inline]
    pub fn randf(&mut self) -> f64 {
        const UPPER_MASK: u64 = 0x3FF0000000000000;
        const LOWER_MASK: u64 = 0xFFFFFFFFFFFFF;
        let tmp = UPPER_MASK | (self.rand64() & LOWER_MASK);
        let result: f64 = f64::from_bits(tmp);
        result - 1.0
    }
    #[inline]
    pub fn gen_bool(&mut self, p: f64) -> bool {
        self.randf() < p
    }
}
#[cargo_snippet::snippet("Xorshift")]
impl Default for Xorshift {
    fn default() -> Self {
        Xorshift::new(0x2b992ddfa23249d6)
    }
}
