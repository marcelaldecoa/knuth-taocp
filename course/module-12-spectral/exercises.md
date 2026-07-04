# Exercises — Module 12 (The Spectral Test)

Self-contained problems on this module's material — the monotonicity of
$\nu_t$, the increment translation, plane counting, Gauss–Lagrange optimality,
Algorithm C's certified search, and spectral figures of merit. You can work
every one **without the books**: each states the problem in full, gives a
**hint** to peek at when stuck, and a worked **answer sketch** to check against
after you try. Computational answers here are reproduced by the code you write
in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). The `W`-numbers are this course's curated set for the $t \le 3$
build; if you own Volume 2, the exercises of §3.3.4 continue the story (several
assume the full Algorithm S).

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | ▶ Monotonicity $\nu_{t+1} \le \nu_t$: pad with a zero | M10 | ⬜ |
| 2 | The increment-translation proof, in full | M18 | ⬜ |
| 3 | Plane count $N + P - 1$; the hidden assumption | M22 | ⬜ |
| 4 | ▶ Gauss–Lagrange optimality, every inequality | M24 | ⬜ |
| 5 | Instrument Algorithm C; the $4/3$ doubling bound | 22 | ⬜ |
| 6 | ▶ Certified search at $t = 4$; RANDU's $\nu_4$ | 30 | ⬜ |
| 7 | A mini Fishman–Moore search for $m = 2^{31} - 1$ | M28 | ⬜ |

## Problems

### 1. ▶ Monotonicity $\nu_{t+1} \le \nu_t$: pad with a zero (rating M10 · cf. W1)

**Problem.** Recall $\nu_t^2 = \min\{\,u_1^2 + \cdots + u_t^2 : u_1 + a u_2 + \cdots
+ a^{t-1}u_t \equiv 0 \pmod{m},\ u \ne 0\,\}$. Prove that $\nu_{t+1} \le \nu_t$ for
every multiplier $a$ and modulus $m$ — so a generator's spectral quality can only
degrade as the dimension grows.

**Hint.** Take a shortest dual vector in dimension $t$ and append a zero
coordinate. Is the result a legal dual vector in dimension $t + 1$, and what is its
length?

**Answer sketch.** Let $u = (u_1, \ldots, u_t)$ achieve the minimum, so
$u_1 + a u_2 + \cdots + a^{t-1}u_t \equiv 0 \pmod m$, $u \ne 0$, and $\|u\|^2 =
\nu_t^2$. Pad it: $u' = (u_1, \ldots, u_t, 0)$. Its dual condition in dimension
$t + 1$ is $u_1 + a u_2 + \cdots + a^{t-1}u_t + a^t\cdot 0 \equiv 0 \pmod m$ — the
*same* congruence, already satisfied. And $u' \ne 0$ with $\|u'\|^2 = \|u\|^2 =
\nu_t^2$. Since $\nu_{t+1}^2$ is the minimum over *all* dimension-$(t+1)$ dual
vectors, it is at most this one: $\nu_{t+1}^2 \le \nu_t^2$. $\blacksquare$
Sanity check against the lesson's Table 1: the toy generator has $\nu_2^2 = 274$,
$\nu_3^2 = 30$ ($30 \le 274$); RANDU has $\nu_2^2 = 2\,147\,221\,514$,
$\nu_3^2 = 118$ ($118 \le \nu_2^2$). The moral: passing the test in low dimensions
guarantees nothing higher up — you must test every $t$ your application consumes.

### 2. The increment-translation proof, in full (rating M18 · cf. W2)

**Problem.** The lesson works with $c = 0$ throughout, claiming the increment
merely *translates* the point set and so changes no $\nu_t$. Prove it rigorously.
Compare the sequence $x_{n+1} = (a x_n + c) \bmod m$ with the $c = 0$ sequence from
the same $x_0$, and show every $t$-tuple $P_n(c)$ equals a lattice point plus one
**fixed** vector $w_t$ independent of $n$. Exhibit $w_t$ explicitly.

**Hint.** Write $S_j = a^{j-1} + \cdots + a + 1$ (so $S_0 = 0$). Prove the
splitting identity $S_{n+j} = a^j S_n + S_j$, then express coordinate $j$ of
$P_n(c)$ in terms of the tuple's own leading value $x_n(c)$.

