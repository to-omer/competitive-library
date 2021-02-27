#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct BitDp(pub usize);
mod bitdp_impls {
    use super::*;
    impl BitDp {
        pub fn is_element(mask: usize, x: usize) -> bool {
            mask & 1 << x != 0
        }
        pub fn elements(&self, mask: usize) -> impl Iterator<Item = usize> {
            (0..self.0).filter(move |&x| Self::is_element(mask, x))
        }
        pub fn not_elements(&self, mask: usize) -> impl Iterator<Item = usize> {
            (0..self.0).filter(move |&x| !Self::is_element(mask, x))
        }
        pub fn is_subset(mask: usize, elements: usize) -> bool {
            mask & elements == elements
        }
        fn next_subset(mask: usize, cur: usize) -> Option<usize> {
            if cur == 0 {
                None
            } else {
                Some((cur - 1) & mask)
            }
        }
        pub fn subsets(mask: usize) -> Subsets {
            Subsets {
                mask,
                cur: Some(mask),
            }
        }
        fn next_combination(cur: usize) -> Option<usize> {
            if cur == 0 {
                None
            } else {
                let x = cur & (!cur + 1);
                let y = cur + x;
                Some(((cur & !y) / x / 2) | y)
            }
        }
        pub fn combinations(&self, k: usize) -> Combinations {
            Combinations {
                mask: 1 << self.0,
                cur: Some((1 << k) - 1),
            }
        }
    }
    #[derive(Debug, Clone)]
    pub struct Subsets {
        mask: usize,
        cur: Option<usize>,
    }
    impl Iterator for Subsets {
        type Item = usize;
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(cur) = self.cur {
                self.cur = BitDp::next_subset(self.mask, cur);
                Some(cur)
            } else {
                None
            }
        }
    }
    #[derive(Debug, Clone)]
    pub struct Combinations {
        mask: usize,
        cur: Option<usize>,
    }
    impl Iterator for Combinations {
        type Item = usize;
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(cur) = self.cur {
                if cur < self.mask {
                    self.cur = BitDp::next_combination(cur);
                    Some(cur)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elements() {
        let b = BitDp(0);
        assert_eq!(b.elements(0).collect::<Vec<_>>(), vec![]);
        assert_eq!(b.elements(1).collect::<Vec<_>>(), vec![]);

        let b = BitDp(1);
        assert_eq!(b.elements(0b0).collect::<Vec<_>>(), vec![]);
        assert_eq!(b.elements(0b1).collect::<Vec<_>>(), vec![0]);

        let b = BitDp(2);
        assert_eq!(b.elements(0b00).collect::<Vec<_>>(), vec![]);
        assert_eq!(b.elements(0b01).collect::<Vec<_>>(), vec![0]);
        assert_eq!(b.elements(0b10).collect::<Vec<_>>(), vec![1]);
        assert_eq!(b.elements(0b11).collect::<Vec<_>>(), vec![0, 1]);

        let b = BitDp(3);
        assert_eq!(b.elements(0b000).collect::<Vec<_>>(), vec![]);
        assert_eq!(b.elements(0b001).collect::<Vec<_>>(), vec![0]);
        assert_eq!(b.elements(0b010).collect::<Vec<_>>(), vec![1]);
        assert_eq!(b.elements(0b011).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(b.elements(0b100).collect::<Vec<_>>(), vec![2]);
        assert_eq!(b.elements(0b101).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(b.elements(0b110).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(b.elements(0b111).collect::<Vec<_>>(), vec![0, 1, 2]);

        let b = BitDp(4);
        assert_eq!(b.elements(0b0000).collect::<Vec<_>>(), vec![]);
        assert_eq!(b.elements(0b0001).collect::<Vec<_>>(), vec![0]);
        assert_eq!(b.elements(0b0010).collect::<Vec<_>>(), vec![1]);
        assert_eq!(b.elements(0b0011).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(b.elements(0b0100).collect::<Vec<_>>(), vec![2]);
        assert_eq!(b.elements(0b0101).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(b.elements(0b0110).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(b.elements(0b0111).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(b.elements(0b1000).collect::<Vec<_>>(), vec![3]);
        assert_eq!(b.elements(0b1001).collect::<Vec<_>>(), vec![0, 3]);
        assert_eq!(b.elements(0b1010).collect::<Vec<_>>(), vec![1, 3]);
        assert_eq!(b.elements(0b1011).collect::<Vec<_>>(), vec![0, 1, 3]);
        assert_eq!(b.elements(0b1100).collect::<Vec<_>>(), vec![2, 3]);
        assert_eq!(b.elements(0b1101).collect::<Vec<_>>(), vec![0, 2, 3]);
        assert_eq!(b.elements(0b1110).collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(b.elements(0b1111).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_not_elements() {
        let b = BitDp(0);
        assert_eq!(b.not_elements(0).collect::<Vec<_>>(), vec![]);
        assert_eq!(b.not_elements(1).collect::<Vec<_>>(), vec![]);

        let b = BitDp(1);
        assert_eq!(b.not_elements(0b0).collect::<Vec<_>>(), vec![0]);
        assert_eq!(b.not_elements(0b1).collect::<Vec<_>>(), vec![]);

        let b = BitDp(2);
        assert_eq!(b.not_elements(0b00).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(b.not_elements(0b01).collect::<Vec<_>>(), vec![1]);
        assert_eq!(b.not_elements(0b10).collect::<Vec<_>>(), vec![0]);
        assert_eq!(b.not_elements(0b11).collect::<Vec<_>>(), vec![]);

        let b = BitDp(3);
        assert_eq!(b.not_elements(0b000).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(b.not_elements(0b001).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(b.not_elements(0b010).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(b.not_elements(0b011).collect::<Vec<_>>(), vec![2]);
        assert_eq!(b.not_elements(0b100).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(b.not_elements(0b101).collect::<Vec<_>>(), vec![1]);
        assert_eq!(b.not_elements(0b110).collect::<Vec<_>>(), vec![0]);
        assert_eq!(b.not_elements(0b111).collect::<Vec<_>>(), vec![]);

        let b = BitDp(4);
        assert_eq!(b.not_elements(0b0000).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        assert_eq!(b.not_elements(0b0001).collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(b.not_elements(0b0010).collect::<Vec<_>>(), vec![0, 2, 3]);
        assert_eq!(b.not_elements(0b0011).collect::<Vec<_>>(), vec![2, 3]);
        assert_eq!(b.not_elements(0b0100).collect::<Vec<_>>(), vec![0, 1, 3]);
        assert_eq!(b.not_elements(0b0101).collect::<Vec<_>>(), vec![1, 3]);
        assert_eq!(b.not_elements(0b0110).collect::<Vec<_>>(), vec![0, 3]);
        assert_eq!(b.not_elements(0b0111).collect::<Vec<_>>(), vec![3]);
        assert_eq!(b.not_elements(0b1000).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(b.not_elements(0b1001).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(b.not_elements(0b1010).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(b.not_elements(0b1011).collect::<Vec<_>>(), vec![2]);
        assert_eq!(b.not_elements(0b1100).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(b.not_elements(0b1101).collect::<Vec<_>>(), vec![1]);
        assert_eq!(b.not_elements(0b1110).collect::<Vec<_>>(), vec![0]);
        assert_eq!(b.not_elements(0b1111).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_subsets() {
        for mask in 0..1 << 12 {
            let mut subsets = BitDp::subsets(mask).collect::<Vec<_>>();
            let n = subsets.len();
            assert_eq!(n, 1 << mask.count_ones());
            assert!(subsets.iter().all(|&s| BitDp::is_subset(mask, s)));
            subsets.sort_unstable();
            subsets.dedup();
            assert_eq!(n, subsets.len());
        }
    }

    #[test]
    fn test_combinations() {
        let mut comb = vec![vec![0; 14]; 14];
        comb[0][0] = 1;
        for i in 0..=12 {
            for j in 0..=12 {
                comb[i + 1][j] += comb[i][j];
                comb[i][j + 1] += comb[i][j];
            }
        }

        for n in 0..=12 {
            let b = BitDp(n);
            for k in 0..=n {
                let mut combinations = b.combinations(k).collect::<Vec<_>>();
                let len = combinations.len();
                assert_eq!(len, comb[n - k][k]);
                assert!(combinations.iter().all(|&s| s.count_ones() as usize == k));
                combinations.sort_unstable();
                combinations.dedup();
                assert_eq!(len, combinations.len());
            }
        }
    }
}
