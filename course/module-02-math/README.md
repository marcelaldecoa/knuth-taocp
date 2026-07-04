# Module 02 — Mathematical Preliminaries

> **Source:** *The Art of Computer Programming*, Vol. 1, 3rd ed., §1.2 —
> specifically §1.2.1 (induction), §1.2.3 (sums), §1.2.6 (binomial
> coefficients), §1.2.7 (harmonic numbers), §1.2.8 (Fibonacci numbers),
> §1.2.10 (analysis of an algorithm), and §1.2.11.1 (O-notation).
> **Lab:** `labs/module-02-math` · **Grade it:** `./grade 2`
>
> This lesson is self-contained: you can complete the module without the
> book. If you own Vol. 1, read the sections above alongside — the lesson
> tells you where each idea lives.
>
> **Companion:** this §1.2 is exactly what *Concrete Mathematics*
> (Graham–Knuth–Patashnik) expands into a whole course. If any tool here
> whets your appetite, [../../docs/concrete-mathematics.md](../../docs/concrete-mathematics.md)
> maps the two together.

Module 01 ended with a promise: $T(m, n)$, the division count of Euclid's
algorithm, was your first *cost function*, and we said Module 02 would build
the toolkit that makes such functions tame. This is that toolkit. It
culminates in the first complete algorithm analysis of the book — Knuth's
Algorithm M for finding the maximum, whose "interesting quantity" $A$ turns out
to average exactly $H_n - 1$, a harmonic number. Everything in this module
(induction, closed-form sums, binomial identities, harmonic numbers,
Fibonacci numbers, O-notation) is a tool you will reach for again in every
later module, from quicksort's average case to hashing's load factors.

---

## 1. Mathematical induction (§1.2.1)

The proof pattern you used for Euclid's algorithm — *an invariant plus a
decreasing quantity* — is mathematical induction wearing work clothes. The
formal template: to prove a statement $P(n)$ for all integers $n \ge 1$,

1. **Basis.** Prove $P(1)$.
2. **Induction.** Prove that $P(1), P(2), \ldots, P(n)$ together imply $P(n + 1)$.

Allowing *all* earlier cases (not just $P(n)$) costs nothing and is often what
you need — Knuth states it this way from the start (it is sometimes called
*strong* induction, but it is the same principle).

**Worked example.** $P(n)$: $1 + 3 + 5 + \cdots + (2n - 1) = n^2$.
*Basis:* $1 = 1^2$. *Induction:* assuming the sum of the first $n$ odd numbers is
$n^2$, adding the next odd number gives $n^2 + (2n + 1) = (n + 1)^2$. ∎

The deeper point of §1.2.1 — worth internalizing now — is that induction is
*the* method for proving algorithms correct. Knuth demonstrates it on
extended Euclid (your Module 01, Stage 3): attach an assertion to each arrow
of the flowchart, check that each step carries its incoming assertion to its
outgoing one, and induction on the number of steps executed does the rest.
Robert W. Floyd and C. A. R. Hoare later built program-verification logics on
exactly this idea. Every invariant comment you write in this course is a
little induction proof waiting to be completed.

---

## 2. Sums and the perturbation method (§1.2.3)

Analyses of algorithms produce sums; you need to turn sums into closed forms.
Three you should know cold, and one *method* that generates such formulas on
demand.

