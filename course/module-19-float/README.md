# Module 19 — Floating-Point Arithmetic

> **Source:** *The Art of Computer Programming*, Vol. 2, 3rd ed., §4.2
> (§4.2.1 the operations, §4.2.2 the accuracy of floating-point arithmetic).
> **Lab:** `labs/module-19-float` · **Grade it:** `./grade 19`
>
> Self-contained: you can complete the module without the book. If you own
> Vol. 2, read §4.2.1–§4.2.2 alongside; the lesson says where to look.

Real numbers are a mathematician's fiction: there are uncountably many of them
and your machine has 64 bits. *Floating point* is the engineering compromise
that lets those 64 bits stand in for the reals across sixty orders of
magnitude — and it is a **leaky abstraction**. It looks like arithmetic, it
mostly behaves like arithmetic, and then one day `0.1 + 0.2 != 0.3`, a Patriot
missile misses its target, and your unit test comparing two floats with `==`
fails on a different CPU. Knuth's answer, and ours, is not to fear floating
point but to *analyze* it: know exactly what each operation does, and carry an
error bound the way a chemist carries significant figures.

By the end of this module you will have built a working binary floating-point
system — pack/unpack, add, subtract, multiply, divide, all correctly rounded —
that agrees with your hardware `f64` **bit for bit**, and you will have proved
the theorems that make numerical computing trustworthy: the fundamental
rounding bound, the non-associativity of `+`, and why Kahan's summation
recovers the bits everyone else throws away.

---

## 1. Positional fractions and the normalized form

Fix a base `b` (we use `b = 2`) and a precision `p` (we use `p = 53`, matching
IEEE 754 binary64). A **floating-point number** is a value

```text
    x = (-1)^s · f · b^e
```

where `s ∈ {0,1}` is the sign, `e` is an integer **exponent**, and `f` is the
**significand** (Knuth calls it the *fraction*), a p-digit base-b number. The
"floating" point is that `e` slides the radix point to wherever the value needs
it: the same p significant digits describe `6.022·10²³` and `1.602·10⁻¹⁹`.

Two different (f, e) pairs can denote the same value — `0.5 = 1·2⁻¹ =
0.1·2⁰` — so we impose a canonical form. A nonzero number is **normalized**
when its leading significand digit is nonzero; in binary that means the leading
bit is 1. We store the 53-bit significand as an *integer* `frac` with its
leading 1 pinned at bit 52:

```text
    2^52 ≤ frac < 2^53,        value = (-1)^sign · frac · 2^exp
```

so `2^exp` is precisely the weight of the **last** significand bit — the *unit
in the last place*, or **ulp**. Two consecutive representable numbers of the
same exponent differ by exactly one ulp.

**Why normalize?** Three reasons, all of which you will feel in the lab:

1. *Uniqueness.* Every nonzero value has exactly one representation, so
   equality and comparison are trivial.
2. *Maximum precision.* A leading zero bit would waste one of your 53 bits of
   accuracy. Normalizing keeps all of them significant.
3. *A free bit.* Because the leading bit is *always* 1, IEEE 754 doesn't even
   store it — the "implicit leading bit" trick buys a 53rd bit of precision
   from a 52-bit field. Our `from_f64` restores that `1`; `to_f64` strips it.

Zero is special: it has no nonzero leading digit, so it can't be normalized.
We represent it as `frac == 0`, and keep the sign bit so that `+0` and `−0` are
distinguishable (they matter at the boundary of underflow and for `1/x`).

The classic cautionary example lives here already. **One tenth has no finite
binary expansion**: `1/10 = 0.0001100110011…₂`, the block `0011` repeating
forever, exactly as `1/3 = 0.333…` never terminates in decimal. So `from_f64(0.1)`
cannot equal `1/10`; it is the *nearest 53-bit number* to `1/10`, off by less
than half a ulp. Every "floating-point is broken" bug report traces back to
this one fact.

---

## 2. The biased (excess) exponent

The exponent `e` can be negative, but hardware likes to compare and sort
numbers by looking at their raw bits as unsigned integers. IEEE stores a
**biased** exponent: an 11-bit field `E` holding `E = e_ieee + 1023`, where
`e_ieee` is the exponent of the value written as `1.fff… · 2^{e_ieee}`. The
bias `1023 = 2¹⁰ − 1` is the "excess" that makes every stored exponent
non-negative, so that (for numbers of the same sign) larger magnitude means a
larger bit pattern — integer comparison sorts floats.

