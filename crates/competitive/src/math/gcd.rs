#[cfg(verus_keep_ghost)]
use vstd::{
    arithmetic::{div_mod::*, power2::*},
    bits::*,
    math::abs,
    prelude::*,
    std_specs::bits::*,
};

#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures
        (result == 0) == (original_a == 0 && original_b == 0),
        forall|d: int| #![trigger original_a as int % d] d > 0 ==> ((original_a as int % d == 0 && original_b as int % d == 0) == (result as int % d == 0)),
))]
pub fn gcd_loop(original_a: u64, original_b: u64) -> u64 {
    let (mut a, mut b) = (original_a, original_b);
    #[cfg_attr(verus_keep_ghost, verus_spec(
        invariant
            (a == 0 && b == 0) == (original_a == 0 && original_b == 0),
            forall|d: int| #![trigger original_a as int % d] d > 0 ==> ((original_a as int % d == 0 && original_b as int % d == 0) == (a as int % d == 0 && b as int % d == 0)),
        decreases b,
    ))]
    while b != 0 {
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|d: int| #![trigger original_a as int % d] d > 0 implies
                ((original_a as int % d == 0 && original_b as int % d == 0)
                    == (b as int % d == 0 && (a as int % b as int) % d == 0)) by {
                assert((original_a as int % d == 0 && original_b as int % d == 0)
                    == (a as int % d == 0 && b as int % d == 0));
                lemma_fundamental_div_mod(a as int, b as int);
                if b as int % d == 0 {
                    vstd::arithmetic::mul::lemma_mul_basics(a as int / b as int);
                    lemma_small_mod(0, d as nat);
                    lemma_mul_mod_noop_left(b as int, a as int / b as int, d);
                    lemma_add_mod_noop(b as int * (a as int / b as int), a as int % b as int, d);
                    lemma_mod_twice(a as int % b as int, d);
                }
            };
        }
        a %= b;
        std::mem::swap(&mut a, &mut b);
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        #[verifier::proof_note("For all u64 inputs, Euclidean GCD returns zero exactly for (0, 0)")]
        assert((a == 0) == (original_a == 0 && original_b == 0));
        #[verifier::proof_note("For all u64 inputs and every positive integer d, d divides the result exactly when d divides both inputs")]
        assert(forall|d: int| #![trigger original_a as int % d] d > 0 ==>
            ((original_a as int % d == 0 && original_b as int % d == 0)
                == (a as int % d == 0)));
    }
    a
}

#[codesnip::entry]
/// binary gcd
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures
        (result == 0) == (original_a == 0 && original_b == 0),
        forall|d: int| #![trigger original_a as int % d] d > 0 ==>
            ((original_a as int % d == 0 && original_b as int % d == 0) == (result as int % d == 0)),
))]
pub fn gcd(original_a: u64, original_b: u64) -> u64 {
    let (mut a, mut b) = (original_a, original_b);
    if a == 0 {
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|d: int| d > 0 implies #[trigger] (0int % d) == 0 by { lemma_small_mod(0, d as nat); }
        }
        return b;
    }
    if b == 0 {
        #[cfg(verus_keep_ghost)]
        proof! {
            assert forall|d: int| d > 0 implies #[trigger] (0int % d) == 0 by { lemma_small_mod(0, d as nat); }
        }
        return a;
    }
    let u = a.trailing_zeros();
    let v = b.trailing_zeros();
    #[cfg(verus_keep_ghost)]
    proof! {
        axiom_u64_trailing_zeros(a);
        axiom_u64_trailing_zeros(b);
    }
    a >>= u;
    b >>= v;
    let k = u.min(v);
    #[cfg(verus_keep_ghost)]
    proof_decl! { let ghost scale = pow2(k as nat) as int; }
    #[cfg(verus_keep_ghost)]
    proof! {
        factor_twos(original_a, k);
        factor_twos(original_b, k);
        lemma_pow2_pos(k as nat);
        let a0 = original_a >> k;
        let b0 = original_b >> k;
        assert(a & 1 == 1 && b & 1 == 1);
        assert(a > 0 && b > 0) by(bit_vector) requires a & 1 == 1, b & 1 == 1;
        assert(a <= a0 && b <= b0) by(bit_vector)
            requires a == original_a >> u, b == original_b >> v, k <= u < 64, k <= v < 64,
                a0 == original_a >> k, b0 == original_b >> k;
        assert((a as int) * scale <= original_a && (b as int) * scale <= original_b) by(nonlinear_arith)
            requires a <= a0, b <= b0, scale > 0,
                original_a == (a0 as int) * scale, original_b == (b0 as int) * scale;
        assert forall|d: int| #![trigger original_a as int % d] d > 0 implies
            ((original_a as int % d == 0 && original_b as int % d == 0)
                == ((a as int) * scale % d == 0 && (b as int) * scale % d == 0)) by {
            if u >= v {
                assert(k == v && b0 == b);
                let shift = (u - k) as u32;
                assert forall|j: u32| j < shift implies #[trigger] ((a0 >> j) & 1) == 0 by {
                    assert((original_a >> (k as u64 + j as u64)) & 1 == 0);
                    assert(((a0 >> j) & 1) == ((original_a >> (k as u64 + j as u64)) & 1)) by(bit_vector)
                        requires a0 == original_a >> k, j < shift, shift == u - k, k <= u < 64;
                }
                strip_twos_common(b, a0, shift, scale, d);
                compose_shifts(original_a, k, u);
            } else {
                assert(k == u && a0 == a);
                let shift = (v - k) as u32;
                assert forall|j: u32| j < shift implies #[trigger] ((b0 >> j) & 1) == 0 by {
                    assert((original_b >> (k as u64 + j as u64)) & 1 == 0);
                    assert(((b0 >> j) & 1) == ((original_b >> (k as u64 + j as u64)) & 1)) by(bit_vector)
                        requires b0 == original_b >> k, j < shift, shift == v - k, k <= v < 64;
                }
                strip_twos_common(a, b0, shift, scale, d);
                compose_shifts(original_b, k, v);
            }
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(
        invariant
            original_a > 0, original_b > 0,
            a > 0, b > 0, a & 1 == 1, b & 1 == 1,
            k < 64, scale == pow2(k as nat), scale > 0,
            (a as int) * scale <= u64::MAX, (b as int) * scale <= u64::MAX,
            forall|d: int| #![trigger original_a as int % d] d > 0 ==>
                ((original_a as int % d == 0 && original_b as int % d == 0)
                    == ((a as int) * scale % d == 0 && (b as int) * scale % d == 0)),
        decreases a + b,
    ))]
    while a != b {
        if a < b {
            std::mem::swap(&mut a, &mut b);
        }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before_a = a; }
        a -= b;
        #[cfg(verus_keep_ghost)]
        proof! {
            axiom_u64_trailing_zeros(a);
            let shift = u64_trailing_zeros(a);
            assert forall|d: int| #![trigger original_a as int % d] d > 0 implies
                ((original_a as int % d == 0 && original_b as int % d == 0)
                    == (((a >> shift) as int) * scale % d == 0 && (b as int) * scale % d == 0)) by {
                assert((original_a as int % d == 0 && original_b as int % d == 0)
                    == ((before_a as int) * scale % d == 0 && (b as int) * scale % d == 0));
                assert((before_a as int) * scale == (a as int) * scale + (b as int) * scale)
                    by(nonlinear_arith) requires before_a == a + b;
                lemma_sub_mod_noop((before_a as int) * scale, (b as int) * scale, d);
                lemma_add_mod_noop((a as int) * scale, (b as int) * scale, d);
                lemma_small_mod(0, d as nat);
                lemma_mod_twice((a as int) * scale, d);
                assert(((before_a as int) * scale % d == 0 && (b as int) * scale % d == 0)
                    == ((a as int) * scale % d == 0 && (b as int) * scale % d == 0));
                assert forall|j: u32| j < shift implies #[trigger] ((a >> j) & 1) == 0 by {
                    assert((a >> j as u64) & 1 == 0);
                }
                strip_twos_common(b, a, shift, scale, d);
            }
            assert((a >> shift) & 1 == 1);
            assert(a >> shift > 0) by(bit_vector) requires (a >> shift) & 1 == 1;
            assert(a >> shift <= a) by(bit_vector);
            assert((a >> shift) as int * scale <= u64::MAX) by(nonlinear_arith)
                requires (before_a as int) * scale <= u64::MAX,
                    a < before_a, a >> shift <= a, scale > 0;
        }
        a >>= a.trailing_zeros();
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        lemma_u64_shl_is_mul(a, k as u64);
        assert((a as int) * scale > 0) by(nonlinear_arith) requires a > 0, scale > 0;
        #[verifier::proof_note("For all u64 inputs, binary GCD returns zero exactly for (0, 0), and every positive integer divides its result exactly when it divides both inputs")]
        assert(((a as int) * scale == 0) == (original_a == 0 && original_b == 0)
            && (forall|d: int| #![trigger original_a as int % d] d > 0 ==>
                ((original_a as int % d == 0 && original_b as int % d == 0)
                    == ((a as int) * scale % d == 0))));
    }
    a << k
}

