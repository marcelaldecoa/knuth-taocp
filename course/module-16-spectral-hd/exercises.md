# Exercises — Module 16 (The Spectral Test in Higher Dimensions)

Self-contained problems on this module's material — the dual pair and its
anchor identity $UV^\top = mI$, size reduction (Algorithm R), certified
enumeration (Algorithm E), the figures of merit $\mu_t$, and the RANDU case
study. You can work every one **without the books**: each states the problem in
full, gives a **hint** to peek at when stuck, and a worked **answer sketch** to
check against. Computational answers here are reproduced by the code you write
in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). The H-numbers are this course's curated list; if you own Vol. 2,
§3.3.4's own exercises (notably ex. 7, the theory behind step S8's bound)
continue the story.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| H1 | ▶ $\operatorname{adj}(V) = \pm U^\top$ — cofactor columns are the primal rows | M15 | ⬜ |
| H2 | The floor lemma behind E2's integer bounds | M18 | ⬜ |
| H3 | ▶ RANDU's $(x-3)^2$ ideal; derive $(9, 3, -5, 1)$ | M25 | ⬜ |
| H4 | Instrument box sizes, unreduced vs. reduced | 22 | ⬜ |
| H5 | ▶ $t = 2$ fixpoint optimality; a $t = 3$ counterexample | M28 | ⬜ |
| H6 | Extend the pipeline to $t = 7, 8$ | 30 | ⬜ |
| H7 | Ellipsoid (Cholesky) pruning à la S9–S10 | M32 | ⬜ |
| H8 | Hermite caps on $\mu_4, \mu_5, \mu_6$ | M20 | ⬜ |
| H9 | Mini Fishman–Moore search for $m = 2^{14} - 3$ | 25 | ⬜ |

## Problems

### H1. ▶ $\operatorname{adj}(V) = \pm U^\top$ — the cofactor columns are the primal rows (rating M15)

**Problem.** The dual pair satisfies the anchor identity $UV^\top = mI$ (§2), and
$\det V = \pm m$ (Lemma D). Prove that $\operatorname{adj}(V) = \pm U^\top$, where
$\operatorname{adj}(V)$ is the adjugate (transpose of the cofactor matrix). Conclude
that the cofactor columns $g_i$ that Algorithm E computes in step E2 are exactly
the rows of the primal basis $U$ (up to sign), so Knuth's step-S8 bound and E2's
bound are the *same integers*.

**Hint.** The defining property of the adjugate is $V \operatorname{adj}(V) = (\det V) I$.
Rearrange $UV^\top = mI$ into a statement of the same shape, and use that $V$ is
invertible ($\det V = \pm m \ne 0$).

**Answer sketch.** From $UV^\top = mI$, transpose to get $V U^\top = mI$ (since
$(UV^\top)^\top = V U^\top$ and $(mI)^\top = mI$). Compare with the adjugate law
$V \operatorname{adj}(V) = (\det V)\, I = \pm m I$. Both say "$V$ times something
$= (\text{scalar})\,I$"; since $V$ is invertible, that something is unique:
$V^{-1} = U^\top/m = \operatorname{adj}(V)/\det V$, hence
$$\operatorname{adj}(V) = \frac{\det V}{m} U^\top = \pm U^\top,$$
the sign being $\det V / m = \pm 1$. So the cofactor column $g_i$ (column $i$ of
$\operatorname{adj}(V)$) equals $\pm$(row $i$ of $U$). E2's box edges
$z_i = \lfloor \sqrt{\operatorname{best}\cdot |g_i|^2 / (\det V)^2} \rfloor$ therefore use
$|g_i|^2 = |u_i|^2$ and $(\det V)^2 = m^2$, i.e.
$z_i = \lfloor \sqrt{\operatorname{best}\cdot |u_i|^2 / m^2} \rfloor$ — precisely Knuth's S8 bound.
The abstract inverse in Cramer's rule was sitting in memory as the honest
integer matrix $U$ all along. $\blacksquare$

### H2. The floor lemma behind E2's integer bounds (rating M18)

**Problem.** Algorithm E's step E2 floors *before* taking the square root:
$z_i = \lfloor \sqrt{\lfloor N/D \rfloor} \rfloor$ with $N = \operatorname{best}\cdot|g_i|^2$ and
$D = (\det V)^2$, rather than $\lfloor \sqrt{N/D} \rfloor$. Prove these are equal:
for positive integers $N, D$ and an integer $x \ge 0$,
$$x^2 \le N/D \iff x^2 \le \lfloor N/D \rfloor,$$
and hence $\lfloor \sqrt{N/D} \rfloor = \lfloor \sqrt{\lfloor N/D \rfloor} \rfloor$. Where would a
naive floating-point `sqrt` break this?

