//! Stage 4 — Harmonic numbers, exactly and asymptotically (§1.2.7).
//!
//! Implement `harmonic` (exact reduced fraction) and `harmonic_f64` in
//! src/lab.rs. The lesson: course/module-02-math/README.md, part 4.

use lab_02_math::{harmonic, harmonic_f64};

/// Euler's constant gamma, to f64 precision (§1.2.7, Eq. (3)).
const GAMMA: f64 = 0.5772156649015329;

fn gcd(mut m: u128, mut n: u128) -> u128 {
    while n != 0 {
        let r = m % n;
        m = n;
        n = r;
    }
    m
}

#[test]
fn small_values_from_the_text() {
    // H_1 = 1, H_2 = 3/2, H_3 = 11/6, H_4 = 25/12, H_5 = 137/60, H_6 = 49/20.
    assert_eq!(harmonic(1), (1, 1));
    assert_eq!(harmonic(2), (3, 2));
    assert_eq!(harmonic(3), (11, 6));
    assert_eq!(harmonic(4), (25, 12));
    assert_eq!(harmonic(5), (137, 60));
    assert_eq!(harmonic(6), (49, 20));
}

#[test]
fn fractions_are_reduced() {
    for n in 1..=30u32 {
        let (p, q) = harmonic(n);
        assert!(q >= 1, "H_{n} has denominator {q}");
        assert_eq!(gcd(p, q), 1, "H_{n} = {p}/{q} is not in lowest terms");
        // H_n > 1 for n > 1, and H_n < n: crude sanity bounds.
        if n > 1 {
            assert!(p > q, "H_{n} = {p}/{q} should exceed 1");
        }
        assert!(p <= q * n as u128, "H_{n} = {p}/{q} should be at most n");
    }
}

#[test]
fn recurrence_h_n_minus_h_n_minus_1_is_1_over_n() {
    // H_n = H_{n-1} + 1/n, checked exactly by cross-multiplication:
    //   n * (p_n q_{n-1} - p_{n-1} q_n) = q_n q_{n-1}.
    for n in 2..=30u32 {
        let (p1, q1) = harmonic(n - 1);
        let (p, q) = harmonic(n);
        assert_eq!(
            n as u128 * (p * q1 - p1 * q),
            q * q1,
            "H_{n} - H_{} != 1/{n}",
            n - 1
        );
    }
}

#[test]
fn h_30_denominator_is_the_known_one() {
    // The reduced denominator of H_30 divides lcm(1..30) and equals
    // 2329089562800; a wrong reduction strategy shows up here immediately.
    let (p, q) = harmonic(30);
    assert_eq!(q, 2329089562800u128);
    assert_eq!(p, 9304682830147u128);
}

#[test]
fn float_agrees_with_exact() {
    for n in 1..=30u32 {
        let (p, q) = harmonic(n);
        let exact = p as f64 / q as f64;
        let approx = harmonic_f64(n as u64);
        assert!(
            (exact - approx).abs() < 1e-12,
            "H_{n}: exact {exact} vs float {approx}"
        );
    }
}

#[test]
fn asymptotic_ln_n_plus_gamma() {
    // §1.2.7, Eq. (3): H_n = ln n + gamma + 1/(2n) - 1/(12n^2) + ...
    // So the error of the two-term approximation ln n + gamma is positive,
    // below 1/(2n), and in fact within about 1/(12n^2) of 1/(2n).
    for &n in &[100u64, 1_000, 10_000, 100_000] {
        let d = harmonic_f64(n) - (n as f64).ln() - GAMMA;
        let half = 1.0 / (2.0 * n as f64);
        assert!(d > 0.0, "H_{n} should exceed ln n + gamma");
        assert!(d < half + 1e-9, "H_{n} - ln n - gamma = {d} exceeds 1/(2n) = {half}");
        assert!(
            (d - half).abs() < 1.0 / (6.0 * (n * n) as f64) + 1e-9,
            "H_{n}: correction {d} too far from 1/(2n) = {half}"
        );
    }
}

#[test]
fn logarithmic_growth_bounds() {
    // The bisection argument of §1.2.7: grouping terms in blocks of powers
    // of two gives  1 + k/2  <=  H_{2^k}  <=  1 + k.  Harmonic numbers grow
    // without bound — but only logarithmically.
    for k in 0..=20u32 {
        let h = harmonic_f64(1u64 << k);
        assert!(h >= 1.0 + k as f64 / 2.0 - 1e-9, "H_2^{k} = {h} below 1 + k/2");
        assert!(h <= 1.0 + k as f64 + 1e-9, "H_2^{k} = {h} above 1 + k");
    }
}

#[test]
#[should_panic(expected = "n >= 1")]
fn h_zero_is_rejected() {
    // Knuth's harmonic numbers start at H_1; make the domain explicit.
    harmonic(0);
}