**The arithmetic sum.** $\sum_{k=1}^{n} k = n(n + 1)/2$. Pair the first and last
terms (Gauss's schoolroom trick), or induct. Note the closed form is exact
integer arithmetic: one of $n$, $n + 1$ is even, so the division by 2 is exact.

**Sum of squares.** $\sum_{k=1}^{n} k^2 = n(n + 1)(2n + 1)/6$.

**Sum of cubes (Nicomachus).** $\sum_{k=1}^{n} k^3 = (n(n + 1)/2)^2$ — the sum of
the first $n$ cubes is the *square* of the $n$-th triangular number. Induction
makes short work of it: adding $(n + 1)^3$ to $(n(n+1)/2)^2$ and factoring gives
$((n+1)(n+2)/2)^2$.

### The perturbation method

Rather than guess-and-induct, §1.2.3 teaches a machine for *deriving* closed
forms: compute $S_{n+1}$ two ways, by splitting off the **last** term and by
splitting off the **first**, then solve the resulting equation.

Derivation of the geometric sum $S_n = \sum_{k=0}^{n} x^k$:

```text
split off the last term:    S_{n+1} = S_n + x^(n+1)
split off the first term:   S_{n+1} = 1 + x·(1 + x + ... + x^n) = 1 + x·S_n
equate and solve:           S_n + x^(n+1) = 1 + x·S_n
                            S_n·(x − 1) = x^(n+1) − 1
                            S_n = (x^(n+1) − 1) / (x − 1)      for x ≠ 1.
```

For $x = 1$ the equation degenerates (both sides say $S + 1 = S + 1$) and the
answer is simply $n + 1$. The same two-way-split trick, applied to
$\sum k \cdot x^k$, produces that closed form too — try it (exercise table below). The
division above is *exact in the integers*: $(x - 1)$ divides $x^{n+1} - 1$
because $x \equiv 1$ makes $x^{n+1} \equiv 1 \pmod{x - 1}$.

How to prove $\sum k^2$ without inspiration: perturb $\sum k^3$. Split $S = \sum_{k=0}^{n} k^3$
at both ends, expand $(k+1)^3$, and the cubes cancel, leaving a linear equation
for $\sum k^2$. Perturbation converts "I need a formula for degree $d$" into "I know
the formula for degree $d + 1$'s telescoping" — a genuinely mechanical method.

---

## 3. Binomial coefficients (§1.2.6)

Knuth calls $\binom{n}{k}$ — "$n$ choose $k$", written with $n$ over $k$ in
parentheses — the most important quantity in the analysis of algorithms.
Definition, for integers $n \ge k \ge 0$:

$$\binom{n}{k} = \frac{n(n-1)\cdots(n-k+1)}{k!} = \frac{n!}{k!(n-k)!}$$

and $\binom{n}{k} = 0$ when $k > n \ge 0$ (no way to choose more things than you
have). Combinatorially: the number of $k$-element subsets of an $n$-element set.

Four identities carry most of the weight. All have one-line *combinatorial*
proofs — learn those, not the algebra:

**Symmetry.** $\binom{n}{k} = \binom{n}{n-k}$. Choosing which $k$ elements are
*in* is the same as choosing which $n - k$ are *out*.

**Pascal's rule.** $\binom{n}{k} = \binom{n-1}{k-1} + \binom{n-1}{k}$. Fix one
element $x$: the $k$-subsets either contain $x$ (choose the other $k - 1$ from
$n - 1$) or they don't (choose all $k$ from $n - 1$). This addition formula
generates Pascal's triangle, and gives an induction proof that every
$\binom{n}{k}$ is an integer — not obvious from the factorial quotient!

**Row sums (the binomial theorem at $x = y = 1$).**
$\sum_k \binom{n}{k} = 2^n$: every subset has some size. With alternating signs,
$\sum_k (-1)^k \binom{n}{k} = 0$ for $n \ge 1$ — set $x = -1$, $y = 1$ in
$(x + y)^n = \sum_k \binom{n}{k} x^k y^{n-k}$.

**Vandermonde's convolution.** $\sum_j \binom{m}{j} \cdot \binom{n}{k-j} = \binom{m+n}{k}$.
*Proof sketch:* a committee of $k$ people drawn from $m$ men and $n$ women must
contain $j$ men and $k - j$ women for exactly one $j$; sum over the ways. Or,
in generating-function clothing: $(1+x)^m \cdot (1+x)^n = (1+x)^{m+n}$, and match
the coefficient of $x^k$ on both sides — the left side's coefficient is
exactly the convolution sum. This is your first taste of the generating-
function method, which reappears in the analysis of Algorithm M below.

### Computing C(n, k) exactly

The factorial formula overflows absurdly early ($21! > 2^{64}$) even when the
answer is small. Two exact strategies:

- **Multiplicative:** evaluate $c \gets c \cdot (n - k + i)/i$ for
  $i = 1, 2, \ldots, k$, left to right. Invariant: after step $i$,
  $c = \binom{n-k+i}{i}$ — an integer by Pascal's-rule induction — so every
  division is exact. Multiply *before* dividing. Apply symmetry first so
  $k \le n/2$.
- **Pascal:** build triangle rows by addition only. Slower, but division-free.

Either is exact in `u128` for all $n \le 100$; the largest value you'll meet is

$$\binom{100}{50} = 100891344545564193334812497256 \approx 1.0 \times 10^{29}.$$

---

## 4. Harmonic numbers (§1.2.7)

Define $H_n = 1 + 1/2 + 1/3 + \cdots + 1/n$. These fractions are the fingerprint
of *divide-by-rank* behavior — they appear whenever "the $k$-th item matters with
weight $1/k$", which in practice is constantly (quicksort, records in
sequences, coupon collecting, treaps…).