**Answer sketch.** By induction, $x_n(c) = a^n x_0 + S_n c \pmod m$. Coordinate
$j$ ($0 \le j \le t-1$) of the tuple starting at position $n$ is
$x_{n+j}(c) = a^{n+j}x_0 + S_{n+j}c$. Using $S_{n+j} = a^j S_n + S_j$ (proved by
splitting the sum $a^{n+j-1} + \cdots + 1$ at the term $a^j$),

$$x_{n+j}(c) = a^j\big(a^n x_0 + S_n c\big) + S_j c = a^j\, x_n(c) + S_j c \pmod m.$$

Therefore, with $v = (1, a, a^2, \ldots, a^{t-1})$,

$$P_n(c) = x_n(c)\cdot v + w_t \pmod m, \qquad w_t = (S_0 c,\, S_1 c,\, \ldots,\, S_{t-1}c) = (0,\, c,\, (a+1)c,\, (a^2+a+1)c,\, \ldots).$$

The first term $x_n(c)\cdot v$ is a lattice point (as $n$ varies, $x_n(c)$ runs
through the same residues the $c = 0$ sequence does), and $w_t$ is a **single
fixed vector** — verified numerically: for $a = 3, m = 100, c = 7$, one finds
$w_3 = (0, 7, 28)$ and $P_n(c) = x_n(c)v + w_3 \pmod m$ for every $n$. So the whole
$c$-point-set is the $c = 0$ set rigidly translated by $w_t$. Translations move no
inter-point spacings and shift plane families bodily without changing their count
or separation, so every quantity the spectral test measures — $\nu_t$, plane
counts, gaps — is independent of $c$. $\blacksquare$

### 3. Plane count $N + P - 1$; the hidden assumption (rating M22 · cf. W3)

**Problem.** A dual vector $u$ with entries of both signs cuts the point set with
the family of parallel planes $u \cdot x = km$. Split $u$ into positive and
negative parts, $P = \sum_{u_i > 0} u_i$ and $N = \sum_{u_i < 0} |u_i|$. Prove that
exactly $N + P - 1$ of these planes meet the cube $[0, m)^t$. Then identify the
assumption the counting argument quietly relies on, and confirm the count for
RANDU's $u = (9, -6, 1)$.

**Hint.** Bound the linear form $u \cdot x$ over $x \in [0, m)^t$; its extreme
values are attained by pushing each coordinate to $0$ or $m - 1$ according to the
sign of $u_i$. The planes are indexed by the integers $k = (u\cdot x)/m$ inside
that range.

**Answer sketch.** For $x \in [0, m)^t$, each coordinate $x_i \in \{0, \ldots,
m-1\}$, so $u \cdot x$ is maximized by setting $x_i = m - 1$ where $u_i > 0$ and
$x_i = 0$ where $u_i < 0$, giving $(m-1)P$; it is minimized symmetrically at
$-(m-1)N$. Thus

$$-(m-1)N \ \le\ u\cdot x \ \le\ (m-1)P.$$

A covering plane has index $k = (u\cdot x)/m$, so $k$ ranges over the integers in
$[-(m-1)N/m,\ (m-1)P/m]$. Since $(m-1)N/m < N$ and $(m-1)P/m < P$, the integer $k$
runs from $-(N-1)$ to $P - 1$ inclusive — that is $(N - 1) + (P - 1) + 1 = N + P - 1$
values. **Hidden assumption:** each of those $N + P - 1$ candidate planes actually
*contains an integer point of the cube* (otherwise it would not "meet" the point
set and the count would be an over-estimate). This holds because $\gcd$ of the
$u_i$ can be taken $1$ for a shortest dual vector, making $u\cdot x = km$ solvable in
the cube for each admissible $k$. For RANDU, $u = (9, -6, 1)$ gives $P = 10$,
$N = 6$, so $N + P - 1 = 15$ planes carry all $2^{29}$ triples — at most
$|9| + |6| + |1| = 16$, and one fewer because both signs are present. Fifteen
gaping slabs in a cube $2^{31}$ cells on a side: Marsaglia's title was not
hyperbole.

### 4. ▶ Gauss–Lagrange optimality, every inequality (rating M24 · cf. W4)

**Problem.** Algorithm G reduces the rank-$2$ dual basis $v_1 = (m, 0)$,
$v_2 = (-a, 1)$ until $\|v_1\| \ge \|v_2\|$ and $|v_1 \cdot v_2| \le \|v_2\|^2/2$,
then declares $v_2$ shortest. Fill in every inequality of the optimality proof:
show that any nonzero lattice vector $w = \alpha v_1 + \beta v_2$
($\alpha, \beta \in \mathbb{Z}$) has $\|w\| \ge \|v_2\|$. Then confirm the hand
trace on $a = 137$, $m = 256$, and explain where integrality is load-bearing.

