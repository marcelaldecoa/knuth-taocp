//! Stage 4 — Figures of merit: judging real generators.
//!
//! Raw nu_t^2 values are hard to compare across moduli; Knuth normalizes:
//!   mu_t = (volume of the t-ball of radius nu_t) / m
//! so that mu_t measures the lattice against the best possible density.
//!   mu_2 = pi·nu_2^2/m,      mu_3 = (4/3)·pi·nu_3^3/m.
//! Rule of thumb (§3.3.4): mu_t >= 0.1 passes, mu_t >= 1 is excellent.

use lab_12_spectral::{mu2, mu3};

const RANDU_A: i64 = 65539;
const RANDU_M: i64 = 1 << 31;
const MINSTD_A: i64 = 16807;
const MINSTD_M: i64 = (1 << 31) - 1;

#[test]
fn randu_mu3_is_catastrophic() {
    // nu_3^2 = 118 gives mu_3 = (4/3)·pi·118^{3/2}/2^31 ≈ 2.5e-6 — five
    // orders of magnitude below the 0.1 pass mark. This single number is
    // the spectral test's verdict on RANDU.
    let mu = mu3(RANDU_A, RANDU_M);
    assert!(mu < 1e-4, "RANDU mu3 = {mu}, expected ≈ 2.5e-6");
    assert!(mu > 1e-7, "did you divide by the right power of m?");
    // Pin it down: the reference computes 2.500240e-6.
    assert!((mu - 2.500_240_320_878e-6).abs() < 1e-12);
}

#[test]
fn minimal_standard_merits_match_reference_bands() {
    // Computed exactly in the reference implementation first, then frozen:
    //   nu_2^2 = 16807^2 + 1 = 282_475_250 -> mu_2 = 0.413238150...
    //   nu_3^2 = 408_197               -> mu_3 = 0.508702013...
    // Both pass Knuth's 0.1 bar; neither is excellent. That is the fair
    // verdict on the minimal standard: usable, dated, unspectacular.
    let m2 = mu2(MINSTD_A, MINSTD_M);
    let m3 = mu3(MINSTD_A, MINSTD_M);
    assert!((m2 - 0.413_238_150_362_94).abs() < 1e-9, "mu2 = {m2}");
    assert!((m3 - 0.508_702_013_718_59).abs() < 1e-9, "mu3 = {m3}");
    assert!(m2 > 0.1 && m3 > 0.1, "minimal standard passes the 0.1 bar");
}

#[test]
fn good_multipliers_beat_randu() {
    // Ordering on mu_3: the revised Park–Miller multiplier 48271 (an
    // excellent lattice, mu_3 ≈ 3.35) beats the minimal standard, which
    // beats RANDU by five orders of magnitude.
    let randu = mu3(RANDU_A, RANDU_M);
    let minstd = mu3(MINSTD_A, MINSTD_M);
    let revised = mu3(48271, MINSTD_M);
    assert!(minstd > 10_000.0 * randu, "minstd {minstd} vs randu {randu}");
    assert!(revised > minstd, "48271 ({revised}) should beat 16807 ({minstd})");
    assert!((revised - 3.349_102_265_47).abs() < 1e-9);
    // And 48271 also shines in two dimensions (mu_2 ≈ 2.91 vs 0.41).
    assert!(mu2(48271, MINSTD_M) > 2.0);
}

#[test]
fn mu2_formula_on_a_tiny_worked_example() {
    // a = 137, m = 256: nu_2^2 = 274 (stage 2's hand trace), so
    // mu_2 = pi·274/256 = 3.3624... — a *great* 2-D lattice for its size.
    let got = mu2(137, 256);
    let want = std::f64::consts::PI * 274.0 / 256.0;
    assert!((got - want).abs() < 1e-12, "mu2(137,256) = {got}, want {want}");
}

#[test]
fn mu3_formula_on_a_tiny_worked_example() {
    // a = 137, m = 256: nu_3^2 = 30 (stage 3), so
    // mu_3 = (4/3)·pi·30^{3/2}/256 = 2.6885...
    let got = mu3(137, 256);
    let want = 4.0 / 3.0 * std::f64::consts::PI * 30.0f64.sqrt().powi(3) / 256.0;
    assert!((got - want).abs() < 1e-12, "mu3(137,256) = {got}, want {want}");
}

#[test]
fn merit_never_exceeds_the_hermite_ceiling() {
    // mu_2 <= 2*pi/sqrt(3) ≈ 3.6276 and mu_3 <= (4/3)*pi*sqrt(2) ≈ 5.9238
    // for EVERY generator — Hermite's constants gamma_2 = 2/sqrt(3),
    // gamma_3 = 2^{1/3} cap how good any lattice can be.
    let mu2_cap = 2.0 * std::f64::consts::PI / 3.0f64.sqrt();
    let mu3_cap = 4.0 / 3.0 * std::f64::consts::PI * 2.0f64.sqrt();
    for (a, m) in [
        (137i64, 256i64),
        (48271, MINSTD_M),
        (16807, MINSTD_M),
        (65539, RANDU_M),
        (1, 4096),
        (33, 1 << 16),
    ] {
        let (m2, m3) = (mu2(a, m), mu3(a, m));
        assert!(m2 <= mu2_cap + 1e-9, "mu2({a},{m}) = {m2} over the cap");
        assert!(m3 <= mu3_cap + 1e-9, "mu3({a},{m}) = {m3} over the cap");
    }
}