#[codesnip::entry(include("gcd"))]
pub fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

// ax + by = gcd(a, b)
// a, b -> gcd(a, b), x, y
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    requires i64::MIN < a, i64::MIN < b,
    ensures
        i64::MIN < result.0,
        (result.0 == 0) == (a == 0 && b == 0),
        (a as int) * result.1 + (b as int) * result.2 == result.0,
        result.0 != 0 ==> (a as int) % (abs(result.0 as int) as int) == 0
            && (b as int) % (abs(result.0 as int) as int) == 0,
        abs(result.1 as int) <= vstd::math::max(abs(b as int) as int, 1),
        abs(result.2 as int) <= vstd::math::max(abs(a as int) as int, 1),
        a >= 0 && b >= 0 ==> result.0 >= 0,
        b == 0 ==> result == (a, 1i64, 0i64),
    decreases abs(b as int),
))]
pub fn extgcd_recurse(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        #[cfg(verus_keep_ghost)]
        proof! {
            vstd::arithmetic::mul::lemma_mul_basics(a as int);
            if a != 0 {
                let d = abs(a as int) as int;
                lemma_mod_self_0(d);
                lemma_small_mod(0, d as nat);
                if a < 0 {
                    lemma_mul_mod_noop_right(-1, d, d);
                    vstd::arithmetic::mul::lemma_mul_basics(d);
                }
            }
        }
        (a, 1, 0)
    } else {
        #[cfg(verus_keep_ghost)]
        proof! { signed_division(a as int, b as int); }
        let (g, x, y) = extgcd_recurse(b, a % b);
        #[cfg(verus_keep_ghost)]
        proof! {
            let q = rust_div(a as int, b as int);
            let r = rust_rem(a as int, b as int);
            let aa = abs(a as int) as int;
            let bb = abs(b as int) as int;
            let qq = abs(q) as int;
            let rr = abs(r) as int;
            lemma_fundamental_div_mod(aa, bb);
            assert(aa == qq * bb + rr && qq * bb <= aa) by(nonlinear_arith)
                requires aa == bb * (aa / bb) + aa % bb, qq == aa / bb, rr == aa % bb, rr >= 0;
            assert(-aa <= q * y <= aa) by(nonlinear_arith)
                requires qq >= 0, bb > 0, -bb <= y <= bb, qq * bb <= aa, qq == abs(q);
            if r != 0 {
                assert(-aa <= x - q * y <= aa) by(nonlinear_arith)
                    requires -rr <= x <= rr, -bb <= y <= bb, aa == qq * bb + rr,
                        qq == abs(q), bb > 0, rr >= 0;
            }
            #[verifier::proof_note("For every i64 input pair excluding i64::MIN, recursive extended GCD terminates and satisfies the exact integer Bezout identity without overflowing its i64 coefficients, including negative inputs")]
            assert((a as int) * y + (b as int) * (x - q * y) == g) by(nonlinear_arith)
                requires (b as int) * x + r * y == g, a == q * b + r;
            if g != 0 {
                let d = abs(g as int) as int;
                vstd::arithmetic::mul::lemma_mul_basics(q);
                vstd::arithmetic::mul::lemma_mul_is_commutative(b as int, q);
                lemma_small_mod(0, d as nat);
                lemma_mul_mod_noop_left(b as int, q, d);
                lemma_add_mod_noop((b as int) * q, r, d);
                lemma_mod_twice(r, d);
                #[verifier::proof_note("The absolute value of the nonzero returned gcd divides both inputs; together with the Bezout identity this characterizes their greatest common divisor")]
                assert((a as int) % d == 0 && (b as int) % d == 0);
            }
        }
        (g, y, x - (a / b) * y)
    }
}

#[codesnip::entry]
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    requires i64::MIN < original_a, i64::MIN < original_b,
    ensures
        i64::MIN < result.0,
        original_a >= 0 && original_b >= 0 ==> result.0 >= 0,
        (result.0 == 0) == (original_a == 0 && original_b == 0),
        (original_a as int) * (result.1 as int) + (original_b as int) * (result.2 as int) == result.0,
        result.0 != 0 ==> (original_a as int) % (abs(result.0 as int) as int) == 0
            && (original_b as int) % (abs(result.0 as int) as int) == 0,
        abs(result.1 as int) <= vstd::math::max(abs(original_b as int) as int, 1),
        abs(result.2 as int) <= vstd::math::max(abs(original_a as int) as int, 1),
))]
pub fn extgcd(original_a: i64, original_b: i64) -> (i64, i64, i64) {
    let (mut a, mut b) = (original_a, original_b);
    let (mut u, mut v, mut x, mut y) = (1, 0, 0, 1);
    #[cfg(verus_keep_ghost)]
    proof! { assert(abs(0) == 0 && abs(1) == 1) by(compute_only); }
    #[cfg_attr(verus_keep_ghost, verus_spec(
        invariant
            i64::MIN < a <= i64::MAX, i64::MIN < b <= i64::MAX,
            i64::MIN < original_a, i64::MIN < original_b,
            original_a >= 0 && original_b >= 0 ==> a >= 0 && b >= 0,
            (a == 0 && b == 0) == (original_a == 0 && original_b == 0),
            (original_a as int) * u + (original_b as int) * v == a,
            (original_a as int) * x + (original_b as int) * y == b,
            (a as int) * b * u * x <= 0, (a as int) * b * v * y <= 0,
            (abs(a as int) as int) * abs(x as int) + (abs(b as int) as int) * abs(u as int) == abs(original_b as int),
            (abs(a as int) as int) * abs(y as int) + (abs(b as int) as int) * abs(v as int) == abs(original_a as int),
            abs(u as int) <= vstd::math::max(abs(original_b as int) as int, 1),
            abs(x as int) <= vstd::math::max(abs(original_b as int) as int, 1),
            abs(v as int) <= vstd::math::max(abs(original_a as int) as int, 1),
            abs(y as int) <= vstd::math::max(abs(original_a as int) as int, 1),
            forall|d: int| #![trigger original_a as int % d] d > 0 ==>
                ((original_a as int % d == 0 && original_b as int % d == 0)
                    == (a as int % d == 0 && b as int % d == 0)),
        decreases abs(a as int),
    ))]
    while a != 0 {
        #[cfg(verus_keep_ghost)]
        proof! { signed_division(b as int, a as int); }
        let k = b / a;
        #[cfg(verus_keep_ghost)]
        proof! {
            signed_coefficient_step(a as int, b as int, u as int, x as int, abs(original_b as int) as int);
            signed_coefficient_step(a as int, b as int, v as int, y as int, abs(original_a as int) as int);
            let aa = abs(a as int) as int;
            let bb = abs(b as int) as int;
            let r = rust_rem(b as int, a as int);
            lemma_fundamental_div_mod(bb, aa);
            assert((bb / aa) * aa <= bb) by(nonlinear_arith)
                requires bb == aa * (bb / aa) + bb % aa, bb % aa >= 0;
            assert(-bb <= (k as int) * a <= bb) by(nonlinear_arith)
                requires abs(k as int) == bb / aa, (bb / aa) * aa <= bb, aa == abs(a as int), aa > 0;
            assert((original_a as int) * (x as int - (k as int) * u)
                + (original_b as int) * (y as int - (k as int) * v) == b as int - (k as int) * a)
                by(nonlinear_arith) requires
                    (original_a as int) * u + (original_b as int) * v == a,
                    (original_a as int) * x + (original_b as int) * y == b;
            assert forall|d: int| #![trigger original_a as int % d] d > 0 implies
                ((original_a as int % d == 0 && original_b as int % d == 0)
                    == (a as int % d == 0 && r % d == 0)) by {
                assert((original_a as int % d == 0 && original_b as int % d == 0)
                    == (a as int % d == 0 && b as int % d == 0));
                if a as int % d == 0 {
                    vstd::arithmetic::mul::lemma_mul_basics(k as int);
                    lemma_small_mod(0, d as nat);
                    lemma_mul_mod_noop_left(a as int, k as int, d);
                    lemma_add_mod_noop((a as int) * k, r, d);
                    lemma_mod_twice(r, d);
                }
            }
        }
        x -= k * u;
        y -= k * v;
        b -= k * a;
        std::mem::swap(&mut x, &mut u);
        std::mem::swap(&mut y, &mut v);
        std::mem::swap(&mut b, &mut a);
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        if b != 0 {
            let d = abs(b as int) as int;
            lemma_small_mod(0, d as nat);
            lemma_mod_self_0(d);
            if b < 0 {
                lemma_mul_mod_noop_right(-1, d, d);
                vstd::arithmetic::mul::lemma_mul_basics(d);
                vstd::arithmetic::mul::lemma_mul_basics(-1);
            }
            lemma_mod_bound(original_a as int, d);
            lemma_mod_bound(original_b as int, d);
            assert((original_a as int) % d == 0 && (original_b as int) % d == 0);
        }
        #[verifier::proof_note("For every i64 input pair excluding i64::MIN, iterative extended GCD terminates without integer overflow and returns a signed gcd with exact Bezout coefficients; its magnitude divides both inputs and the result is zero exactly for (0, 0)")]
        assert((b == 0) == (original_a == 0 && original_b == 0)
            && (original_a as int) * x + (original_b as int) * y == b
            && (b != 0 ==> (original_a as int) % (abs(b as int) as int) == 0
                && (original_b as int) % (abs(b as int) as int) == 0));
    }
    (b, x, y)
}

