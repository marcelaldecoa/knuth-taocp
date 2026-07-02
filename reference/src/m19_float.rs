//! Module 19 — Floating-Point Arithmetic.
//! Source: TAOCP Vol. 2, 3rd ed., §4.2 (§4.2.1 the operations, §4.2.2 the
//! error analysis).
//!
//! We model Knuth's normalized "drum" floating point as a clean binary format
//! with a `p = 53`-bit significand, matching IEEE 754 binary64 so we can
//! cross-check every operation against the hardware `f64`.
//!
//! A finite value is represented as
//!
//! ```text
//!     value = (-1)^sign * frac * 2^exp
//! ```
//!
//! where `frac` is a 53-bit integer. A *normalized* nonzero value keeps the
//! leading 1 pinned at bit 52, i.e. `2^52 <= frac < 2^53`; then `2^exp` is the
//! weight of the least-significant fraction bit — the *unit in the last place*
//! (ulp). Zero is `frac == 0` (the sign bit distinguishes ±0).
//!
//! Scope: this is a *finite* model. `from_f64` rejects non-finite inputs, and
//! `to_f64` saturates to ±∞ only when a legitimately-computed magnitude leaves
//! the binary64 normal range. Infinities and NaNs as first-class values are
//! deliberately out of scope (§4.2.1's algorithms are stated for finite
//! normalized numbers); the interesting mathematics — normalization, rounding,
//! and error analysis — lives entirely in the finite realm.

/// Precision: number of significand bits (the leading 1 included).
pub const P: u32 = 53;

/// The classification of a `Float` in this finite model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// `frac == 0`: a signed zero.
    Zero,
    /// A normalized finite value, `2^52 <= frac < 2^53`.
    Normal,
}

/// A normalized binary floating-point number, value `(-1)^sign * frac * 2^exp`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Float {
    /// Sign: `true` means negative.
    pub sign: bool,
    /// Binary exponent: the weight of the least-significant bit of `frac`.
    pub exp: i32,
    /// 53-bit significand; normalized nonzero values keep bit 52 set.
    pub frac: u64,
}

impl Float {
    /// Pack raw fields into a `Float`. This is the "unpack/pack" primitive of
    /// stage 1: it stores the fields verbatim and does *not* normalize. The
    /// significand must fit in 53 bits.
    pub fn new(sign: bool, exp: i32, frac: u64) -> Float {
        debug_assert!(frac < (1u64 << P), "significand must fit in 53 bits");
        Float { sign, exp, frac }
    }

    /// The signed zero with the given sign.
    pub fn zero(sign: bool) -> Float {
        Float { sign, exp: 0, frac: 0 }
    }

    /// Normalize in place: shift the significand so its leading 1 sits at
    /// bit 52, adjusting `exp` to preserve the value. Lossless (the caller
    /// guarantees `frac < 2^53`) and idempotent. Zero is left as a signed zero.
    pub fn normalize(&mut self) {
        if self.frac == 0 {
            // A zero has no leading 1; canonicalize the exponent.
            self.exp = 0;
            return;
        }
        // Bring the leading 1 up to bit 52. For a legal 53-bit significand we
        // only ever need to shift left (frac < 2^52) — never right.
        while self.frac < (1u64 << (P - 1)) {
            self.frac <<= 1;
            self.exp -= 1;
        }
    }

    /// Decode an IEEE 754 `f64` into this model, exactly. Panics on non-finite
    /// input — the model is finite by construction.
    pub fn from_f64(x: f64) -> Float {
        assert!(x.is_finite(), "Float models finite values only");
        let bits = x.to_bits();
        let sign = (bits >> 63) != 0;
        let biased = ((bits >> 52) & 0x7ff) as i32;
        let mantissa = bits & 0x000f_ffff_ffff_ffff; // low 52 bits
        if biased == 0 {
            if mantissa == 0 {
                return Float::zero(sign);
            }
            // Subnormal: value = mantissa * 2^-1074. Normalize it up.
            let mut f = Float { sign, exp: -1074, frac: mantissa };
            f.normalize();
            return f;
        }
        // Normal: value = (2^52 + mantissa) * 2^(biased - 1023 - 52).
        let frac = (1u64 << (P - 1)) | mantissa;
        let exp = biased - 1023 - (P as i32 - 1);
        Float { sign, exp, frac }
    }

