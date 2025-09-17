use super::Unsigned;

/// Garner's algorithm with precomputation for fixed moduli.
pub struct Garner<T>
where
    T: Unsigned,
{
    moduli: Vec<T>,
    coeff: Vec<T>,
    inv: Vec<T>,
}

impl<T> Garner<T>
where
    T: Unsigned,
{
    pub fn new<M>(moduli: M, modulo: T) -> Option<Self>
    where
        M: IntoIterator<Item = T>,
    {
        if modulo == T::zero() {
            return None;
        }
        let moduli: Vec<_> = moduli.into_iter().collect();
        if moduli.iter().any(|&m| m.is_zero()) {
            return None;
        }
        let n = moduli.len();
        for i in 0..n {
            for j in 0..i {
                if moduli[i].gcd(moduli[j]) != T::one() {
                    return None;
                }
            }
        }
        Some(Self::new_unchecked(moduli, modulo))
    }

    pub fn new_unchecked<M>(moduli: M, modulo: T) -> Self
    where
        M: IntoIterator<Item = T>,
    {
        let mut moduli: Vec<_> = moduli.into_iter().collect();
        let n = moduli.len();
        moduli.push(modulo);
        let coeff_len = n * (n + 1) / 2;
        let mut coeff = Vec::with_capacity(coeff_len);
        let mut inv = Vec::with_capacity(n);
        let mut prefix = vec![T::one(); moduli.len()];
        for i in 0..n {
            let modulus = moduli[i];
            inv.push(prefix[i].mod_inv(modulus));
            for j in i + 1..=n {
                coeff.push(prefix[j]);
                prefix[j] = prefix[j].mod_mul(modulus, moduli[j]);
            }
        }
        Self { moduli, coeff, inv }
    }

    pub fn solve<B, I>(&self, residues: B) -> Option<T>
    where
        B: IntoIterator<Item = T, IntoIter = I>,
        I: ExactSizeIterator<Item = T>,
    {
        let residues = residues.into_iter();
        if residues.len() != self.inv.len() {
            return None;
        }
        let n = residues.len();
        let mut constants = vec![T::zero(); n + 1];
        let mut start = 0;
        for (((i, residue), &modulus), &inv) in residues
            .into_iter()
            .enumerate()
            .zip(&self.moduli)
            .zip(&self.inv)
        {
            debug_assert!(residue < modulus);
            let t = residue.mod_sub(constants[i], modulus).mod_mul(inv, modulus);
            let coeff = &self.coeff[start..start + n - i];
            start += n - i;
            for ((constant, &modulus), &coeff) in constants
                .iter_mut()
                .zip(&self.moduli)
                .skip(i + 1)
                .zip(coeff)
            {
                *constant = coeff.mod_mul(t, modulus).mod_add(*constant, modulus);
            }
        }
        Some(constants[n])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::solve_simultaneous_linear_congruence, tools::Xorshift};

    #[test]
    fn test_garner() {
        let mut rng = Xorshift::default();
        for _ in 0..200 {
            let mut mod_candidates = [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41];
            rng.shuffle(&mut mod_candidates);
            let len = rng.random(1..=mod_candidates.len());
            let moduli: Vec<_> = mod_candidates[..len].to_vec();
            let product: u64 = moduli.iter().copied().product();
            let final_mods: Vec<_> = rng.random_iter(2..=product).take(10).collect();
            for final_mod in final_mods {
                let solver = Garner::new(moduli.iter().copied(), final_mod).unwrap();
                for _ in 0..10 {
                    let residues: Vec<_> = moduli
                        .iter()
                        .map(|&modulus| rng.random(0..modulus))
                        .collect();
                    let value = solver.solve(residues.clone()).unwrap();
                    let pairs: Vec<_> = residues
                        .iter()
                        .zip(moduli.iter())
                        .map(|(&b, &modulus)| (b, modulus))
                        .collect();
                    let (expected, _) = solve_simultaneous_linear_congruence(
                        pairs.iter().copied().map(|(b, modulus)| (1u64, b, modulus)),
                    )
                    .unwrap();
                    assert_eq!(value, expected % final_mod);
                }
            }
        }
    }
}
