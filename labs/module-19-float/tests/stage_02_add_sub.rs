//! Stage 2 — Addition and subtraction with rounding (Algorithm 4.2.1A).
//!
//! Implement `Float::add` and `Float::sub`. Lesson: README §"Algorithm A".

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

fn fadd(a: f64, b: f64) -> f64 {
    Float::from_f64(a).add(&Float::from_f64(b)).to_f64()
}
fn fsub(a: f64, b: f64) -> f64 {
    Float::from_f64(a).sub(&Float::from_f64(b)).to_f64()
}

#[test]
fn add_agrees_with_hardware_to_the_last_bit() {
    // Correctly-rounded addition equals IEEE `+` bit-for-bit.
    let mut g = Lcg(0x1111);
    for _ in 0..40_000 {
        let a = g.f64_in(200);
        let b = g.f64_in(200);
        assert_eq!(fadd(a, b).to_bits(), (a + b).to_bits(), "{a} + {b}");
        assert_eq!(fsub(a, b).to_bits(), (a - b).to_bits(), "{a} - {b}");
    }
}

#[test]
fn add_agrees_when_magnitudes_are_far_apart() {
    // The hard case for exponent alignment: huge + tiny (and its cancellation).
    let mut g = Lcg(0x2222);
    for _ in 0..20_000 {
        let a = g.f64_in(250);
        let b = g.f64_in(20);
        assert_eq!(fadd(a, b).to_bits(), (a + b).to_bits(), "{a} + {b}");
        assert_eq!(fsub(a, b).to_bits(), (a - b).to_bits(), "{a} - {b}");
    }
}

#[test]
fn adding_zero_is_identity() {
    let mut g = Lcg(0x3333);
    for _ in 0..2_000 {
        let a = g.f64_in(150);
        assert_eq!(fadd(a, 0.0).to_bits(), a.to_bits());
        assert_eq!(fadd(0.0, a).to_bits(), a.to_bits());
        assert_eq!(fsub(a, 0.0).to_bits(), a.to_bits());
    }
}

#[test]
fn addition_is_not_associative() {
    // (1 + h) + h = 1  (each half-ulp tie rounds to even, staying 1),
    // but 1 + (h + h) = 1 + 2^-52 = 1 + ulp.  A worked counterexample.
    let one = Float::from_f64(1.0);
    let h = Float::from_f64((2f64).powi(-53));
    let left = one.add(&h).add(&h).to_f64();
    let right = one.add(&h.add(&h)).to_f64();
    assert_eq!(left, 1.0);
    assert_eq!(right, 1.0 + (2f64).powi(-52));
    assert_ne!(left, right);
    // Hardware floats are non-associative in exactly the same way.
    assert_eq!(left, (1.0 + (2f64).powi(-53)) + (2f64).powi(-53));
    assert_eq!(right, 1.0 + ((2f64).powi(-53) + (2f64).powi(-53)));
}

#[test]
fn catastrophic_cancellation() {
    // (1 + eps) - 1 recovers eps exactly when eps is representable at 1's
    // scale, but small perturbations below the ulp are annihilated.
    let eps = (2f64).powi(-52); // one ulp at 1.0
    assert_eq!(fsub(1.0 + eps, 1.0), eps);
    // A quantity a quarter-ulp below the ulp is lost entirely by 1 + x.
    let tiny = (2f64).powi(-54);
    assert_eq!(fadd(1.0, tiny), 1.0);
    assert_eq!(fsub(fadd(1.0, tiny), 1.0), 0.0); // the tiny term is gone
}

#[test]
fn round_to_even_tie_cases() {
    let one = Float::from_f64(1.0);
    // 1 + 2^-54  (quarter ulp)  -> rounds down to 1.
    assert_eq!(one.add(&Float::from_f64((2f64).powi(-54))).to_f64(), 1.0);
    // 1 + 2^-53  (exact half ulp, a tie) -> round to even -> 1.
    assert_eq!(one.add(&Float::from_f64((2f64).powi(-53))).to_f64(), 1.0);
    // 1 + (2^-53 + 2^-105)  (just over half) -> rounds up to 1 + ulp.
    let over = Float::from_f64((2f64).powi(-53)).add(&Float::from_f64((2f64).powi(-105)));
    assert_eq!(one.add(&over).to_f64(), 1.0 + (2f64).powi(-52));
    // The next tie up, at 3 * 2^-53 above 1: rounds to the even neighbour.
    // 1 + 2^-52 + 2^-53 is a tie between 1+2^-52 and 1+2^-51; even is 1+2^-51.
    let a = Float::from_f64(1.0 + (2f64).powi(-52));
    let tie = a.add(&Float::from_f64((2f64).powi(-53))).to_f64();
    assert_eq!(tie, (1.0 + (2f64).powi(-52)) + (2f64).powi(-53));
}

#[test]
fn subtraction_is_negation_plus_add() {
    let mut g = Lcg(0x4444);
    for _ in 0..5_000 {
        let a = g.f64_in(120);
        let b = g.f64_in(120);
        assert_eq!(fsub(a, b), fadd(a, -b));
    }
}