Small values, exactly: $H_1 = 1$, $H_2 = 3/2$, $H_3 = 11/6$, $H_4 = 25/12$,
$H_5 = 137/60$, $H_6 = 49/20$. Note the denominators are lcm-like and grow fast;
$H_{30}$ in lowest terms is $9304682830147 / 2329089562800$.

**$H_n$ is unbounded — but barely.** Group the terms in blocks of lengths
$1, 1, 2, 4, 8, \ldots$:

```text
H_{2^k} = 1 + (1/2) + (1/3 + 1/4) + (1/5 + ... + 1/8) + ...
```

Each parenthesized block of $2^j$ terms is at most $2^j \cdot (1/2^j) = 1$ and at
least $2^j \cdot (1/2^{j+1}) = 1/2$, so

$$1 + k/2 \le H_{2^k} \le 1 + k.$$

Sandwiching $k$ between logarithms: $H_n$ grows like $\log n$. The precise
statement (§1.2.7, Eq. (3)) is one of Euler's gems:

$$H_n = \ln n + \gamma + \frac{1}{2n} - \frac{1}{12n^2} + \varepsilon, \qquad 0 < \varepsilon < \frac{1}{120n^4},$$

where $\gamma = 0.5772156649015329\ldots$ is Euler's constant — a number so
mysterious that whether it is irrational is *still* an open problem. Your
Stage 4 tests verify this expansion numerically: the error of the two-term
approximation $\ln n + \gamma$ is squeezed between $1/(2n) - 1/(12n^2)$ and $1/(2n)$.

**"Almost log":** for algorithm analysis, remember $H_n = \ln n + \gamma + O(1/n)$.
When you see an expected cost of $H_n$, read "natural log of $n$, plus 0.577".

---

## 5. Fibonacci numbers (§1.2.8)

$F_0 = 0$, $F_1 = 1$, $F_{n+1} = F_n + F_{n-1}$ — Knuth's indexing, which we use
everywhere (beware off-by-one against other books). First values:

| $n$ | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---|---|---|---|---|---|---|---|---|---|----|----|----|----|----|----|
| $F_n$ | 0 | 1 | 1 | 2 | 3 | 5 | 8 | 13 | 21 | 34 | 55 | 89 | 144 | 233 | 377 | 610 |

You met them already: consecutive Fibonacci numbers are the worst case of
Euclid's algorithm (Lamé, Module 01, Stage 4). They also govern Fibonacci
heaps, polyphase merge sorting (Vol. 3), and Fibonacci hashing (Vol. 3,
§6.4).