    /// Encode back to the nearest `f64`. For a normalized value in binary64's
    /// normal range this is exact (a true inverse of `from_f64`). Magnitudes
    /// above the range saturate to ±∞; tiny magnitudes fall through to a
    /// correctly-rounded scaling.
    pub fn to_f64(&self) -> f64 {
        if self.frac == 0 {
            return if self.sign { -0.0 } else { 0.0 };
        }
        // Work on a normalized copy so bit 52 is the implicit leading 1.
        let mut n = *self;
        n.normalize();
        let biased = n.exp + 1023 + (P as i32 - 1);
        if biased >= 0x7ff {
            return if n.sign { f64::NEG_INFINITY } else { f64::INFINITY };
        }
        if biased >= 1 {
            let mantissa = n.frac & 0x000f_ffff_ffff_ffff;
            let bits = ((n.sign as u64) << 63) | ((biased as u64) << 52) | mantissa;
            return f64::from_bits(bits);
        }
        // Subnormal / underflow: recover by exact scaling (correctly rounded).
        let mag = (n.frac as f64) * two_pow(n.exp);
        if n.sign {
            -mag
        } else {
            mag
        }
    }

    /// The unit in the last place of this (nonzero, normalized) value, returned
    /// as a `Float`: the positive quantity `2^exp`, the gap to the next
    /// representable number of the same exponent.
    pub fn ulp(&self) -> Float {
        // 2^exp = 2^52 * 2^(exp-52); already normalized.
        Float::new(false, self.exp - (P as i32 - 1), 1u64 << (P - 1))
    }

    /// Classify: `Zero` or `Normal`.
    pub fn classify(&self) -> Class {
        if self.frac == 0 {
            Class::Zero
        } else {
            Class::Normal
        }
    }

    /// Negation (flip the sign bit).
    pub fn neg(&self) -> Float {
        Float { sign: !self.sign, exp: self.exp, frac: self.frac }
    }

    /// Algorithm 4.2.1A — floating-point addition with round-to-nearest-even.
    ///
    /// ```text
    /// A1. [Unpack.]      Separate signs, exponents, fractions of u and v.
    /// A2. [Assume e_u >= e_v.]  If not, swap so u has the larger exponent.
    /// A3. [Set e = e_u.] The result exponent starts here.
    /// A4. [Scale right.] Shift v's fraction right by e_u - e_v, keeping the
    ///                    bits that fall off as a sticky trail (guard/round/
    ///                    sticky), so nothing that can affect rounding is lost.
    /// A5. [Add.]         Add or subtract the aligned fractions per the signs.
    /// A6. [Normalize.]   Renormalize the sum and round to 53 bits (step N).
    /// ```
    pub fn add(&self, other: &Float) -> Float {
        // A1 / zero shortcuts: 0 + v = v, u + 0 = u.
        if self.frac == 0 {
            return *other;
        }
        if other.frac == 0 {
            return *self;
        }
        // A2. Order so `a` holds the larger (or equal) exponent.
        let (a, b) = if self.exp >= other.exp {
            (self, other)
        } else {
            (other, self)
        };
        let shift = (a.exp - b.exp) as u32;

        // A4. Align to the common exponent `a.exp - GUARD`, keeping GUARD low
        // bits exact. Beyond GUARD, `b` cannot reach the round bit, so its only
        // trace is a sticky bit — this is why a bounded window suffices.
        const GUARD: u32 = 64;
        let a_wide = (a.frac as u128) << GUARD;
        let (b_wide, sticky) = if shift <= GUARD {
            ((b.frac as u128) << (GUARD - shift), false)
        } else {
            (0u128, b.frac != 0)
        };
        let e = a.exp - GUARD as i32;

        // A5. Combine by sign.
        let (sign, mut mag) = if a.sign == b.sign {
            (a.sign, a_wide + b_wide)
        } else if a_wide >= b_wide {
            (a.sign, a_wide - b_wide)
        } else {
            (b.sign, b_wide - a_wide)
        };
        if sticky {
            // Safe: whenever `sticky` can be set, the low GUARD bits are zero.
            mag |= 1;
        }
        if mag == 0 {
            // Exact cancellation yields +0 under round-to-nearest.
            return Float::zero(false);
        }

        // A6. Renormalize + round (step N).
        round_wide(sign, e, mag)
    }

    /// Subtraction: `self - other = self + (-other)`.
    pub fn sub(&self, other: &Float) -> Float {
        self.add(&other.neg())
    }

