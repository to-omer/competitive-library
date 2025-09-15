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

    #[test]
    fn test_id_generator() {
        let mut g = IdGenerator::new();
        assert_eq!(g.create(), 0);
        assert_eq!(g.create(), 1);
        assert_eq!(g.create_n(3), 2..5);
        assert_eq!(g.create_vec(4), vec![5, 6, 7, 8]);
        assert_eq!(g.create(), 9);
    }
}
