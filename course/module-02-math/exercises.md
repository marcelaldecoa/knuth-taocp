# Exercises — Module 02 (Mathematical Preliminaries, §1.2)

Self-contained problems on this module's toolkit — induction, sums by the
perturbation method, binomial coefficients, Fibonacci numbers, and the analysis
of Algorithm M. You can work every one **without the book**: each states the
problem in full, gives a **hint** to peek at when stuck, and a worked **answer
sketch** to check against. Every numeric value below is reproduced by a few
lines at a REPL (or the exact-arithmetic code you write in the lab).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 1.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Induction warm-up: sum of cubes (Nicomachus) | 10 | ⬜ |
| 2 | ▶ Perturbation method: derive $\sum_{k} k\,x^k$ | 20 | ⬜ |
| 3 | A binomial identity by Pascal's rule: $\sum_k \binom{n}{k} = 2^n$ | 17 | ⬜ |
| 4 | ▶ $k!$ divides any product of $k$ consecutive integers | 25 | ⬜ |
| 5 | A Fibonacci identity by induction: Cassini | 20 | ⬜ |
| 6 | The variance of $A$ in Algorithm M | 27 | ⬜ |

## Problems

### 1. Induction warm-up: sum of cubes (Nicomachus) (rating 10 · cf. 1.2.1–1)

**Problem.** Prove by mathematical induction that

$$
\sum_{k=1}^{n} k^3 = \left(\frac{n(n+1)}{2}\right)^2 \quad\text{for all } n \ge 1.
$$

That is: the sum of the first $n$ cubes equals the square of the $n$-th
triangular number.

**Hint.** Basis $n = 1$ is immediate. For the step, add $(n+1)^3$ to the assumed
formula and factor the result into $\big((n+1)(n+2)/2\big)^2$ — the lesson states
exactly this factoring move.

**Answer sketch.** *Basis:* $1^3 = 1 = (1 \cdot 2 / 2)^2$. *Induction:* assume
$\sum_{k=1}^{n} k^3 = \big(n(n+1)/2\big)^2$. Then

$$
\sum_{k=1}^{n+1} k^3 = \left(\frac{n(n+1)}{2}\right)^2 + (n+1)^3
= (n+1)^2\left(\frac{n^2}{4} + (n+1)\right)
= (n+1)^2 \cdot \frac{n^2 + 4n + 4}{4} = \left(\frac{(n+1)(n+2)}{2}\right)^2,
$$

which is the formula at $n+1$. $\blacksquare$ (Numerical check: $1+8+27+64 = 100 =
(4\cdot 5/2)^2 = 10^2$.)

### 2. ▶ Perturbation method: derive $\sum_{k} k\,x^k$ (rating 20 · cf. 1.2.3–16)

**Problem.** Using the perturbation method (compute $S_{n+1}$ two ways, by
splitting off the last term and the first term, then solve), derive a closed
form for $S = \sum_{k=0}^{n} k\,x^k$. Specialize to $x = 2$ and confirm the result
$\sum_{k=0}^{n} k\,2^k = (n-1)\,2^{n+1} + 2$.

**Hint.** You will need the geometric sum $\sum_{k=0}^{n} x^k = \dfrac{x^{n+1}-1}{x-1}$
(derived in the lesson by the same trick) as an input, because splitting off the
first term leaves a $\sum x^k$ behind.

**Answer sketch.** Let $S = \sum_{k=0}^{n} k\,x^k$. Split $S_{n+1} = \sum_{k=0}^{n+1} k x^k$
two ways. Last term: $S_{n+1} = S + (n+1)x^{n+1}$. First term (reindex
$k \to k+1$): $S_{n+1} = \sum_{k=0}^{n}(k+1)x^{k+1} = x\sum_{k=0}^{n} k x^k + x\sum_{k=0}^{n} x^k
= xS + x\,G$, where $G = \dfrac{x^{n+1}-1}{x-1}$. Equating,
$S + (n+1)x^{n+1} = xS + xG$, so

$$
S(1-x) = xG - (n+1)x^{n+1} = \frac{x(x^{n+1}-1)}{x-1} - (n+1)x^{n+1},
\qquad S = \frac{x - (n+1)x^{n+1} + n\,x^{n+2}}{(1-x)^2}\quad (x \ne 1).
$$

