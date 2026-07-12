# Exercises — Module 19 (Floating-Point Arithmetic, §4.2)

Self-contained problems on this module's material — exact vs. inexact binary
fractions, the guard/round/sticky machinery of correct rounding, exponent
overflow in multiplication, Sterbenz's exactness lemma, error bounds for naive
and Kahan summation, and the non-associativity of floating-point addition. You
can work every one **without the book**: each states the problem in full, gives
a **hint** to peek at when stuck, and a worked **answer sketch** to check
against. Numeric answers here are reproduced by the `Float` you build in the lab
(or a few lines at a REPL), and they agree with hardware `f64` bit for bit.

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Each problem cites the section of Vol. 2 (§4.2.1 / §4.2.2) whose
material it exercises. Throughout,
$p = 53$ is the binary precision, $u = 2^{-52} = \operatorname{ulp}(1)$ is the
machine epsilon (the gap $1 \to 1^{+}$), and the unit roundoff is
$u/2 = 2^{-53}$.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Which decimal fractions are exact in binary? | 10 | ⬜ |
| 2 | ▶ Why the sticky bit is necessary for correct rounding | 20 | ⬜ |
| 3 | Overflow from the exponent sum in multiplication | 22 | ⬜ |
| 4 | ▶ Sterbenz's lemma: $x - y$ exact when $y/2 \le x \le 2y$ | M25 | ⬜ |
| 5 | Error bounds for naive vs. Kahan summation | M28 | ⬜ |
| 6 | ▶ When does $(a{+}b){+}c$ differ from $a{+}(b{+}c)$, and by how much? | M30 | ⬜ |

## Problems

### 1. Which decimal fractions are exact in binary? (rating 10 · cf. §4.2.1)

**Problem.** Give the normalized binary form $(-1)^s \cdot f \cdot 2^e$ (leading
significand bit $= 1$) of $0.5$, $0.75$, $0.375$, and $0.1$. Which of these — and
more generally which decimal fractions $m/n$ in lowest terms — are represented
*exactly* by a finite binary significand, and which are not?

**Hint.** A number has a finite binary expansion iff, written in lowest terms
$m/n$, its denominator $n$ divides some power of two — i.e. $n$ is itself a power
of two. Factor each denominator. Recall §1's headline fact: $1/10 =
0.0001100110011\ldots_2$ repeats forever.

**Answer sketch.** $0.5 = 1.0_2 \cdot 2^{-1}$ (exact), $0.75 = 1.1_2 \cdot
2^{-1}$ (exact), $0.375 = 3/8 = 1.1_2 \cdot 2^{-2}$ (exact). But $0.1 = 1/10$ is
**not** exact: $10 = 2 \cdot 5$ has an odd prime factor $5$, so no power of two
is a multiple of $10$, and the expansion $0.0001100110011\ldots_2$ (block `0011`
repeating) never terminates. General rule: $m/n$ in lowest terms is exactly
representable in binary iff $n = 2^k$ for some $k \ge 0$ — equivalently $n$ has no
odd prime factor. So $0.25, 0.75, 0.125, 0.375$ (denominators $4, 4, 8, 8$) are
exact, while $0.1, 0.2, 0.3, 0.6$ (denominators $10, 5, 10, 5$) are not. This is
why `from_f64(0.1)` stores the *nearest* 53-bit number to $1/10$, within half a
ulp — the root of every "floating point is broken" bug report.

### 2. ▶ Why the sticky bit is necessary for correct rounding (rating 20 · cf. §4.2.1)

**Problem.** Round-to-nearest-ties-to-even needs, beyond the kept bits, a
**round bit** $r$ (first discarded bit) *and* a **sticky bit** $t$ (OR of all
bits below $r$). Show by a concrete example that $r$ alone is not enough: exhibit
two additions with identical kept bits and identical round bit but different
correct results, distinguished only by $t$. Explain why alignment in Algorithm A
makes $t$ exactly the right summary of the discarded tail.