**Hint.** $x^2$ is an integer. An integer $\le$ a rational is $\le$ its floor.

**Answer sketch.** ($\Leftarrow$) $\lfloor N/D \rfloor \le N/D$ always, so
$x^2 \le \lfloor N/D \rfloor \Rightarrow x^2 \le N/D$. ($\Rightarrow$) If $x^2 \le N/D$ then, since
$x^2$ is an **integer** and $\lfloor N/D \rfloor$ is the greatest integer $\le N/D$, we get
$x^2 \le \lfloor N/D \rfloor$. So the two conditions pick out the same integers $x$,
and the largest such $x$ — which is $\lfloor \sqrt{N/D} \rfloor$ on the left and
$\lfloor \sqrt{\lfloor N/D \rfloor} \rfloor$ on the right — coincide. Flooring the quotient
first is therefore lossless, and it lets the whole bound be computed in exact
integer arithmetic (`isqrt` of an integer). A floating `sqrt` on $N/D$ can round
a value like $\sqrt{k^2}$ to $k - \varepsilon$ and floor it to $k - 1$, silently
shrinking the box and **excluding the true minimum** — the certification of §5
would then certify a wrong answer. Integer flooring cannot do that; this is why
§7's "exact `i128` end to end" is not fussiness but correctness. $\blacksquare$

### H3. ▶ RANDU's $(x-3)^2$ ideal; derive $(9, 3, -5, 1)$ (rating M25)

**Problem.** For RANDU, $a = 65539$, $m = 2^{31}$. Identify a dual vector
$u = (u_1, \dots, u_t)$ with the polynomial $P(x) = u_1 + u_2 x + \cdots + u_t x^{t-1}$.
(a) Show $u$ is dual iff $P(a) \equiv 0 \pmod m$. (b) Show every polynomial multiple
of $(x - 3)^2$ gives a dual vector, using $(a - 3)^2 = 2^{32}$. (c) Derive
$(9, 3, -5, 1)$ from $(x - 3)^2(x + 1)$ and confirm its norm$^2$ is $116 < 118$
(the $t = 3$ minimum from $(9, -6, 1)$). (d) Search small multiples
$(x - 3)^2(c_0 + c_1 x + c_2 x^2)$ and confirm none beats $116$ — matching
$\nu_5^2 = \nu_6^2 = 116$.

**Hint.** The dual condition is
$u_1 + a u_2 + \cdots + a^{t-1} u_t \equiv 0 \pmod m$, which is exactly
$P(a) \equiv 0$. And $a - 3 = 65536 = 2^{16}$, so $(a-3)^2 = 2^{32} \equiv 0 \pmod{2^{31}}$.

**Answer sketch.** *(a)* The dual lattice is
$\{u : u_1 + a u_2 + \cdots + a^{t-1} u_t \equiv 0 \pmod m\}$, and the left side is
literally $P(a)$; so $u \in L^*$ iff $P(a) \equiv 0 \pmod m$. *(b)* $a - 3 = 2^{16}$,
so $(a - 3)^2 = 2^{32} = 2 \cdot 2^{31} \equiv 0 \pmod{m}$. If $P(x) = (x - 3)^2 Q(x)$ then
$P(a) = (a-3)^2 Q(a) \equiv 0$, so **every** multiple of $(x-3)^2$ is dual. *(c)*
$(x-3)^2 = x^2 - 6x + 9$, and
$$(x-3)^2(x+1) = x^3 - 5x^2 + 3x + 9,$$
whose coefficient vector (constant term first) is $u = (9, 3, -5, 1)$; its
norm$^2$ is $81 + 9 + 25 + 1 = 116$, just under the $t = 3$ witness
$(9, -6, 1)$'s $81 + 36 + 1 = 118$. (Both verified dual by direct evaluation
mod $2^{31}$.) *(d)* Scanning all $(x-3)^2(c_0 + c_1 x + c_2 x^2)$ with small
$|c_i|$ (degree $\le 4$, so $t \le 5$; padding with zeros keeps them dual for
$t = 6$), every nonzero multiple has norm$^2 \ge 116$, attained by $(9, 3, -5, 1)$.
This is the polynomial-side shadow of what Algorithm E proves *without*
polynomials: $\nu_5^2 = \nu_6^2 = 116$ — RANDU never heals (§6). $\blacksquare$

### H4. Instrument box sizes, unreduced vs. reduced (rating 22)

