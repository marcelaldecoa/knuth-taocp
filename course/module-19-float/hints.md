# Hints — Module 19 (Floating-Point Arithmetic)

Graduated hints. Try the next one only after the previous fails you. They never
give the whole solution.

## Stage 1: Representation: pack, unpack, normalize

1. The model is `value = (-1)^sign · frac · 2^exp` with `frac` a 53-bit integer
   normalized so bit 52 is its leading 1. Everything reduces to splitting and
   reassembling an `f64`'s 64 bits: 1 sign, 11 biased exponent, 52 mantissa.
   The implicit leading 1 is not stored — you add it on decode, strip it on
   encode.
2. Use `x.to_bits()` / `f64::from_bits()`. For a *normal* `f64` (biased field in
   `1..=2046`): `frac = (1<<52) | mantissa` and `exp = biased - 1075` (since the
   value is `1.m · 2^{biased-1023}` and our exp is the weight of `frac`'s last
   bit). `normalize` left-shifts `frac` (decrementing `exp`) until `frac >= 2^52`;
   zero stays `frac == 0`.
3. Encode: `biased = exp + 1075`, `mantissa = frac & ((1<<52)-1)`, then
   `bits = (sign<<63) | (biased<<52) | mantissa`. `ulp(&self)` is `2^exp`, i.e.
   `Float::new(false, exp - 52, 1<<52)`. For the `0.1` test, note
   `|10·frac - (1<<(-exp))| < 10` is the "within one ulp of 1/10" check.

## Stage 2: Addition and subtraction with rounding

1. You cannot add significands until their exponents match. Align the
   smaller-exponent operand by shifting it right, but you must *not* lose bits
   that affect rounding: keep guard/round/sticky. Then add (or subtract, if
   signs differ), renormalize, and round to 53 bits, ties to even.
2. Work in a `u128`. Put the larger operand's `frac` at a fixed high position by
   shifting left by a `GUARD = 64` bits; shift the smaller one right by the
   exponent difference. If that difference exceeds 64, the small operand can't
   reach the round bit — collapse it to a single sticky bit (`frac != 0`). Same
   sign → add the two `u128`s; opposite sign → subtract smaller magnitude from
   larger, and the result's sign is the larger's.
3. Factor rounding into one helper `round_wide(sign, exp, mag: u128)`: find the
   leading bit `msb = 127 - mag.leading_zeros()`, set `shift = msb - 52`; if
   `shift <= 0` shift left (exact); else round bit is `(mag>>(shift-1))&1`,
   sticky is `mag & ((1<<(shift-1))-1) != 0`, and round up iff
   `round_bit==1 && (sticky || (kept&1)==1)`, handling carry-out to `1<<53`.
   `sub` is `self.add(&other.neg())`.

## Stage 3: Multiplication and division

1. Multiplication: signs xor, exponents add, significands multiply. Two 53-bit
   integers multiply to at most 106 bits, which fits exactly in a `u128` — so
   the product is exact and only the final normalization rounds. Division's
   exact quotient is generally infinite, so you must generate guard/round/sticky
   yourself.
2. `mul`: `prod = (self.frac as u128) * (other.frac as u128)`, then
   `round_wide(sign, self.exp + other.exp, prod)` (reuse stage 2's helper).
   Handle a zero operand up front, returning a signed zero.
3. `div`: assert `other.frac != 0` with a message containing `"division by
   zero"`. Compute `num = (self.frac as u128) << 64`, `q = num / den`,
   `r = num % den`; if `r != 0` set `q |= 1` (the remainder is the sticky bit).
   Then `round_wide(sign, self.exp - other.exp - 64, q)`.

## Stage 4: Error analysis: ulps and compensated summation

1. `machine_epsilon()` is `2^{-52}` — the gap between `1.0` and its successor.
   `ulp_f64(x)` is `2^{exponent(x)-52}`. Naive summation just folds with `+`;
   Kahan keeps a correction term for the bits lost each addition.
2. `ulp_f64`: read the biased exponent from `x.abs().to_bits()`, unbias
   (`e = biased - 1023`), return `2^{e-52}` (build the power via `from_bits` or
   `powi`). `ulp_error(c, x) = |c - x| / ulp_f64(x)`.
3. Kahan: `s=0; c=0; for x { let y = x - c; let t = s + y; c = (t - s) - y;
   s = t; } s`. The magic line `c = (t - s) - y` recovers exactly the low bits
   that `t = fl(s+y)` discarded; subtracting `c` next iteration cancels them.