**Hint.** The round bit alone cannot tell an *exact tie* (round bit $1$, nothing
below) from a value *just above* the midpoint (round bit $1$, something below).
Those two round differently when the last kept bit is even. Build both as
$1.0 + (\text{tiny})$ where the tiny addend places a $1$ at the round position.

**Answer sketch.** Work at the scale of $1.0$, where $\operatorname{ulp}(1) =
2^{-52}$ and the round position is bit $2^{-53}$ (half a ulp). Compare two
*single* correctly-rounded additions:
- $1.0 + 2^{-53}$: the exact sum is $1.\underbrace{0\ldots0}_{52}\,1_2$ — round
  bit $r = 1$, sticky $t = 0$: an **exact tie**. Ties-to-even keeps the last bit
  even, and $1.0$'s last kept bit is $0$, so it rounds **down** to $1.0$.
- $1.0 + (2^{-53} + 2^{-57})$: the addend is one representable value; the exact
  sum has round bit $r = 1$ *and* a nonzero bit at $2^{-57}$, so $t = 1$ — it is
  **above** the midpoint and rounds **up** to $1 + 2^{-52} = $ `1.0000000000000002`.

Both have identical kept bits and identical round bit $r = 1$; only $t$ differs,
and it flips the result. (Verified on hardware `f64`: the two single additions
yield `1.0` and `1.0000000000000002`.) Without the sticky bit you would treat
the second case as a tie and wrongly round it down. Why $t$ suffices: RNE's
decision depends on *whether the exact value is below, at, or above* the
midpoint, and "above" is detected purely by "is any discarded bit below $r$
set?" — that single OR is $t$. In Algorithm A's alignment step A4, a small
addend shifted right by many places dumps all its bits past the round position;
collapsing every one of them into $t$ loses nothing rounding cares about, which
is why the implementation keeps a fixed guard window and ORs the rest into
sticky.

### 3. Overflow from the exponent sum in multiplication (rating 22 · cf. §4.2.1)

**Problem.** In Algorithm M the exponents simply add: $e = e_u + e_v$. Show that
$x \cdot y$ can **overflow** (exceed the largest finite float) even when both $x$
and $y$ are comfortably inside the representable range. Characterize, in terms of
the exponents, exactly when this happens, and give a concrete binary64 example.

**Hint.** binary64 writes normals as $1.f \cdot 2^{e_{\text{ieee}}}$ with
$-1022 \le e_{\text{ieee}} \le 1023$; the largest finite value is just under
$2^{1024}$. The product's exponent is (about) $e_{u,\text{ieee}} +
e_{v,\text{ieee}}$, so two mid-range magnitudes can push the sum past the top.

**Answer sketch.** Since significands lie in $[1, 2)$, $x \cdot y = (1.f_u \cdot
1.f_v) \cdot 2^{e_{u} + e_{v}}$ with the significand product in $[1, 4)$, so the
result's exponent is $e_u + e_v$ or $e_u + e_v + 1$ (a normalizing carry).
Overflow occurs when this exceeds $1023$, i.e. roughly when $e_u + e_v \ge
1024$ — a condition on the **sum** of exponents, each of which may individually
be far below the ceiling. Concretely, $x = y = 2^{512}$ are both finite (well
under $2^{1024}$), yet $x \cdot y = 2^{1024}$ overflows to $\infty$; likewise
$x = y = 2^{600}$ give $2^{1200} = \infty$ (verified on hardware). So "$x$ and
$y$ in range" does **not** imply "$x \cdot y$ in range": the danger lives in the
additive exponent, not in either operand. (The mirror failure is *underflow*
when $e_u + e_v$ drops below $-1022$; both are why robust code scales operands or
uses $\log$-domain products. The significand rounding at M3 is exact-then-round
and never itself causes overflow beyond that one carry bit.)