/// Requires nonnegative inputs.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    requires original_a >= 0, original_b >= 0,
    ensures
        result.0 >= 0,
        (result.0 == 0) == (original_a == 0 && original_b == 0),
        (original_a as int) * result.1 + (original_b as int) * result.2 == result.0,
        result.0 > 0 ==> original_a % result.0 == 0 && original_b % result.0 == 0,
))]
pub fn extgcd_binary(original_a: i64, original_b: i64) -> (i64, i64, i64) {
    if original_b == 0 {
        #[cfg(verus_keep_ghost)]
        proof! {
            vstd::arithmetic::mul::lemma_mul_basics(original_a as int);
            if original_a > 0 {
                lemma_mod_self_0(original_a as int);
                lemma_small_mod(0, original_a as nat);
            }
        }
        return (original_a, 1, 0);
    }
    if original_a == 0 {
        #[cfg(verus_keep_ghost)]
        proof! {
            vstd::arithmetic::mul::lemma_mul_basics(original_b as int);
            lemma_mod_self_0(original_b as int);
            lemma_small_mod(0, original_b as nat);
        }
        return (original_b, 0, 1);
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        assert((original_a | original_b) >= 0) by(bit_vector)
            requires original_a >= 0, original_b >= 0;
    }
    let k = ((original_a | original_b) as u64).trailing_zeros();
    #[cfg(verus_keep_ghost)]
    proof! {
        let joined = (original_a | original_b) as u64;
        let aa = original_a as u64;
        let bb = original_b as u64;
        axiom_u64_trailing_zeros(joined);
        axiom_u64_trailing_zeros(aa);
        axiom_u64_trailing_zeros(bb);
        assert(joined > 0) by(bit_vector)
            requires joined == (original_a | original_b) as u64, original_a > 0, original_b > 0;
        let ta = u64_trailing_zeros(aa);
        let tb = u64_trailing_zeros(bb);
        if k > ta {
            assert((joined >> (ta as u64)) & 1 == 0);
            assert(false) by(bit_vector)
                requires joined == aa | bb, (joined >> (ta as u64)) & 1 == 0, (aa >> ta) & 1 == 1, ta < 64;
        }
        if k > tb {
            assert((joined >> (tb as u64)) & 1 == 0);
            assert(false) by(bit_vector)
                requires joined == aa | bb, (joined >> (tb as u64)) & 1 == 0, (bb >> tb) & 1 == 1, tb < 64;
        }
        factor_twos(aa, k);
        factor_twos(bb, k);
        assert(((original_a >> k) as u64) == aa >> k && ((original_b >> k) as u64) == bb >> k)
            by(bit_vector) requires original_a >= 0, original_b >= 0, k < 64,
                aa == original_a as u64, bb == original_b as u64;
        assert(((aa >> k) | (bb >> k)) & 1 == 1) by(bit_vector)
            requires (joined >> k) & 1 == 1, joined == aa | bb, k < 64;
    }
    #[cfg(verus_keep_ghost)]
    proof_decl! { let ghost scale = pow2(k as nat) as int; }
    let (mut c, mut d) = ((original_a >> k) as u64, (original_b >> k) as u64);
    #[cfg(verus_keep_ghost)]
    proof! { assert(((c | d) & 1) == 1); }
    let swapped = d & 1 == 0;
    if swapped {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost cc = c; let ghost dd = d; }
        std::mem::swap(&mut c, &mut d);
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(((c | d) & 1) == 1) by(bit_vector)
                requires ((cc | dd) & 1) == 1, c == dd, d == cc;
        }
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        lemma_pow2_pos(k as nat);
        assert(d & 1 == 1) by(bit_vector)
            requires ((c | d) & 1) == 1, !swapped ==> d & 1 != 0, swapped ==> c & 1 == 0;
        assert(d % 2 == 1) by(bit_vector) requires d & 1 == 1;
        assert(c <= i64::MAX && d <= i64::MAX) by(nonlinear_arith)
            requires (c as int) * scale <= i64::MAX, (d as int) * scale <= i64::MAX, scale >= 1;
        lemma_mod_self_0(d as int);
        lemma_small_mod(0, d as nat);
        vstd::arithmetic::mul::lemma_mul_basics(c as int);
    }
    let (mut a, mut b) = (c, d);
    // c*u == a and c*s == b modulo the odd d, with 0 <= u,s <= d.
    let (mut u, mut s) = (1u64, 0u64);
    let shift = a.trailing_zeros();
    #[cfg(verus_keep_ghost)]
    proof! {
        factor_twos(c, shift);
        assert(c >> shift <= c) by(bit_vector);
        assert forall|h: int| #![trigger (original_a as int) % h] h > 0 implies
            (((original_a as int) % h == 0 && (original_b as int) % h == 0)
                == (((c >> shift) as int) * scale % h == 0 && (d as int) * scale % h == 0)) by {
            assert forall|j: u32| j < shift implies #[trigger] ((c >> j) & 1) == 0 by {
                assert((c >> (j as u64)) & 1 == 0);
            }
            strip_twos_common(d, c, shift, scale, h);
        }
        assert(c >> 0u32 == c) by(bit_vector);
    }
    a >>= shift;
    #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
        invariant
            c > 0, 0 < d <= i64::MAX, d % 2 == 1,
            shift == u64_trailing_zeros(c), shift < 64, a == c >> shift,
            u <= d,
            (c as int) * u % (d as int) == ((c >> (iter.index() as u32)) as int) % (d as int),
    ))]
    for _i in 0..shift {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before = u; }
        if u & 1 == 1 {
            u += d;
        }
        u /= 2;
        #[cfg(verus_keep_ghost)]
        proof! {
            let value = c >> _i;
            assert((c >> _i) & 1 == 0);
            assert(value % 2 == 0) by(bit_vector) requires value & 1 == 0;
            assert((before & 1 == 1) == (before % 2 == 1)) by(bit_vector);
            modular_halving(c as int, value as int, before as int, d as int);
            assert(value / 2 == c >> (_i + 1)) by(bit_vector)
                requires value == c >> _i, _i < 63;
        }
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        assert((a as int) * scale <= (c as int) * scale) by(nonlinear_arith)
            requires a <= c, scale > 0;
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(
        invariant
            original_a > 0, original_b > 0, k < 64, scale == pow2(k as nat), scale > 0,
            (c as int) * scale == (if swapped { original_b as int } else { original_a as int }),
            (d as int) * scale == (if swapped { original_a as int } else { original_b as int }),
            0 < c <= i64::MAX, 0 < d <= i64::MAX, d % 2 == 1,
            0 < a <= i64::MAX, 0 < b <= i64::MAX, a & 1 == 1, b & 1 == 1, u <= d, s <= d,
            (a as int) * scale <= vstd::math::max(original_a as int, original_b as int),
            (b as int) * scale <= vstd::math::max(original_a as int, original_b as int),
            (c as int) * u % (d as int) == (a as int) % (d as int),
            (c as int) * s % (d as int) == (b as int) % (d as int),
            forall|h: int| #![trigger (original_a as int) % h] h > 0 ==>
                (((original_a as int) % h == 0 && (original_b as int) % h == 0)
                    == ((a as int) * scale % h == 0 && (b as int) * scale % h == 0)),
        decreases a + b,
    ))]
    while a != b {
        if b < a {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut u, &mut s);
        }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before_b = b; let ghost before_s = s; }
        b -= a;
        if s < u {
            s += d;
        }
        s -= u;
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(s <= d);
            let adjust = if before_s < u { d as int } else { 0 };
            assert(s == before_s + adjust - u);
            lemma_mod_self_0(d as int);
            lemma_small_mod(0, d as nat);
            lemma_mul_mod_noop_right(c as int, adjust, d as int);
            vstd::arithmetic::mul::lemma_mul_basics(c as int);
            assert((c as int) * adjust % (d as int) == 0);
            lemma_add_mod_noop((c as int) * before_s, (c as int) * adjust, d as int);
            lemma_mod_twice((c as int) * before_s, d as int);
            lemma_sub_mod_noop((c as int) * (before_s + adjust), (c as int) * u, d as int);
            lemma_sub_mod_noop(before_b as int, a as int, d as int);
            assert((c as int) * s == (c as int) * (before_s + adjust) - (c as int) * u)
                by(nonlinear_arith) requires s == before_s + adjust - u;
            assert((c as int) * (before_s + adjust) == (c as int) * before_s + (c as int) * adjust)
                by(nonlinear_arith);
            assert((c as int) * s % (d as int) == (b as int) % (d as int));
            assert(b % 2 == 0) by(bit_vector)
                requires b == before_b - a, a & 1 == 1, before_b & 1 == 1, a < before_b;
            assert((b as int) * scale == (before_b as int) * scale - (a as int) * scale)
                by(nonlinear_arith) requires b == before_b - a;
            assert((b as int) * scale <= (before_b as int) * scale) by(nonlinear_arith)
                requires b <= before_b, scale > 0;
        }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before_shift = b; }
        let shift = b.trailing_zeros();
        #[cfg(verus_keep_ghost)]
        proof! {
            factor_twos(b, shift);
            assert((b >> shift) & 1 == 1);
            assert(b >> shift <= b) by(bit_vector);
            assert(((b >> shift) as int) * scale <= (b as int) * scale) by(nonlinear_arith)
                requires b >> shift <= b, scale > 0;
            assert forall|h: int| #![trigger (original_a as int) % h] h > 0 implies
                (((original_a as int) % h == 0 && (original_b as int) % h == 0)
                    == ((a as int) * scale % h == 0 && ((b >> shift) as int) * scale % h == 0)) by {
                assert forall|j: u32| j < shift implies #[trigger] ((b >> j) & 1) == 0 by {
                    assert((b >> (j as u64)) & 1 == 0);
                }
                strip_twos_common(a, b, shift, scale, h);
                lemma_sub_mod_noop((before_b as int) * scale, (a as int) * scale, h);
                lemma_add_mod_noop((b as int) * scale, (a as int) * scale, h);
                lemma_small_mod(0, h as nat);
                assert((b as int) * scale + (a as int) * scale == (before_b as int) * scale)
                    by(nonlinear_arith) requires b == before_b - a;
            }
        }
        b >>= shift;
        #[cfg(verus_keep_ghost)]
        proof! { assert(before_shift >> 0u32 == before_shift) by(bit_vector); }
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                0 < c <= i64::MAX, 0 < d <= i64::MAX, d % 2 == 1,
                before_shift > 0, shift == u64_trailing_zeros(before_shift), shift < 64,
                b == before_shift >> shift, s <= d,
                (c as int) * s % (d as int)
                    == ((before_shift >> (iter.index() as u32)) as int) % (d as int),
        ))]
        for _i in 0..shift {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost before = s; }
            if s & 1 == 1 {
                s += d;
            }
            s /= 2;
            #[cfg(verus_keep_ghost)]
            proof! {
                axiom_u64_trailing_zeros(before_shift);
                let value = before_shift >> _i;
                assert((before_shift >> _i) & 1 == 0);
                assert(value % 2 == 0) by(bit_vector) requires value & 1 == 0;
                assert((before & 1 == 1) == (before % 2 == 1)) by(bit_vector);
                modular_halving(c as int, value as int, before as int, d as int);
                assert(value / 2 == before_shift >> (_i + 1)) by(bit_vector)
                    requires value == before_shift >> _i, _i < 63;
            }
        }
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        assert(0 < (a as int) * scale <= i64::MAX) by(nonlinear_arith)
            requires a > 0, scale > 0,
                (a as int) * scale <= vstd::math::max(original_a as int, original_b as int),
                0 < original_a <= i64::MAX, 0 < original_b <= i64::MAX;
        lemma_mod_self_0((a as int) * scale);
        assert((original_a as int) % ((a as int) * scale) == 0
            && (original_b as int) % ((a as int) * scale) == 0);
        lemma_u64_mul_pow2_le_max_iff_max_shr(a, k as u64, u64::MAX);
        lemma_u64_shl_is_mul(a, k as u64);
        let product = (c as int) * s;
        assert(0 <= product <= i64::MAX as int * i64::MAX) by(nonlinear_arith)
            requires 0 < c <= i64::MAX, 0 <= s <= d <= i64::MAX, product == (c as int) * s;
        lemma_sub_mod_noop(a as int, product, d as int);
        lemma_small_mod(0, d as nat);
        assert((a as int - product) % (d as int) == 0);
        lemma_fundamental_div_mod(a as int - product, d as int);
        let difference = a as int - product;
        let quotient = difference / (d as int);
        assert(-i64::MAX as int <= quotient <= i64::MAX as int) by(nonlinear_arith)
            requires difference == (d as int) * quotient, difference == a as int - product,
                0 < a <= i64::MAX, 0 < c <= i64::MAX, 0 < d <= i64::MAX, 0 <= s <= d,
                product == (c as int) * s;
        assert(-difference == (d as int) * (-quotient)) by(nonlinear_arith)
            requires difference == (d as int) * quotient;
        lemma_div_multiples_vanish(-quotient, d as int);
        assert((-difference) / (d as int) == -quotient);
    }
    let x = s as i64;
    let y = ((a as i128 - c as i128 * s as i128) / d as i128) as i64;
    let g = (a << k) as i64;
    #[cfg(verus_keep_ghost)]
    proof! {
        assert((c as int) * x + (d as int) * y == a) by(nonlinear_arith)
            requires (a as int - (c as int) * s) == (d as int) * ((a as int - (c as int) * s) / (d as int)),
                x == s, y == (a as int - (c as int) * s) / (d as int);
        assert((c as int) * scale * x + (d as int) * scale * y == g) by(nonlinear_arith)
            requires (c as int) * x + (d as int) * y == a, g == (a as int) * scale;
        lemma_mod_self_0(g as int);
        assert((b as int) * scale == g);
        assert(((original_a as int) % (g as int) == 0 && (original_b as int) % (g as int) == 0)
            == ((a as int) * scale % (g as int) == 0 && (b as int) * scale % (g as int) == 0));
        #[verifier::proof_note("For all nonnegative i64 inputs, binary extended GCD terminates without overflow and returns a nonnegative common divisor with exact Bezout coefficients; the result is zero exactly for (0, 0)")]
        assert(g > 0 && original_a % g == 0 && original_b % g == 0
            && (if swapped { (original_a as int) * y + (original_b as int) * x == g }
                else { (original_a as int) * x + (original_b as int) * y == g }));
    }
    if swapped { (g, y, x) } else { (g, x, y) }
}

