//! Stage 1 — Representation: pack, unpack, normalize (§4.2.1).
//!
//! Implement `Float::new`, `zero`, `normalize`, `from_f64`, `to_f64`, `ulp`,
//! `classify` in src/lab.rs. Lesson: course/module-19-float/README.md.

use lab_19_float::{Class, Float};

// Deterministic LCG (no external crates, no std::random).
struct Lcg(u64);
impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    /// A finite normal f64 with binary exponent in [-lo, +lo].
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
fn roundtrip_is_identity_on_normals() {
    // from_f64 then to_f64 reproduces the exact bit pattern for normal doubles.
    let mut g = Lcg(0xC0FFEE);
    for _ in 0..30_000 {
        let x = g.f64_in(250);
        let back = Float::from_f64(x).to_f64();
        assert_eq!(back.to_bits(), x.to_bits(), "roundtrip failed for {x}");
    }
}

#[test]
fn normalization_is_idempotent() {
    // Normalizing an already-normalized value changes nothing.
    let mut g = Lcg(0x5EED);
    for _ in 0..5_000 {
        let f = Float::from_f64(g.f64_in(100));
        let mut once = f;
        once.normalize();
        let mut twice = once;
        twice.normalize();
        assert_eq!(once, twice);
        assert_eq!(once, f, "from_f64 must already be normalized");
    }
}

#[test]
fn normalize_lifts_the_leading_one() {
    // 5 * 2^0 is not normalized (bit 52 is not set); normalize preserves value.
    let mut f = Float::new(false, 0, 5);
    f.normalize();
    assert_eq!(f.to_f64(), 5.0);
    // The normalized significand must have its leading 1 at bit 52.
    assert!(f.frac >= (1u64 << 52) && f.frac < (1u64 << 53));
}

#[test]
fn powers_of_two_are_exact() {
    for k in -80..80i32 {
        let p = (2f64).powi(k);
        assert_eq!(Float::from_f64(p).to_f64(), p, "2^{k}");
    }
    // ...and so is any small integer.
    for n in 0..1000u64 {
        let x = n as f64;
        assert_eq!(Float::from_f64(x).to_f64(), x);
    }
}

#[test]
fn one_tenth_is_not_exact_but_matches_hardware() {
    let f = Float::from_f64(0.1);
    // Bit-for-bit equal to the hardware double 0.1.
    assert_eq!(f.to_f64().to_bits(), (0.1f64).to_bits());
    // Yet within one ulp of the true rational 1/10:
    //   |v - 1/10| < ulp = 2^exp   <=>   |10*frac - 2^-exp| < 10.
    assert!(f.exp < 0, "0.1 has a negative binary exponent");
    let scaled = f.frac as i128 * 10;
    let one = 1i128 << (-f.exp);
    assert!((scaled - one).abs() < 10, "0.1 within one ulp of 1/10");
    // And it is definitely NOT the exact rational.
    assert!(scaled != one, "0.1 is not exactly 1/10 in binary");
}

#[test]
fn signs_and_zero() {
    assert_eq!(Float::zero(false).classify(), Class::Zero);
    assert_eq!(Float::zero(true).classify(), Class::Zero);
    assert_eq!(Float::zero(false).to_f64(), 0.0);
    assert!(Float::zero(true).to_f64().is_sign_negative());
    assert_eq!(Float::from_f64(0.0).classify(), Class::Zero);
    assert_eq!(Float::from_f64(3.5).classify(), Class::Normal);
    // Negation flips the sign of the value.
    assert_eq!(Float::from_f64(3.5).neg().to_f64(), -3.5);
    assert_eq!(Float::from_f64(-2.0).to_f64(), -2.0);
}

#[test]
fn ulp_is_the_gap_to_the_next_number() {
    // ulp(1.0) = 2^-52; adding it must change the value, half of it must not.
    let one = Float::from_f64(1.0);
    let u = one.ulp().to_f64();
    assert_eq!(u, (2f64).powi(-52));
    assert_ne!(1.0 + u, 1.0);
    // ulp scales with magnitude: ulp(2^k) = 2^(k-52).
    for k in -30..30i32 {
        let x = Float::from_f64((2f64).powi(k));
        assert_eq!(x.ulp().to_f64(), (2f64).powi(k - 52));
    }
}