### 4. ▶ Sterbenz's lemma: $x - y$ exact when $y/2 \le x \le 2y$ (rating M25 · cf. §4.2.2)

**Problem.** Prove **Sterbenz's lemma**: if $x$ and $y$ are floating-point
numbers of the same sign with $y/2 \le x \le 2y$ (equivalently, within a factor
of two of each other), then the exact difference $x - y$ is itself a
floating-point number — so $\mathrm{fl}(x - y) = x - y$ with *no* rounding error.

**Hint.** Assume WLOG $0 < y \le x \le 2y$. Both $x$ and $y$ are integer
multiples of the smaller number's ulp; show $x - y$ is a multiple of that same
ulp and small enough in magnitude to be representable at that scale (no bits
below the ulp, and it fits in $p$ significant bits).

**Answer sketch.** WLOG take $0 < y \le x$ (same sign; the difference's sign is
handled symmetrically). The hypothesis gives $x \le 2y$. Write $y = f_y \cdot
2^{e}$ where $e$ is $y$'s exponent (ulp $= 2^{e}$) and $2^{p-1} \le f_y < 2^{p}$.
Since $y \le x \le 2y < 2^{e+1} \cdot 2^{p}$, the number $x$ has exponent $e$ or
$e+1$, so **$x$ is an integer multiple of $2^{e}$** — the ulp of $y$ — as is $y$.
Hence $d = x - y$ is an integer multiple of $2^{e}$: it has no bits below
position $e$. And $0 \le d = x - y \le 2y - y = y < 2^{e+p}$, so $d$ needs at
most $p$ significant bits above position $e$. A value that is a multiple of
$2^{e}$ and lies in $[0, 2^{e+p})$ is exactly representable with a $p$-bit
significand and exponent $\le e$. Therefore $x - y$ is a float and the machine
subtraction is exact. $\blacksquare$ (Empirically: over $200{,}000$ random pairs
with $y/2 \le x \le 2y$, $\mathrm{fl}(x - y)$ matches the exact rational
difference every time.) This benign-cancellation fact is exactly what makes the
$t - s$ step in Kahan summation (Exercise 5) recover the lost bits precisely.

### 5. Error bounds for naive vs. Kahan summation (rating M28 · cf. §4.2.2)

**Problem.** Let $x_1, \ldots, x_n$ be floats summed left to right. Using the
$(1+\delta)$ model — each addition returns the exact sum times $(1 + \delta)$
with $|\delta| \le u/2$ — bound the absolute error of **naive** summation, then of
**Kahan compensated** summation, and show the Kahan bound is essentially
*independent of $n$*.

**Hint.** For naive, the $k$-th partial sum picks up one factor $(1 + \delta_k)$;
expand the product and keep first order in $u$, bounding each partial magnitude
by $\sum |x_i|$. For Kahan, the correction $c$ removes each step's rounding error
to first order (that is the point of the algorithm), leaving only a *second*-order
$O(nu^2)$ residue on top of a single $O(u)$ term.

**Answer sketch.** Write $S_k = \mathrm{fl}(S_{k-1} + x_k) = (S_{k-1} + x_k)(1 +
\delta_k)$, $|\delta_k| \le u/2$. Unrolling, the computed total is $\hat{S}_n =
\sum_k x_k \prod_{j \ge k}(1 + \delta_j)$, and since $\prod(1 + \delta_j) = 1 +
\theta$ with $|\theta| \le (n-1)(u/2) + O(u^2)$,

$$
\bigl|\hat{S}_n - \textstyle\sum x_k\bigr| \;\le\; (n-1)\,\tfrac{u}{2}
\sum_{k=1}^{n} |x_k| \;+\; O(u^2).
$$

