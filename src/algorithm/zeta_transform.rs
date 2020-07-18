//! fast zeta transform and fast mobius transform
//!
//! Convolution theorem
//! - min convolution: subset
//! - max convolution: superset
//! - gcd convolution: divisor
//! - lcm convolution: multiple

use crate::algebra::{Group, Monoid};

#[cargo_snippet::snippet("zeta_transform")]
pub fn zeta_transform_for_subset<M: Monoid>(v: &mut [M::T], monoid: M) {
    let n = v.len();
    let mut i = 1;
    while i < n {
        for j in 0..n {
            if j & i != 0 {
                v[j] = monoid.operate(&v[j], &v[j ^ i]);
            }
        }
        i <<= 1;
    }
}

#[test]
fn test_zeta_transform_for_subset() {
    use crate::algebra::AdditiveOperation;
    let mut f: Vec<u64> = vec![
        1, // 000
        2, // 001
        3, // 010
        4, // 011
        5, // 100
        6, // 101
        7, // 110
        8, // 111
    ];
    let g: Vec<u64> = vec![
        f[0b000],
        f[0b000] + f[0b001],
        f[0b000] + f[0b010],
        f[0b000] + f[0b001] + f[0b010] + f[0b011],
        f[0b000] + f[0b100],
        f[0b000] + f[0b001] + f[0b100] + f[0b101],
        f[0b000] + f[0b010] + f[0b100] + f[0b110],
        f[0b000] + f[0b001] + f[0b010] + f[0b011] + f[0b100] + f[0b101] + f[0b110] + f[0b111],
    ];
    zeta_transform_for_subset(&mut f, AdditiveOperation::new());
    assert_eq!(f, g);
}

#[cargo_snippet::snippet("zeta_transform")]
pub fn mobius_transform_for_subset<G: Group>(v: &mut [G::T], group: G) {
    let n = v.len();
    let mut i = 1;
    while i < n {
        for j in 0..n {
            if j & i != 0 {
                v[j] = group.operate(&v[j], &group.inverse(&v[j ^ i]));
            }
        }
        i <<= 1;
    }
}

#[test]
fn test_mobius_transform_for_subset() {
    use crate::algebra::AdditiveOperation;
    let f: Vec<i64> = vec![
        1, // 000
        2, // 001
        3, // 010
        4, // 011
        5, // 100
        6, // 101
        7, // 110
        8, // 111
    ];
    let mut g: Vec<i64> = vec![
        f[0b000],
        f[0b000] + f[0b001],
        f[0b000] + f[0b010],
        f[0b000] + f[0b001] + f[0b010] + f[0b011],
        f[0b000] + f[0b100],
        f[0b000] + f[0b001] + f[0b100] + f[0b101],
        f[0b000] + f[0b010] + f[0b100] + f[0b110],
        f[0b000] + f[0b001] + f[0b010] + f[0b011] + f[0b100] + f[0b101] + f[0b110] + f[0b111],
    ];
    mobius_transform_for_subset(&mut g, AdditiveOperation::new());
    assert_eq!(f, g);
}

#[cargo_snippet::snippet("zeta_transform")]
pub fn zeta_transform_for_superset<M: Monoid>(v: &mut [M::T], monoid: M) {
    let n = v.len();
    let mut i = 1;
    while i < n {
        for j in 0..n {
            if j & i == 0 {
                v[j] = monoid.operate(&v[j], &v[j | i]);
            }
        }
        i <<= 1;
    }
}

#[test]
fn test_zeta_transform_for_superset() {
    use crate::algebra::AdditiveOperation;
    let mut f: Vec<u64> = vec![
        1, // 000
        2, // 001
        3, // 010
        4, // 011
        5, // 100
        6, // 101
        7, // 110
        8, // 111
    ];
    let g: Vec<u64> = vec![
        f[0b000] + f[0b001] + f[0b010] + f[0b011] + f[0b100] + f[0b101] + f[0b110] + f[0b111],
        f[0b001] + f[0b011] + f[0b101] + f[0b111],
        f[0b010] + f[0b011] + f[0b110] + f[0b111],
        f[0b011] + f[0b111],
        f[0b100] + f[0b101] + f[0b110] + f[0b111],
        f[0b101] + f[0b111],
        f[0b110] + f[0b111],
        f[0b111],
    ];
    zeta_transform_for_superset(&mut f, AdditiveOperation::new());
    assert_eq!(f, g);
}

#[cargo_snippet::snippet("zeta_transform")]
pub fn mobius_transform_for_superset<G: Group>(v: &mut [G::T], group: G) {
    let n = v.len();
    let mut i = 1;
    while i < n {
        for j in 0..n {
            if j & i == 0 {
                v[j] = group.operate(&v[j], &group.inverse(&v[j | i]));
            }
        }
        i <<= 1;
    }
}

