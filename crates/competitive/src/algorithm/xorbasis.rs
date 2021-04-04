/// Basis of xor operation.
#[derive(Debug, Clone)]
pub struct XorBasis {
    /// (reduced basis, coordinates, inserted basis)
    bases: Vec<(u64, u64, u64)>,
}
impl XorBasis {
    /// Create a empty space.
    pub fn new() -> Self {
        Default::default()
    }
    /// Return (reduced basis, coordinate).
    /// Coordinate means if i-th bit is 1, x was reduced by i-th inserted basis.
    pub fn reduce(&self, mut x: u64) -> (u64, u64) {
        let mut coord = 0u64;
        for (i, (u, c, _)) in self.bases.iter().enumerate() {
            if x > u ^ x {
                coord ^= c ^ 1 << i;
                x ^= u;
            }
        }
        (x, coord)
    }
    /// Return true if inserted element cannot be consisted by current basis and be added as a new basis.
    /// Return false if inserted element can be consisted by current basis.
    pub fn insert(&mut self, x: u64) -> bool {
        let (y, coord) = self.reduce(x);
        if y != 0 {
            self.bases.push((y, coord, x));
        }
        y != 0
    }
    /// Return coordinate if element can be consisted by current basis.
    pub fn find(&self, x: u64) -> Option<u64> {
        let (y, coord) = self.reduce(x);
        if y == 0 {
            Some(coord)
        } else {
            None
        }
    }
    /// Return coordinate if element can be consisted by current basis.
    pub fn basis(&self, x: u64) -> Option<Vec<u64>> {
        let (y, coord) = self.reduce(x);
        if y == 0 {
            Some(
                self.bases
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (_, _, b))| if coord & 1 << i != 0 { Some(*b) } else { None })
                    .collect(),
            )
        } else {
            None
        }
    }
}
impl Default for XorBasis {
    fn default() -> Self {
        XorBasis { bases: Vec::new() }
    }
}
impl std::iter::FromIterator<u64> for XorBasis {
    fn from_iter<T: IntoIterator<Item = u64>>(iter: T) -> Self {
        let mut basis = XorBasis::default();
        for x in iter {
            basis.insert(x);
        }
        basis
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rand, tools::Xorshift};

    #[test]
    fn test_xor_basis() {
        let mut rng = Xorshift::default();
        const Q: usize = 200;

        for _ in 0..Q {
            let mut basis = XorBasis::new();
            for x in rng.gen_iter(0u64..).take(Q) {
                if let Some(b) = basis.basis(x) {
                    assert_eq!(x, b.into_iter().fold(0, std::ops::BitXor::bitxor));
                }
                basis.insert(x);
            }
        }
    }

    fn consistables(b: &[u64]) -> std::collections::HashSet<u64> {
        assert!(b.len() <= 20);
        (0..1 << b.len())
            .map(|i| {
                b.iter()
                    .enumerate()
                    .map(|(j, &b)| if i & 1 << j != 0 { b } else { 0 })
                    .fold(0, std::ops::BitXor::bitxor)
            })
            .collect()
    }

    #[test]
    fn test_xor_basis_find() {
        let mut rng = Xorshift::default();
        const Q: usize = 100;
        const L: usize = 12;

        for _ in 0..Q {
            rand!(rng, k: (0usize..=L + 2), b: [0u64..1 << L; k]);
            let cons = consistables(&b);
            let basis: XorBasis = b.into_iter().collect();
            for x in rng.gen_iter(0u64..1 << L).take(Q) {
                assert_eq!(cons.contains(&x), basis.find(x).is_some());
            }
        }
    }
}