The naive error grows **linearly in $n$** (worst case), which is why summing
$0.1$ a hundred thousand times drifts to `10000.000000018848` instead of
`10000`. For Kahan, the invariant is that $c$ holds, to first order, exactly the
rounding error dropped by the previous $t = \mathrm{fl}(s + y)$ (proved by
Sterbenz's lemma, Exercise 4: when $|s| \ge |y|$, $t - s$ is exact, so $c = (t -
s) - y$ is the true lost part). Feeding $-c$ back cancels each step's leading
error, leaving

$$
\bigl|\hat{S}^{\text{Kahan}}_n - \textstyle\sum x_k\bigr| \;\le\; \bigl(2\cdot
\tfrac{u}{2} + O(nu^2)\bigr)\sum_{k=1}^{n}|x_k| \;=\; \bigl(u + O(nu^2)\bigr)
\sum_{k=1}^{n}|x_k|,
$$

an $O(u)$ bound whose $n$-dependence is pushed to *second* order — effectively
constant. Empirically Kahan lands on `10000.0` to the bit, and on the adversarial
$[10^{16}, 1, \ldots, 1, -10^{16}]$ (ten thousand ones) it recovers `10000` while
naive returns `0` — every `1` was swamped and rounded away, but Kahan's $c$ held
each lost `1` and fed it back. Cost: about $4\times$ the flops, hence a choice,
not a default.

### 6. ▶ When does $(a{+}b){+}c$ differ from $a{+}(b{+}c)$? (rating M30 · cf. §4.2.2)

**Problem.** Floating-point addition is commutative but **not** associative.
Characterize when $(a + b) + c$ and $a + (b + c)$ give different results,
exhibit a concrete triple that differs, compute the exact difference, and bound
how large the discrepancy between the two groupings can be.

**Hint.** Each grouping is *correctly rounded at every step*, so each equals the
true sum $a + b + c$ times a product of two $(1 + \delta)$ factors. They differ
exactly when the two roundings the groupings perform are not the same — e.g. one
grouping forms an intermediate that rounds a bit away that the other grouping
preserves. Use $1$, $2^{-53}$, $2^{-53}$ and trace both.

**Answer sketch.** Let $h = 2^{-53}$ (half a ulp at $1$). $(1 + h) + h$: the
first add $1 + h$ is an exact tie, rounds to even $= 1$; then $1 + h$ ties again
$= 1$. But $1 + (h + h)$: $h + h = 2^{-52} = \operatorname{ulp}(1)$ *exactly*
(no rounding), and $1 + 2^{-52} = $ `1.0000000000000002` exactly. So

$$
(1 + h) + h = 1, \qquad 1 + (h + h) = 1 + 2^{-52}, \qquad
\text{difference} = 2^{-52},
$$

verified on hardware `f64`. **When do they differ?** Using TwoSum, each addition
$\mathrm{fl}(p + q) = (p + q) - \varepsilon$ with $\varepsilon$ the exact
rounding error; $(a+b)+c$ carries the errors of forming $a+b$ then adding $c$,
while $a+(b+c)$ carries the errors of forming $b+c$ then adding $a$. The two
agree iff those accumulated errors coincide; they differ precisely when at least
one grouping performs an inexact rounding the other avoids (as above: $1 + h$ is
inexact and destructive, $h + h$ is exact). **How much?** Each grouping equals
$(a + b + c)(1 + \theta)$ with $|\theta| \le 2(u/2) + O(u^2) = u + O(u^2)$, so

$$
\bigl|((a{+}b){+}c) - (a{+}(b{+}c))\bigr| \;\le\; \bigl(u + O(u^2)\bigr)\,
|a + b + c| \;+\; \text{(cancellation terms)} \;\lesssim\; 2\cdot\tfrac{u}{2}
\bigl(|a| + |b| + |c|\bigr),
$$

i.e. a couple of ulps of the operands — small in *absolute* terms, but (as when
$a + b + c$ nearly cancels) potentially unbounded in *relative* terms. The
practical moral of §6/§8: you may not freely reorder floating-point sums, and
`-ffast-math` reorderings can change results — the discrepancy is exactly this
non-associativity.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