Our model keeps a *signed* `exp: i32` for clarity, but `from_f64`/`to_f64` must
translate. If a binary64 has biased field `E` and 52 stored mantissa bits `m`,
then for a normal number

```text
    value = 1.m · 2^{E-1023} = (2^52 + m) · 2^{E-1023-52},
```

so our `frac = 2^52 + m` and our `exp = E − 1023 − 52 = E − 1075`. Going back,
`E = exp + 1075` and `m = frac − 2^52 = frac & (2^52 − 1)`. The three reserved
patterns — `E = 0` (zero and subnormals) and `E = 2047` (∞ and NaN) — mark the
edges of the representable world; we handle zero and treat the rest as out of
scope for this finite model.

---

## 3. Rounding: nearest, ties to even

When an exact result needs more than 53 bits, we must **round**. IEEE's default
(and ours) is *round to nearest, ties to even* (RNE):

- round to the closest representable number;
- if exactly halfway, round to the one whose last bit is **0** (even).

To round correctly you cannot simply chop; you must know three things about the
discarded tail, the celebrated **guard / round / sticky** bits:

- the **guard/round bit** `r`: the first bit past the ones you keep;
- the **sticky bit** `t`: the OR of *all* bits below the round bit — "is there
  anything else down there?"

Then, keeping value `q` with last kept bit `q₀`:

```text
    round up  ⟺  r = 1 AND (t = 1 OR q₀ = 1).
```

If `r = 0` you're below halfway → round down. If `r = 1, t = 1` you're above
halfway → round up. If `r = 1, t = 0` you're *exactly* halfway (a tie) → round
to even, i.e. up iff `q₀ = 1`. Our `round_wide` computes exactly these from a
wide integer: the round bit is bit `s−1`, the sticky is "any bit below `s−1`
set", where `s` is how many bits we're dropping.

**Why ties to even? A no-drift argument.** Suppose you always rounded ties *up*
("round half up", the schoolbook rule). Then over many operations the tiny
half-ulp errors would all have the same sign and **accumulate**: sum a million
numbers and your bias marches steadily away from the truth, growing like `n`.
Rounding ties to even sends half the ties up and half down (the last bit is
equidistributed between 0 and 1 over generic data), so the errors are **mean-zero**
and tend to cancel — the accumulated bias grows like `√n` rather than `n`, by
the same central-limit reasoning that makes a random walk stay near the origin.
"To even" specifically (rather than randomly) also has the pleasant property
that `round(round(x, p+1), p) = round(x, p)` — rounding in two steps agrees
with rounding once, so it composes without introducing *double-rounding* error.
That is why every serious system defaults to it.

---

## 4. Algorithm A — addition (and subtraction) — §4.2.1

Adding two floats is not "add the significands." Their radix points are in
different places (different exponents), so you must **align** first. Knuth's
Algorithm A, in our binary dress:

```text
A1. [Unpack.]         Extract signs, exponents, significands of u and v.
A2. [Assume e_u ≥ e_v.]  If not, swap u ↔ v so u has the larger exponent.
A3. [Set e ← e_u.]    The result's tentative exponent.
A4. [Scale right.]    Shift v's significand right by (e_u − e_v) places,
                      catching the bits that fall off in guard/round/sticky.
A5. [Add or subtract.]  If signs agree, add the aligned significands; else
                      subtract the smaller from the larger; sign follows the
                      larger magnitude.
A6. [Normalize.]      Renormalize the result (it may have grown a bit on carry
                      or lost many on cancellation) and round to p bits (step N).
```

The subtlety is A4. If `v` is much smaller, shifting it right by (say) 200
places would need a 250-bit register. But you don't need every bit — you need
the value *rounded* to 53 bits, and rounding only cares about guard + round +
sticky. So once `v` has shifted past the round position, all its remaining bits
just OR into the sticky bit. Our implementation keeps a fixed 64-bit guard
window exact and collapses everything beyond it into sticky — provably enough,
because a `v` shifted more than 64 places cannot reach even the round bit.

Subtraction is just `u − v = u + (−v)`; negate `v`'s sign and call add.

### Hand-trace: aligning and rounding

Let's add, in a toy `p = 4`-bit binary system (leading 1 explicit, RNE):

```text
    u =  1.011₂ · 2^3   =  1011₂ · 2^0   = 11
    v =  1.101₂ · 2^0   =  1101₂ · 2^{-3} = 1.625
```

