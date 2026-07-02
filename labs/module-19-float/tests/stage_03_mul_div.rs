//! Stage 3 — Multiplication and division (Algorithm 4.2.1M and §4.2.1 divide).
//!
//! Implement `Float::mul` and `Float::div`. Lesson: README §"Algorithm M".

use lab_19_float::Float;

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

fn fmul(a: f64, b: f64) -> f64 {
    Float::from_f64(a).mul(&Float::from_f64(b)).to_f64()
}
fn fdiv(a: f64, b: f64) -> f64 {
    Float::from_f64(a).div(&Float::from_f64(b)).to_f64()
}

#[test]
fn mul_agrees_with_hardware_to_the_last_bit() {
    // The exact 106-bit product, correctly rounded, equals IEEE `*`.
    let mut g = Lcg(0xAAAA);
    for _ in 0..40_000 {
        let a = g.f64_in(120);
        let b = g.f64_in(120);
        assert_eq!(fmul(a, b).to_bits(), (a * b).to_bits(), "{a} * {b}");
    }
}

#[test]
fn div_agrees_with_hardware_to_the_last_bit() {
    let mut g = Lcg(0xBBBB);
    for _ in 0..40_000 {
        let a = g.f64_in(120);
        let b = g.f64_in(120);
        assert_eq!(fdiv(a, b).to_bits(), (a / b).to_bits(), "{a} / {b}");
    }
}

#[test]
fn multiply_by_power_of_two_is_exact() {
    // Multiplying by 2^k only shifts the exponent — no rounding at all.
    let mut g = Lcg(0xCCCC);
    for _ in 0..5_000 {
        let a = g.f64_in(100);
        for k in -20..20i32 {
            let p = (2f64).powi(k);
            assert_eq!(fmul(a, p).to_bits(), (a * p).to_bits());
        }
    }
    assert_eq!(fmul(0.1, 2.0), 0.2);
    assert_eq!(fmul(0.1, 4.0), 0.4);
}

#[test]
fn identity_and_zero() {
    let mut g = Lcg(0xDDDD);
    for _ in 0..5_000 {
        let a = g.f64_in(100);
        assert_eq!(fmul(a, 1.0).to_bits(), a.to_bits());
        assert_eq!(fdiv(a, 1.0).to_bits(), a.to_bits());
        assert_eq!(fmul(a, 0.0), 0.0);
        assert_eq!(fdiv(0.0, a), 0.0);
    }
}

#[test]
fn div_by_self_is_one() {
    let mut g = Lcg(0xEEEE);
    for _ in 0..5_000 {
        let a = g.f64_in(150);
        assert_eq!(fdiv(a, a), 1.0);
    }
}

#[test]
fn reciprocal_within_half_ulp() {
    // 1 / (1 / x) returns to within a couple ulps of x (two roundings), and
    // each division is itself correctly rounded (matches hardware exactly).
    let mut g = Lcg(0x1234);
    for _ in 0..10_000 {
        let x = g.f64_in(80);
        let r = fdiv(1.0, x);
        assert_eq!(r.to_bits(), (1.0f64 / x).to_bits());
    }
}

#[test]
fn sign_rules() {
    assert_eq!(fmul(-2.0, 3.0), -6.0);
    assert_eq!(fmul(-2.0, -3.0), 6.0);
    assert_eq!(fdiv(-6.0, 2.0), -3.0);
    assert_eq!(fdiv(6.0, -2.0), -3.0);
    assert_eq!(fdiv(-6.0, -2.0), 3.0);
}

#[test]
#[should_panic(expected = "division by zero")]
fn division_by_zero_is_rejected() {
    // Infinities are out of scope in this finite model.
    Float::from_f64(1.0).div(&Float::zero(false));
}