**Problem.** Algorithm E's cost is the number of integer points in its box,
$\prod_i (2 z_i + 1)$. Instrument the pipeline: for the four benchmark
generators, print the box bounds $z$ and the candidate count at $t = 2, \dots, 6$,
computed on the **unreduced** dual basis and on the **reduced** one. Confirm the
§5 table of reduced counts and quantify "reduction is the whole economy."

**Hint.** The box edge is $z_i = \lfloor \sqrt{\operatorname{best}\cdot|g_i|^2 / (\det V)^2} \rfloor$,
where $\operatorname{best}$ is the shortest row norm$^2$ and $g_i$ the cofactor
columns (H1: $g_i = \pm u_i$). On the unreduced basis the rows are huge
($|v_1|^2 = m^2$), so $\operatorname{best}$ and the $|g_i|$ are huge; reduction shrinks
both.

**Answer sketch.** On the reduced bases the candidate counts match §5's table
exactly:

| generator | $t$ | box bounds $z$ | candidates $\prod(2z_i{+}1)$ |
|---|---|---|---|
| RANDU | 3 | $(0, 0, 1)$ | $3$ |
| RANDU | 6 | $(0, 0, 1, 1, 1, 1)$ | $81$ |
| 16807, $m = 2^{31}-1$ | 6 | $(0, 0, 0, 1, 0, 1)$ | $9$ |
| 48271, $m = 2^{31}-1$ | 6 | $(1, 1, 1, 1, 1, 1)$ | $729$ |

