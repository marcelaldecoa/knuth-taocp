//! Stage 2 — The two-dimensional spectral test, exactly.
//!
//! nu_2^2 = min{ u1^2 + u2^2 : u1 + a·u2 ≡ 0 (mod m), u ≠ 0 }, computed
//! EXACTLY by Gauss–Lagrange reduction on the dual basis (m, 0), (-a, 1).
//! The lesson proves the reduction terminates and that the vector it
//! returns is truly shortest; these tests hold you to "exact" by
//! cross-checking against brute force.

use lab_12_spectral::nu2_squared;

/// Brute-force nu_2^2, justified by Hermite's theorem: every rank-2 lattice
/// of determinant d has a nonzero vector of squared length <= γ_2·d with
/// γ_2 = 2/√3, and our dual lattice has determinant m. So
///     nu_2^2 <= (2/√3)·m,
/// and any dual vector with |u2| > sqrt((2/√3)·m) is already longer than
/// that bound — scanning |u2| <= B = ceil(sqrt((2/√3)·m)) is exhaustive.
/// For each u2 > 0 the shortest u1 with u1 ≡ -a·u2 (mod m) is the centered
/// residue (|u1| <= m/2); u2 = 0 forces u1 = ±m; negating u changes nothing.
fn nu2_squared_brute(a: i64, m: i64) -> i128 {
    let (a, m) = (a as i128, m as i128);
    let b = ((2.0 / 3.0f64.sqrt()) * m as f64).sqrt().ceil() as i128;
    let mut best = m * m; // the class u2 = 0: shortest nonzero member (m, 0)
    for u2 in 1..=b {
        let r = (a * u2).rem_euclid(m);
        let u1 = if 2 * r > m { m - r } else { -r };
        best = best.min(u1 * u1 + u2 * u2);
    }
    best
}

#[test]
fn hand_traced_example_from_the_lesson() {
    // a = 137, m = 256. The lesson's Gauss–Lagrange trace:
    //   (256,0), (-137,1) -> q=-2 -> (-18,2) -> q=8 -> (7,-15) -> q=-1 stop.
    // Shortest dual vector (7, -15): nu_2^2 = 49 + 225 = 274.
    // (Check the duality by hand: -7 + 137·15 = 2048 = 8·256.)
    assert_eq!(nu2_squared(137, 256), 274);
}

#[test]
fn derived_a_second_way() {
    // Same value from the Hermite-bounded exhaustive search — two
    // independent derivations of the frozen constant 274.
    assert_eq!(nu2_squared_brute(137, 256), 274);
    assert_eq!(nu2_squared(137, 256), nu2_squared_brute(137, 256));
}

#[test]
fn reduction_matches_brute_force_for_all_small_generators() {
    for m in 2i64..=80 {
        for a in 1..m {
            assert_eq!(
                nu2_squared(a, m),
                nu2_squared_brute(a, m),
                "nu2^2 mismatch at a = {a}, m = {m}"
            );
        }
    }
}

#[test]
fn reduction_matches_brute_force_on_lcg_generated_sample() {
    // Deterministic pseudo-random (a, m) pairs from the course's hand-rolled
    // LCG — no external crates, same data on every run.
    let mut x: u64 = 20260702;
    let mut next = || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        x >> 33
    };
    for _ in 0..60 {
        let m = (next() % 2_000_000 + 2) as i64;
        let a = (next() % (m as u64 - 1) + 1) as i64;
        assert_eq!(
            nu2_squared(a, m),
            nu2_squared_brute(a, m),
            "nu2^2 mismatch at a = {a}, m = {m}"
        );
    }
}

#[test]
fn known_structures_are_detected() {
    // a = 1: consecutive pairs (x, x) sit on the diagonal; the dual vector
    // (1, -1) has length^2 = 2 — the spectral test sees it instantly.
    assert_eq!(nu2_squared(1, 1 << 20), 2);
    // a = m - 1: pairs alternate on an anti-diagonal; (1, 1) is dual.
    assert_eq!(nu2_squared((1 << 20) - 1, 1 << 20), 2);
    // Minimal standard a = 16807, m = 2^31 - 1: a^2 < m, so the vector
    // (-a, 1) can't be beaten (any |u2| >= 2 wraps u1 far from 0) —
    // nu_2^2 = a^2 + 1. Mediocre-but-not-broken in two dimensions.
    assert_eq!(nu2_squared(16807, (1 << 31) - 1), 16807i128 * 16807 + 1);
}

#[test]
fn respects_the_hermite_bound() {
    // Sanity net: nu_2^2 <= (2/√3)·m for every generator we try.
    for m in [64i64, 101, 256, 1 << 16, 1_000_003] {
        for a in [1i64, 2, 3, 5, 17, 33] {
            if a < m {
                let bound = 2.0 / 3.0f64.sqrt() * m as f64;
                let got = nu2_squared(a, m) as f64;
                assert!(got <= bound + 1e-6, "a={a}, m={m}: {got} > {bound}");
            }
        }
    }
}