At $x = 2$: denominator $(1-2)^2 = 1$, numerator $2 - (n+1)2^{n+1} + n\,2^{n+2}
= 2 + 2^{n+1}\big(2n - (n+1)\big) = (n-1)2^{n+1} + 2$. So
$\sum_{k=0}^{n} k\,2^k = (n-1)2^{n+1} + 2$. (Checks: $n=2$ gives $1\cdot 8 + 2 = 10 =
0 + 2 + 8$; $n=4$ gives $3\cdot 32 + 2 = 98$.)

### 3. A binomial identity by Pascal's rule: $\sum_k \binom{n}{k} = 2^n$ (rating 17 · cf. 1.2.6)

**Problem.** Prove the row-sum identity $\sum_{k=0}^{n} \binom{n}{k} = 2^n$ by
induction on $n$, using **Pascal's rule** $\binom{n}{k} = \binom{n-1}{k-1} +
\binom{n-1}{k}$ (rather than the binomial theorem).

**Hint.** Let $R_n = \sum_k \binom{n}{k}$. Substitute Pascal's rule into $R_n$ and
watch each row-$(n-1)$ term get counted twice.

**Answer sketch.** *Basis:* $R_0 = \binom{0}{0} = 1 = 2^0$. *Induction:* assume
$R_{n-1} = 2^{n-1}$. Using Pascal's rule on every interior term (with the
boundary convention $\binom{n-1}{-1} = \binom{n-1}{n} = 0$),

$$
R_n = \sum_{k=0}^{n}\binom{n}{k}
= \sum_{k=0}^{n}\left(\binom{n-1}{k-1} + \binom{n-1}{k}\right)
= \sum_{k}\binom{n-1}{k-1} + \sum_{k}\binom{n-1}{k} = R_{n-1} + R_{n-1} = 2R_{n-1}.
$$