/// Requires 0 < a < m < 2^63 and gcd(a, m) == 1.
pub fn modinv_recurse(a: u64, m: u64) -> u64 {
    extgcd_recurse(a as i64, m as i64).1.rem_euclid(m as i64) as u64
}

#[codesnip::entry(include("extgcd"))]
/// Requires 0 < a < m < 2^63 and gcd(a, m) == 1.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    requires
        0 < a < m <= i64::MAX,
        forall|d: int| #![trigger (a as int) % d]
            d > 0 && (a as int) % d == 0 && (m as int) % d == 0 ==> d == 1,
    ensures result < m, (a as int) * (result as int) % (m as int) == 1,
))]
pub fn modinv(a: u64, m: u64) -> u64 {
    #[cfg(verus_keep_ghost)]
    proof_decl! { let ghost original_a = a as int; }
    let (mut a, mut b) = (a as i64, m as i64);
    let (mut u, mut x) = (1, 0);
    #[cfg(verus_keep_ghost)]
    proof! {
        lemma_small_mod(original_a as nat, m as nat);
        lemma_small_mod(0, m as nat);
        lemma_mod_self_0(m as int);
        vstd::arithmetic::mul::lemma_mul_basics(original_a);
        vstd::arithmetic::mul::lemma_mul_basics(m as int);
        assert(abs(0) == 0 && abs(1) == 1) by(compute_only);
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(
        invariant
            0 <= a < b <= m <= i64::MAX,
            1 < m,
            -m <= u <= m, -m < x < m,
            (u as int) * x <= 0,
            (a as int) * abs(x as int) + (b as int) * abs(u as int) == m,
            (original_a * u) % (m as int) == a,
            (original_a * x) % (m as int) == (b as int) % (m as int),
            forall|d: int| #![trigger (a as int) % d]
                d > 0 && (a as int) % d == 0 && (b as int) % d == 0 ==> d == 1,
        decreases a,
    ))]
    while a != 0 {
        let k = b / a;
        #[cfg(verus_keep_ghost)]
        proof! {
            let r = (b as int) % (a as int);
            let next = x as int - (k as int) * u;
            lemma_fundamental_div_mod(b as int, a as int);
            coefficient_step(a as int, b as int, u as int, x as int, m as int);
            assert(abs(u as int) < m) by(nonlinear_arith)
                requires (b as int) * abs(u as int) <= m,
                    b >= 2, abs(u as int) >= 0, m > 0;
            assert forall|d: int| #![trigger r % d]
                d > 0 && r % d == 0 && (a as int) % d == 0 implies d == 1 by {
                lemma_mul_mod_noop_left(a as int, k as int, d);
                lemma_add_mod_noop((a as int) * k, r, d);
                lemma_mod_twice(r, d);
                assert((b as int) % d == 0);
            }
            lemma_mul_mod_noop_right(k as int, original_a * u, m as int);
            lemma_sub_mod_noop(original_a * x, (k as int) * (original_a * u), m as int);
            lemma_sub_mod_noop(b as int, (k as int) * a, m as int);
            assert(original_a * next == original_a * x - (k as int) * (original_a * u))
                by(nonlinear_arith) requires next == x as int - (k as int) * u;
            lemma_small_mod(r as nat, m as nat);
            lemma_small_mod(a as nat, m as nat);
            assert((original_a * next) % (m as int) == r);
        }
        x -= k * u;
        b -= k * a;
        std::mem::swap(&mut x, &mut u);
        std::mem::swap(&mut b, &mut a);
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        lemma_small_mod(0, b as nat);
        lemma_mod_self_0(b as int);
        assert(b == 1);
        lemma_small_mod(0, m as nat);
        lemma_small_mod(1, m as nat);
        if x < 0 {
            lemma_add_mod_noop(original_a * x, original_a * m, m as int);
            lemma_mul_mod_noop_right(original_a, m as int, m as int);
            lemma_mod_self_0(m as int);
            assert(original_a * (x as int + m as int) == original_a * x + original_a * m)
                by(nonlinear_arith);
        }
        #[verifier::proof_note("For every 0 < a < m < 2^63 with coprime a and m, iterative modular inversion returns a multiplicative inverse in [0, m) without overflowing i64 intermediates")]
        assert({let result = if x < 0 { x as int + m as int } else { x as int };
            0 <= result < m && original_a * result % (m as int) == 1});
    }
    (if x < 0 { x + m as i64 } else { x }) as _
}

