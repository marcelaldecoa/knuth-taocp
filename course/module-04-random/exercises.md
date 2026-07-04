# Exercises — Module 04 (Random Numbers)

Self-contained problems on this module's material — the LCG period, the
Hull–Dobell full-period test, the chi-square test, shuffling, and reservoir
sampling. You can work every one **without the books**: each states the problem
in full, gives a **hint** to peek at when stuck, and a worked **answer sketch**
to check against after you try. Computational answers here are reproduced by the
code you write in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 2.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Period is at most $m$; exhibit a full-period generator | 10 | ⬜ |
| 2 | ▶ Hull–Dobell "if" direction for $m = 2^e$ | M23 | ⬜ |
| 3 | Cycle structure of the $7,7,7 \bmod 10$ generator | 10 | ⬜ |
| 4 | Chi-square: compute $V$ and decide | 20 | ⬜ |
| 5 | ▶ Reservoir sampling keeps each item with probability $1/n$ | 22 | ⬜ |
| 6 | Expected replacements in reservoir sampling | M25 | ⬜ |

## Problems

### 1. Period is at most $m$; exhibit a full-period generator (rating 10 · cf. 3.2.1.2–1)

**Problem.** Show that the linear congruential sequence
$X_{k+1} = (aX_k + c) \bmod m$ has period at most $m$, from any seed. Then
exhibit values $a, c$ giving period exactly $m$ for $m = 16$.

**Hint.** Each $X_k$ lies in $\{0, 1, \dots, m-1\}$, and $X_{k+1}$ is a function
of $X_k$ alone. What must happen once you have generated $m+1$ values?

**Answer sketch.** There are only $m$ possible values, so among
$X_0, X_1, \dots, X_m$ ($m+1$ of them) two must coincide, say $X_i = X_j$ with
$i < j$. Because the next value depends only on the current one, the sequence
repeats from there with period dividing $j - i \le m$. Hence the period is at
most $m$. For $m = 16$, Hull–Dobell (Problem 2) gives full period $16$ whenever
$a \equiv 1 \pmod 4$ and $c$ is odd — e.g. `period(5, 3, 16, 0) == 16`. By
contrast `period(3, 1, 16, 0) < 16` ($3 \equiv 3 \pmod 4$) and
`period(5, 2, 16, 0) < 16` ($c$ even).

### 2. ▶ Hull–Dobell "if" direction for $m = 2^e$ (rating M23 · cf. 3.2.1.2–4)

**Problem.** Theorem 3.2.1.2A says the LCG has full period $m$ iff (i)
$\gcd(c, m) = 1$; (ii) $a \equiv 1 \pmod p$ for every prime $p \mid m$; and
(iii) $a \equiv 1 \pmod 4$ if $4 \mid m$. Prove the *if* direction for the common
case $m = 2^e$ with $e \ge 2$: if $a \equiv 1 \pmod 4$ and $c$ is odd, the period
is $2^e$.

**Hint.** For $m = 2^e$ the three conditions collapse — the only prime dividing
$m$ is $2$, and $4 \mid m$. Translate each condition into a statement about $a$
and $c$ modulo small powers of two.

**Answer sketch.** With $m = 2^e$, $e \ge 2$: condition (i) $\gcd(c, 2^e) = 1$
is exactly "$c$ is odd"; condition (ii) requires $a \equiv 1 \pmod 2$, i.e. $a$
odd; condition (iii) requires $a \equiv 1 \pmod 4$. Now $a \equiv 1 \pmod 4$
already forces $a$ odd, so it implies (ii) and (iii); and $c$ odd is (i). Thus
the hypotheses give all three conditions, and the theorem yields period exactly
$2^e$. (The full proof that the three conditions are *sufficient* — the harder
"if" of the general theorem — is Knuth's; here we only reduce the $2^e$ case to
it, which is the instructive step and matches what Stage 1 checks empirically.)

### 3. Cycle structure of the $7,7,7 \bmod 10$ generator (rating 10 · cf. 3.2.1–2)

**Problem.** Consider $X_{k+1} = (7X_k + 7) \bmod 10$. Describe its full cycle
structure: which values lead into which cycles, and what periods occur? Does it
have full period?

**Hint.** Is the map $x \mapsto (7x + 7) \bmod 10$ a bijection on
$\{0, \dots, 9\}$? Compare $\gcd(7, 10)$ with $1$. A bijection has no "tails."

