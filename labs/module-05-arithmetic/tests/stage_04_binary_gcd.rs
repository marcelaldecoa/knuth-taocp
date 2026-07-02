//! Stage 4 — The binary gcd algorithm (Algorithm 4.5.2B).
//!
//! Implement `binary_gcd` in src/lab.rs.
//! Lesson: course/module-05-arithmetic/README.md.

use lab_05_arithmetic::binary_gcd;

/// Reference oracle: Euclid's algorithm (module 01), with gcd(0, n) = n.
fn euclid(mut m: u64, mut n: u64) -> u64 {
    while n != 0 {
        let r = m % n;
        m = n;
        n = r;
    }
    m
}

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn knuth_worked_example() {
    // §4.5.2 traces Algorithm B on u = 40902, v = 24140. B1 strips the one
    // shared factor of 2 (k = 1), the subtract-and-shift loop grinds down
    // to u = v = 17, and the answer is 17 * 2^1 = 34.
    assert_eq!(binary_gcd(40902, 24140), 34);
    assert_eq!(binary_gcd(24140, 40902), 34);
}

#[test]
fn zero_operands() {
    // Convention: gcd(0, n) = gcd(n, 0) = n, and gcd(0, 0) = 0.
    for n in [0u64, 1, 2, 17, 1 << 40, u64::MAX] {
        assert_eq!(binary_gcd(0, n), n, "gcd(0, {n})");
        assert_eq!(binary_gcd(n, 0), n, "gcd({n}, 0)");
    }
}

#[test]
fn powers_of_two() {
    // Step B1 alone solves these: gcd(2^a, 2^b) = 2^min(a,b).
    for a in 0..=62u32 {
        for b in 0..=62u32 {
            assert_eq!(
                binary_gcd(1u64 << a, 1u64 << b),
                1u64 << a.min(b),
                "gcd(2^{a}, 2^{b})"
            );
        }
    }
    // Mixed: one power of two, one odd number.
    assert_eq!(binary_gcd(1 << 20, 3), 1);
    assert_eq!(binary_gcd(96, 36), 12); // 2^5*3 vs 2^2*3^2
}

#[test]
fn agrees_with_euclid_on_a_grid() {
    for u in 0..=120u64 {
        for v in 0..=120u64 {
            assert_eq!(binary_gcd(u, v), euclid(u, v), "gcd({u}, {v})");
        }
    }
}

#[test]
fn agrees_with_euclid_on_lcg_samples() {
    let mut s = 45272u64;
    for _ in 0..2000 {
        let u = lcg(&mut s);
        let v = lcg(&mut s);
        assert_eq!(binary_gcd(u, v), euclid(u, v), "gcd({u}, {v})");
        // Scale invariance under shared powers of 2 (that's step B1).
        let sh = lcg(&mut s) % 8;
        let (us, vs) = (u >> 8 << sh, v >> 8 << sh);
        assert_eq!(binary_gcd(us, vs), euclid(us, vs), "gcd({us}, {vs})");
    }
}

#[test]
fn gcd_contract_divides_and_is_greatest() {
    // Check the *definition*, not just agreement: d | u, d | v, and no
    // larger common divisor exists (via the Bezout-free bound: any common
    // divisor of u and v divides gcd computed by Euclid).
    let mut s = 123456789u64;
    for _ in 0..200 {
        let u = lcg(&mut s) >> 20;
        let v = lcg(&mut s) >> 20;
        let d = binary_gcd(u, v);
        assert!(d > 0);
        assert_eq!(u % d, 0, "{d} must divide {u}");
        assert_eq!(v % d, 0, "{d} must divide {v}");
        assert_eq!(euclid(u, v) % d, 0);
        assert_eq!(d % euclid(u, v), 0);
    }
}

#[test]
fn stein_worked_example() {
    // The classic Stein illustration: gcd(48, 18).
    //   B1: (48,18) -> (24,9), k=1.  Then 24 halves to 3 via t;
    //   |3 - 9| -> 6 -> 3; 3 = 3 stops. Answer 3 * 2 = 6.
    assert_eq!(binary_gcd(48, 18), 6);
    assert_eq!(binary_gcd(18, 48), 6);
    // And u = v (B6 hits t = 0 immediately after B1/B2).
    assert_eq!(binary_gcd(12, 12), 12);
    assert_eq!(binary_gcd(1, 1), 1);
}

#[test]
fn large_coprime_pair() {
    // Two Mersenne primes: gcd(2^61 - 1, 2^31 - 1) = 2^gcd(61,31) - 1 = 1.
    let m61 = (1u64 << 61) - 1;
    let m31 = (1u64 << 31) - 1;
    assert_eq!(binary_gcd(m61, m31), 1);
    // Near the top of u64: consecutive integers are always coprime.
    assert_eq!(binary_gcd(u64::MAX, u64::MAX - 1), 1);
    // And a large non-coprime pair for contrast: 3 * m61 vs 5 * m61.
    assert_eq!(binary_gcd(3 * m61, 5 * m61), m61);
}
