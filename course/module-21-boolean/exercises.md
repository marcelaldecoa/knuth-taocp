# Exercises — Module 21 (Boolean Functions and Optimal Evaluation)

Self-contained problems on this module's material — counting special functions,
normal forms, monotonicity, Boolean chains, and the combinational complexity
$C(f)$. You can work every one **without the books**: each states the problem in
full, gives a **hint** to peek at when stuck, and a worked **answer sketch** to
check against after you try. Numeric answers here are reproduced by the code you
write in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number in §7.1.1–7.1.2
is noted for readers who own Volume 4A.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Count the self-dual functions of $n$ variables | 10 | ⬜ |
| 2 | ▶ Symmetric functions $\leftrightarrow$ $n+1$ weight bits | 20 | ⬜ |
| 3 | Monotone $\iff$ positive-literal DNF | 22 | ⬜ |
| 4 | ▶ Optimum 4-gate median-of-3; no 3-gate chain | 15 | ⬜ |
| 5 | $C$ of the full adder's two outputs; can they share? | 20 | ⬜ |
| 6 | ▶ A random $f$ needs $\approx 2^n/n$ gates (Shannon/Lupanov) | 30 | ⬜ |
| 7 | Compute the Dedekind number $M(5)$; the $M(9)$ story | 40 | ⬜ |

## Problems

### 1. Count the self-dual functions of $n$ variables (rating 10 · cf. 7.1.1–2)

**Problem.** A function is **self-dual** if $f(\lnot x) = \lnot f(x)$ for every
input $x$ (complementing all inputs complements the output). How many self-dual
Boolean functions of $n$ variables are there?

**Hint.** The $2^n$ inputs pair up under complementation: $x$ with $\lnot x$.
How many such pairs are there, and how much freedom does self-duality leave you
on each pair?

**Answer sketch.** The map $x \mapsto \lnot x$ has no fixed points (an $n$-bit
string never equals its own complement for $n \ge 1$), so the $2^n$ inputs split
into exactly $2^n / 2 = 2^{n-1}$ complementary pairs. Self-duality forces
$f(\lnot x) = \lnot f(x)$, so **fixing $f$ on one member of a pair fixes it on
the other**. You may choose $f$ freely (0 or 1) on one representative of each of
the $2^{n-1}$ pairs, and nothing else. Hence there are

$$
\boxed{2^{2^{n-1}}}
$$

self-dual functions. Check: $n = 1 \Rightarrow 2^{1} = 2$ (the two projections
$x_1, \lnot x_1$); $n = 2 \Rightarrow 2^{2} = 4$; $n = 3 \Rightarrow 2^{4} = 16$
— all confirmed by brute-force enumeration. Compare the grand total $2^{2^n}$ of
all functions: self-dual ones are the square root of that count, a vanishing
fraction, yet they include the important median/majority function the lesson
singles out.

### 2. ▶ Symmetric functions $\leftrightarrow$ $n+1$ weight bits (rating M20 · cf. 7.1.1–5)

**Problem.** A function is **symmetric** if its value depends only on *how many*
inputs are 1, not which. Prove that every symmetric function of $n$ variables is
determined by the $n+1$ bits $w_0, \dots, w_n$ (where $w_j$ is the value when
exactly $j$ inputs are 1), and count how many symmetric functions there are.

**Hint.** Group the $2^n$ inputs by popcount. What does symmetry say about all
inputs in one group? Then count the free choices.