/// 0 < a < p < 2^63, gcd(a, p) == 1, p is odd
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    requires
        0 < original_a < p <= i64::MAX, p % 2 == 1,
        forall|d: int| #![trigger (original_a as int) % d]
            d > 0 && (original_a as int) % d == 0 && (p as int) % d == 0 ==> d == 1,
    ensures result < p, (original_a as int) * result % (p as int) == 1,
))]
pub fn modinv_extgcd_binary(original_a: u64, p: u64) -> u64 {
    let (mut a, mut b, mut u, mut s) = (original_a, p, 1, 0);
    let k = a.trailing_zeros();
    #[cfg(verus_keep_ghost)]
    proof! {
        factor_twos(original_a, k);
        assert(p & 1 == 1) by(bit_vector) requires p % 2 == 1;
        assert((original_a >> k) & 1 == 1);
        lemma_small_mod(0, p as nat);
        lemma_small_mod(original_a as nat, p as nat);
        assert forall|d: int| #![trigger ((original_a >> k) as int) % d]
            d > 0 && ((original_a >> k) as int) % d == 0 && (p as int) % d == 0
                implies d == 1 by {
            assert forall|j: u32| j < k implies #[trigger] ((original_a >> j) & 1) == 0 by {
                assert((original_a >> (j as u64)) & 1 == 0);
            }
            strip_twos_common(p, original_a, k, 1, d);
        }
    }
    a >>= k;
    #[cfg(verus_keep_ghost)]
    proof! {
        assert(original_a >> 0u32 == original_a) by(bit_vector);
        assert(original_a >> k <= original_a) by(bit_vector);
        vstd::arithmetic::mul::lemma_mul_basics(original_a as int);
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
        invariant
            0 < original_a < p <= i64::MAX, p % 2 == 1,
            k == u64_trailing_zeros(original_a), k < 64, a == original_a >> k,
            u < p,
            (original_a as int) * u % (p as int)
                == ((original_a >> (iter.index() as u32)) as int) % (p as int),
    ))]
    for _i in 0..k {
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before = u; }
        if u & 1 == 1 {
            u += p;
        }
        u /= 2;
        #[cfg(verus_keep_ghost)]
        proof! {
            let value = original_a >> _i;
            assert((original_a >> _i) & 1 == 0);
            assert(value % 2 == 0) by(bit_vector) requires value & 1 == 0;
            assert((before & 1 == 1) == (before % 2 == 1)) by(bit_vector);
            modular_halving(original_a as int, value as int, before as int, p as int);
            assert(value / 2 == original_a >> (_i + 1)) by(bit_vector)
                requires value == original_a >> _i, _i < 63;
        }
    }
    #[cfg_attr(verus_keep_ghost, verus_spec(
        invariant
            0 < original_a < p <= i64::MAX, p % 2 == 1,
            0 < a <= p, 0 < b <= p, a & 1 == 1, b & 1 == 1, u < p, s < p,
            (original_a as int) * u % (p as int) == (a as int) % (p as int),
            (original_a as int) * s % (p as int) == (b as int) % (p as int),
            forall|d: int| #![trigger (a as int) % d]
                d > 0 && (a as int) % d == 0 && (b as int) % d == 0 ==> d == 1,
        decreases a + b,
    ))]
    while a != b {
        if b < a {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut u, &mut s);
        }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before_b = b; let ghost before_s = s; }
        b -= a;
        if s < u {
            s += p;
        }
        s -= u;
        #[cfg(verus_keep_ghost)]
        proof! {
            assert(s < p);
            let adjust = if before_s < u { p as int } else { 0 };
            assert(s == before_s + adjust - u);
            lemma_mod_self_0(p as int);
            lemma_small_mod(0, p as nat);
            lemma_mul_mod_noop_right(original_a as int, adjust, p as int);
            vstd::arithmetic::mul::lemma_mul_basics(original_a as int);
            assert((original_a as int) * adjust % (p as int) == 0);
            lemma_add_mod_noop((original_a as int) * before_s, (original_a as int) * adjust, p as int);
            lemma_mod_twice((original_a as int) * before_s, p as int);
            lemma_sub_mod_noop((original_a as int) * (before_s + adjust), (original_a as int) * u, p as int);
            lemma_sub_mod_noop(before_b as int, a as int, p as int);
            assert((original_a as int) * s == (original_a as int) * (before_s + adjust) - (original_a as int) * u)
                by(nonlinear_arith) requires s == before_s + adjust - u;
            assert((original_a as int) * (before_s + adjust)
                == (original_a as int) * before_s + (original_a as int) * adjust) by(nonlinear_arith);
            assert((original_a as int) * s % (p as int) == (b as int) % (p as int));
            assert(b % 2 == 0) by(bit_vector)
                requires b == before_b - a, a & 1 == 1, before_b & 1 == 1, a < before_b;
        }
        #[cfg(verus_keep_ghost)]
        proof_decl! { let ghost before_shift = b; }
        let k = b.trailing_zeros();
        #[cfg(verus_keep_ghost)]
        proof! {
            factor_twos(b, k);
            assert((b >> k) & 1 == 1);
            assert(b >> k <= b) by(bit_vector);
            assert forall|d: int| #![trigger (a as int) % d]
                d > 0 && (a as int) % d == 0 && ((b >> k) as int) % d == 0 implies d == 1 by {
                assert forall|j: u32| j < k implies #[trigger] ((b >> j) & 1) == 0 by {
                    assert((b >> (j as u64)) & 1 == 0);
                }
                strip_twos_common(a, b, k, 1, d);
                lemma_add_mod_noop(a as int, b as int, d);
                lemma_small_mod(0, d as nat);
                assert((before_b as int) % d == 0);
            }
        }
        b >>= k;
        #[cfg(verus_keep_ghost)]
        proof! { assert(before_shift >> 0u32 == before_shift) by(bit_vector); }
        #[cfg_attr(verus_keep_ghost, verus_spec(iter =>
            invariant
                0 < original_a < p <= i64::MAX, p % 2 == 1,
                0 < before_shift, k == u64_trailing_zeros(before_shift), k < 64,
                b == before_shift >> k, s < p,
                (original_a as int) * s % (p as int)
                    == ((before_shift >> (iter.index() as u32)) as int) % (p as int),
        ))]
        for _i in 0..k {
            #[cfg(verus_keep_ghost)]
            proof_decl! { let ghost before = s; }
            if s & 1 == 1 {
                s += p;
            }
            s /= 2;
            #[cfg(verus_keep_ghost)]
            proof! {
                axiom_u64_trailing_zeros(before_shift);
                let value = before_shift >> _i;
                assert((before_shift >> _i) & 1 == 0);
                assert(value % 2 == 0) by(bit_vector) requires value & 1 == 0;
                assert((before & 1 == 1) == (before % 2 == 1)) by(bit_vector);
                modular_halving(original_a as int, value as int, before as int, p as int);
                assert(value / 2 == before_shift >> (_i + 1)) by(bit_vector)
                    requires value == before_shift >> _i, _i < 63;
            }
        }
    }
    #[cfg(verus_keep_ghost)]
    proof! {
        lemma_mod_self_0(a as int);
        assert(a == 1);
        lemma_small_mod(1, p as nat);
        #[verifier::proof_note("For every odd modulus 1 < p < 2^63 and coprime input 0 < a < p, binary modular inversion terminates without overflow and returns the inverse in [0, p)")]
        assert(s < p && (original_a as int) * s % (p as int) == 1);
    }
    s
}