**Hint.** The cross case ($\alpha, \beta$ both nonzero) reduces to showing
$\alpha^2 - |\alpha\beta| + \beta^2 \ge 1$ for nonzero integers. Complete the
square.

**Answer sketch.** At termination two facts hold: $\|v_1\| \ge \|v_2\|$ (from the
ordering step), and $|v_1\cdot v_2| \le \|v_2\|^2/2$ (from rounding the shear
coefficient to the *nearest* integer). Take $w = \alpha v_1 + \beta v_2$, $w \ne 0$.
If $\beta = 0$: $\|w\| = |\alpha|\,\|v_1\| \ge \|v_2\|$. If $\alpha = 0$:
$\|w\| = |\beta|\,\|v_2\| \ge \|v_2\|$. Otherwise,

$$
\begin{aligned}
\|w\|^2 &= \alpha^2\|v_1\|^2 + 2\alpha\beta(v_1\cdot v_2) + \beta^2\|v_2\|^2 \\
&\ge \alpha^2\|v_2\|^2 - 2|\alpha||\beta|\cdot\tfrac12\|v_2\|^2 + \beta^2\|v_2\|^2
 = \big(\alpha^2 - |\alpha||\beta| + \beta^2\big)\|v_2\|^2,
\end{aligned}
$$

using $\|v_1\|^2 \ge \|v_2\|^2$ and $|v_1\cdot v_2| \le \|v_2\|^2/2$. Finally
$\alpha^2 - |\alpha||\beta| + \beta^2 = \tfrac12\big((|\alpha|-|\beta|)^2 + \alpha^2
+ \beta^2\big) \ge 1$ for nonzero integers $\alpha, \beta$ (the bracket is a sum of
squares, at least $2$). Hence $\|w\| \ge \|v_2\|$: $v_2$ is genuinely shortest.
$\blacksquare$ The hand trace confirms $\nu_2^2(137, 256) = 274$ with shortest
vector $(7, -15)$ (three reduction passes, reproduced by the lab). **Integrality
is load-bearing** in the final step: over the reals $\alpha = \beta = \tfrac12$
makes $\alpha^2 - |\alpha||\beta| + \beta^2 = \tfrac14 < 1$, so the argument
collapses — which is exactly why shortest-vector is a hard *discrete* optimization
while its continuous relaxation (linear algebra) is trivial.

### 5. Instrument Algorithm C; the $4/3$ doubling bound (rating 22 · cf. W5)

**Problem.** Algorithm C finds $\nu_3^2$ by scanning $(u_2, u_3) \in [-B, B]^2$,
doubling $B$ until $B^2 \ge \text{best}$. Instrument it: report the final $B$ and
the total number of scanned cells for RANDU ($a = 65539$, $m = 2^{31}$), $16807$,
and $48271$ (both mod $2^{31} - 1$). Then check the claim that the doubling
schedule costs at most $\tfrac43$ of the final scan alone.

**Hint.** The scan at bound $B$ visits $(2B+1)^2 - 1$ cells (excluding
$(0,0)$). Summed over $B = 1, 2, 4, \ldots, B_{\text{final}}$, consecutive terms
grow by roughly $4\times$, so the geometric series is $\le \tfrac{4}{3}$ of the
last term — asymptotically.