**Binet's closed form.** Let $\varphi = (1 + \sqrt5)/2 \approx 1.618$ (the
*golden ratio*) and $\hat\varphi = (1 - \sqrt5)/2 = -1/\varphi \approx -0.618$,
the two roots of $x^2 = x + 1$. Then

$$F_n = \frac{\varphi^n - \hat\varphi^n}{\sqrt5}.$$

*Proof sketch:* both $\varphi^n$ and $\hat\varphi^n$ satisfy the recurrence
$x_{n+1} = x_n + x_{n-1}$ (because $\varphi^2 = \varphi + 1$, same for
$\hat\varphi$), hence so does any linear combination; choose the coefficients
$(1/\sqrt5, -1/\sqrt5)$ to match $F_0 = 0$ and $F_1 = 1$, and induction finishes
it. Since $|\hat\varphi| < 1$, the second term dies away geometrically, giving
the remarkable statement

$$F_n = \varphi^n/\sqrt5, \text{ rounded to the nearest integer,}$$

so Fibonacci growth is exponential with base $\varphi$ — that is why
$F_{186} \approx 3.3 \times 10^{38}$ already exhausts a `u128`.

**Identities you will implement against (all provable by induction):**

- *Addition law* (§1.2.8, Eq. (6)): $F_{m+n} = F_m F_{n+1} + F_{m-1} F_n$.
  Setting $m = n$ yields the *fast doubling* formulas
  $F_{2n} = F_n(2F_{n+1} - F_n)$ and $F_{2n+1} = F_n^2 + F_{n+1}^2$, which compute
  $F_n$ in $O(\log n)$ arithmetic operations — worth trying as an extension.
- *Cassini's identity* (Eq. (8)): $F_{n-1}F_{n+1} - F_n^2 = (-1)^n$. (This is
  the algebra behind the classic $8 \times 8$-square-into-$5 \times 13$-rectangle "paradox".)
- *gcd law:* $\gcd(F_m, F_n) = F_{\gcd(m,n)}$ — the Fibonacci sequence is a
  "strong divisibility sequence". Proof sketch: the addition law gives
  $F_{m+n} \equiv F_{m-1} F_n \pmod{F_m}$... iterating, $F_n \bmod F_m$ runs through a
  Euclid-like descent on the *indices*, so the gcd computation on values
  mirrors the gcd computation on subscripts. It is Euclid's algorithm and
  induction, twice over.

---

## 6. O-notation in one page (§1.2.11.1)

$f(n) = O(g(n))$ means: there exist a constant $C$ and a threshold $n_0$ with
$|f(n)| \le C \cdot |g(n)|$ for all $n \ge n_0$. Read $O(g(n))$ as "some quantity
we won't name, bounded by a constant times $g(n)$".

The three rules of hygiene:

1. **Equations with O are one-way.** $n = O(n^2)$ is true; $O(n^2) = n$ is
   nonsense. The convention: the left side is *more specific* than the
   right. You may transform left-to-right only.
2. **Constants are absorbed:** $3n^2 + 10n \log n = O(n^2)$; $H_n = O(\log n)$;
   $H_n = \ln n + O(1)$; $\ln n + \gamma + O(1/n)$ is *sharper* than $O(\log n)$ — carry
   the most precise form your argument can afford.
3. **O is an upper bound only.** To say quicksort *takes* order $n \log n$ on
   average you want $\Theta$ (matching upper and lower bounds); O alone permits
   wild overestimates.

From this module's inventory: $H_n = \ln n + \gamma + O(1/n)$;
$F_n = \varphi^n/\sqrt5 + O(\hat\varphi^n) = \varphi^n/\sqrt5 + O(0.62^n)$;
$\binom{2n}{n} = O(4^n/\sqrt n)$ (via Stirling, §1.2.11.2); and — coming right up
— the running time of Algorithm M is $a + b \cdot n + c \cdot A$ where only $A$
varies with the *arrangement* of the data.

---

## 7. Algorithm M and its analysis (§1.2.10)