    /// Algorithm 4.2.1M — floating-point multiplication.
    ///
    /// ```text
    /// M1. [Unpack.]  Signs xor; exponents add; fractions multiply exactly.
    /// M2. [Multiply.] The exact product of two 53-bit fractions is 106 bits;
    ///                 keep it in a u128, no information lost.
    /// M3. [Normalize + round.] Renormalize to 53 bits (step N).
    /// ```
    pub fn mul(&self, other: &Float) -> Float {
        let sign = self.sign ^ other.sign;
        if self.frac == 0 || other.frac == 0 {
            return Float::zero(sign);
        }
        // M1/M2: exact 106-bit product.
        let prod = (self.frac as u128) * (other.frac as u128);
        let exp = self.exp + other.exp;
        // M3.
        round_wide(sign, exp, prod)
    }

    /// Floating-point division with round-to-nearest-even (§4.2.1).
    ///
    /// The exact quotient of two 53-bit fractions is not finite in binary, so
    /// we compute `frac_a * 2^64 / frac_b` (64 guard bits below the answer) and
    /// remember whether the division had a remainder — that remainder is the
    /// sticky bit. Sixty-four guard bits is far more than the two the rounding
    /// needs, so the result is correctly rounded.
    pub fn div(&self, other: &Float) -> Float {
        let sign = self.sign ^ other.sign;
        assert!(other.frac != 0, "floating-point division by zero");
        if self.frac == 0 {
            return Float::zero(sign);
        }
        let num = (self.frac as u128) << 64;
        let den = other.frac as u128;
        let mut q = num / den;
        let r = num % den;
        if r != 0 {
            q |= 1; // sticky: real bits live below q's last bit
        }
        round_wide(sign, self.exp - other.exp - 64, q)
    }
}

/// Step N of §4.2.1: given an *exact* magnitude `mag * 2^exp` (with `mag` a wide
/// unsigned integer that may carry guard/round/sticky bits), renormalize to a
/// 53-bit significand and round to nearest, ties to even. Returns a normalized
/// `Float`.
fn round_wide(sign: bool, mut exp: i32, mag: u128) -> Float {
    if mag == 0 {
        return Float::zero(sign);
    }
    // Position of the leading 1 (0-based).
    let msb = 127 - mag.leading_zeros() as i32;
    let target = P as i32 - 1; // we want the leading 1 at bit 52
    let shift = msb - target;

    let frac: u64;
    if shift <= 0 {
        // Fewer than 53 bits: shift left, exact, no rounding.
        frac = (mag << (-shift) as u32) as u64;
        exp -= -shift;
    } else {
        // Drop `shift` low bits with round-to-nearest-even.
        let s = shift as u32;
        let round_bit = (mag >> (s - 1)) & 1;
        let sticky = (mag & (((1u128) << (s - 1)) - 1)) != 0;
        let mut kept = (mag >> s) as u64;
        exp += shift;
        // Round half to even.
        if round_bit == 1 && (sticky || (kept & 1) == 1) {
            kept += 1;
            if kept == (1u64 << P) {
                // Carry out of the top: e.g. 0x1F...F + 1. Re-normalize.
                kept >>= 1;
                exp += 1;
            }
        }
        frac = kept;
    }
    Float { sign, exp, frac }
}

/// Exactly compute `2^n` as an `f64` for `n` in binary64's representable range
/// (including subnormal exponents), falling back to 0 / ∞ outside it.
fn two_pow(n: i32) -> f64 {
    if n < -1074 {
        return 0.0;
    }
    if n > 1023 {
        return f64::INFINITY;
    }
    if n >= -1022 {
        // Normal: assemble the exponent field directly.
        let biased = (n + 1023) as u64;
        f64::from_bits(biased << 52)
    } else {
        // Subnormal power of two: a single set bit below the implicit one.
        f64::from_bits(1u64 << (n + 1074))
    }
}

// ---------------------------------------------------------------------------
// §4.2.2 — error analysis and compensated summation.
// ---------------------------------------------------------------------------

/// The unit roundoff `u = 2^-52` for binary64: `1.0 + u` is the smallest
/// representable number greater than `1.0`, while `1.0 + u/2` rounds back to
/// `1.0`. (Some texts call `u` "machine epsilon"; beware the factor-of-two
/// conventions.)
pub fn machine_epsilon() -> f64 {
    two_pow(-(P as i32 - 1)) // 2^-52
}

