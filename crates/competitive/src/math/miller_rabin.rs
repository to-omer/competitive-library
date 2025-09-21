use super::BarrettReduction;

macro_rules! impl_test_mr {
    ($name:ident, $ty:ty, $upty:ty) => {
        fn $name(n: $ty, br: &BarrettReduction<$upty>, a: $ty) -> bool {
            if br.rem(a as $upty) == 0 {
                return true;
            }
            let d = n - 1;
            let k = d.trailing_zeros();
            let mut d = d >> k;
            let mut y = {
                let mut a = a as $upty;
                let mut y: $upty = 1;
                while d > 0 {
                    if d & 1 == 1 {
                        y = br.rem(y * a);
                    }
                    a = br.rem(a * a);
                    d >>= 1;
                }
                y as $ty
            };
            if y == 1 || y == n - 1 {
                true
            } else {
                for _ in 0..k - 1 {
                    y = br.rem(y as $upty * y as $upty) as $ty;
                    if y == n - 1 {
                        return true;
                    }
                }
                false
            }
        }
    };
}
impl_test_mr!(test_mr32, u32, u64);
impl_test_mr!(test_mr64, u64, u128);

/// http://miller-rabin.appspot.com/
macro_rules! impl_mr {
    ($name:ident, $test:ident, $ty:ty, $upty:ty, [$($th:expr, [$($a:expr),+]),+], |$n:ident, $br:ident|$last:expr) => {
        fn $name($n: $ty, $br: &BarrettReduction<$upty>) -> bool {
            $(
                if $n >= $th {
                    return $($test($n, $br, $a))&&+
                }
            )+
            $last
        }
    };
}
impl_mr!(
    mr32,
    test_mr32,
    u32,
    u64,
    [316349281, [2, 7, 61], 49141, [11000544, 31481107]],
    |n, br| test_mr32(n, br, 921211727)
);
impl_mr!(
    mr64,
    test_mr64,
    u64,
    u128,
    [
        585226005592931977,
        [2, 325, 9375, 28178, 450775, 9780504, 1795265022],
        7999252175582851,
        [
            2,
            123635709730000,
            9233062284813009,
            43835965440333360,
            761179012939631437,
            1263739024124850375
        ],
        55245642489451,
        [
            2,
            4130806001517,
            149795463772692060,
            186635894390467037,
            3967304179347715805
        ],
        350269456337,
        [
            2,
            141889084524735,
            1199124725622454117,
            11096072698276303650
        ],
        1050535501,
        [
            4230279247111683200,
            14694767155120705706,
            16641139526367750375
        ]
    ],
    |n, br| mr32(n as u32, &BarrettReduction::<u64>::new(n))
);

pub fn miller_rabin_with_br(n: u64, br: &BarrettReduction<u128>) -> bool {
    if n % 2 == 0 {
        return n == 2;
    }
    if n % 3 == 0 {
        return n == 3;
    }
    if n % 5 == 0 {
        return n == 5;
    }
    if n % 7 == 0 {
        return n == 7;
    }
    if n < 121 { n > 2 } else { mr64(n, br) }
}

pub fn miller_rabin(n: u64) -> bool {
    miller_rabin_with_br(n, &BarrettReduction::<u128>::new(n as u128))
}

#[test]
fn test_miller_rabin() {
    const N: u32 = 1_000_000;
    let primes = super::PrimeTable::new(N);
    for i in 1..=N {
        assert_eq!(primes.is_prime(i), miller_rabin(i as _), "{}", i);
    }
    assert!(miller_rabin(1_000_000_007));
    assert!(!miller_rabin(1_000_000_011));
}