Here everything converges. The algorithm itself is almost insultingly
simple — find the maximum of n values — but Knuth chose it precisely so the
*analysis* could be carried out completely, and it introduces the questions
this whole book asks of every algorithm: what is the best case, the worst
case, the *average* case, and the distribution around it?

### Algorithm M (Find the maximum)

Given elements $X[1], X[2], \ldots, X[n]$ with $n \ge 1$, find $m$ and $j$ such that
$m = X[j] = \max_{1 \le i \le n} X[i]$, where $j$ is the **largest** index attaining the
maximum.

```text
M1. [Initialize.]   Set j <- n, k <- n - 1, m <- X[n].
M2. [All tested?]   If k = 0, the algorithm terminates.
M3. [Compare.]      If X[k] <= m, go to M5.
M4. [Change m.]     Set j <- k, m <- X[k].
                    (m is the new current maximum.)
M5. [Decrease k.]   Decrease k by one, return to M2.
```

Note the two conventions with teeth: the scan runs from the **right** end
leftward, and M3's test is `<=` — a tie does *not* displace the current
maximum, which is how j ends up as the largest maximizing index.

**Hand trace** on $X = (7, 2, 9, 4, 8, 3)$, $n = 6$ (indices 1-based as in the
book; your Rust returns $j - 1$):

| step | k | X[k] | compare | j | m | A so far |
|------|---|------|---------|---|---|----------|
| M1 | — | — | — | 6 | 3 | 0 |
| M3 | 5 | 8 | 8 > 3 → M4 | 5 | 8 | 1 |
| M3 | 4 | 4 | 4 ≤ 8 | 5 | 8 | 1 |
| M3 | 3 | 9 | 9 > 8 → M4 | 3 | 9 | 2 |
| M3 | 2 | 2 | 2 ≤ 9 | 3 | 9 | 2 |
| M3 | 1 | 7 | 7 ≤ 9 | 3 | 9 | 2 |
| M2 | 0 | — | terminate | **3** | **9** | **2** |

### The analysis

Count each step's executions. M1 runs once; M2, M3, M5 run a fixed number of
times (M3 runs exactly $n - 1$ times — every element except the last is
compared exactly once). The only data-dependent quantity is

$$A = (\text{number of times step M4 executes}) = (\text{number of times the running maximum changes}).$$

Best case $A = 0$ (the maximum is already at position $n$); worst case $A = n - 1$
(strictly decreasing input, so every comparison wins). But what is $A$ *on
average*, over the $n!$ orderings of $n$ distinct values, each equally likely?

**The key observation.** Step M4 fires at position $k$ exactly when $X[k]$ is
larger than everything to its right: $X[k] > \max(X[k+1], \ldots, X[n])$. Call such
a position a *right-to-left maximum*. Position $n$ is always one (it seeds $m$
in M1, and does not count toward $A$); so

$$A = (\text{number of right-to-left maxima}) - 1.$$

**$E[A] = H_n - 1$.** For each $k$, look at the $n - k + 1$ values
$X[k], X[k+1], \ldots, X[n]$. In a random permutation, each of them is equally
likely to be the largest of the group, so

$$P(X[k] \text{ is a right-to-left maximum}) = \frac{1}{n - k + 1}.$$

By linearity of expectation (indicators $I_k$ for $k = 1, \ldots, n - 1$):

$$E[A] = \sum_{k=1}^{n-1} \frac{1}{n - k + 1} = \frac{1}{2} + \frac{1}{3} + \cdots + \frac{1}{n} = H_n - 1.$$

The average number of maximum-updates is *logarithmic* — that is harmonic
numbers earning their keep. For $n = 20$, $E[A] \approx 2.598$; for $n = 1000$, about
6.49. (Aside: the indicator events are in fact independent — knowing that
$X[k]$ beats everything to its right says nothing about the relative order
within $X[k+1..n]$ — which makes the next two results clean.)

