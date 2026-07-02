# Module 04 — Random Numbers

> **Source:** *The Art of Computer Programming*, Vol. 2, 3rd ed., Ch. 3 —
> §3.1 (what randomness means), §3.2.1 (the linear congruential method and
> Theorem 3.2.1.2A), §3.3.1 (the chi-square test), §3.4.2 (random sampling
> and shuffling).
> **Lab:** `labs/module-04-random` · **Grade it:** `./grade 4`
>
> Self-contained: you can complete the module without the book. If you own
> Vol. 2, read §3.2.1 and §3.3.1 first — this lesson tells you where to look.

"Anyone who considers arithmetical methods of producing random digits is, of
course, in a state of sin." Knuth quotes von Neumann on the first page of
Chapter 3, and the joke is also the whole problem: a deterministic program
cannot produce a random sequence, only a sequence that *behaves* random enough
for the job. This module builds the classic engine (a linear congruential
generator), a scientific instrument to judge it (the chi-square test), and two
algorithms that *consume* randomness correctly (shuffling and reservoir
sampling) — plus the counting arguments that prove the good ones fair and the
plausible-looking ones biased.

---

## 1. Pseudo-randomness: the caveat (§3.1)

A *random sequence* is an idealization; what a program emits is a
**pseudo-random sequence** — fully determined by its starting conditions, yet
passing the statistical tests we would apply to a truly random one. Two traps
frame the subject:

- **Complex ≠ random.** Von Neumann's own "middle-square" method (square the
  number, take the middle digits) looks chaotic and is terrible: it quickly
  falls into short cycles or the fixed point 0. Complexity of the recipe is no
  guarantee of quality of the output.
- **Randomness is a property of the *process*, not any one sequence.** No
  finite string is "random" or "not random" by itself. We can only ask whether
  a *generator* produces output indistinguishable, under stated tests, from
  independent uniform draws.

Because the state is finite (say all 64-bit words), the sequence
X₀, X₁, X₂, … must eventually repeat: it is **ultimately periodic**. A short
tail may lead into a cycle, and the cycle length is the **period**. A usable
generator needs, at minimum, a period far longer than the amount of output you
will draw — but a long period is necessary, never sufficient.

---

## 2. The linear congruential method (§3.2.1)

Lehmer's 1949 generator is still the textbook workhorse. Pick four **magic
quantities**: the modulus *m*, the multiplier *a*, the increment *c*, and the
seed X₀. Then

```text
    X_{n+1} = (a · X_n + c) mod m .
```

### Algorithm L (one LCG step)

```text
L1. [Multiply and add.]  Set X <- a·X + c.
L2. [Reduce.]            Set X <- X mod m, and output X.
```

Two implementation notes carried by the lab's `Lcg`:

- **The word-size modulus.** Taking m = 2^64 makes the "mod" *free*: it is
  exactly the wrap of 64-bit machine arithmetic. The lab encodes this as
  `m == 0` and uses `wrapping_mul`/`wrapping_add`. For any other `m > 0` the
  product `a·X + c` is formed in 128-bit arithmetic before reducing, so it
  cannot overflow.
- **Indexing.** `new` stores X₀; the first `next()` returns X₁. Knuth traces
  sequences from X₁, and so do the tests.