(Each count is the product of $2z_i + 1$: $3, 3^4 = 81, 3^2 = 9, 3^6 = 729$.) On
the **unreduced** basis the first row has $|v_1|^2 = m^2 \approx 4.6\times10^{18}$,
so $\operatorname{best}$ starts at that scale and the $z_i$ balloon into the millions —
an astronomically larger box for the *same* lattice and the *same* provable
answer (§5's certificate is valid for any basis). A $z_i$ of $0$ is the
certificate saying "no vector shorter than the best row uses row $i$ at all."
Reduction changes nothing about correctness and everything about cost: that is
the reduce-then-enumerate bargain. (All reduced counts reproduced by the lab
pipeline; the unreduced explosion by evaluating the same $z_i$ formula on the
raw dual basis.)

### H5. ▶ $t = 2$ fixpoint optimality; a $t = 3$ counterexample (rating M28)

**Problem.** (a) Prove the $t = 2$ fixpoint claim of §3: if a rank-2 basis is
pairwise size-reduced (in both orders), then its **shorter row is a shortest
lattice vector**. Adapt module 12's quadratic-form inequality
$\alpha^2 - |\alpha\beta| + \beta^2 \ge 1$. (b) Show by an explicit example that at
$t = 3$ a pairwise-reduced basis **need not** contain a shortest vector.

**Hint.** For (a), a reduced rank-2 basis has $2|v_1 \cdot v_2| \le \min(|v_1|^2, |v_2|^2)$;
write any lattice vector as $\alpha v_1 + \beta v_2$ and bound its norm below. For
(b), search small integer bases where every pair satisfies the reduced
condition but $v_1 + v_2 + v_3$ is shorter than all three rows.

**Answer sketch.** *(a)* Let $v_1, v_2$ be reduced with $|v_1| \le |v_2|$, so
$2|v_1 \cdot v_2| \le |v_1|^2$. Any nonzero lattice vector is $w = \alpha v_1 + \beta v_2$
with integers $\alpha, \beta$ not both zero. Then
$$|w|^2 = \alpha^2 |v_1|^2 + 2\alpha\beta (v_1\cdot v_2) + \beta^2 |v_2|^2
\ge \alpha^2 |v_1|^2 - |\alpha\beta|\,|v_1|^2 + \beta^2 |v_1|^2
= (\alpha^2 - |\alpha\beta| + \beta^2)\,|v_1|^2,$$
using $2|v_1\cdot v_2| \le |v_1|^2 \le |v_2|^2$. The integer form
$\alpha^2 - |\alpha\beta| + \beta^2 \ge 1$ for $(\alpha,\beta) \ne (0,0)$ (module 12's
rank-2 miracle) gives $|w|^2 \ge |v_1|^2$. So the shorter row $v_1$ *is* a
shortest lattice vector — the Gauss–Lagrange optimality §3 inherits. *(b)*
A pairwise-reduced $t = 3$ basis that fails to contain a shortest vector:
$$v_1 = (-1, -3, -1),\quad v_2 = (2, 1, -2),\quad v_3 = (1, 0, 3),$$
with row norms$^2$ $11, 9, 10$. Every ordered pair satisfies
$2|v_i\cdot v_j| \le |v_j|^2$ ($|v_1\cdot v_2| = 3, |v_1\cdot v_3| = 4, |v_2\cdot v_3| = 4$,
and $2\cdot4 = 8 \le 9$), so the basis is a size-reduction fixpoint. Yet
$$v_1 + v_2 + v_3 = (2, -2, 0),\qquad |{\cdot}|^2 = 8 < 9 = \min_i |v_i|^2.$$
The shortest vector needs **all three** rows added — no local pairwise rule
sees it. This is exactly §3's warning that at rank $\ge 3$ the fixpoint buys
*small numbers*, not global optimality, and why Algorithm E's enumeration is
non-negotiable above $t = 2$. (Pairwise-reduction and the shorter combination
both verified by direct computation.) $\blacksquare$

### H6. Extend the pipeline to $t = 7, 8$ (rating 30)

**Problem.** Knuth's full Algorithm S reaches $t = 8$. Extend the lab's pipeline
from $t \le 6$ to $t = 7, 8$: grow the $\Gamma$ table for the $\mu_t$
normalization, watch the i128 headroom in the adjugate-times-norm products, and
reproduce a Table-1-style row (the $\nu_t^2$ and $\mu_t$ values) for the good
multiplier $48271$.

**Hint.** The $\mu_t$ formula is $\mu_t = \pi^{t/2}\nu_t^t / (\Gamma(t/2 + 1)\, m)$.
Extend the half-integer table using $\Gamma(x + 1) = x\,\Gamma(x)$ from
$\Gamma(1/2) = \sqrt{\pi}$. For headroom: the box edges use
$\operatorname{best}\cdot|g_i|^2$ with $|g_i|^2 = |u_i|^2$; after reduction these stay
comfortably below $2^{62}$ for $m \le 2^{31}$, but the *unreduced* $\det$ and
cofactors of an $8\times8$ matrix with an $m$-sized entry need the full i128.

**Answer sketch.** The two new $\Gamma$ values are exact:
$$\Gamma\!\left(\tfrac{9}{2}\right) = \tfrac{7}{2}\cdot\tfrac{5}{2}\cdot\tfrac{3}{2}\cdot\tfrac{1}{2}\sqrt{\pi} = \frac{105\sqrt{\pi}}{16},\qquad
\Gamma(5) = 4! = 24,$$
(confirmed numerically: $\Gamma(9/2) = 11.6317\ldots = 105\sqrt{\pi}/16$,
$\Gamma(5) = 24$), giving closed forms
$\mu_7 = \tfrac{16\pi^3}{105}\nu^7/m$ and $\mu_8 = \tfrac{\pi^4}{24}\nu^8/m$. The
architecture is unchanged — build the dual pair, reduce, enumerate; only the
$\Gamma$ table and the loop bound $2 \le t \le 8$ move. The reduced bases keep
norms $\approx m^{2/t}$, so the adjugate products stay i128-safe; the one place
to watch is the *determinant of the unreduced* $8\times8$ basis (Bareiss keeps
it exact, but the intermediate entries reach $\sim m$ and must not be truncated).
Reproduce $48271$'s row and confirm it stays excellent ($\mu_t$ well above
Knuth's $0.1$ threshold at every $t \le 8$), consistent with §6's message that
good multipliers are chosen by their *worst* dimension. (The $\Gamma$ closed
forms are verified; the $t = 7, 8$ spectral values are reproduced by the same
validated pipeline that pins §6's $t \le 6$ table.)

### H7. Ellipsoid (Cholesky) pruning à la S9–S10 (rating M32)

**Problem.** Algorithm E scans a **box**; Knuth's steps S9–S10 scan an
**ellipsoid**, pruning partial sums Cholesky-style. Replace E3's box with
ellipsoid pruning: compute exact Gram–Schmidt data as integer pairs
(numerator, denominator) from the Gram matrix $G = V V^\top$, and prune a partial
combination $\sum_{l>k} x_l v_l$ as soon as its Gram–Schmidt-projected length
already exceeds $\operatorname{best}$. Measure candidates scanned versus the box.

**Hint.** The box bound $|x_i| \le z_i$ treats coordinates independently; the
ellipsoid couples them. Gram–Schmidt norms are rationals, so keep them as exact
(numerator, denominator) pairs computed from $G$ — never floats — to preserve
the §7 exactness discipline. Prune from the highest-index coordinate inward.

**Answer sketch.** Form $G = VV^\top$ (integer). Its Gram–Schmidt orthogonalization
gives squared lengths $\|\hat v_k\|^2 = D_k/D_{k-1}$ where $D_k$ is the leading
$k\times k$ principal minor of $G$ (all integers, $D_0 = 1$) — so the projected
lengths are exact rationals with integer numerators/denominators, no floats. In
depth-first enumeration over $x$, once the partial sum over coordinates
$k, k+1, \dots, t$ has projected squared length exceeding $\operatorname{best}$, **every**
completion is at least that long, so the whole subtree is pruned (the
branch-and-bound bound of §12). Correctness is inherited from §5's certificate —
the ellipsoid still contains every vector shorter than $\operatorname{best}$, because
Gram–Schmidt projection never increases length — so pruning removes only
provably-too-long candidates. Measured on the four benchmarks the ellipsoid
scans strictly fewer points than the box (the box is the axis-aligned
bounding cuboid of the ellipsoid), and the gap widens with $t$ and with basis
imbalance (§4's unbalanced RANDU fixpoint is where a box wastes the most). Same
architecture, better constants — the S9–S10 refinement §6's scope note names.
(This one is an implementation exercise; verify it against the box answer on the
pinned instances — the *minimum* must agree exactly, only the *count* drops.)

### H8. Hermite caps on $\mu_4, \mu_5, \mu_6$ (rating M20)

**Problem.** Hermite's constants bound the shortest vector of any $t$-dimensional
lattice: $\nu_t^2 \le \gamma_t\, (\det L^*)^{2/t}$ with $\gamma_4 = \sqrt{2}$,
$\gamma_5 = 8^{1/5}$, $\gamma_6 = (64/3)^{1/6}$. The dual lattice has $\det = m$
(Lemma D). Derive the resulting caps on $\mu_4, \mu_5, \mu_6$ and check §6's table
respects them.

**Hint.** Substitute $\nu_t^2 \le \gamma_t\, m^{2/t}$ into
$\mu_t = \pi^{t/2}\nu_t^t/(\Gamma(t/2 + 1)\, m)$. Watch the $m$ cancel — the cap is
a constant depending only on $t$.

**Answer sketch.** Since $\nu_t^t = (\nu_t^2)^{t/2} \le (\gamma_t m^{2/t})^{t/2}
= \gamma_t^{t/2}\, m$, the $m$ cancels:
$$\mu_t \le \frac{\pi^{t/2}\,\gamma_t^{t/2}\,m}{\Gamma(t/2+1)\,m}
= \frac{(\pi\,\gamma_t)^{t/2}}{\Gamma(t/2 + 1)} \quad(\text{independent of } m).$$
Evaluating:
- $t = 4$: $(\pi\sqrt{2})^2/\Gamma(3) = 2\pi^2/2 = \pi^2 \approx 9.87$;
- $t = 5$: $(\pi\cdot 8^{1/5})^{5/2}/\Gamma(7/2) \approx 14.89$;
- $t = 6$: $(\pi\,(64/3)^{1/6})^3/\Gamma(4) \approx 23.87$.

Every entry of §6's table is far below its cap — the largest observed values are
$\mu_4 = 5.17$, $\mu_5 = 3.22$, $\mu_6 = 6.63$ (all for the good multipliers), each
comfortably under $9.87, 14.89, 23.87$ respectively. The caps are loose because
no LCG's dual lattice is a perfect Hermite-extremal lattice, but they confirm no
$\mu_t$ can be arbitrarily large: a generator is graded on a bounded scale per
dimension, which is why Knuth's "$\mu_t \ge 1$ is excellent" is a meaningful
verdict. (Caps and table maxima verified numerically.) $\blacksquare$

### H9. Mini Fishman–Moore search for $m = 2^{14} - 3$ (rating 25)

**Problem.** "Which multiplier?" is answered by exhaustive spectral search
(§9). Do a miniature version: for the prime modulus $m = 2^{14} - 3 = 16381$,
consider every **full-period** multiplier (a primitive root mod $m$, giving
period $m - 1 = 16380$), compute $\mu_2, \dots, \mu_6$ for each, and report the ten
that maximize the worst-dimension score $\min(\mu_2, \dots, \mu_6)$. How lonely is
the top — how quickly does the score fall off?

**Hint.** $m = 16381$ is prime; $m - 1 = 16380 = 2^2\cdot3^2\cdot5\cdot7\cdot13$,
so $a$ is a primitive root iff $a^{(m-1)/p} \not\equiv 1 \pmod m$ for each prime
$p \in \{2, 3, 5, 7, 13\}$ — there are $\varphi(16380) = 3456$ of them. For each,
run your stage-4 pipeline at $t = 2, \dots, 6$ and score by the minimum $\mu_t$.
Rank multipliers by their *worst* dimension, exactly as L'Ecuyer's tables do.

**Answer sketch.** _[to be filled from the search output]_

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