**A2/A3.** `e_u = 3 ≥ e_v = 0`, so `e = 3`, shift = 3.

**A4.** Shift `v` right 3 places, tracking the tail:

```text
    v significand:  1.101       (at 2^0)
    aligned to 2^3: 0.001 101   → keep "0.001", tail bits "101"
                    guard r = 1, sticky t = (0 OR 1) = 1
```

**A5.** Same sign, add the kept parts at exponent 3:

```text
       1.011
     + 0.001
     -------
       1.100     with pending round bits r=1, t=1  (we are above halfway)
```

**A6 (step N).** `r = 1, t = 1` → round up the last kept bit:

```text
       1.100  + 0.001  =  1.101   · 2^3  =  1101₂ · 2^0 = 13
```

The exact sum is `11 + 1.625 = 12.625`; the nearest 4-bit float is `13` (the
representable neighbours are `12 = 1.100·2^3` and `13 = 1.101·2^3`, and
`12.625` is closer to `13`). Round-to-nearest did its job. Notice how the guard
and sticky bits, *not* the discarded value itself, drove the decision.

---

## 5. Algorithm M — multiplication — and division — §4.2.1

Multiplication is the easy one, because significands multiply cleanly:

```text
M1. [Unpack.]     sign ← s_u XOR s_v;  e ← e_u + e_v.
M2. [Multiply.]   The product of two 53-bit integers is at most 106 bits —
                  hold it exactly in a 128-bit register. No information lost.
M3. [Normalize.]  Renormalize the ~106-bit product to 53 bits (step N) and
                  round. Exponents simply added; only the significand rounds.
```

Because the exact product fits in `u128`, multiplication is *exact before
rounding* — the single rounding at M3 makes it correctly rounded. Multiplying
by a power of two touches only the exponent: `x · 2^k` shifts `exp` by `k` with
no rounding at all, a fact the tests check.

**Division** is the hard one, because a quotient of two finite binaries is
generally *not* finite (`1/3` again). We compute enough bits and round:

```text
    q = ⌊ frac_u · 2^64 / frac_v ⌋,     r = (frac_u · 2^64) mod frac_v.
```

Sixty-four extra low bits is far more than the two (round + sticky) that
rounding needs, and the remainder `r ≠ 0` tells us there is *more* below — it
**is** the sticky bit. Then step N rounds `q` at exponent `e_u − e_v − 64`. The
result is correctly rounded, matching hardware `/` to the last bit.

### Hand-trace: `3.0 / 7.0` in binary64

`3 = 1.1₂·2¹` (`frac = 3·2^51`, `exp = −50`), `7 = 1.11₂·2²`. The exact
quotient `3/7 = 0.011011011…₂` repeats forever. Long division produces
`0.01101101101101…`; normalized that's `1.1011011…·2⁻²`. Keep 53 bits, look at
bit 54 (the round bit) and the infinite tail (sticky = 1, since the pattern
never terminates), apply RNE. You get exactly `f64::from_bits((3.0f64/7.0).to_bits())` —
which the lab asserts over 40000 random pairs.

---

## 6. Error analysis — the unit roundoff — §4.2.2

Here is the theorem that makes numerical analysis possible.

**Definition.** The **unit roundoff** is `u = 2^{-p} = 2^{-53}` in the "distance
to the tie" convention, or (as we and much C use it) `u = 2^{-(p-1)} = 2^{-52}`,
the gap `ulp(1)` between `1` and its successor. We take `machine_epsilon() =
2^{-52}`: the smallest `ε` with `1 + ε ≠ 1`, while `1 + ε/2 = 1`.

**Theorem (fundamental bound of floating-point arithmetic).** Let `op` be one of
`+ − × ÷`, let `x, y` be representable, and suppose `x op y` is in the normal
range. Then the computed result satisfies

```text
    fl(x op y) = (x op y)(1 + δ)      for some δ with |δ| ≤ u/2.
```

*Proof sketch.* `fl` returns the representable number nearest `x op y`. Adjacent
representables straddle `x op y` at a spacing of one ulp of the result, so the
nearest is within half a ulp: `|fl(x op y) − (x op y)| ≤ (1/2)·ulp(x op y)`. For
a normalized result `z`, `ulp(z) ≤ u·|z| = 2^{-52}|z|` (the last bit's weight is
at most `2^{-52}` of the value). Combine: `|fl − exact| ≤ (u/2)|exact|`, i.e.
`δ = (fl − exact)/exact` has `|δ| ≤ u/2`. ∎