**Trace (§3.1's opener).** X₀ = a = c = 7, m = 10:

| n | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| X_n | 7 | 6 | 9 | 0 | 7 | 6 |

The sequence is 7, 6, 9, 0, 7, 6, 9, 0, … — period **4**. Only ten values were
possible, and we squandered more than half of them. The lesson of the example:
parameters must be *chosen*, not guessed.

### 2.1 Full period — Theorem 3.2.1.2A

We want the longest possible period, m, achieved from *every* seed (so the
generator visits all of Z_m before repeating). This happens exactly under
three conditions.

**Theorem 3.2.1.2A.** The linear congruential sequence has period length m if
and only if

1. **c is relatively prime to m**  (gcd(c, m) = 1);
2. **a ≡ 1 (mod p)** for every prime p dividing m;
3. **a ≡ 1 (mod 4)** if 4 divides m.

*Proof sketch.* The three conditions are exactly what is needed for the map
x ↦ ax + c to be a single m-cycle. Condition 1 rules out an increment that
shares a factor with m (which would trap the sequence in a coset). Conditions
2–3 control the multiplier: writing the closed form
X_n = a^n X₀ + c(a^n − 1)/(a − 1) (mod m), one analyzes, prime power by prime
power via the Chinese Remainder Theorem, when a^n ≡ 1 and the geometric-sum
factor first returns to 0. For a prime power p^e the order works out to require
p | (a − 1), with the extra 4 | (a − 1) needed at the prime 2 because the group
of units mod 2^e is not cyclic for e ≥ 3. Knuth gives the full number-theoretic
argument in §3.2.1.2; the payoff is a clean, checkable recipe. ∎

**Worked predicate, m = 16 = 2⁴.** The only prime dividing m is 2, and 4 | m,
so conditions 2–3 collapse to a ≡ 1 (mod 4), i.e. a ∈ {1, 5, 9, 13}; condition
1 says c must be odd. So full period ⟺ (a mod 4 = 1) and (c odd). For example
`period(5, 3, 16, 0) = 16`, while `period(3, 1, 16, 0) < 16` (3 ≡ 3 mod 4) and
`period(5, 2, 16, 0) < 16` (c even). Stage 1 verifies the *entire* "if and only
if" by enumerating all 256 (a, c) pairs.

**Worked predicate, m = 100 = 2²·5².** Primes 2 and 5 divide m, and 4 | m, so
a ≡ 1 (mod 4) and a ≡ 1 (mod 5), i.e. **a ≡ 1 (mod 20)** (a ∈ {1,21,41,61,81});
and gcd(c, 100) = 1. Try a = 21, c = 3: full period 100 from any seed.

### 2.2 The low-order-bit weakness

When m = 2^e, the low bits of an LCG are far less random than the high bits.
Reducing modulo 2^b keeps only the bottom b bits, and

```text
    X_{n+1} mod 2^b = (a·X_n + c) mod 2^b
```

depends only on X_n mod 2^b. So the last b bits are themselves a little LCG
with modulus 2^b — and hence have period at most 2^b. The lowest bit of a
maximal-period power-of-two LCG simply alternates 0,1,0,1,…; the low two bits
cycle with period 4; and so on. **Never** derive a small integer with
`X mod k` (or, worse, use the low bit as a coin flip). Instead take the **high**
bits. The lab's test closures use the multiply-shift map (Lemire's method),

```text
    rng(bound) = (X · bound) >> 64 ,       (X a 64-bit output)
```

which is `⌊(X / 2^64) · bound⌋`: it reads the *top* bits of X and lands
uniformly in `0..bound`.

### 2.3 The spectral flaw: RANDU and the 9−6−1 identity

IBM's notorious **RANDU** used a = 65539 = 2¹⁶ + 3, c = 0, m = 2³¹. Its period
issues aside, it has a fatal *structural* defect. Compute a² modulo m:

```text
    a² = (2¹⁶ + 3)² = 2³² + 6·2¹⁶ + 9
       = 2·2³¹ + 6·(2¹⁶ + 3) − 18 + 9
       = 2·2³¹ + 6a − 9  ≡  6a − 9   (mod 2³¹).
```

Since (with c = 0) X_{n+2} = a²X_n and X_{n+1} = a X_n modulo m,

```text
    X_{n+2} ≡ (6a − 9) X_n = 6 X_{n+1} − 9 X_n     (mod 2³¹),

    i.e.   9 X_n − 6 X_{n+1} + X_{n+2} ≡ 0  (mod 2³¹)   for all n.
```

Every consecutive triple (X_n, X_{n+1}, X_{n+2}), plotted as a point in the
unit cube, therefore satisfies one linear equation 9x − 6y + z = (integer)·2³¹
— so **all** the points lie on just **15 parallel planes**. This is the crude
end of §3.3.4's *spectral test*, which measures exactly how far apart such
planes are for a given multiplier. Stage 1 checks the identity holds for 200
consecutive outputs. Good multipliers make the lattice of points as fine and
isotropic as possible; RANDU is the cautionary tale of what a bad one does.

---

## 3. Judging a generator: the chi-square test (§3.3.1)

We need to *measure* "random enough." The chi-square test compares observed
category counts against a probability model. Suppose n independent observations
each fall in one of k categories, category s having model probability p_s (so
Σ p_s = 1). Let Y_s be the observed count in category s; the model expects
n·p_s. Define the statistic

