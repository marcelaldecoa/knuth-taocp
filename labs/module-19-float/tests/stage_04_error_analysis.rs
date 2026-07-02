//! Stage 4 — Error analysis: ulps and compensated summation (§4.2.2).
//!
//! Implement `machine_epsilon`, `ulp_f64`, `ulp_error`, `naive_sum`,
//! `kahan_sum`. Lesson: README §"Error analysis" and §"Kahan summation".

use lab_19_float::{kahan_sum, machine_epsilon, naive_sum, ulp_error, ulp_f64};

struct Lcg(u64);
impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn f64_in(&mut self, lo: i32) -> f64 {
        let r = self.next_u64();
        let sign = (r >> 63) as u64;
        let mantissa = r & 0x000f_ffff_ffff_ffff;
        let span = (2 * lo + 1) as u64;
        let e = (r >> 12) % span;
        let biased = (1023 - lo as u64) + e;
        f64::from_bits((sign << 63) | (biased << 52) | mantissa)
    }
}

#[test]
fn machine_epsilon_is_the_threshold() {
    let u = machine_epsilon();
    assert_eq!(u, (2f64).powi(-52));
    // 1 + u is the next representable number; 1 + u/2 rounds back to 1.
    assert_ne!(1.0 + u, 1.0);
    assert_eq!(1.0 + u / 2.0, 1.0);
}

#[test]
fn ulp_and_ulp_error() {
    // ulp of a power of two is the classic 2^(k-52).
    for k in -40..40i32 {
        assert_eq!(ulp_f64((2f64).powi(k)), (2f64).powi(k - 52));
    }
    // The error between adjacent doubles is exactly one ulp.
    let x = 1.0f64;
    let nxt = f64::from_bits(x.to_bits() + 1);
    assert!((ulp_error(nxt, x) - 1.0).abs() < 1e-12);
    assert_eq!(ulp_error(x, x), 0.0);
    // Correctly-rounded results sit within half a ulp of the true value.
    // (1/3 rounded is within 0.5 ulp of the exact third, checked via the fact
    // that 3 * fl(1/3) differs from 1 by less than 3 * ulp.)
    let third = 1.0f64 / 3.0;
    assert!(ulp_error(third, third) == 0.0);
    assert!((3.0 * third - 1.0).abs() <= 3.0 * ulp_f64(1.0));
}

#[test]
fn fundamental_relative_error_bound() {
    // |fl(a+b) - (a+b)| <= (u/2) |a+b|, verified statistically. The exact
    // error is recovered with Knuth's TwoSum, so no extra precision is needed.
    let u = machine_epsilon();
    let mut g = Lcg(0x777);
    for _ in 0..60_000 {
        let a = g.f64_in(60);
        let b = g.f64_in(60);
        let s = a + b;
        let bv = s - a;
        let av = s - bv;
        let err = (a - av) + (b - bv); // exact: a + b = s + err
        assert!(
            err.abs() <= 0.5 * u * s.abs() + f64::MIN_POSITIVE,
            "bound violated: a={a} b={b} err={err}"
        );
    }
}

#[test]
fn kahan_recovers_swamped_terms() {
    // Adversarial: a huge value swamps a run of ones, then is cancelled away.
    // Naive summation loses every 1.0; Kahan keeps them.
    let mut xs = vec![1e16];
    xs.extend(std::iter::repeat(1.0).take(10_000));
    xs.push(-1e16);
    let naive = naive_sum(&xs);
    let kahan = kahan_sum(&xs);
    assert_eq!(naive, 0.0, "naive loses all 10000 ones");
    assert!((kahan - 10_000.0).abs() < 4.0, "kahan recovers them: {kahan}");
}

#[test]
fn kahan_error_stays_bounded_while_naive_drifts() {
    // Sum 0.1 a hundred thousand times; the true value is 10000.
    let xs = vec![0.1f64; 100_000];
    let naive = naive_sum(&xs);
    let kahan = kahan_sum(&xs);
    // Kahan's error is O(u), independent of n: a handful of ulps at 10^4.
    assert!((kahan - 10_000.0).abs() < 1e-9, "kahan={kahan}");
    // ...and never worse than naive.
    assert!((kahan - 10_000.0).abs() <= (naive - 10_000.0).abs());
}

#[test]
fn naive_drift_dwarfs_kahan_at_large_n() {
    // At n = 100000, naive summation of 0.1 has drifted many ulps off 10000,
    // while Kahan sits at the roundoff floor: Kahan is orders of magnitude
    // closer. This is the O(n*u) vs O(u) contrast made concrete.
    let xs = vec![0.1f64; 100_000];
    let naive_err = (naive_sum(&xs) - 10_000.0).abs();
    let kahan_err = (kahan_sum(&xs) - 10_000.0).abs();
    assert!(kahan_err < 1e-9, "kahan near-exact: {kahan_err}");
    assert!(naive_err > 1e-9, "naive visibly drifts: {naive_err}");
    assert!(naive_err > kahan_err, "naive drifts far more than kahan");
}