**The full distribution.** Knuth pushes on to $P(A = k)$, and the generating
function is beautiful. Let $p_{nk} = P(A = k)$ and $G_n(z) = \sum_k p_{nk} z^k$.
Condition on where the *smallest* element sits, or directly multiply the
independent indicators: position $k$ contributes a factor
$((n-k) + z)/(n-k+1)$ — "no new maximum" with probability $(n-k)/(n-k+1)$,
"new maximum" (one factor of $z$) with probability $1/(n-k+1)$. Hence

$$G_n(z) = \frac{z+1}{2} \cdot \frac{z+2}{3} \cdots \frac{z+n-1}{n} = \frac{z(z+1)(z+2)\cdots(z+n-1)}{z \cdot n!}.$$

Sanity checks: $G_n(1) = 1$ ✓; the mean is $G_n'(1) = \sum_{k=2}^{n} 1/k = H_n - 1$
✓; the variance comes out to $H_n - H_n^{(2)}$ where $H_n^{(2)} = \sum 1/k^2$ (so the
standard deviation is about $\sqrt{\ln n}$ — $A$ is tightly concentrated). Expanding
the numerator polynomial $z(z+1)\cdots(z+n-1) = \sum_m {n \brack m} z^m$ defines the
**Stirling cycle numbers** ${n \brack m}$ (Stirling numbers of the first kind), so

$$P(A = k) = {n \brack k+1} \big/ n!.$$

For $n = 6$: the counts of permutations with $A = 0, 1, \ldots, 5$ are exactly
120, 274, 225, 85, 15, 1 (total $720 = 6!$) — your Stage 5 verifies this
histogram by brute force over all 720 permutations, then confirms
$E[A] = H_n - 1$ by Monte Carlo at $n = 20$. When a computation, a theorem, and
a simulation all agree, you may start to believe all three.

---

## 8. Stage-by-stage lab guide

Open `labs/module-02-math/src/lab.rs`. Each stage below has a test file
`labs/module-02-math/tests/stage_NN_*.rs`; run `./grade 2` to take the
stages in order. Everything is exact `u128`/`i128` integer arithmetic except
`harmonic_f64`. You may add private helpers (you *will* want a gcd).

### Stage 1 — `sum_first_n`, `sum_squares`, `sum_cubes`, `geometric_sum` (§1.2.3)

Closed forms only — the tests call `sum_first_n(10^12)` and friends, where a
loop would take hours. Mind two things: **exactness of division** (order
operations so each division is exact: $n(n+1)/2$ first, and for squares
multiply that by $(2n+1)$ before dividing by 3) and **the $x = 1$ case** of the
geometric sum, where the closed form divides by zero. `geometric_sum(x, n)`
sums $x^k$ for $k = 0, 1, \ldots, n$ (so $n + 1$ terms, and $x^0 = 1$ even at $x = 0$). The
tests re-derive the perturbation identities $S_{n+1} = S_n + x^{n+1} = 1 + x \cdot S_n$
— if your function satisfies both, you have re-proved the closed
form.

### Stage 2 — `binomial` (§1.2.6)

Exact $\binom{n}{k}$ in `u128`, zero when $k > n$, no overflow anywhere for $n \le 100$.
Use the multiplicative method (multiply-then-divide, symmetry first) or
build Pascal's triangle. The tests check Pascal's rule, symmetry, row sums
$2^n$, alternating sums, Vandermonde's convolution over a $20 \times 20$ grid, and the
anchor value $\binom{100}{50}$ — any `f64` detour or premature division dies there.

### Stage 3 — `fibonacci` (§1.2.8)

$F_n$ exact for $n \le 186$ (the `u128` limit); panic with a message containing
"overflows" beyond that. A simple iteration is fine — but be careful not to
compute $F_{n+1}$ when $n = 186$! The tests verify the table, $F_{100} =
354224848179261915075$, the addition law, Cassini, the gcd law over a $60 \times 60$
grid, and Binet's rounding property. Fast doubling is an optional flex.

