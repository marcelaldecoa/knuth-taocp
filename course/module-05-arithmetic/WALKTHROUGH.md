# Walkthrough — Module 05: Arithmetic

Read this AFTER a stage is green — it explains how the reference solution is
built and why.

## Stage 1: Multiple-precision addition and subtraction

The reference makes the *representation* do the work. Canonical form (no
trailing zero limbs, empty = 0) is definiteness applied to data: one value has
exactly one encoding, so `big_cmp` can shortcut on length alone — a longer limb
vector is unconditionally larger — and equality is just `Vec` equality. `big_add`
carries the column sum in a `u64` and extracts the carry with `k = t >> 32`
rather than a division; the invariant that `k ∈ {0,1}` is what guarantees this
shift never loses bits, and the final carry limb is pushed only when nonzero so
the result stays canonical. `big_sub` is the mirror image with an `i64` borrow:
when `t < 0` it adds `1<<32` back and sets the borrow to −1. The one idiom worth
stealing is the `assert!(big_cmp(u,v) != Less, "...nonnegative...")` guard — the
algorithm is *defined* only for `u ≥ v`, so instead of returning garbage it
panics with a message the tests pin. Note the `debug_assert_eq!(k, 0)` after the
loop: since `u ≥ v` the final borrow is provably zero, and the assertion
documents that reasoning rather than trusting it silently.

## Stage 2: Classical multiplication

`big_mul` is Algorithm M transcribed step-for-step, and its correctness is one
inequality: `t = uᵢ·vⱼ + w_{i+j} + k ≤ b² − 1 < b²`, so the whole accumulation
fits a single `u64` and `k = t >> 32` is a legal single-limb carry. The design
choice to zero the entire `m+n` product area up front (rather than growing it)
means the inner loop can read-modify-write `w[i+j]` freely, and the `if v[j] == 0
{ continue; }` micro-optimization skips an all-zero inner pass without breaking
the layout. `big_to_decimal` is the quietly clever part: instead of dividing by
10 a digit at a time, it divides the whole limb vector by `10⁹` per pass — the
largest power of ten under 2³² — so each short-division sweep (most-significant
limb first, carrying the remainder downward via `(rem << 32) | *d`) yields nine
decimal digits at once. The top chunk prints bare and the rest zero-padded to
width 9; getting that padding wrong is the classic bug that drops interior
zeros, which is exactly why the tests decode `10!` and check the full string.

## Stage 3: Faster multiplication by divide and conquer

`big_mul_karatsuba` shows the payoff of building the primitives first: the entire
divide-and-conquer body is written in terms of `big_add`, `big_sub`, and
`big_mul` from the earlier stages. The subtractive form `z1 = mid − z2 − z0`
computes the cross term `u₁v₀ + u₀v₁` with one multiplication instead of two, and
because that quantity is provably nonnegative the reuse of `big_sub` (which panics
on negatives) doubles as a correctness check. The `KARATSUBA_CUTOFF = 32` guard
is not a detail — it is the whole point of "classical before clever": below a few
dozen limbs the recursion's three calls, additions, and allocations cost more
than the digit products they save, so it falls back to `big_mul`, and the base
case is what makes the recursion terminate. `split_at_limb` strips trailing zeros
from the low half so recursive inputs stay canonical (the high half is already
canonical because the parent was), and `shift_limbs` maps "multiply by `b^p`" to
"prepend `p` zero limbs", with the special case that shifting zero stays the
empty vector. This is a miniature of how GMP is structured.

## Stage 4: The binary gcd algorithm

`binary_gcd` is Stein's algorithm, and the reference's cleverness is the *signed*
helper `t: i128` that lets one variable stand for both "the current `u`" (when
positive) and "minus the current `v`" (when negative), collapsing Knuth's two
symmetric cases into one loop. The `k` counter accumulates the common factor of
two pulled out in B1, restored at the end as `u << k`. The subtle control-flow
point the reference handles with `entering_at_b4` is that step B2 sometimes jumps
*into* the middle of the halving loop (when `u` is odd, `t = −v` and we must test
parity before halving) — a naive `loop { t /= 2; ... }` would wrongly halve an
odd `t` on the first pass. The termination argument is that every trip through
B3–B6 halves `|t|` at least once and never increases `max(u,v)`, giving
`O(log uv)` shift/subtract steps and no division at all — which is why
constant-time variants of this live inside crypto libraries where a division's
data-dependent timing would leak key bits.

## Stage 5: Probabilistic primality testing

The Stage 5 functions stack cleanly: `mul_mod` promotes to `u128` so a full
64-bit product never overflows; `pow_mod` is right-to-left binary exponentiation
built on it (`1 % m` initializes correctly even for `m = 1`); and
`is_strong_probable_prime` is the Miller–Rabin witness test on top of `pow_mod`.
The heart of the witness test is the √1 argument made operational: after
`x = a^d mod n`, the loop squares repeatedly, and the *only* way to pass is to
start at 1 or hit `n−1` before reaching 1 — because a square that lands on 1
without having passed through `−1` would be a nontrivial square root of 1, which
a prime modulus forbids. So "never saw `n−1`" simultaneously catches both a
nontrivial root and an outright Fermat failure. The idiom worth stealing is
`is_prime_u64`'s two-phase structure: handle the twelve witness primes and their
multiples directly (so tiny inputs and the bases themselves are correct), then
require *all twelve* strong tests — a proven deterministic set for the entire
`u64` range, turning a probabilistic algorithm into an exact one. The tests pin
the instructive traps: 2047 fools base 2 alone, 561 fools Fermat but not the
strong test, and `2⁶¹−1` is prime.