#[cfg(verus_keep_ghost)]
verus! {
proof fn signed_division(a: int, b: int)
    requires i64::MIN < a <= i64::MAX, i64::MIN < b <= i64::MAX, b != 0,
    ensures
        i64::MIN < rust_div(a, b) <= i64::MAX,
        i64::MIN < rust_rem(a, b) <= i64::MAX,
        abs(rust_div(a, b)) == (abs(a) as int) / (abs(b) as int),
        abs(rust_rem(a, b)) == (abs(a) as int) % (abs(b) as int),
        a == rust_div(a, b) * b + rust_rem(a, b),
        a >= 0 ==> rust_rem(a, b) >= 0,
{
    let aa = abs(a) as int;
    let bb = abs(b) as int;
    lemma_div_pos_is_pos(aa, bb);
    lemma_div_nonincreasing(aa, bb);
    lemma_mod_bound(aa, bb);
    lemma_fundamental_div_mod(aa, bb);
    if b < 0 {
        assert(aa / (-bb) == -(aa / bb) && aa % (-bb) == aa % bb) by(nonlinear_arith)
            requires aa >= 0, bb > 0;
    }
    #[verifier::proof_note("For signed i64 operands excluding i64::MIN and a nonzero divisor, Rust's truncating quotient and remainder preserve a = q*b + r and have the unsigned quotient and remainder as their magnitudes")]
    assert(a == rust_div(a, b) * b + rust_rem(a, b)) by(nonlinear_arith)
        requires aa == abs(a), bb == abs(b), aa == (aa / bb) * bb + aa % bb,
            a != 0 || rust_div(a, b) == 0,
            a > 0 ==> rust_div(a, b) == (if b > 0 { aa / bb } else { -(aa / bb) }),
            a < 0 ==> rust_div(a, b) == (if b > 0 { -(aa / bb) } else { aa / bb }),
            rust_rem(a, b) == (if a < 0 { -(aa % bb) } else { aa % bb });
}

proof fn modular_halving(original: int, value: int, coefficient: int, modulus: int)
    requires
        modulus > 0, modulus % 2 == 1, value >= 0, value % 2 == 0,
        0 <= coefficient <= modulus, original * coefficient % modulus == value % modulus,
    ensures
        0 <= (coefficient + if coefficient % 2 == 1 { modulus } else { 0 }) / 2 <= modulus,
        coefficient < modulus ==> (coefficient + if coefficient % 2 == 1 { modulus } else { 0 }) / 2 < modulus,
        original * ((coefficient + if coefficient % 2 == 1 { modulus } else { 0 }) / 2) % modulus
            == (value / 2) % modulus,
{
    let adjust = if coefficient % 2 == 1 { modulus } else { 0 };
    let half = (coefficient + adjust) / 2;
    lemma_fundamental_div_mod(coefficient, 2);
    lemma_fundamental_div_mod(modulus, 2);
    lemma_fundamental_div_mod(value, 2);
    lemma_add_mod_noop(coefficient, adjust, 2);
    lemma_fundamental_div_mod(coefficient + adjust, 2);
    assert((coefficient + adjust) % 2 == 0);
    lemma_mod_self_0(modulus);
    lemma_small_mod(0, modulus as nat);
    assert(adjust % modulus == 0);
    lemma_mul_mod_noop_right(original, adjust, modulus);
    vstd::arithmetic::mul::lemma_mul_basics(original);
    lemma_add_mod_noop(original * coefficient, original * adjust, modulus);
    lemma_mod_twice(original * coefficient, modulus);
    lemma_mod_twice(value, modulus);
    assert(original * adjust % modulus == 0);
    assert((original * coefficient + original * adjust) % modulus == value % modulus);
    assert(original * (coefficient + adjust) == original * coefficient + original * adjust) by(nonlinear_arith);
    lemma_sub_mod_noop(original * (coefficient + adjust), value, modulus);
    assert((original * (coefficient + adjust) - value) % modulus == 0);
    let difference = original * half - value / 2;
    assert(2 * difference == original * (coefficient + adjust) - value) by(nonlinear_arith)
        requires coefficient + adjust == 2 * half, value == 2 * (value / 2),
            difference == original * half - value / 2;
    lemma_mul_mod_noop_left(2 * difference, modulus / 2 + 1, modulus);
    lemma_mul_mod_noop_right(difference, modulus, modulus);
    vstd::arithmetic::mul::lemma_mul_basics(modulus / 2 + 1);
    vstd::arithmetic::mul::lemma_mul_basics(difference);
    lemma_sub_mod_noop(2 * difference * (modulus / 2 + 1), difference * modulus, modulus);
    assert(2 * difference * (modulus / 2 + 1) - difference * modulus == difference)
        by(nonlinear_arith) requires modulus == 2 * (modulus / 2) + 1;
    assert(difference % modulus == 0);
    lemma_add_mod_noop(difference, value / 2, modulus);
    assert(difference + value / 2 == original * half);
    lemma_mod_twice(value / 2, modulus);
    #[verifier::proof_note("For a positive odd modulus, adding it to an odd coefficient before halving preserves the modular representation of an even value and keeps the coefficient in [0, modulus], or [0, modulus) when it started there")]
    assert(0 <= half <= modulus && (coefficient < modulus ==> half < modulus)
        && original * half % modulus == (value / 2) % modulus);
}

proof fn coefficient_step(a: int, b: int, u: int, x: int, limit: int)
    requires a > 0, b >= 0, limit >= 0, u * x <= 0, a * abs(x) + b * abs(u) == limit,
    ensures
        -limit <= (b / a) * u <= limit,
        -limit <= x - (b / a) * u <= limit,
        u * (x - (b / a) * u) <= 0, (x - (b / a) * u) * u <= 0,
        a * abs(x - (b / a) * u) + (b % a) * abs(u) == limit,
{
    let k = b / a;
    let r = b % a;
    let next = x - k * u;
    lemma_fundamental_div_mod(b, a);
    if u > 0 {
        assert(x <= 0) by(nonlinear_arith) requires u * x <= 0, u > 0;
    } else if u < 0 {
        assert(x >= 0) by(nonlinear_arith) requires u * x <= 0, u < 0;
    }
    assert(abs(next) == abs(x) + k * abs(u)) by(nonlinear_arith)
        requires next == x - k * u, k >= 0, (u > 0 ==> x <= 0), (u < 0 ==> x >= 0);
    assert(k * abs(u) <= limit) by(nonlinear_arith)
        requires b * abs(u) <= limit, 0 <= k <= b, abs(u) >= 0;
    assert(-limit <= k * u <= limit) by(nonlinear_arith)
        requires k * abs(u) <= limit, k >= 0;
    assert(a * abs(next) + r * abs(u) == limit) by(nonlinear_arith)
        requires a * abs(x) + b * abs(u) == limit, b == k * a + r,
            abs(next) == abs(x) + k * abs(u);
    assert(abs(next) <= limit) by(nonlinear_arith)
        requires a * abs(next) + r * abs(u) == limit, a >= 1, r >= 0,
            abs(next) >= 0, abs(u) >= 0;
    assert(u * next <= 0 && next * u <= 0) by(nonlinear_arith)
        requires next == x - k * u, u * x <= 0, k >= 0;
    #[verifier::proof_note("An extended-Euclidean step preserves opposite coefficient signs and the weighted absolute-value invariant, bounding both the product and updated coefficient by that invariant")]
    assert(-limit <= k * u <= limit && -limit <= next <= limit
        && a * abs(next) + r * abs(u) == limit && u * next <= 0);
}

proof fn signed_coefficient_step(a: int, b: int, u: int, x: int, limit: int)
    requires
        i64::MIN < a <= i64::MAX, i64::MIN < b <= i64::MAX, a != 0, limit >= 0,
        a * b * u * x <= 0, (abs(a) as int) * abs(x) + (abs(b) as int) * abs(u) == limit,
    ensures
        -limit <= rust_div(b, a) * u <= limit,
        -limit <= x - rust_div(b, a) * u <= limit,
        rust_rem(b, a) * a * (x - rust_div(b, a) * u) * u <= 0,
        (abs(rust_rem(b, a)) as int) * abs(u) + (abs(a) as int) * abs(x - rust_div(b, a) * u) == limit,
{
    signed_division(b, a);
    let aa = abs(a) as int;
    let bb = abs(b) as int;
    let k = rust_div(b, a);
    let kk = abs(k) as int;
    let r = rust_rem(b, a);
    let next = x - k * u;
    if a < 0 {
        assert(bb / (-aa) == -(bb / aa)) by(nonlinear_arith) requires bb >= 0, aa > 0;
    }
    if b == 0 {
        assert(abs(0) == 0) by(compute_only);
        vstd::arithmetic::mul::lemma_mul_basics(abs(u) as int);
        assert(abs(next) <= limit) by(nonlinear_arith)
            requires next == x, aa >= 1, aa * abs(x) == limit;
    } else {
        let direction = if (a > 0) == (b > 0) { 1int } else { -1int };
        let adjusted = direction * u;
        assert(adjusted * x <= 0) by(nonlinear_arith)
            requires a * b * u * x <= 0, a != 0, b != 0, adjusted == direction * u,
                direction == (if (a > 0) == (b > 0) { 1int } else { -1int });
        assert(abs(adjusted) == abs(u)) by(nonlinear_arith)
            requires adjusted == direction * u, direction == 1 || direction == -1;
        coefficient_step(aa, bb, adjusted, x, limit);
        assert(k == direction * kk && next == x - (bb / aa) * adjusted) by(nonlinear_arith)
            requires next == x - k * u, adjusted == direction * u, kk == bb / aa, kk == abs(k),
                direction == 1 || direction == -1, direction == 1 ==> k >= 0, direction == -1 ==> k <= 0;
    }
    assert(r * a * next * u <= 0) by(nonlinear_arith)
        requires next == x - k * u, a * b * u * x <= 0, a != 0,
            b > 0 ==> r >= 0, b < 0 ==> r <= 0, b == 0 ==> r == 0,
            (a > 0 && b >= 0 || a < 0 && b <= 0) ==> k >= 0,
            (a > 0 && b <= 0 || a < 0 && b >= 0) ==> k <= 0;
    #[verifier::proof_note("A signed extended-Euclidean step preserves the weighted absolute-value invariant and bounds the coefficient update by it for either sign of each non-minimum i64 operand")]
    assert(-limit <= k * u <= limit && -limit <= next <= limit
        && (abs(r) as int) * abs(u) + aa * abs(next) == limit);
}

proof fn factor_twos(value: u64, shift: u32)
    requires value > 0, shift <= u64_trailing_zeros(value),
    ensures
        shift < 64,
        value as int == (value >> shift) as int * pow2(shift as nat),
        value >> shift > 0,
{
    axiom_u64_trailing_zeros(value);
    let tz = u64_trailing_zeros(value);
    assert(value << ((64 - tz) as u64) == 0);
    assert((value >> shift) << shift == value) by(bit_vector)
        requires shift <= tz < 64, value << ((64 - tz) as u64) == 0;
    assert(value >> shift <= u64::MAX >> shift) by(bit_vector);
    lemma_u64_mul_pow2_le_max_iff_max_shr(value >> shift, shift as u64, u64::MAX);
    lemma_u64_shl_is_mul(value >> shift, shift as u64);
    lemma_pow2_pos(shift as nat);
    #[verifier::proof_note("For any nonzero u64 and any shift up to its trailing-zero count, right shift divides out exactly 2^shift and leaves a positive value")]
    assert(value as int == (value >> shift) as int * pow2(shift as nat) && value >> shift > 0);
}

proof fn halve_common(a: int, b: int, scale: int, d: int)
    requires a > 0, a % 2 == 1, b >= 0, b % 2 == 0, scale > 0, d > 0,
    ensures
        (a * scale % d == 0 && b * scale % d == 0)
            == (a * scale % d == 0 && (b / 2) * scale % d == 0),
{
    lemma_fundamental_div_mod(a, 2);
    lemma_fundamental_div_mod(b, 2);
    assert((a * scale) * (b / 2) - (b * scale) * (a / 2) == (b / 2) * scale)
        by(nonlinear_arith) requires a == 2 * (a / 2) + 1, b == 2 * (b / 2);
    assert(b * scale == 2 * ((b / 2) * scale)) by(nonlinear_arith)
        requires b == 2 * (b / 2);
    lemma_mul_mod_noop_left(a * scale, b / 2, d);
    lemma_mul_mod_noop_left(b * scale, a / 2, d);
    lemma_sub_mod_noop((a * scale) * (b / 2), (b * scale) * (a / 2), d);
    lemma_mul_mod_noop_right(2, (b / 2) * scale, d);
    vstd::arithmetic::mul::lemma_mul_basics(a / 2);
    vstd::arithmetic::mul::lemma_mul_basics(b / 2);
    lemma_small_mod(0, d as nat);
    #[verifier::proof_note("Halving the even member of an odd/even pair preserves every positive common divisor, including after a shared positive scaling")]
    assert((a * scale % d == 0 && b * scale % d == 0)
        == (a * scale % d == 0 && (b / 2) * scale % d == 0));
}

proof fn strip_twos_common(a: u64, b: u64, shift: u32, scale: int, d: int)
    requires
        a & 1 == 1, shift < 64, scale > 0, d > 0,
        forall|j: u32| j < shift ==> #[trigger] ((b >> j) & 1) == 0,
    ensures
        ((a as int) * scale % d == 0 && (b as int) * scale % d == 0)
            == ((a as int) * scale % d == 0 && ((b >> shift) as int) * scale % d == 0),
    decreases shift,
{
    if shift == 0 {
        assert(b >> shift == b) by(bit_vector) requires shift == 0;
    } else {
        assert((b >> 0u32) & 1 == 0);
        assert(a % 2 == 1 && b % 2 == 0 && a > 0) by(bit_vector)
            requires a & 1 == 1, (b >> 0u32) & 1 == 0;
        halve_common(a as int, b as int, scale, d);
        let half = b / 2;
        assert forall|j: u32| j < shift - 1 implies #[trigger] ((half >> j) & 1) == 0 by {
            assert((b >> (j + 1) as u32) & 1 == 0);
            assert((half >> j) & 1 == (b >> (j + 1) as u32) & 1) by(bit_vector)
                requires half == b / 2, j < 63;
        }
        strip_twos_common(a, half, (shift - 1) as u32, scale, d);
        assert((half >> (shift - 1) as u32) == b >> shift) by(bit_vector)
            requires half == b / 2, 0 < shift < 64;
    }
    #[verifier::proof_note("Removing any number of trailing zero bits from the even member of an odd/even pair preserves common divisibility, including after a shared positive scaling")]
    assert(((a as int) * scale % d == 0 && (b as int) * scale % d == 0)
        == ((a as int) * scale % d == 0 && ((b >> shift) as int) * scale % d == 0));
}
proof fn compose_shifts(value: u64, first: u32, last: u32)
    requires first <= last < 64,
    ensures (value >> first) >> (last - first) as u32 == value >> last,
{
    let delta = (last - first) as u32;
    lemma_u64_shr_is_div(value, first as u64);
    lemma_u64_shr_is_div(value, last as u64);
    lemma_u64_shr_is_div(value >> first, delta as u64);
    lemma_pow2_pos(first as nat);
    lemma_pow2_pos(delta as nat);
    lemma_pow2_adds(first as nat, delta as nat);
    lemma_div_denominator(value as int, pow2(first as nat) as int, pow2(delta as nat) as int);
    #[verifier::proof_note("Two u64 right shifts compose exactly when their total is below 64")]
    assert((value >> first) >> (last - first) as u32 == value >> last);
}
}

