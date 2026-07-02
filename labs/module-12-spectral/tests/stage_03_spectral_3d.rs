//! Stage 3 — Short dual vectors in three dimensions.
//!
//! nu_3^2 = min{ u1^2 + u2^2 + u3^2 : u1 + a·u2 + a^2·u3 ≡ 0 (mod m), u ≠ 0 },
//! found by a *certified* bounded search: scan (u2, u3) in [-B, B]^2 with
//! the centered residue for u1, and grow B until B^2 >= best — at that point
//! any vector sticking out of the square is provably longer than the best
//! one found. This is the honest small cousin of Knuth's Algorithm 3.3.4S.

use lab_12_spectral::{is_dual_vector, nu3_squared_certified};

const RANDU_A: i64 = 65539;
const RANDU_M: i64 = 1 << 31;

/// Unbounded-by-construction brute force for tiny m: every class (u2, u3)
/// of the dual lattice has a representative with |u2|, |u3| <= m (classes
/// repeat mod m), and nu_3^2 <= m^2 always (witness (m, 0, 0)), so larger
/// coordinates can never win. Slow but obviously correct.
fn nu3_squared_brute_tiny(a: i64, m: i64) -> i128 {
    let (a, m) = (a as i128, m as i128);
    let a2 = a * a % m;
    let mut best = m * m;
    for u3 in -m..=m {
        for u2 in -m..=m {
            if (u2, u3) == (0, 0) {
                continue;
            }
            let r = (a * u2 + a2 * u3).rem_euclid(m);
            let u1 = if 2 * r > m { m - r } else { -r };
            best = best.min(u1 * u1 + u2 * u2 + u3 * u3);
        }
    }
    best
}

#[test]
fn randu_nu3_squared_is_118() {
    // The most famous failure in the history of random-number generation:
    // RANDU's shortest 3-D dual vector is (9, -6, 1), so
    //   nu_3^2 = 81 + 36 + 1 = 118,
    // and all 2^29 triples fall on ~|9|+|-6|+|1| = 16 parallel planes
    // (15 meet the cube). For a good generator with m = 2^31 we could have
    // had nu_3 ≈ 1400, i.e. on the order of a thousand plane families.
    assert_eq!(nu3_squared_certified(RANDU_A, RANDU_M), 118);
    assert!(is_dual_vector(&[9, -6, 1], RANDU_A, RANDU_M));
}

#[test]
fn minimal_standard_is_orders_of_magnitude_better() {
    // a = 16807, m = 2^31 - 1 (Lewis–Goodman–Miller). Computed by the
    // reference implementation and frozen: nu_3^2 = 408_197 (nu_3 ≈ 639).
    // Not stellar — but ~3460 times RANDU's 118.
    let got = nu3_squared_certified(16807, (1 << 31) - 1);
    assert_eq!(got, 408_197);
    assert!(got >= 100_000, "must beat RANDU by orders of magnitude");
    assert!(got > 1000 * 118);
}

#[test]
fn toy_generator_from_the_lesson() {
    // a = 137, m = 256: nu_3^2 = 30, witnessed by u = (2, 5, 1):
    // 2 + 137·5 + 137^2·1 = 2 + 685 + 18769 = 19456 = 76·256.
    assert_eq!(nu3_squared_certified(137, 256), 30);
    assert!(is_dual_vector(&[2, 5, 1], 137, 256));
}

#[test]
fn certified_search_agrees_with_full_brute_force_on_tiny_moduli() {
    for m in 2i64..=40 {
        for a in 1..m {
            assert_eq!(
                nu3_squared_certified(a, m),
                nu3_squared_brute_tiny(a, m),
                "nu3^2 mismatch at a = {a}, m = {m}"
            );
        }
    }
}

#[test]
fn result_is_witnessed_by_a_dual_vector() {
    // The certified minimum must be attained by an actual dual vector:
    // re-search a small window and demand a witness with that exact norm.
    for (a, m) in [(137i64, 256i64), (5, 64), (21, 100), (65539, 1 << 31)] {
        let nu3sq = nu3_squared_certified(a, m);
        let (a128, m128) = (a as i128, m as i128);
        let a2 = a128 * a128 % m128;
        let b = ((nu3sq as f64).sqrt().ceil() as i128).max(1);
        let mut found = false;
        'outer: for u3 in -b..=b {
            for u2 in -b..=b {
                if (u2, u3) == (0, 0) {
                    continue;
                }
                let r = (a128 * u2 + a2 * u3).rem_euclid(m128);
                let u1 = if 2 * r > m128 { m128 - r } else { -r };
                if u1 * u1 + u2 * u2 + u3 * u3 == nu3sq {
                    assert!(is_dual_vector(&[u1 as i64, u2 as i64, u3 as i64], a, m));
                    found = true;
                    break 'outer;
                }
            }
        }
        assert!(found, "no dual vector of norm^2 = {nu3sq} for a={a}, m={m}");
    }
}

#[test]
fn power_of_two_moduli_with_small_odd_multipliers_are_terrible() {
    // Multipliers close to a small root of a quadratic mod m produce
    // RANDU-like collapses. a = 2^16 + 3 is bad for ANY power-of-two
    // modulus >= 2^25 or so: (9, -6, 1) stays dual because
    // a^2 - 6a + 9 = 2^32 ≡ 0 (mod 2^k) for k <= 32.
    for k in [26, 28, 30, 31] {
        let m = 1i64 << k;
        assert!(is_dual_vector(&[9, -6, 1], 65539, m));
        assert!(
            nu3_squared_certified(65539, m) <= 118,
            "a = 65539 must be RANDU-bad for m = 2^{k}"
        );
    }
}