**Answer sketch.** Since $\gcd(7, 10) = 1$, multiplication by $7$ is invertible
mod $10$, so $x \mapsto 7x + 7$ is a **bijection** — every state has a unique
predecessor, hence there are **no tails**, only disjoint cycles. Tracing them:
$\{0 \to 7 \to 6 \to 9 \to 0\}$ (period $4$), $\{1 \to 4 \to 5 \to 2 \to 1\}$
(period $4$), and $\{3 \to 8 \to 3\}$ (period $2$). The period from a seed is the
length of its cycle, so this generator does **not** have full period $10$ — its
multiplier $a = 7 \equiv 3 \pmod 4$ fails Hull–Dobell condition (iii). (Starting
from $X_0 = 7$ you see $7, 6, 9, 0, 7, 6, 9, 0, \dots$.)

### 4. Chi-square: compute $V$ and decide (rating 20 · cf. 3.3.1–8)

**Problem.** A die is rolled $n = 60$ times with observed face counts
$Y = (16, 8, 9, 6, 12, 9)$ for faces $1$–$6$. Under the fair-die hypothesis each
face is expected $n/6 = 10$ times. Compute the chi-square statistic $V$ and
decide, at the $95\%$ level, whether to reject fairness. (The $95\%$ point of the
chi-square distribution with $5$ degrees of freedom is $\approx 11.07$.)

**Hint.** $V = \sum_{s=1}^{6} \dfrac{(Y_s - np_s)^2}{np_s}$ with $np_s = 10$.
This is exactly what `chi_square_uniform(&[16,8,9,6,12,9])` returns; reject if
$V$ exceeds the tabulated critical value.

**Answer sketch.** With every expected count $10$,

$$
V = \frac{(6)^2 + (-2)^2 + (-1)^2 + (-4)^2 + (2)^2 + (-1)^2}{10}
= \frac{36 + 4 + 1 + 16 + 4 + 1}{10} = \frac{62}{10} = 6.2.
$$

There are $k - 1 = 5$ degrees of freedom. Since $6.2 < 11.07$, $V$ is **not** in
the upper $5\%$ tail, so we do **not** reject the fair-die hypothesis — the
counts are consistent with a fair die. (A $V$ near $0$ would be *too* good, and a
$V$ above $\approx 11$ would be grounds to suspect the die.)

### 5. ▶ Reservoir sampling keeps each item with probability $1/n$ (rating 22 · cf. 3.4.2–8)

**Problem.** Algorithm R (Stage 3, single-element reservoir) reads a stream
$x_1, x_2, \dots, x_n$ of unknown length: it keeps $x_1$, then for each
$i \ge 2$ replaces the stored sample by $x_i$ with probability $1/i$. Prove that
after processing all $n$ items, every $x_j$ is the retained sample with
probability exactly $1/n$.

**Hint.** Induct on the stream length. Assume the claim holds after $n-1$ items;
what happens to those probabilities when the $n$-th item arrives and is accepted
with probability $1/n$?

**Answer sketch.** Induction on $n$. *Base* $n = 1$: $x_1$ is kept with
probability $1 = 1/1$. *Step:* suppose after $n-1$ items each of
$x_1, \dots, x_{n-1}$ is stored with probability $\tfrac{1}{n-1}$. On item $n$:
$x_n$ becomes the sample with probability $\tfrac{1}{n}$ (correct). Each earlier
$x_j$ remains stored iff it was stored *and* $x_n$ is rejected, i.e. with
probability $\tfrac{1}{n-1}\cdot\big(1 - \tfrac1n\big)
= \tfrac{1}{n-1}\cdot\tfrac{n-1}{n} = \tfrac1n.$ So all $n$ items are equally
likely, each with probability $1/n$. $\blacksquare$

### 6. Expected replacements in reservoir sampling (rating M25 · cf. 3.4.2–10)

**Problem.** Over a stream of $n$ items, how many times does the single-element
reservoir of Problem 5 get *replaced* (the stored sample overwritten), on
average?

**Hint.** Let $R_i$ be the indicator that item $i$ triggers a replacement.
$\mathbb{E}[R_i] = 1/i$ for $i \ge 2$. Use linearity of expectation — no
independence needed.

**Answer sketch.** The stored sample is overwritten at step $i$ (for
$i = 2, \dots, n$) with probability $1/i$, independently of the value stored.
By linearity,

$$
\mathbb{E}[\text{replacements}] = \sum_{i=2}^{n} \frac{1}{i} = H_n - 1,
$$

where $H_n = \sum_{i=1}^{n} 1/i$ is the $n$-th harmonic number. So the count
grows like $\ln n$: about $1.93$ replacements for $n = 10$, and $\approx \ln n +
\gamma - 1$ for large $n$ — strikingly few, which is why reservoir sampling is
cheap. (You met $H_n$ in Module 02; it is the recurring cost signature of "touch
item $i$ with probability $1/i$.")

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
