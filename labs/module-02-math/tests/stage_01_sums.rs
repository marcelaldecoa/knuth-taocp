//! Stage 1 — Sums in closed form (§1.2.3).
//!
//! Implement `sum_first_n`, `sum_squares`, `sum_cubes`, `geometric_sum` in
//! src/lab.rs. The lesson: course/module-02-math/README.md, part 2.

use lab_02_math::{geometric_sum, sum_cubes, sum_first_n, sum_squares};

/// x^k by repeated multiplication — the "direct" side of each comparison.
fn ipow(x: i128, k: u32) -> i128 {
    let mut p = 1i128;
    for _ in 0..k {
        p *= x;
    }
    p
}

#[test]
fn gauss_schoolroom_example() {
    // The story goes that young Gauss summed 1..100 instantly: 5050.
    assert_eq!(sum_first_n(100), 5050);
    assert_eq!(sum_first_n(0), 0);
    assert_eq!(sum_first_n(1), 1);
}

#[test]
fn closed_forms_match_direct_summation() {
    for n in 0..=500u64 {
        let (mut s1, mut s2, mut s3) = (0u128, 0u128, 0u128);
        for k in 1..=n as u128 {
            s1 += k;
            s2 += k * k;
            s3 += k * k * k;
        }
        assert_eq!(sum_first_n(n), s1, "sum_first_n({n})");
        assert_eq!(sum_squares(n), s2, "sum_squares({n})");
        assert_eq!(sum_cubes(n), s3, "sum_cubes({n})");
    }
}

#[test]
fn closed_forms_hold_far_beyond_looping_range() {
    // At n = 10^12 a summation loop would take hours; the closed forms are
    // instant and still exact. Check them against their defining identities.
    for &n in &[1_000_000u64, 1_000_000_007, 1_000_000_000_000] {
        let big = n as u128;
        assert_eq!(2 * sum_first_n(n), big * (big + 1));
        assert_eq!(6 * sum_squares(n), big * (big + 1) * (2 * big + 1));
    }
    // sum_first_n must survive the extreme of its domain (u128 headroom).
    assert_eq!(2 * sum_first_n(u64::MAX), u64::MAX as u128 * (u64::MAX as u128 + 1));
}

#[test]
fn sum_of_cubes_is_square_of_sum() {
    // Nicomachus: (1 + 2 + ... + n)^2 = 1^3 + 2^3 + ... + n^3.
    for n in 0..=2000u64 {
        let t = sum_first_n(n);
        assert_eq!(sum_cubes(n), t * t, "n = {n}");
    }
    let t = sum_first_n(3_000_000_000);
    assert_eq!(sum_cubes(3_000_000_000), t * t);
}

#[test]
fn geometric_sum_matches_direct_summation() {
    for x in -6i128..=6 {
        for n in 0..=20u32 {
            let mut s = 0i128;
            for k in 0..=n {
                s += ipow(x, k);
            }
            assert_eq!(geometric_sum(x, n), s, "x = {x}, n = {n}");
        }
    }
}

#[test]
fn perturbation_identity() {
    // The §1.2.3 derivation itself: peel a term off each end of S_n,
    //   S_{n+1} = S_n + x^{n+1}   and   S_{n+1} = 1 + x * S_n,
    // so equating gives the closed form. Both peels must hold.
    for x in -5i128..=5 {
        for n in 0..=18u32 {
            let s = geometric_sum(x, n);
            let s1 = geometric_sum(x, n + 1);
            assert_eq!(s1, s + ipow(x, n + 1), "x = {x}, n = {n} (append)");
            assert_eq!(s1, 1 + x * s, "x = {x}, n = {n} (prepend)");
        }
    }
}

#[test]
fn geometric_closed_form_identity() {
    // (x - 1) * S_n = x^{n+1} - 1, the closed form cross-multiplied so it
    // also covers x = 1 (both sides 0).
    for x in -5i128..=5 {
        for n in 0..=20u32 {
            assert_eq!(
                (x - 1) * geometric_sum(x, n),
                ipow(x, n + 1) - 1,
                "x = {x}, n = {n}"
            );
        }
    }
}

#[test]
fn geometric_edge_cases() {
    // x = 1: n + 1 copies of 1 — the case the closed form's division misses.
    assert_eq!(geometric_sum(1, 0), 1);
    assert_eq!(geometric_sum(1, 41), 42);
    // x = 0: only the k = 0 term survives (0^0 = 1 by the empty-product rule).
    assert_eq!(geometric_sum(0, 7), 1);
    // x = -1: the sum telescopes to 1 or 0 by parity.
    assert_eq!(geometric_sum(-1, 10), 1);
    assert_eq!(geometric_sum(-1, 11), 0);
    // x = 2: Mersenne numbers, 1 + 2 + ... + 2^n = 2^{n+1} - 1.
    for n in 0..=100u32 {
        assert_eq!(geometric_sum(2, n), (1i128 << (n + 1)) - 1);
    }
}
