//! Stage 4 — ν_t and μ_t for real generators, t <= 6 (Algorithm 3.3.4S).
//!
//! The assembled pipeline (dual pair -> reduce -> certified search) and
//! Knuth's figure of merit mu_t = pi^{t/2}·nu_t^t / (Γ(t/2+1)·m), with the
//! half-integer Gamma values taken exactly. Verdicts to reproduce: RANDU
//! is catastrophic in EVERY dimension t >= 3, while L'Ecuyer's/Park–Miller's
//! a = 48271 stays comfortably good all the way to t = 6.

use lab_16_spectral_hd::{mu_t, nu_t_squared};

const RANDU_A: i64 = 65539;
const RANDU_M: i64 = 1 << 31;
const MM: i64 = (1 << 31) - 1;

#[test]
fn consistency_with_module_12_pinned_values() {
    // Labs cannot depend on each other, so module 12's frozen constants
    // are restated here; nu_t_squared must reproduce every one of them.
    assert_eq!(nu_t_squared(RANDU_A, RANDU_M, 2), 2_147_221_514);
    assert_eq!(nu_t_squared(RANDU_A, RANDU_M, 3), 118);
    assert_eq!(nu_t_squared(16807, MM, 2), 16807i128 * 16807 + 1); // 282_475_250
    assert_eq!(nu_t_squared(16807, MM, 3), 408_197);
    assert_eq!(nu_t_squared(48271, MM, 2), 1_990_735_345);
    assert_eq!(nu_t_squared(48271, MM, 3), 1_433_881);
    assert_eq!(nu_t_squared(137, 256, 2), 274);
    assert_eq!(nu_t_squared(137, 256, 3), 30);
}

#[test]
fn randu_is_catastrophic_in_every_dimension_from_3_up() {
    // Padding (9, -6, 1) with zeros keeps it dual (multiply the congruence
    // by nothing at all), so nu_t^2 <= 118 for every t >= 3. At t = 4 the
    // polynomial (x-3)^2·(x+1) evaluated at a yields the slightly shorter
    // (9, 3, -5, 1), norm 116 — and nothing shorter ever appears.
    assert_eq!(nu_t_squared(RANDU_A, RANDU_M, 4), 116);
    assert_eq!(nu_t_squared(RANDU_A, RANDU_M, 5), 116);
    assert_eq!(nu_t_squared(RANDU_A, RANDU_M, 6), 116);
    for t in 3..=6 {
        assert!(nu_t_squared(RANDU_A, RANDU_M, t) <= 118, "t={t}");
    }
    // Merit: microscopic. (mu_t creeps upward with t only because the
    // ball-volume normalization weakens while nu_t stays frozen.)
    assert!(mu_t(RANDU_A, RANDU_M, 3) < 1e-4);
    assert!(mu_t(RANDU_A, RANDU_M, 4) < 1e-3);
    assert!(mu_t(RANDU_A, RANDU_M, 5) < 1e-2);
    assert!(mu_t(RANDU_A, RANDU_M, 6) < 1e-1);
}

#[test]
fn lecuyer_48271_is_good_in_all_dimensions() {
    // Knuth's Table 1 (§3.3.4) and L'Ecuyer's good-multiplier tables both
    // endorse a = 48271 for m = 2^31 - 1; C++'s std::minstd_rand ships it.
    // Frozen by the reference implementation:
    assert_eq!(nu_t_squared(48271, MM, 4), 47_418);
    assert_eq!(nu_t_squared(48271, MM, 5), 4_404);
    assert_eq!(nu_t_squared(48271, MM, 6), 1_402);
    for t in 2..=6 {
        let mu = mu_t(48271, MM, t);
        assert!(mu > 0.29, "mu_{t}(48271) = {mu} dips below 0.29");
    }
    // And the 1969 minimal standard for comparison — passable, never great:
    assert_eq!(nu_t_squared(16807, MM, 4), 21_682);
    assert_eq!(nu_t_squared(16807, MM, 5), 4_439);
    assert_eq!(nu_t_squared(16807, MM, 6), 895);
    for t in 2..=6 {
        let mu = mu_t(16807, MM, t);
        assert!(mu > 0.1, "the minimal standard passes Knuth's 0.1 rule at t={t}");
        assert!(
            mu_t(48271, MM, t) > mu || t == 5,
            "48271 beats 16807 at every t except a near-tie at t=5"
        );
    }
}

#[test]
fn merit_formula_matches_module_12_and_the_exact_gamma_table() {
    // t = 2 and t = 3 must reproduce module 12's mu2/mu3 to 1e-9 (this
    // pins Γ(2) = 1 and Γ(5/2) = 3√π/4):
    assert!((mu_t(16807, MM, 2) - 0.413_238_150_362_94).abs() < 1e-9);
    assert!((mu_t(16807, MM, 3) - 0.508_702_013_718_59).abs() < 1e-9);
    assert!((mu_t(48271, MM, 2) - 2.912_282_728_59).abs() < 1e-9);
    assert!((mu_t(48271, MM, 3) - 3.349_102_265_47).abs() < 1e-9);
    // t = 4, 5, 6 against independently expanded closed forms (this pins
    // Γ(3) = 2, Γ(7/2) = 15√π/8, Γ(4) = 6):
    //   mu_4 = pi^2·nu^4/(2m),  mu_5 = (8 pi^2/15)·nu^5/m,
    //   mu_6 = pi^3·nu^6/(6m).
    use std::f64::consts::PI;
    let m = MM as f64;
    let nu4 = (nu_t_squared(48271, MM, 4) as f64).sqrt();
    assert!((mu_t(48271, MM, 4) - PI.powi(2) * nu4.powi(4) / (2.0 * m)).abs() < 1e-9);
    let nu5 = (nu_t_squared(48271, MM, 5) as f64).sqrt();
    assert!((mu_t(48271, MM, 5) - 8.0 * PI.powi(2) / 15.0 * nu5.powi(5) / m).abs() < 1e-9);
    let nu6 = (nu_t_squared(48271, MM, 6) as f64).sqrt();
    assert!((mu_t(48271, MM, 6) - PI.powi(3) * nu6.powi(6) / (6.0 * m)).abs() < 1e-9);
}

#[test]
fn every_nu_is_positive_and_the_search_terminates() {
    // nu_{t+1} <= nu_t is NOT asserted pointwise as a general law here —
    // it happens to hold for dual lattices (pad with a zero), but the
    // contract this stage certifies is: the pipeline terminates and yields
    // a positive integer for every supported (a, m, t).
    for &m in &[64i64, 101, 256, 6075] {
        for &a in &[5i64, 21, 106, m - 3] {
            if a <= 0 || a >= m {
                continue;
            }
            for t in 2..=6 {
                let nusq = nu_t_squared(a, m, t);
                assert!(nusq > 0, "nu_t^2 must be positive: a={a}, m={m}, t={t}");
                assert!(
                    nusq <= (m as i128) * (m as i128),
                    "nu_t^2 can never exceed m^2 (witness (m,0,...,0))"
                );
                let mu = mu_t(a, m, t);
                assert!(mu.is_finite() && mu > 0.0);
            }
        }
    }
}

#[test]
#[should_panic(expected = "2 <= t <= 6")]
fn dimensions_beyond_six_are_rejected() {
    // Honest scope: Knuth's Algorithm S reaches t = 8 with extra
    // machinery; this module stops at 6 and says so.
    nu_t_squared(48271, MM, 7);
}