**Answer sketch.** Partition the $2^n$ inputs by their number of 1s: group $j$
holds the $\binom{n}{j}$ inputs with popcount $j$, for $j = 0, 1, \dots, n$.
Symmetry means $f$ is **constant on each group** (any two inputs with the same
popcount give the same value), so $f$ is completely described by its value $w_j$
on group $j$ — exactly the $n+1$ bits $w_0, \dots, w_n$, giving
$f(x) = w_{\operatorname{popcount}(x)}$ (the module's `symmetric_function`).
Conversely any choice of the $w_j$ yields a symmetric function. So the symmetric
functions are in bijection with $(n+1)$-bit strings, and there are exactly

$$
2^{\,n+1}
$$

of them. Check: $n = 2 \Rightarrow 2^3 = 8$, $n = 3 \Rightarrow 2^4 = 16$ —
confirmed by enumeration. This is a spectacular compression: $n+1$ bits instead
of $2^n$, and it is why majority, threshold, and parity — all symmetric — are
among the *cheap* functions the counting bound (Problem 6) says must be rare.

### 3. Monotone $\iff$ positive-literal DNF (rating 22 · cf. 7.1.1–16)

**Problem.** Call $f$ **monotone (nondecreasing)** if $x \subseteq y \Rightarrow
f(x) \le f(y)$ under the bitwise-subset order (flipping any input $0 \to 1$ never
flips the output $1 \to 0$). Prove that $f$ is monotone **iff** it can be written
as a DNF (an OR of product terms) using only **positive** literals — no
complemented variables.

**Hint.** One direction: show a positive term is monotone and that OR preserves
monotonicity. Other direction: for a monotone $f$, build a DNF from its *minimal*
1-inputs, using for each the product of the variables that are 1 there.

**Answer sketch.** ($\Leftarrow$) A positive product term $x_{i_1} \land \cdots
\land x_{i_k}$ is monotone: raising an input from 0 to 1 can only turn the AND
from 0 to 1, never the reverse. The OR of monotone functions is monotone (raising
an input cannot turn any disjunct off, so it cannot turn the OR off). Hence any
positive-literal DNF is monotone. ($\Rightarrow$) Let $f$ be monotone. Call an
input $x$ **minimal** if $f(x) = 1$ but $f(x') = 0$ for every $x' \subsetneq x$.
For each minimal $x$ form the positive term $T_x = \bigwedge_{j:\,x_j = 1} x_j$,
and let $D = \bigvee_{x \text{ minimal}} T_x$. Then $D = f$: if $f(y) = 1$, walk
down from $y$ by clearing 1-bits while the value stays 1 until you reach a minimal
$x \subseteq y$; then $T_x$ is satisfied by $y$ (all its variables are 1 in $y$),
so $D(y) = 1$. Conversely if $T_x$ is satisfied by $y$ then $x \subseteq y$, so
by monotonicity $f(y) \ge f(x) = 1$. Thus $D$ and $f$ agree on every input, and
$D$ uses only positive literals. $\blacksquare$ (The minimal 1-inputs are exactly
the *prime implicants* of a monotone function; this is a *proof by exhaustive
agreement* — the two formulas match on all $2^n$ rows.)

### 4. ▶ Optimum 4-gate median-of-3; no 3-gate chain (rating 15 · cf. 7.1.2–1)

**Problem.** Exhibit a 4-gate Boolean chain computing the median (majority) of
three inputs, and argue that no chain of 3 gates can compute it — so
$C(\text{majority}_3) = 4$ over the full 16-operation basis.

**Hint.** For the witness, combine an AND, an OR, and one more AND/OR. For the
lower bound, recall why the "reachable functions" shortcut that reports 3 is
*wrong* — it assumes both operands of a gate are free, ignoring that sharing is
what a real chain provides.

**Answer sketch.** **Upper bound (witness).** The chain

$$
\begin{aligned}
v_3 &= x_1 \land x_2, & v_4 &= x_1 \lor x_2,\\
v_5 &= x_3 \land v_4, & v_6 &= v_3 \lor v_5 \;(\text{output}),
\end{aligned}
$$

computes $\text{majority}(x_1,x_2,x_3) = (x_1 \land x_2) \lor (x_3 \land (x_1 \lor
x_2))$ in 4 gates. Check the cases: if $x_1 = x_2 = 1$, $v_3 = 1$; if exactly one
of $x_1, x_2$ is 1 then $v_4 = 1$ and $v_5 = x_3$, so the output is 1 iff $x_3$
supplies the second vote; if $x_1 = x_2 = 0$ everything is 0. So the output is 1
iff at least two inputs are 1 — the majority. **Lower bound.** The state-BFS of
§6 — which carries the *whole set* of computed values forward so any later gate
can reuse an earlier one — finds no 3-gate chain for majority; its first
appearance is at depth 4. The tempting "reachable set" shortcut ($R_c = R_{c-1}
\cup \{g(a,b)\}$) reports 3, but it double-counts by pretending both operands are
available for free when each may itself cost gates; it computes only a *lower
bound* on cost, not the cost. Since the shortcut's 3 is not achievable and the
witness achieves 4, $C(\text{majority}_3) = 4$.

### 5. $C$ of the full adder's two outputs; can they share? (rating 20 · cf. 7.1.2–2)

**Problem.** A full adder has inputs $x_1, x_2, x_3$ and two outputs: the **sum**
$s = x_1 \oplus x_2 \oplus x_3$ and the **carry** $c = \text{majority}(x_1, x_2,
x_3)$. Individually, $C(s) = 2$ and $C(c) = 4$ over the full basis. If we build
one chain producing *both* outputs, can it cost fewer than $2 + 4 = 6$ gates by
sharing an intermediate value? Find the smallest such chain.

**Hint.** Is there a subexpression common to a cheap chain for $s$ and a cheap
chain for $c$? Rewrite the carry using $x_1 \oplus x_2$ instead of $x_1 \lor x_2$
where they agree.

**Answer sketch.** Yes — the two outputs **share the half-sum $x_1 \oplus x_2$**,
so both fit in **5 gates**, not 6. When $x_1 \land x_2 = 0$ (the region that
matters for the second disjunct) $x_1 \lor x_2$ equals $x_1 \oplus x_2$, giving
the identity $c = (x_1 \land x_2) \lor (x_3 \land (x_1 \oplus x_2))$. Now:

$$
\begin{aligned}
g_1 &= x_1 \oplus x_2, & g_2 &= g_1 \oplus x_3 \;(=\text{sum } s),\\
g_3 &= x_1 \land x_2, & g_4 &= x_3 \land g_1, & g_5 &= g_3 \lor g_4 \;(=\text{carry } c).
\end{aligned}
$$

Gate $g_1$ feeds *both* the sum (via $g_2$) and the carry (via $g_4$) — that is
the sharing. Evaluating all $2^3 = 8$ inputs confirms $g_2 = x_1 \oplus x_2
\oplus x_3$ and $g_5 = \text{majority}(x_1, x_2, x_3)$ on every row. So the joint
(multi-output) complexity is 5, strictly below the sum $6$ of the separate
complexities — a concrete instance of why circuit cost is about *sharing*, the
theme behind the module's state-BFS.

### 6. ▶ A random $f$ needs $\approx 2^n/n$ gates (Shannon/Lupanov) (rating M30 · cf. 7.1.2–23)

**Problem.** Explain why *almost every* Boolean function of $n$ variables has
combinational complexity $C(f) = \Theta(2^n / n)$: most functions require
exponentially many gates. Give the counting argument for the lower bound and
state the matching Lupanov upper bound.

**Hint.** Count how many distinct functions a chain of $r$ gates can possibly
compute, and compare with the total number of functions $2^{2^n}$. If there are
too few small chains, most functions cannot have one.

**Answer sketch.** **Lower bound (counting chains).** A chain of $r$ gates over
the 16-operation basis is specified by choosing, per gate, an operation (16 ways)
and two earlier operands (at most $(n + r)^2$ ways). So at most

$$
\big(16\,(n+r)^2\big)^r = 2^{O(r \log r)}
$$

distinct functions have an $r$-gate chain. To realize even a constant fraction of
all $2^{2^n}$ functions we need $2^{O(r \log r)} \ge \text{const} \cdot 2^{2^n}$,
i.e. $O(r \log r) \ge 2^n - O(1)$, which forces $r = \Omega(2^n / n)$. Because
the count of realizable functions is so much smaller than the count of functions
whenever $r \ll 2^n/n$, the fraction of functions with a small chain tends to 0:
**almost every function needs $\Omega(2^n/n)$ gates.** **Upper bound (Lupanov).**
Lupanov proved $O(2^n/n)$ gates always *suffice* — every function has a chain of
that order — so the bound is tight and the typical complexity is
$C(f) = \Theta(2^n / n)$. **Moral.** This is the double exponential of §1 biting
back: there are $2^{2^n}$ functions but only $2^{O(r \log r)}$ small circuits, so
small circuits are precious. The functions we actually build hardware for
(adders, comparators, symmetric functions) are the rare cheap exceptions.

### 7. Compute the Dedekind number $M(5)$; the $M(9)$ story (rating 40 · cf. §7.1.1)

**Problem.** The **Dedekind number** $M(n)$ counts the monotone Boolean functions
of $n$ variables. Compute $M(5)$ by brute force — enumerate the relevant space
and count the monotone functions — and describe why $M(8)$ and $M(9)$ needed
heroic computation rather than a simple loop.

**Hint.** For a direct count you would sweep all $2^{2^n}$ functions and test
monotonicity (single-bit raises), as the lab does for $n \le 4$. At $n = 5$ the
space $2^{2^5} = 2^{32}$ is already large; think about what makes $n = 8, 9$
astronomically worse.

**Answer sketch.** By definition (README §4) $M(0..4) = 2, 3, 6, 20, 168$, and
the next value is

$$
M(5) = 7581.
$$

A brute-force enumeration of all $2^{2^5} = 2^{32} \approx 4.3 \times 10^9$
functions, testing each for monotonicity via single-bit raises, reproduces
$7581$ — feasible but no longer instant, which is why the lab caps its live
computation at $n \le 4$ ($65536$ functions). The wall past $n = 5$ is the
**double exponential** $2^{2^n}$: the search space is $2^{64}$ at $n = 6$,
$2^{256}$ at $n = 8$, $2^{512}$ at $n = 9$ — far beyond any exhaustive sweep, and
there is no known closed form for $M(n)$. So $M(8)$ was found only in 1991 on a
supercomputer, and **$M(9)$ was computed only in 2023** — independently by
Christian Jäkel and by Van Hirtum et al. — using months of FPGA and GPU time on
cleverly structured counts (not naïve enumeration), giving
$M(9) = 286386577668298411128469151667598498812366$. Dedekind posed the problem
in 1897; each new value has been a milestone, and the count remains open past
$n = 9$.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