This "`(1+δ)` model" is the entire engine of rounding-error analysis: you carry
one `(1+δ)` per operation and bound the product `∏(1+δ_i)`. The lab verifies the
bound *empirically and exactly* using **TwoSum** — Knuth's trick that, for `s =
fl(a+b)`, computes the exact rounding error `err = (a − (s − (s − a))) + (b − (s
− a))` with no extra precision, so `a + b = s + err` exactly (in the absence of
overflow). You then check `|err| ≤ (u/2)|s|` over 60000 random pairs.

### Why floating-point addition is NOT associative

The `(1+δ)` per operation means the *order* of operations changes the answer.
Concretely, with `h = 2^{-53}` (half a ulp at 1):

```text
    (1 + h) + h :  1 + h ties to even → 1 ;  then 1 + h ties again → 1.
    1 + (h + h) :  h + h = 2^{-52} = ulp(1) exactly ;  1 + ulp = 1 + 2^{-52}.
```

So `(1 + h) + h = 1` but `1 + (h + h) = 1 + 2^{-52}`. **Different answers from
the same three numbers.** Addition is commutative but not associative; the
lesson (and every compiler's `-ffast-math` footgun) is that you may not freely
reorder floating-point sums. The lab reproduces this on both our `Float` and
hardware `f64` — they disagree with associativity in exactly the same way.

### Catastrophic cancellation

When you subtract two nearly-equal numbers, the leading bits cancel and the
result is built entirely from the *low, error-contaminated* bits — the relative
error can explode even though each operation obeyed the fundamental bound.
`(1 + ε) − 1` recovers `ε` cleanly when `ε` is representable at 1's scale, but
compute `ε` as `fl(1 + tiny) − 1` with `tiny < u/2` and you get **0**: the tiny
term was rounded away by the addition, and the subtraction has nothing left to
reveal. The fix is never to *form* the cancelling difference — rearrange the
algebra (e.g. `√(x+1) − √x = 1/(√(x+1) + √x)`) so the catastrophic subtraction
never happens. This is the single most important practical lesson of §4.2.2.

---

## 7. Kahan summation — buying back the lost bits

Summing `n` numbers naively accumulates one rounding error per addition; in the
worst case the total error grows like `n·u` times the running magnitude. **Kahan's
compensated summation** (1965) keeps a running *correction* `c` that holds the
low-order bits dropped by the previous addition and feeds them back:

```text
    s ← 0 ;  c ← 0
    for each x:
        y ← x − c            # add back what we lost last time
        t ← s + y            # rounds; loses the low part of y
        c ← (t − s) − y      # exactly the low part just lost (TwoSum!)
        s ← t
    return s