### Stage 4 — `harmonic`, `harmonic_f64` (§1.2.7)

`harmonic(n)` returns $H_n$ as a reduced fraction `(numerator, denominator)`.
Accumulate $\text{num}/\text{den} + 1/k = (\text{num} \cdot k + \text{den})/(\text{den} \cdot k)$ and **reduce by the gcd
every step** — unreduced denominators reach $30! \approx 2.7 \times 10^{32}$ by $n = 30$ and
things only get worse. Panic for $n = 0$ (message containing "n >= 1").
`harmonic_f64` is a plain float loop; summing from $k = n$ down to 1 keeps
rounding error negligible. The asymptotic test brackets
$H_n - \ln n - \gamma$ between $1/(2n) - 1/(12n^2)$ and $1/(2n)$ for $n$ up to $10^5$ —
Euler's expansion, verified on your own machine.

### Stage 5 — `find_max`, `find_max_counting` (Algorithm 1.2.10M)

Implement Algorithm M step-faithfully: scan **right-to-left**, replace $m$
only on a **strict** increase (M3 tests $X[k] \le m$), return the 0-based index
(so Knuth's $j$ minus one) — ties keep the *rightmost* maximum, and the tests
check exactly that. `find_max_counting` additionally returns $A$, counting
executions of M4 only (M1's initialization is not a change). Then the tests
run §1.2.10's analysis against your code: the exact distribution
(120, 274, 225, 85, 15, 1) over all permutations of six elements, and a
10,000-trial Monte Carlo at $n = 20$ whose mean must land within 0.08 of
$H_{20} - 1 \approx 2.5977$. Panic on the empty slice (message containing "n >= 1").

---

## 9. Check your understanding

Answer before moving on (no code needed):

1. The perturbation method applied to $\sum_{k=0}^{n} k \cdot 2^k$ needs the geometric
   sum as an input. Derive the closed form $(n - 1) \cdot 2^{n+1} + 2$ and check it
   at $n = 2$.
2. Why is the multiplicative method's division `c * (n-k+i) / i` always
   exact, and why would `c * ((n-k+i) / i)` be wrong?
3. $H_{1{,}000{,}000} \approx {?}$ (Two terms of the expansion suffice: $\ln 10^6 + \gamma \approx
   13.816 + 0.577 \approx 14.39$.) How many terms of the harmonic series until the
   sum exceeds 100?  (About $e^{100-\gamma} \approx 1.5 \times 10^{43}$ — "unbounded, but
   barely.")
4. In Algorithm M, why does $P(X[k] \text{ is a right-to-left maximum}) = 1/(n-k+1)$
   need the values to be *distinct*? What happens to $E[A]$ if all elements
   are equal?
5. If Algorithm M scanned left-to-right instead (with $<$ vs $\le$ chosen to
   keep the *leftmost* maximum), would the distribution of $A$ change? (No —
   by symmetry of random permutations under reversal.)

## 10. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 fifteen minutes
to an hour · 30 hours · 40 term project · 50 open research problem; M =
mathematically oriented, ▶ = especially instructive. Statements are
paraphrased; match by content if your printing numbers differ. Log your work
in `course/module-02-math/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 1.2.1-8 | M25 | Fermat's little theorem territory: prove $n^p \equiv n \pmod{p}$ for prime $p$, by induction on $n$ using the binomial theorem. |
| 1.2.3-16 | M20 | Derive $\sum_{k=0}^{n} k \cdot 2^k$ in closed form by the perturbation method (see check-question 1). |
| ▶1.2.6-10 | M20 | Show directly that $k!$ divides the product of any $k$ consecutive integers — i.e., $\binom{n}{k}$ is an integer *without* invoking Pascal's rule. (Hint: count multiples of each prime, or interpret combinatorially.) |
| 1.2.6-21 | M20 | Prove Vandermonde's convolution both ways: committee-counting and coefficient-matching in $(1+x)^m (1+x)^n$. |
| 1.2.7-21 | M30 | Prove that $H_n$ is never an integer for $n > 1$. (Hint: exactly one term has the maximal power of 2 in its denominator.) |
| 1.2.8-25 | M22 | Derive the fast-doubling formulas $F_{2n} = F_n(2F_{n+1} - F_n)$, $F_{2n+1} = F_n^2 + F_{n+1}^2$ from the addition law, and use them for an $O(\log n)$ `fibonacci`. |
| ▶1.2.10-... | M21 | Compute the variance of $A$ in Algorithm M from $G_n(z)$, and verify $\operatorname{Var}(A) = H_n - H_n^{(2)}$ numerically for $n = 6$ against the exact histogram. |
| 1.2.11.1-... | M15 | True or false, with proof or counterexample: (a) $O(n) + O(n) = O(n)$; (b) $2^{O(n)} = O(2^n)$; (c) if $f = O(g)$ then $\ln f = O(\ln g)$. |

## Why it's done this way

Knuth spends a hundred pages on "preliminaries" because analysis of
algorithms is *applied* discrete mathematics: every later cost function is a
sum, every average is an expectation, every bound is an asymptotic statement.
The module's shape — closed forms before asymptotics, exact rationals before
floating approximations — mirrors his discipline: *compute exactly first,
approximate knowingly second*. That is why stage 4 makes you build $H_n$ as an
exact fraction before comparing it with $\ln n + \gamma$.

## In the real world

Harmonic numbers are the most-quoted constant in practical average-case
analysis: the expected number of times a "best so far" record updates
(Algorithm M — your stage 5) is $H_n - 1$, which is why streaming max/min
loops and secretary-problem-style cutoff rules are cheap in practice.
Binomial coefficients price out test-coverage combinatorics and reliability
calculations. Fast-doubling Fibonacci is the toy case of evaluating any
linear recurrence in $O(\log n)$ time — the trick behind Markov-chain powers
and path counting in graphs. And O-notation is simply the contract language
of the industry; this module is where you learn to read its fine print
(the constants, and where they hide).

## Proof techniques you practiced

- **Induction**, stated honestly with base case and hypothesis — the engine
  under every identity here.
- **The perturbation method** — derive a sum by shifting it against itself
  (the geometric series here; quicksort's recurrence in Module 06).
- **Double counting** — Vandermonde's convolution counts one committee two
  ways.
- **Linearity of expectation** — $E[A] = H_n - 1$ without ever touching the
  (messy) distribution of $A$ itself.
- **Exact-then-asymptotic** — $H_n$ as a reduced fraction first, then the
  $|H_n - \ln n - \gamma| < 1/(2n)$ approximation with an error you can certify.

## 11. Where this leads

- **$H_n - 1$ is a template.** The same indicator-plus-linearity argument
  gives quicksort's average comparisons $\approx 2n \ln n$ (Module 06), the expected
  depth of a random BST $\approx 2 \ln n$ (Module 07), and the analysis of records,
  cycles, and runs throughout Vols. 1–3.
- **Stirling cycle numbers** ${n \brack m}$, met here as the distribution of $A$,
  return as the permutation-cycle statistics of §1.3.3 and the analysis of
  in-situ permutation.
- **Binomial coefficients** are the currency of Module 08 (combinatorial
  generation: Algorithm 7.2.1.3T generates the $\binom{n}{k}$ combinations) and of
  every counting argument in Vol. 4.
- **Fibonacci numbers** reappear in polyphase merging (Vol. 3, §5.4.2) and
  Fibonacci hashing (§6.4); the golden ratio's continued fraction
  $[1; 1, 1, \ldots]$ is *why* Fibonacci inputs are Euclid's worst case.
- **The LCG** you used to shuffle test data is the star of Module 04
  (Vol. 2, Ch. 3) — where you'll finally test whether it deserves the name
  "random".