Each row-$(n-1)$ entry appears once in each of the two shifted sums, so the total
doubles. Hence $R_n = 2 \cdot 2^{n-1} = 2^n$. $\blacksquare$ (This is the same
addition-formula engine that proves every $\binom{n}{k}$ is an integer — the
recurrence $R_n = 2R_{n-1}$ is Pascal's rule summed across a row.)

### 4. ▶ $k!$ divides any product of $k$ consecutive integers (rating 25 · cf. 1.2.6–10)

**Problem.** Show directly — *without* invoking Pascal's rule or an induction on
the triangle — that $k!$ divides the product of any $k$ consecutive integers,
i.e. that

$$
\binom{n}{k} = \frac{n(n-1)\cdots(n-k+1)}{k!}
$$

is always an integer for integers $n \ge k \ge 0$.

**Hint.** The lesson's *combinatorial* definition of $\binom{n}{k}$ is the key:
it counts something, and a count is a non-negative integer regardless of the
formula you compute it with.

**Answer sketch.** By the lesson's definition, $\binom{n}{k}$ is the number of
$k$-element subsets of an $n$-element set — an integer, because it counts
objects. But that same quantity equals $\dfrac{n(n-1)\cdots(n-k+1)}{k!}$: form an
ordered selection of $k$ distinct elements ($n(n-1)\cdots(n-k+1)$ ways), then
note each $k$-subset arises from exactly $k!$ orderings, so dividing by $k!$
counts unordered subsets. Since the left side (a count) is an integer, $k!$ must
divide the numerator $n(n-1)\cdots(n-k+1)$ — the product of $k$ consecutive
integers. $\blacksquare$ This is *double counting*: the integrality is free once
you see the quotient is really a count. (An alternative prime-power argument —
show each prime $p$ divides the numerator at least as often as the denominator —
also works but is longer; the combinatorial one is the instructive route.)

### 5. A Fibonacci identity by induction: Cassini (rating 20 · cf. 1.2.8, Eq. 8)

**Problem.** With Knuth's indexing $F_0 = 0$, $F_1 = 1$, $F_{n+1} = F_n + F_{n-1}$,
prove **Cassini's identity**

$$
F_{n-1}\,F_{n+1} - F_n^2 = (-1)^n \quad\text{for all } n \ge 1
$$

by induction on $n$.

**Hint.** Basis $n = 1$: $F_0 F_2 - F_1^2 = 0 \cdot 1 - 1 = -1 = (-1)^1$. For the
step, express $F_{n+2} = F_{n+1} + F_n$ and reduce $F_n F_{n+2} - F_{n+1}^2$ to the
previous case; the sign flips each time.

**Answer sketch.** *Basis* ($n=1$): $F_0 F_2 - F_1^2 = 0\cdot 1 - 1^2 = -1 =
(-1)^1$. ✓ *Induction:* assume $F_{n-1}F_{n+1} - F_n^2 = (-1)^n$. Then, using
$F_{n+2} = F_{n+1} + F_n$,

$$
F_n F_{n+2} - F_{n+1}^2 = F_n(F_{n+1}+F_n) - F_{n+1}^2
= F_{n+1}(F_n - F_{n+1}) + F_n^2 = -F_{n+1}F_{n-1} + F_n^2
$$

(since $F_n - F_{n+1} = -F_{n-1}$) $= -\big(F_{n-1}F_{n+1} - F_n^2\big)
= -(-1)^n = (-1)^{n+1}$, which is Cassini at $n+1$. $\blacksquare$ (Spot check:
$n = 6$, $F_5 F_7 - F_6^2 = 5\cdot 13 - 8^2 = 65 - 64 = 1 = (-1)^6$.) The sign
alternation is the algebra behind the classic $8\times 8 \to 5\times 13$
dissection "paradox" mentioned in the lesson.

### 6. The variance of $A$ in Algorithm M (rating 27 · cf. 1.2.10)

**Problem.** Algorithm M scans $X[1..n]$ right-to-left and $A$ counts how often
the running maximum changes (executions of step M4). Its generating function is

$$
G_n(z) = \frac{z+1}{2}\cdot\frac{z+2}{3}\cdots\frac{z+n-1}{n}.
$$

Show that $\operatorname{Var}(A) = H_n - H_n^{(2)}$, where $H_n = \sum_{k=1}^{n} 1/k$
and $H_n^{(2)} = \sum_{k=1}^{n} 1/k^2$, and verify it numerically for $n = 6$
against the exact histogram of $A$ (counts $120, 274, 225, 85, 15, 1$ for
$A = 0, 1, \dots, 5$).

**Hint.** Write $A = \sum_{k=1}^{n-1} I_k$ as a sum of **independent** indicators,
where $I_k = 1$ with probability $p_k = 1/(n-k+1)$ (the lesson proves position
$k$ is a right-to-left maximum with this probability, and notes the indicators
are independent). For independent indicators, variances add, and
$\operatorname{Var}(I_k) = p_k(1 - p_k)$.

**Answer sketch.** Because the $I_k$ are independent Bernoulli$(p_k)$ with
$p_k = 1/(n-k+1)$, letting $j = n-k+1$ range over $2, 3, \dots, n$,

$$
\operatorname{Var}(A) = \sum_{k=1}^{n-1} p_k(1-p_k)
= \sum_{j=2}^{n}\frac{1}{j}\left(1 - \frac{1}{j}\right)
= \sum_{j=2}^{n}\left(\frac{1}{j} - \frac{1}{j^2}\right)
= (H_n - 1) - (H_n^{(2)} - 1) = H_n - H_n^{(2)}.
$$

(Equivalently this drops out of $\operatorname{Var}(A) = G_n''(1) + G_n'(1) -
G_n'(1)^2$ with $G_n'(1) = E[A] = H_n - 1$.) *Numerical check, $n = 6$:* the exact
histogram $(120, 274, 225, 85, 15, 1)/720$ has mean $E[A] = 1044/720 = 1.45 =
H_6 - 1$ and $E[A^2] = 2204/720$, giving $\operatorname{Var}(A) = E[A^2] - E[A]^2
\approx 0.95861$. And $H_6 - H_6^{(2)} = \tfrac{49}{20} - \tfrac{5369}{3600}
\approx 2.45 - 1.49139 = 0.95861$. The two agree. $\blacksquare$ (So the standard
deviation is only about $\sqrt{\ln n}$: $A$ is tightly concentrated.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