#[cfg(verus_keep_ghost)]
mod verification {
    use super::*;

    verus! {
    pub fn binary_gcd_matches_euclidean(a: u64, b: u64) {
        let actual = gcd(a, b);
        let expected = gcd_loop(a, b);
        proof {
            if actual > 0 {
                lemma_mod_self_0(actual as int);
                lemma_mod_self_0(expected as int);
                assert((a as int) % (actual as int) == 0 && (b as int) % (actual as int) == 0);
                assert((a as int) % (expected as int) == 0 && (b as int) % (expected as int) == 0);
                assert((actual as int) % (expected as int) == 0);
                assert((expected as int) % (actual as int) == 0);
                if actual < expected { lemma_small_mod(actual as nat, expected as nat); }
                if expected < actual { lemma_small_mod(expected as nat, actual as nat); }
            }
            #[verifier::proof_note("Binary GCD and Euclidean GCD return the same result for all u64 input pairs, including zeros and u64::MAX")]
            assert(actual == expected);
        }
    }

    pub fn inverse_boundaries(m: u64)
        requires 2 <= m <= i64::MAX,
    {
        assert forall|d: int| #![trigger 1int % d]
            d > 0 && 1int % d == 0 && (m as int) % d == 0 implies d == 1 by {
            if d > 1 { lemma_small_mod(1, d as nat); }
        }
        let first = modinv(1, m);
        proof {
            lemma_small_mod(first as nat, m as nat);
            vstd::arithmetic::mul::lemma_mul_basics(first as int);
            #[verifier::proof_note("For every modulus 2 <= m < 2^63, the inverse of 1 exists and is 1")]
            assert(first == 1);
        }

        assert forall|d: int| #![trigger (m as int - 1) % d]
            d > 0 && (m as int - 1) % d == 0 && (m as int) % d == 0 implies d == 1 by {
            lemma_sub_mod_noop(m as int, m as int - 1, d);
            lemma_small_mod(0, d as nat);
            if d > 1 { lemma_small_mod(1, d as nat); }
        }
        let last = modinv(m - 1, m);
        proof {
            lemma_small_mod(0, m as nat);
            assert(last > 0);
            assert((m as int - 1) * last == (m as int) * (last as int - 1) + (m as int - last))
                by(nonlinear_arith);
            lemma_fundamental_div_mod_converse(
                (m as int - 1) * last, m as int, last as int - 1, m as int - last);
            #[verifier::proof_note("For every modulus 2 <= m < 2^63, the inverse of m - 1 exists and is m - 1")]
            assert(last == m - 1);
        }
    }

    pub fn extended_gcd_matches_euclidean(a: i64, b: i64)
        requires a >= 0, b >= 0,
    {
        let (g, x, y) = extgcd_recurse(a, b);
        let expected = gcd_loop(a as u64, b as u64);
        proof {
            if g > 0 {
                lemma_mod_self_0(expected as int);
                assert((a as int) % (expected as int) == 0 && (b as int) % (expected as int) == 0);
                lemma_mul_mod_noop_left(a as int, x as int, expected as int);
                lemma_mul_mod_noop_left(b as int, y as int, expected as int);
                lemma_add_mod_noop((a as int) * x, (b as int) * y, expected as int);
                vstd::arithmetic::mul::lemma_mul_basics(x as int);
                vstd::arithmetic::mul::lemma_mul_basics(y as int);
                lemma_small_mod(0, expected as nat);
                assert((g as int) % (expected as int) == 0);
                assert((expected as int) % (g as int) == 0);
                if g < expected { lemma_small_mod(g as nat, expected as nat); }
                if expected < g { lemma_small_mod(expected as nat, g as nat); }
            }
            #[verifier::proof_note("For all nonnegative i64 inputs, recursive extended GCD and Euclidean GCD return the same greatest common divisor")]
            assert(g as int == expected as int);
        }
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        Xorshift,
        testutil::{integer_cases, integer_pairs},
    };
    const A: i64 = i64::MAX;

    #[test]
    fn test_gcd() {
        let mut rng = Xorshift::default();
        let pairs = integer_pairs(0..=u64::MAX).chain(
            rng.random_iter((0..=u64::MAX, 0..=u64::MAX))
                .take(3_000_000),
        );
        for (a, b) in (0..=30)
            .flat_map(|a| (0..=30).map(move |b| (a, b)))
            .chain(pairs)
        {
            assert_eq!(gcd_loop(a, b), gcd(a, b));
        }
    }

    #[test]
    fn test_extgcd() {
        let mut rng = Xorshift::default();
        for (a, b) in integer_pairs(-A..=A).chain(rng.random_iter((-A..=A, -A..=A)).take(1_400_000))
        {
            for (g, x, y) in [extgcd(a, b), extgcd_recurse(a, b)] {
                assert_eq!(
                    g.unsigned_abs(),
                    gcd_loop(a.unsigned_abs(), b.unsigned_abs())
                );
                assert_eq!(a as i128 * x as i128 + b as i128 * y as i128, g as i128);
            }
        }
    }

    #[test]
    fn test_extgcd_binary() {
        let mut rng = Xorshift::default();
        for (a, b) in integer_pairs(0..=A).chain(rng.random_iter((0..=A, 0..=A)).take(1_500_000)) {
            let (g, x, y) = extgcd_binary(a, b);
            assert_eq!(g as u64, gcd_loop(a as u64, b as u64));
            assert_eq!(a as i128 * x as i128 + b as i128 * y as i128, g as i128);
        }
    }

    #[test]
    fn test_modinv() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        let boundaries = integer_cases(2..=A as u64).flat_map(|m| [(1, m), (m - 1, m)]);
        let general = integer_cases(2..=A as u64)
            .chain(input_rng.random_iter(2..=A as u64).take(1_500_000))
            .map(|m| {
                let a = rng
                    .random_iter(1..m)
                    .find(|&a| gcd_loop(a, m) == 1)
                    .unwrap();
                (a, m)
            });
        for (a, m) in boundaries.chain(general) {
            for x in [modinv(a, m), modinv_recurse(a, m)] {
                assert!(x < m);
                assert_eq!(a as u128 * x as u128 % m as u128, 1);
            }
        }
    }

    #[test]
    fn test_modinv_extgcd_binary() {
        let mut rng = Xorshift::default();
        let mut input_rng = Xorshift::default();
        let boundaries = integer_cases(3..=A as u64)
            .filter(|&m| m & 1 == 1)
            .flat_map(|m| [(1, m), (m - 1, m)]);
        let general = integer_cases(3..=A as u64)
            .chain(input_rng.random_iter(3..=A as u64).take(2_500_000))
            .filter(|&m| m & 1 == 1)
            .map(|m| {
                let a = rng
                    .random_iter(1..m)
                    .find(|&a| gcd_loop(a, m) == 1)
                    .unwrap();
                (a, m)
            });
        for (a, m) in boundaries.chain(general) {
            let x = modinv_extgcd_binary(a, m);
            assert!(x < m);
            assert_eq!(a as u128 * x as u128 % m as u128, 1);
        }
    }
}