```

**Why `c` is exactly the lost bits (first-order proof).** Look at the third
line. In exact real arithmetic `(t − s) − y = 0`; the reason it isn't zero in
floating point is precisely that `t = fl(s + y)` discarded some low bits of the
true `s + y`. When `|s| ≥ |y|` (the usual case — you're adding small terms to a
large running total), `t − s` is computed *exactly* (Sterbenz's lemma: the
difference of two floats within a factor of two of each other is exact, and
more generally the leading cancellation here is benign), so `(t − s) − y`
evaluates to the exact difference `(s + y) − t = −(the part of s+y that t threw
away)`. Thus `c` holds, to first order in `u`, exactly the rounding error of the
step — and subtracting it next iteration cancels that error. The upshot is a
total error bound of `O(u)·Σ|x_i|` **independent of n**, versus `O(nu)·Σ|x_i|`
for naive. The lab makes this vivid: sum `0.1` a hundred thousand times and
Kahan lands on `10000` to the bit while naive drifts; sum `[10¹⁶, 1, 1, …, 1,
−10¹⁶]` and naive returns `0` (every `1` was swamped) while Kahan recovers all
ten thousand ones.

Kahan does about 4× the flops per element (the lab's benchmark shows the
constant factor), which is why it is a *choice* — but when accuracy matters it
is nearly free next to the alternative of using double the precision.

---

## 8. Knuth's warning: a leaky abstraction

Knuth is blunt in §4.2: floating-point arithmetic *resembles* real arithmetic
but obeys different laws, and "the programmer who ignores this fact does so at
his peril." Associativity fails. Distributivity fails. `x == y` can be false for
two computations that are mathematically equal. The abstraction leaks, and the
only defence is *analysis*: know the error bound of every operation, and design
so the bounds stay small. That is the mindset this module trains.

---

## 9. Stage-by-stage lab guide

Open `labs/module-19-float/src/lab.rs`. Run `./grade 19`; stages run in order,
stopping at the first failure. The model is `value = (−1)^sign · frac · 2^exp`
with `frac` a 53-bit significand, normalized to bit 52.

### Stage 1 — Representation: pack, unpack, normalize

Implement `new` (raw pack), `zero`, `normalize` (shift the leading 1 to bit 52),
`from_f64`/`to_f64` (the bias arithmetic of §2), `ulp` (return `2^exp`), and
`classify`. Keep `from_f64` and `to_f64` exact inverses on the normal range —
that bit-for-bit round-trip is what lets every later stage cross-check against
hardware. The `0.1` test encodes the "not exactly 1/10, but within a ulp" fact
as an *integer* inequality `|10·frac − 2^{-exp}| < 10`, so no bignums needed.

### Stage 2 — Addition and subtraction (Algorithm A)

Implement `add` and `sub`. Align to the larger exponent (A2–A4), keeping a
64-bit guard window and a sticky bit; combine by sign (A5); renormalize and
round (A6, RNE). The tests demand bit-exact agreement with `f64 +`/`−` — even
in the huge-plus-tiny case — plus the round-to-even ties, non-associativity,
and catastrophic-cancellation phenomena as concrete assertions.

### Stage 3 — Multiplication and division (Algorithm M)

Implement `mul` (exact 106-bit product in `u128`, then round) and `div` (scaled
long division with the remainder as sticky). Bit-exact vs hardware `*` and `/`;
multiply-by-power-of-two exact; `x/x = 1`; sign rules; and a `should_panic` on
division by zero (∞ is out of scope). The single rounding after an *exact* wide
product/quotient is the whole trick to correct rounding.

### Stage 4 — Error analysis: ulps and compensated summation

Implement `machine_epsilon` (`2^{-52}`), `ulp_f64`, `ulp_error`, `naive_sum`,
`kahan_sum`. The tests verify the fundamental bound via TwoSum, the
`1 + ε ≠ 1 = 1 + ε/2` threshold, and that Kahan crushes naive on two adversarial
inputs. This is where representation turns into *numerical analysis*.

---

## 10. Check your understanding

1. Why can't `0.1` be stored exactly in binary64, and how far from `1/10` is the
   stored value? (Repeating expansion; within half a ulp = under `2^{-56}`.)
2. In `1.0 + 2^{-53}`, what are the guard, round, and sticky bits, and which way
   does RNE go? (Round bit 1, sticky 0 → a tie; last kept bit of `1.0` is 0
   (even) → round down → `1.0`.)
3. Give three floats `a, b, c` with `(a+b)+c ≠ a+(b+c)` and explain in one
   sentence. (E.g. `1, 2^{-53}, 2^{-53}`: the two half-ulps round away
   separately but survive when added first.)
4. Why does multiplication need only *one* rounding to be correctly rounded,
   while a naive division might need several? (The exact product fits in 106
   bits; the exact quotient is infinite, so you must generate guard/round/sticky
   yourself.)
5. In Kahan's loop, what does `c` hold after the `n`-th step, and why does
   subtracting it next time help? (The low bits lost when `t = fl(s+y)` rounded;
   feeding them back cancels that step's error to first order.)

## 11. Exercises from the text

Ratings are Knuth's: 00 immediate · 10 a minute · 20 up to an hour · 30 hours ·
40 term project · 50 research. ▶ marks especially instructive ones. Log your
work in `course/module-19-float/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 4.2.1-1 | 10 | Give the normalized binary form of a few decimal fractions; which are exact? |
| ▶4.2.1-3 | 20 | Show why the sticky bit (not just guard+round) is needed for correct rounding of addition. |
| 4.2.1-6 | 22 | When can `x · y` overflow even though `x` and `y` are well inside range? Analyze the exponent sum. |
| ▶4.2.2-9 | 25 | Prove Sterbenz's lemma: if `y/2 ≤ x ≤ 2y` then `x − y` is computed exactly. |
| 4.2.2-15 | 28 | Bound the error of naive summation of `n` terms; then Kahan's, showing the `n`-independence. |
| ▶4.2.2-21 | 30 | Analyze `(a+b)+c` vs `a+(b+c)`: characterize when they differ and by how much. |