```text
    V = Σ_s  (Y_s − n·p_s)² / (n·p_s) .
```

Each term measures a squared deviation, *scaled by* the expected count so that
naturally large categories are not unfairly penalized. V near 0 means an almost
suspiciously perfect fit; V very large means the model is a poor description of
the data.

**How large is "too large"?** Under the model, V is approximately distributed
as a **chi-square variate with ν = k − 1 degrees of freedom** (one is lost to
the constraint Σ Y_s = n). We reject the model when V exceeds a high percentile
of that distribution. A tiny table of the ν=9 row (used by stages 2–4):

| percentile | 1% | 25% | 50% | 75% | 95% | 99% |
|---|---|---|---|---|---|---|
| chi-square, ν = 9 | 2.09 | 5.90 | 8.34 | 11.39 | 16.92 | **21.67** |

So with 10 categories a fair generator should give V below ≈ 21.67 about 99% of
the time. **Rule of thumb (Knuth):** every expected count n·p_s should be ≳ 5,
or the chi-square approximation is unreliable — merge sparse categories first.
The lab's `chi_square` panics on a non-positive expected count for this reason.

**Worked example (two dice).** Throw two dice 144 times; category s = 2..=12
has probability p_s from the familiar 1,2,3,4,5,6,5,4,3,2,1 over 36, so the
expected counts are 4, 8, 12, 16, 20, 24, 20, 16, 12, 8, 4. For the observed
table 2, 4, 10, 12, 22, 29, 21, 15, 14, 9, 6,

```text
    V = (2−4)²/4 + (4−8)²/8 + … + (6−4)²/4 = 343/48 ≈ 7.15 .
```

With ν = 10, that sits between the 25% and 50% points — an unremarkable, and
therefore believable, fit. `chi_square_uniform` is the special case where every
p_s = 1/k, so each category expects n/k; e.g. counts (3, 7) give
V = 4/5 + 4/5 = 1.6.

---

## 4. Consuming randomness

### 4.1 Shuffling — Algorithm 3.4.2P (Fisher–Yates)

Given `rng(bound)` uniform in `0..bound`, permute t items uniformly in place.
In 0-based terms:

```text
P1. [Initialize.]  Set j <- t − 1.
P2. [Generate U.]  Set k <- rng(j + 1)     (uniform in 0..=j).
P3. [Exchange.]    Swap items[k] <-> items[j]; then j <- j − 1;
                   if j >= 1, return to P2.
```

The candidate range **shrinks**: position j swaps only with a position in
0..=j. That is the entire trick.

**Theorem.** Algorithm P produces each of the t! permutations with probability
exactly 1/t!.

*Proof (induction on t).* For t = 1 there is one permutation, probability 1.
Assume the claim for t − 1. In the first pass P chooses k uniformly from the t
positions 0..=t−1 and moves that element to the last slot; each of the t
choices has probability 1/t, and each fixes a *distinct* final element. The
remaining t − 1 elements occupy positions 0..=t−2 and are then shuffled by the
same algorithm on a slice of length t − 1, which by hypothesis produces each of
their (t−1)! arrangements with probability 1/(t−1)!. A given permutation of all
t items determines its last element (probability 1/t) and then an arrangement
of the rest (probability 1/(t−1)!), so its total probability is
(1/t)·(1/(t−1)!) = 1/t!. Since the outcomes partition all t! permutations, each
occurs with probability 1/t!. ∎

Equivalently: P consumes one draw from each of the bounds t, t−1, …, 2, giving
t·(t−1)···2 = t! equally likely "tapes", and the argument shows the map
tape ↦ permutation is a **bijection** onto the t! permutations. Stage 3
enumerates the 6 tapes for t = 3 (all distinct outputs) and, for t = 6, drives
43 200 shuffles (≈ 60 per permutation) through the stage-2 chi-square test over
all 720 permutations (ν = 719, comfortably below the ≈ 810 that marks the 99%
point).

### 4.2 Why the "naive" shuffle is biased (a counting proof)

A tempting variant draws from the **full** range at every step:

```text
for i in 0..n:   swap items[i] <-> items[rng(n)] .
```