**Answer sketch.** Measured (reproduced by the lab's instrumentation):

| generator | $\nu_3^2$ | final $B$ | total cells | final scan | ratio |
|---|---|---|---|---|---|
| RANDU | $118$ | $16$ | $1488$ | $1088$ | $1.368$ |
| $16807$ | $408\,197$ | $1024$ | $5\,600\,592$ | $4\,198\,400$ | $1.334$ |
| $48271$ | $1\,433\,881$ | $2048$ | $22\,386\,000$ | $16\,785\,408$ | $1.334$ |

The bound comes from the geometric series: cells at bound $B$ number
$\approx 4B^2$, and $\sum_{k=0}^{K} 4^k = \tfrac{4^{K+1}-1}{3} \approx \tfrac43\cdot
4^K$, so total work $\le \tfrac43$ of the final scan in the **leading term**. The
two large searches sit right at $1.334 \approx \tfrac43$. RANDU comes in slightly
*over* ($1.368$) because its search certifies at a tiny $B = 16$, where the
lower-order $+1$ in $(2B+1)^2$ has not yet washed out — an honest reminder that
$\tfrac43$ is the asymptotic constant, not a hard cap at small $B$. Either way the
cost is dominated by the last scan, exactly the table-doubling accounting from
earlier modules.

### 6. ▶ Certified search at $t = 4$; RANDU's $\nu_4$ (rating 30 · cf. W6)

**Problem.** Extend Algorithm C to $t = 4$: scan $(u_2, u_3, u_4) \in [-B, B]^3$,
pin $u_1$ to the centered residue of $-(a u_2 + a^2 u_3 + a^3 u_4) \bmod m$, and
certify with the same $B^2 \ge \text{best}$ test (Hermite's $\gamma_4 = \sqrt2$
guarantees termination). Estimate the cost from the Hermite bound, then compute
$\nu_4^2$ for RANDU. Is it just the padded value from $t = 3$?

**Hint.** The dimension-$3$ shortest dual vector $(9, -6, 1)$ pads to
$(9, -6, 1, 0)$, a legal dimension-$4$ dual vector of squared length $118$ (Problem
1). So $\nu_4^2 \le 118$ — but the true minimum could be smaller.

**Answer sketch.** Hermite gives $\nu_4^2 \le \gamma_4\, m^{1/2} = \sqrt2\cdot
(2^{31})^{1/2} = 65536$ for RANDU, so once $B \ge 256$ (here $B^2 \ge 65536$) the
scan is certified; in fact RANDU certifies at $B = 16$. The computed value is

$$\nu_4^2(\text{RANDU}) = 116,$$

**strictly below** the padded $118$: the witness $u = (-9, -3, 5, -1)$ satisfies
$-9 - 3a + 5a^2 - a^3 \equiv 0 \pmod{2^{31}}$ and has squared length
$81 + 9 + 25 + 1 = 116$. So padding the $t = 3$ optimum gives only an *upper bound*
(consistent with $\nu_4 \le \nu_3$, i.e. $116 \le 118$); the extra dimension opens
a genuinely shorter direction that the $t = 3$ search could not see. This is why
each dimension must be searched afresh — and why Knuth's Algorithm S transforms
the basis as $t$ grows rather than reusing lower-$t$ answers.

### 7. A mini Fishman–Moore search for $m = 2^{31} - 1$ (rating M28 · cf. W7)

**Problem.** The multiplier $48271$ was found by an exhaustive spectral search
(Fishman–Moore style) over multipliers for the Mersenne modulus $m = 2^{31} - 1$.
Reproduce the method in miniature: for a range of candidate multipliers $a$
(coprime to $m$), compute $\mu_2 = \pi\,\nu_2^2/m$ and $\mu_3 = \tfrac43\pi\,
\nu_3^3/m$ and rank by the figure of merit $\min(\mu_2, \mu_3)$. How close to
$48271$-quality can you get?

**Hint.** $\nu_2^2$ comes from Algorithm G (fast); $\nu_3^2$ from Algorithm C
(slower). Knuth's rule of thumb: $\mu_t \ge 0.1$ passes, $\mu_t \ge 1$ is
excellent. Because $\nu_3$ dominates the cost, screen candidates on $\mu_2$ first
and only compute $\mu_3$ for survivors.

**Answer sketch.** Evaluating a handful of named multipliers reproduces the
verdicts of the lesson's Table 1 and shows the search *works*:

| $a$ | $\mu_2$ | $\mu_3$ | $\min(\mu_2, \mu_3)$ |
|---|---|---|---|
| $16807$ | $0.41$ | $0.51$ | $0.41$ (passes, unspectacular) |
| $48271$ | $2.91$ | $3.35$ | $2.91$ (excellent) |
| $742938285$ | $2.73$ | $3.78$ | $2.73$ (excellent) |

A full exhaustive sweep of all $a$ in a wide band, ranked by $\min(\mu_2, \mu_3)$,
is the term-project version (hence M28): it must skip $a$ with $\gcd(a, m) \ne 1$,
prune early on $\mu_2$, and — because $\nu_3$ is costly per candidate — parallelize
or restrict to a smart sub-band, which is precisely what Fishman and Moore did in
1986 to certify $48271$ as a best-in-class multiplier for this modulus. The
takeaway that the search makes concrete: $16807$ and $48271$ cost the same to
evaluate at runtime, yet $48271$ is an order of magnitude better in three
dimensions — spectral vetting, not folklore, is why the standard was revised.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