/// The gap between `x` and the next representable `f64` of larger magnitude —
/// one ulp of `x`. Used to measure error in ulps.
pub fn ulp_f64(x: f64) -> f64 {
    if x == 0.0 {
        return f64::from_bits(1); // smallest positive subnormal
    }
    let bits = x.abs().to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    if biased == 0 {
        return f64::from_bits(1); // subnormal region: constant ulp
    }
    let e = biased - 1023; // unbiased exponent of x
    two_pow(e - (P as i32 - 1)) // 2^(e-52)
}

/// Distance from `computed` to `exact`, measured in ulps of `exact`. This is
/// the natural yardstick for §4.2.2's error bounds: a correctly-rounded
/// operation lands within 0.5 ulp of the true result.
pub fn ulp_error(computed: f64, exact: f64) -> f64 {
    if computed == exact {
        return 0.0;
    }
    (computed - exact).abs() / ulp_f64(exact)
}

/// Naive left-to-right summation. Each addition rounds; the errors accumulate,
/// and in the worst case the total error grows like `n * u` times the running
/// magnitude.
pub fn naive_sum(xs: &[f64]) -> f64 {
    let mut s = 0.0;
    for &x in xs {
        s += x;
    }
    s
}

/// Kahan compensated summation (§4.2.2). The variable `c` carries the low-order
/// bits lost in the previous addition and feeds them back into the next one,
/// so the total error stays `O(u)` independent of `n`.
///
/// ```text
///   y = x_i - c            # restore the previously-lost bits
///   t = s + y             # rounds; loses the low part of y
///   c = (t - s) - y       # exactly that lost low part (TwoSum trick)
///   s = t
/// ```
pub fn kahan_sum(xs: &[f64]) -> f64 {
    let mut s = 0.0;
    let mut c = 0.0;
    for &x in xs {
        let y = x - c;
        let t = s + y;
        c = (t - s) - y;
        s = t;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    // A small linear congruential generator for deterministic pseudo-random
    // tests (no external crates, no std::random).
    struct Lcg(u64);
    impl Lcg {
        fn next_u64(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
        /// A finite normal f64 with exponent in [-lo, +lo] (moderate range so
        /// products/sums stay normal for bit-exact comparison with hardware).
        fn f64_in(&mut self, lo: i32) -> f64 {
            let r = self.next_u64();
            let sign = (r >> 63) as u64;
            let mantissa = r & 0x000f_ffff_ffff_ffff;
            let span = (2 * lo + 1) as u64;
            let e = (r >> 12) % span; // 0 ..= 2*lo
            let biased = (1023 - lo as u64) + e; // (1023-lo) ..= (1023+lo)
            f64::from_bits((sign << 63) | (biased << 52) | mantissa)
        }
    }

    #[test]
    fn roundtrip_from_to_f64() {
        let mut g = Lcg(0x1234_5678);
        for _ in 0..20_000 {
            let x = g.f64_in(200);
            assert_eq!(Float::from_f64(x).to_f64().to_bits(), x.to_bits());
        }
    }

    #[test]
    fn one_tenth_is_not_exact() {
        // §4.2 opens with the fact that 1/10 has no finite binary expansion.
        let f = Float::from_f64(0.1);
        // Bit-for-bit equal to the hardware double.
        assert_eq!(f.to_f64().to_bits(), (0.1f64).to_bits());
        // ...yet differs from the true rational 1/10 by less than one ulp:
        // |10*frac*2^exp - 1| < 10*2^exp  <=>  |10*frac - 2^-exp| < 10.
        assert!(f.exp < 0);
        let scaled = f.frac as i128 * 10;
        let one = 1i128 << (-f.exp);
        assert!((scaled - one).abs() < 10, "0.1 within 1 ulp of 1/10");
    }

    #[test]
    fn normalize_is_idempotent_and_powers_exact() {
        let mut f = Float::new(false, 0, 5); // 5 * 2^0, unnormalized
        f.normalize();
        let g = {
            let mut g = f;
            g.normalize();
            g
        };
        assert_eq!(f, g);
        assert_eq!(f.to_f64(), 5.0);
        for k in -60..60i32 {
            let p = (2f64).powi(k);
            assert_eq!(Float::from_f64(p).to_f64(), p);
        }
    }

    #[test]
    fn add_matches_hardware() {
        let mut g = Lcg(0xABCDEF);
        for _ in 0..20_000 {
            let a = g.f64_in(200);
            let b = g.f64_in(200);
            let got = Float::from_f64(a).add(&Float::from_f64(b)).to_f64();
            assert_eq!(got.to_bits(), (a + b).to_bits(), "{a} + {b}");
        }
    }

    #[test]
    fn addition_is_not_associative() {
        // (1 + 2^-53) + 2^-53 rounds each tie to even -> stays 1.0,
        // but 1 + (2^-53 + 2^-53) = 1 + 2^-52 = 1 + ulp.
        let one = Float::from_f64(1.0);
        let h = Float::from_f64((2f64).powi(-53));
        let left = one.add(&h).add(&h);
        let right = one.add(&h.add(&h));
        assert_eq!(left.to_f64(), 1.0);
        assert_eq!(right.to_f64(), 1.0 + (2f64).powi(-52));
        assert_ne!(left.to_f64(), right.to_f64());
    }

    #[test]
    fn round_to_even_ties() {
        let one = Float::from_f64(1.0);
        // 1 + 2^-54 = quarter ulp -> rounds to 1.
        assert_eq!(one.add(&Float::from_f64((2f64).powi(-54))).to_f64(), 1.0);
        // 1 + 2^-53 = half ulp, tie -> even -> 1.
        assert_eq!(one.add(&Float::from_f64((2f64).powi(-53))).to_f64(), 1.0);
        // just over half a ulp -> rounds up.
        let over = Float::from_f64((2f64).powi(-53)).add(&Float::from_f64((2f64).powi(-105)));
        assert_eq!(one.add(&over).to_f64(), 1.0 + (2f64).powi(-52));
    }

    #[test]
    fn mul_div_match_hardware() {
        let mut g = Lcg(0x0F0F0F0F);
        for _ in 0..20_000 {
            let a = g.f64_in(100);
            let b = g.f64_in(100);
            let m = Float::from_f64(a).mul(&Float::from_f64(b)).to_f64();
            assert_eq!(m.to_bits(), (a * b).to_bits(), "{a} * {b}");
            let d = Float::from_f64(a).div(&Float::from_f64(b)).to_f64();
            assert_eq!(d.to_bits(), (a / b).to_bits(), "{a} / {b}");
        }
    }

    #[test]
    fn mul_by_power_of_two_and_identities() {
        let two = Float::from_f64(2.0);
        let x = Float::from_f64(0.1);
        assert_eq!(x.mul(&two).to_f64(), 0.2);
        assert_eq!(x.div(&x).to_f64(), 1.0);
        assert_eq!(x.mul(&Float::from_f64(1.0)).to_f64(), 0.1);
        assert!(x.mul(&Float::zero(false)).to_f64() == 0.0);
    }

    #[test]
    fn kahan_beats_naive() {
        // Adversarial: a huge value swamps a run of ones, then is cancelled.
        let mut xs = vec![1e16];
        xs.extend(std::iter::repeat(1.0).take(10_000));
        xs.push(-1e16);
        assert_eq!(naive_sum(&xs), 0.0); // every one was lost
        assert!((kahan_sum(&xs) - 10_000.0).abs() < 4.0);
    }

    #[test]
    fn kahan_stays_close_on_many_tenths() {
        let xs = vec![0.1; 100_000];
        let k = kahan_sum(&xs);
        let n = naive_sum(&xs);
        assert!((k - 10_000.0).abs() < 1e-9);
        assert!((k - 10_000.0).abs() <= (n - 10_000.0).abs());
    }

    #[test]
    fn fundamental_bound_via_twosum() {
        // |fl(a+b) - (a+b)| <= (u/2)|a+b|, checked with the exact TwoSum error.
        let u = machine_epsilon();
        let mut g = Lcg(0x99);
        for _ in 0..50_000 {
            let a = g.f64_in(60);
            let b = g.f64_in(60);
            let s = a + b;
            let bv = s - a;
            let av = s - bv;
            let err = (a - av) + (b - bv); // exact: a + b = s + err
            assert!(err.abs() <= 0.5 * u * s.abs() + f64::MIN_POSITIVE);
        }
    }

    #[test]
    fn epsilon_threshold() {
        let u = machine_epsilon();
        assert_eq!(u, (2f64).powi(-52));
        assert_ne!(1.0 + u, 1.0);
        assert_eq!(1.0 + u / 2.0, 1.0);
        // ulp_error of an exactly-equal pair is zero.
        assert!(ulp_error(0.1f64, 0.1f64) == 0.0);
    }
}
