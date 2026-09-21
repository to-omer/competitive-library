use std::ops::Range;

#[derive(Debug, Default)]
pub struct IdGenerator {
    next_id: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn create_n(&mut self, n: usize) -> Range<usize> {
        let start = self.next_id;
        self.next_id += n;
        start..self.next_id
    }

    pub fn create_vec(&mut self, n: usize) -> Vec<usize> {
        self.create_n(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::IdGenerator;
    use crate::tools::Xorshift;

    #[test]
    fn test_id_generator() {
        let mut rng = Xorshift::default();
        for _ in 0..100 {
            let mut g = IdGenerator::new();
            let mut next = 0;
            for _ in 0..100 {
                let n = rng.random(0..=32);
                match rng.random(0..3) {
                    0 => {
                        assert_eq!(g.create(), next);
                        next += 1;
                    }
                    1 => {
                        assert_eq!(g.create_n(n), next..next + n);
                        next += n;
                    }
                    _ => {
                        assert_eq!(g.create_vec(n), (next..next + n).collect::<Vec<_>>());
                        next += n;
                    }
                }
            }
        }
    }
}