It is *not* uniform, and the proof is pure counting. There are n independent
draws each with n outcomes, hence **nⁿ** equally likely tapes. If every one of
the n! permutations were equally likely, n! would have to divide nⁿ. But for
n > 2 it does not: n! contains the prime factors of n−1, n−2, …, which nⁿ (a
power of n) generally lacks. For n = 3: 3³ = 27 tapes, 3! = 6 permutations, and
27 / 6 is not an integer — so the six permutations *cannot* be equally likely.

Enumerating all 27 tapes gives the exact distribution (lexicographic order of
the resulting permutation of [0,1,2]):

| permutation | 012 | 021 | 102 | 120 | 201 | 210 |
|---|---|---|---|---|---|---|
| tapes | 4 | 5 | 5 | 5 | 4 | 4 |

Never the fair 4.5/27. Stage 3 reproduces this table exactly. The moral: an
algorithm that "obviously" mixes everything can still be provably biased — you
must count.

### 4.3 Reservoir sampling — Algorithm 3.4.2R

Choose k items uniformly from a stream whose length is **unknown in advance**,
in one pass and O(k) memory. With t = number of records seen so far:

```text
R1. [Initialize.]   Fill the reservoir with the first k records; set t <- k.
R2. [Next record.]  If the stream is exhausted, stop: the reservoir is the
                    sample. Otherwise read a record and set t <- t + 1.
R3. [Draw.]         Set M <- rng(t)          (uniform in 0..t).
R4. [Replace?]      If M < k, put the new record into reservoir[M]. Go to R2.
```

**Theorem (the invariant).** After t ≥ k records have been read, every one of
them is in the reservoir with probability exactly k/t.

*Proof (induction on t).* Base t = k: the first k records fill the reservoir,
each present with probability 1 = k/k. Inductive step: assume the claim after
t records; consider record t + 1. In step R3 we draw M uniform in 0..t+1.
- The **new** record is kept iff M < k, probability k/(t+1). ✓
- An **old** record was present with probability k/t (hypothesis). It survives
  unless the new record displaces its exact slot: the new record enters with
  probability k/(t+1), and given that, lands on any of the k slots with
  probability 1/k, so it evicts a *particular* old element with probability
  1/(t+1). Thus the old element remains with probability
  1 − 1/(t+1) = t/(t+1), giving total (k/t)·(t/(t+1)) = k/(t+1). ✓

So after t + 1 records every record has probability k/(t+1); induction
completes the proof. When the stream finally ends at length N, each item is in
the sample with probability k/N, and every k-subset is equally likely. ∎

Stage 4 checks exact uniformity for N = 4, k = 2 (the 12 tapes split the 6
subsets 2 apiece) and, over 30 000 LCG-driven trials with N = 10, k = 3,
confirms each item appears with frequency ≈ k/N via the chi-square test.

---

## 5. Stage-by-stage lab guide

Open `labs/module-04-random/src/lab.rs`. Run `./grade 4`; stages are taken in
order, stopping at the first failure.

### Stage 1 — `Lcg` and `period`
Implement `new` (reduce seed/a/c mod m when m > 0; store as-is when m == 0),
`next` (wrapping arithmetic for m == 0, u128 otherwise), and `period` (direct
cycle detection: remember the step each state was first seen; the first repeat
closes the cycle, so the tail is excluded automatically). Panic in `period`
when m == 0 — the message must contain "finite modulus". Tests reproduce §3.1's
opener, the MMIX constants, the full "iff" of Theorem 3.2.1.2A for m = 16 and
m = 100, and RANDU's 9−6−1 identity.

### Stage 2 — `chi_square`, `chi_square_uniform`
`V = Σ (obs − exp)²/exp`. Panic on length mismatch ("same number"), empty input
("at least one category"), non-positive expected ("positive"), and all-zero
counts ("at least one observation"). `chi_square_uniform` builds equal expected
counts n/k and delegates. Anchored on the two-dice V = 343/48.