## Why it's done this way

- **Normalized significand + separate exponent** gives one representation per
  value (cheap comparison), maximum precision (no wasted leading zeros), and the
  implicit-leading-bit trick that buys a free bit — the whole reason IEEE packs
  53 bits of precision into 52 stored.
- **A biased exponent** makes the raw bit pattern sort in numeric order, so
  hardware can compare floats with integer instructions.
- **Round to nearest, *even*** is the unique tie rule that is unbiased (errors
  cancel, drift is `√n` not `n`) *and* composes without double-rounding — which
  is why it is everyone's default.
- **Guard/round/sticky** is the minimum state that makes rounding correct after
  a shift or an infinite quotient; carry those three bits and you never need the
  whole discarded tail.
- **Exact-then-round** (106-bit products, 64-guard-bit quotients) is the clean
  road to *correctly rounded* operations: do the math exactly in a wider
  integer, round exactly once.

## In the real world

- **IEEE 754 is everywhere** — every `f32`/`f64` in every language, GPU, and
  phone implements exactly the pack/round/align machinery you just built. Your
  `Float` agrees with it bit for bit precisely because the algorithm is the same.
- **The Patriot missile failure (Dhahran, 1991)** killed 28 soldiers because the
  system tracked time in tenths of a second, and `0.1`'s tiny representation
  error — the very fact from §1 — accumulated over 100 hours into a third of a
  second of clock drift, enough to miss an incoming Scud. Floating-point error
  analysis is not academic.
- **Kahan (and pairwise) summation lives in NumPy and pandas**: `numpy.sum` uses
  pairwise summation for exactly the accuracy reasons in §7, and libraries expose
  compensated variants (`math.fsum` in Python is fully exact). When you `df.mean()`
  a million rows, someone did this analysis so you didn't drift.
- **`==` on floats is a bug** in almost every context: two mathematically equal
  computations can differ in their last bits (non-associativity, different
  evaluation order, fused multiply-add). Compare with a tolerance, or compare
  the exact quantities you actually care about.
- **ML trains in bf16/fp16** — 8 and 11 significand bits, a tiny fraction of
  binary64. It works only because the community re-derived §4.2.2 for low
  precision: loss scaling to dodge underflow, `float32` master weights,
  Kahan-style compensated optimizer accumulators, and stochastic rounding to
  keep gradient sums unbiased. Every one of those tricks is this module's theory
  applied where the ulps are enormous.

## Proof techniques you practiced

- **Nearest-point + spacing bound** — the fundamental theorem falls out of
  "adjacent representables are one ulp apart," turned into `|δ| ≤ u/2`.
- **The `(1+δ)` model** — replace each rounded operation by an exact one times
  `(1 + δ)`, `|δ| ≤ u/2`, then bound the accumulated product. The backbone of all
  rounding-error analysis.
- **Exact error extraction (TwoSum/TwoProduct)** — recover the rounding error
  itself in floating point with no extra precision; used both to *verify* the
  bound and to *build* Kahan summation.
- **Bias/no-drift argument** — the mean-zero, random-walk reasoning that makes
  round-to-even's error grow like `√n` instead of `n`.
- **Counterexample by construction** — non-associativity proved by exhibiting
  `1, 2^{-53}, 2^{-53}` and tracing both groupings.
- **Invariant maintenance** — Kahan's `c` maintains "the bits lost so far,"
  proved preserved each iteration (a loop invariant, as in Module 01).

## 12. Where this leads

- **Interval and higher-precision arithmetic** (double-double, the `TwoSum`/
  `TwoProduct` "error-free transformations") build directly on stage 4 to get
  guaranteed bounds and extra digits from ordinary hardware.
- **Numerical linear algebra** — the `(1+δ)` model is how you prove Gaussian
  elimination, QR, and iterative solvers are backward-stable; the conditioning of
  a problem times the stability of an algorithm bounds the error.
- **The rest of Vol. 2** — §4.2.3 (double precision), §4.2.4 (distribution of
  floating-point numbers, and why leading digits obey Benford's law), and the
  polynomial and power-series evaluation of §4.6 all assume the fluency you built
  here.
- **Reproducibility** — deterministic, order-independent summation (superaccumulators)
  is an active systems topic in HPC and blockchain, and it starts from exactly the
  non-associativity you demonstrated.