#[test]
fn test_mobius_transform_for_superset() {
    use crate::algebra::AdditiveOperation;
    let f: Vec<i64> = vec![
        1, // 000
        2, // 001
        3, // 010
        4, // 011
        5, // 100
        6, // 101
        7, // 110
        8, // 111
    ];
    let mut g: Vec<i64> = vec![
        f[0b000] + f[0b001] + f[0b010] + f[0b011] + f[0b100] + f[0b101] + f[0b110] + f[0b111],
        f[0b001] + f[0b011] + f[0b101] + f[0b111],
        f[0b010] + f[0b011] + f[0b110] + f[0b111],
        f[0b011] + f[0b111],
        f[0b100] + f[0b101] + f[0b110] + f[0b111],
        f[0b101] + f[0b111],
        f[0b110] + f[0b111],
        f[0b111],
    ];
    mobius_transform_for_superset(&mut g, AdditiveOperation::new());
    assert_eq!(f, g);
}

#[cargo_snippet::snippet("zeta_transform")]
pub fn zeta_transform_for_divisor<I: Iterator<Item = usize>, M: Monoid>(
    v: &mut [M::T],
    primes: I,
    monoid: M,
) {
    for p in primes {
        for (i, j) in (0..v.len()).step_by(p).enumerate() {
            v[j] = monoid.operate(&v[j], &v[i]);
        }
    }
}

#[test]
fn test_zeta_transform_for_divisor() {
    use crate::algebra::AdditiveOperation;
    let mut f = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let g = vec![
        0,
        f[1],
        f[1] + f[2],
        f[1] + f[3],
        f[1] + f[2] + f[4],
        f[1] + f[5],
        f[1] + f[2] + f[3] + f[6],
        f[1] + f[7],
        f[1] + f[2] + f[4] + f[8],
        f[1] + f[3] + f[9],
        f[1] + f[2] + f[5] + f[10],
    ];
    zeta_transform_for_divisor(
        &mut f,
        [2, 3, 5, 7].iter().cloned(),
        AdditiveOperation::new(),
    );
    assert_eq!(f, g);
}

#[cargo_snippet::snippet("zeta_transform")]
pub fn mobius_transform_for_divisor<I: Iterator<Item = usize>, G: Group>(
    v: &mut [G::T],
    primes: I,
    group: G,
) {
    for p in primes {
        for (i, j) in (0..v.len()).step_by(p).enumerate().rev() {
            v[j] = group.operate(&v[j], &group.inverse(&v[i]));
        }
    }
}

#[test]
fn test_mobius_transform_for_divisor() {
    use crate::algebra::AdditiveOperation;
    let f = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut g = vec![
        0,
        f[1],
        f[1] + f[2],
        f[1] + f[3],
        f[1] + f[2] + f[4],
        f[1] + f[5],
        f[1] + f[2] + f[3] + f[6],
        f[1] + f[7],
        f[1] + f[2] + f[4] + f[8],
        f[1] + f[3] + f[9],
        f[1] + f[2] + f[5] + f[10],
    ];
    mobius_transform_for_divisor(
        &mut g,
        [2, 3, 5, 7].iter().cloned(),
        AdditiveOperation::new(),
    );
    assert_eq!(f, g);
}

#[cargo_snippet::snippet("zeta_transform")]
pub fn zeta_transform_for_multiple<I: Iterator<Item = usize>, M: Monoid>(
    v: &mut [M::T],
    primes: I,
    monoid: M,
) {
    for p in primes {
        for (i, j) in (0..v.len()).step_by(p).enumerate().rev() {
            v[i] = monoid.operate(&v[i], &v[j]);
        }
    }
}

#[test]
fn test_zeta_transform_for_multiple() {
    use crate::algebra::AdditiveOperation;
    let mut f = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let g = vec![
        0,
        f[1] + f[2] + f[3] + f[4] + f[5] + f[6] + f[7] + f[8] + f[9] + f[10],
        f[2] + f[4] + f[6] + f[8] + f[10],
        f[3] + f[6] + f[9],
        f[4] + f[8],
        f[5] + f[10],
        f[6],
        f[7],
        f[8],
        f[9],
        f[10],
    ];
    zeta_transform_for_multiple(
        &mut f,
        [2, 3, 5, 7].iter().cloned(),
        AdditiveOperation::new(),
    );
    assert_eq!(f, g);
}

#[cargo_snippet::snippet("zeta_transform")]
pub fn mobius_transform_for_multiple<I: Iterator<Item = usize>, G: Group>(
    v: &mut [G::T],
    primes: I,
    group: G,
) {
    for p in primes {
        for (i, j) in (0..v.len()).step_by(p).enumerate() {
            v[i] = group.operate(&v[i], &group.inverse(&v[j]));
        }
    }
}

#[test]
fn test_mobius_transform_for_multiple() {
    use crate::algebra::AdditiveOperation;
    let f = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut g = vec![
        0,
        f[1] + f[2] + f[3] + f[4] + f[5] + f[6] + f[7] + f[8] + f[9] + f[10],
        f[2] + f[4] + f[6] + f[8] + f[10],
        f[3] + f[6] + f[9],
        f[4] + f[8],
        f[5] + f[10],
        f[6],
        f[7],
        f[8],
        f[9],
        f[10],
    ];
    mobius_transform_for_multiple(
        &mut g,
        [2, 3, 5, 7].iter().cloned(),
        AdditiveOperation::new(),
    );
    assert_eq!(f, g);
}
