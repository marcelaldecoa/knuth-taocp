# Walkthrough — Module 19 (Floating-Point Arithmetic)

Read this *after* a stage is green. It is a design commentary on
`reference/src/m19_float.rs`: why the code is shaped the way it is, the
invariant that makes it correct, and the idioms worth stealing. You do not need
any of it to pass — only to see why the reference beats a naive version.

## Stage 1: Representation

The whole module rests on one representation invariant: **a nonzero `Float` is
normalized, `2^52 ≤ frac < 2^53`, and `value = (-1)^sign · frac · 2^exp`.**
Keeping `exp` as the weight of `frac`'s *last* bit (rather than IEEE's "weight
of the leading bit") is the single decision that makes everything downstream
clean: `ulp` is literally `2^exp`, and the wide-integer arithmetic of later
stages never has to track a fractional point.

`from_f64`/`to_f64` are deliberately built as *exact bit inverses* on the normal
range, not as `frac as f64 * 2f64.powi(exp)`. The naive scaling double-rounds
(once forming `2^exp`, once multiplying) and loses the bottom of the subnormal
range; the bit-assembly version (`(sign<<63)|(biased<<52)|mantissa`) is exact
and, crucially, is what lets every later stage assert `== hardware` bit for bit.
That single design choice turns the hardware `f64` into a free oracle for
40000-case property tests.

`normalize` only ever shifts *left*. That is not laziness — it is the invariant
talking: a legal 53-bit significand is already `< 2^53`, so it can only be
*under*-normalized (leading 1 too low), never over. Rounding (which is the only
thing that can produce more than 53 bits) is a *different* operation, factored
out into `round_wide`, so `normalize` stays lossless and idempotent. The `0.1`
test is worth stealing: it proves "within one ulp of 1/10" with pure integer
arithmetic (`|10·frac - 2^{-exp}| < 10`), sidestepping the need for a bignum
rational just to talk about accuracy.

## Stage 2: Addition and subtraction

The naive approach — shift the smaller operand right and hope — either loses
rounding information or needs a 2000-bit register for `1e300 + 1e-300`. The
reference's idiom is a **bounded exact window plus a sticky bit**: put the large
operand's `frac` at bit 64 of a `u128` (`a_wide = frac << 64`), giving 64 exact
guard bits below it; shift the small operand into that window; and if it would
shift past the window (`shift > 64`), collapse *all* of it into a single sticky
bit. The correctness argument is the payoff of §4.2.1's guard/round/sticky
theory: an operand shifted more than 64 places cannot reach even the round bit,
so its only possible influence on the result is "is there anything nonzero down
there?" — exactly one bit of information.

Two subtleties the code gets right and a naive version botches:

- **Sign handling via magnitude.** For opposite signs it subtracts the smaller
  `u128` from the larger and takes the larger's sign, so cancellation is exact
  and the result sign is always correct — no special cases for "which is
  bigger" beyond the one comparison.
- **The `mag |= 1` sticky merge is provably safe** because whenever `sticky` can
  be set (`shift > 64`), the low 64 bits of `a_wide` are zero, so OR-ing a 1
  there cannot corrupt a real bit. This is why a *single* rounding routine can
  serve add, mul, and div.

All roundings funnel through `round_wide`, which implements step N once and
correctly (leading-bit find via `leading_zeros`, then the RNE decision
`round_bit == 1 && (sticky || kept&1)`, with carry-out to `1<<53` renormalized).
Centralizing rounding is the reason the three operations are each only a dozen
lines.

## Stage 3: Multiplication and division

The governing idea is **exact-then-round**. `mul` forms the full 106-bit product
in a `u128` — exact, no information lost — and hands it to the same `round_wide`.
Because there is exactly one rounding, of an exact product, the result is
*correctly rounded* by construction; the tests confirm it matches hardware `*`
to the bit. A naive `(self.to_f64() * other.to_f64())` would work here only
because `f64` already does this — the point of the exercise is to *be* the
hardware, and the `u128` product is how.

Division cannot be exact, so the reference manufactures the guard/round/sticky
it needs: `q = (frac_u << 64) / frac_v` gives 64 fractional bits of quotient
(vastly more than the 2 rounding needs), and the remainder `r = (frac_u<<64) %
frac_v` *is* the sticky bit — `r != 0` means "the true quotient has more below
what we computed." Setting `q |= 1` folds that into the low bit, and `round_wide`
finishes. The elegance is that division reuses the exact same rounding path as
add and mul; the only division-specific line is turning the remainder into
sticky. Note the `assert!(other.frac != 0, "...division by zero")` with a
message the tests match — infinities are out of scope, and the model says so
loudly rather than returning garbage.

## Stage 4: Error analysis

The functions are short; the *insight density* is high. `ulp_f64` and
`machine_epsilon` are built from the same bias arithmetic as stage 1 — reuse,
not reinvention. The two summation functions are nearly identical in shape,
which is the whole pedagogical point: four extra flops per element (`kahan`'s
`y`, `t`, `c`) buy an error bound that no longer grows with `n`.

The idiom to steal from `kahan_sum` is the **error-free transformation** hiding
in `c = (t - s) - y`. In exact arithmetic that is zero; in floating point it is
exactly the rounding error of `t = fl(s + y)`. This is TwoSum, and the same
three lines that verify the fundamental bound in the tests are what make Kahan
work — recovering the rounding error *in the same precision* is one of the most
useful tricks in all of numerical computing (double-double arithmetic, exact dot
products, and reproducible summation all start here). A naive summation loop
throws that error on the floor every iteration; Kahan picks it back up. The
reference deliberately keeps both so you can diff them and see that accuracy is
a handful of instructions away, not a change of number type.