### Stage 3 — `shuffle`, `naive_shuffle`
Algorithm P with j running backwards from t−1 to 1; leave slices of length < 2
untouched (don't touch the rng). Implement `naive_shuffle` faithfully *wrong*
(full-range swap) — the test proves its bias on all 27 tapes.

### Stage 4 — `reservoir_sample`
Algorithm R. If k == 0 return empty without drawing; if the stream is shorter
than k return everything in order. Otherwise fill, then for each further record
draw M in 0..t and overwrite slot M when M < k.

---

## 6. Check your understanding

1. The generator X₀ = a = c = 7, m = 10 has period 4. Which condition of
   Theorem 3.2.1.2A fails? (c = 7 is coprime to 10 ✓; a = 7 ≡ 1 mod 2? no —
   fails condition 2 at p = 2, and also a ≢ 1 mod 5.)
2. Why is `X % 2` a terrible coin flip for a power-of-two-modulus LCG, but
   `(X · 2) >> 64` fine? (The low bit has period ≤ 2; the high bits carry the
   full period.)
3. For the two-dice example V ≈ 7.15 with ν = 10 — do you reject the generator?
   (No: it is near the median, a perfectly ordinary fit.)
4. In Algorithm P, what breaks if step P2 draws from `rng(t)` (the full range)
   every time instead of `rng(j+1)`? (You get the biased naive shuffle.)
5. In reservoir sampling, why does the new (t+1)-th record enter with
   probability k/(t+1) rather than 1/(t+1)? (It enters whenever M < k, and M is
   uniform over t+1 values.)

## 7. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 up to an hour · 30
hours · 40 term project · 50 open problem. ▶ marks especially instructive ones.
Log attempts in `course/module-04-random/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 3.2.1.2-1 | 10 | Show the period of an LCG is at most m, and find a case where it is exactly m. |
| ▶3.2.1.2-4 | M23 | Prove the "if" direction of Theorem 3.2.1.2A for m a power of 2. |
| 3.2.1-2 | 10 | For what seeds does X₀ = a = c = 7, m = 10 enter its cycle, and how long is any tail? |
| 3.3.1-8 | 20 | Compute V for a given table and decide whether to accept the hypothesis. |
| ▶3.4.2-8 | 22 | Prove Algorithm P generates each permutation with probability 1/t! (you did — stage 3; write it up). |
| 3.4.2-10 | M25 | Analyze the expected number of moves (slot replacements) reservoir sampling makes over a stream of length N. |

## Why it's done this way

Every stage pairs a *generator* with a *judgment about it*: an LCG with a
period theorem, a shuffle with a uniformity proof, a statistic with a
distribution. That pairing is the module's real lesson — Knuth's Chapter 3
is not a recipe box of generators, it is a training course in *how to
distrust one*. The famous cautionary tale of §3.1 (his elaborate
"super-random" Algorithm K that promptly collapsed into a 3178-cycle... or
into a fixed point) opens the chapter for exactly this reason.

## In the real world

RANDU is not a hypothetical: results computed with it in the 1960s–70s had
to be re-examined once its 15-plane geometry surfaced, and it remains the
canonical example of "passes 1-D tests, fails in 3-D". The naive shuffle's
bias is a production bug class — a random-comparator shuffle famously
skewed a 2010 browser-choice ballot — and your 27-tape enumeration in
stage 3 is precisely the audit that catches it. Reservoir sampling runs in
every stream-analytics stack that must sample fairly from data too big to
hold. Chi-square is the same machinery behind A/B-test sanity checks:
stage 2's function is a data-science primitive.

## Proof techniques you practiced

- **Number-theoretic structure** — the full-period theorem ties a program's
  observable behavior to congruence conditions you can check by hand.
- **Induction for uniformity** — Algorithm P is proved fair by inducting on
  the suffix already shuffled.
- **Divisibility obstruction** — the naive shuffle *cannot* be fair because
  3³ = 27 outcomes can't split evenly over 6 permutations; no amount of
  testing is as final as this one-line counting argument.
- **Hypothesis testing done honestly** — a statistic, its distribution, a
  significance threshold, and the discipline of not over-reading one run.

## 8. Where this leads

- The **chi-square test** is the entry point to §3.3's full battery (equi-
  distribution, gap, poker, coupon-collector, runs, serial correlation) and to
  §3.3.4's **spectral test**, the sharp tool that quantifies the RANDU flaw you
  saw crudely here.
- **Full-period theory** (§3.2.1.2) generalizes to combined and lagged
  generators; the low-bit weakness motivates modern designs (xorshift, PCG,
  counter-based) that this module's `Lcg` interface can host unchanged.
- **Uniform shuffling and sampling** underpin randomized algorithms throughout
  the course — quicksort's random pivots (Module 06), randomized data
  structures, and Monte-Carlo estimation.
