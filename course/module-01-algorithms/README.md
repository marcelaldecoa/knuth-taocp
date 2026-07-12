# Module 01 — The Notion of an Algorithm

> **Source:** *The Art of Computer Programming*, Vol. 1, 3rd ed., §1.1 (with a
> look ahead to §1.2.1 and §4.5.3).
> **Lab:** `labs/module-01-algorithms` · **Grade it:** `./grade 1`
> **Concrete Mathematics companion:** Chapter 4 (Number Theory) develops the gcd
> and divisibility this module rests on — see [../../docs/concrete-mathematics.md](../../docs/concrete-mathematics.md).
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 1, read §1.1 first — the lesson tells you where to look.

The word *algorithm* itself is where Knuth begins, and so do we. By the end of
this module you will have implemented Euclid's algorithm three ways, proved it
correct, and reproduced the first nontrivial algorithm analysis in history —
Gabriel Lamé's 1845 theorem about its worst case.

> **Companion exhibit — _The Invisible Ruler_.** This module has a visual twin in
> the [Museum of Algorithms](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-1.1-invisible-ruler.html):
> an interactive instrument where Euclid's algorithm carves squares out of a
> rectangle, the Bézout coefficients (stage 3) balance on a scale, and the
> Fibonacci worst case (stage 4) draws itself. Open it in a second window and
> run it as you read — or reach it any time from the **Museum** tab in the top
> navigation.

---

## 1. What is an algorithm?

An algorithm is not just "a recipe." Knuth demands five properties, and each
one earns its keep:

1. **Finiteness.** It terminates after a finite number of steps — always, for
   every allowed input. A procedure that *may* run forever is a *computational
   method*, not an algorithm. (An operating system's event loop is a perfectly
   respectable computational method.)
2. **Definiteness.** Every step is rigorously and unambiguously specified.
   This is why our implementations reject inputs the algorithm was never
   defined for, loudly, instead of guessing.
3. **Input.** Zero or more quantities, drawn from specified sets. Euclid's
   algorithm takes two *positive integers* — not "numbers."
4. **Output.** One or more quantities with a specified relation to the inputs.
5. **Effectiveness.** Each operation must be basic enough that a person with
   pencil and paper could do it exactly, in finite time. "Divide m by n" is
   effective for integers; "print the exact decimal expansion of $\pi$" is not.

Knuth also fixes a *style* for writing algorithms that we will keep for the
whole course. An algorithm has a letter (Algorithm **E**), and numbered steps
**E1**, **E2**, … Each step has a bracketed summary phrase, an action, and
sometimes a parenthetical *assertion* — a fact that is true whenever execution
reaches that point. Those assertions are the seeds of correctness proofs.

### Algorithm E (Euclid's algorithm)

Given two positive integers `m` and `n`, find their greatest common divisor —
the largest positive integer that evenly divides both.

```text
E1. [Find remainder.]  Divide m by n and let r be the remainder.
                       (We will have 0 <= r < n.)
E2. [Is it zero?]      If r = 0, the algorithm terminates; n is the answer.
E3. [Reduce.]          Set m <- n, n <- r, and go back to step E1.
```

Trace it on m = 544, n = 119 with pencil and paper before you code anything:

| step | m | n | r |
|---|---|---|---|
| E1 | 544 | 119 | 68 |
| E1 | 119 | 68 | 51 |
| E1 | 68 | 51 | 17 |
| E1 | 51 | 17 | **0** |

Four divisions; the answer is **17**. Notice something pleasant: if m < n, the
first division leaves r = m and step E3 simply swaps the operands. No special
case needed — a hallmark of a well-stated algorithm.

### Why it's correct

Two facts, both worth proving on paper (they are stage-1 knowledge):

**Lemma (the reduction step preserves the answer).** For $n > 0$,
$\gcd(m, n) = \gcd(n, m \bmod n)$.

*Proof.* Write $m = qn + r$ with $r = m \bmod n$. Any $d$ dividing both $m$ and
$n$ divides $r = m - qn$; any $d$ dividing both $n$ and $r$ divides
$m = qn + r$. So the pairs $(m, n)$ and $(n, r)$ have exactly the same set of
common divisors — in particular the same greatest one. ∎

**Termination.** After each execution of **E3** the second operand strictly
decreases: $n > r \ge 0$. A strictly decreasing sequence of non-negative
integers is finite, so **E2** must eventually see $r = 0$. And when it does,
$\gcd(m, n) = \gcd(n, 0) = n$. ∎

Together: finiteness ✓, and the output relation holds ✓. This little proof
pattern — *an invariant plus a decreasing quantity* — will carry you through
the entire course, from sorting networks to SAT solvers.

### Computational methods, formally (optional but beautiful)

Knuth closes §1.1 by making "computational method" precise: a quadruple
$(Q, I, \Omega, f)$ where $Q$ is a set of *states*, $I \subseteq Q$ the inputs,
$\Omega \subseteq Q$ the outputs, and $f : Q \to Q$ a transition function that
fixes every output state ($f(q) = q$ for $q \in \Omega$). A computation is the
sequence $x, f(x), f(f(x)), \ldots$; it *terminates* when it first hits
$\Omega$. An **algorithm** is a computational method that terminates for every
$x \in I$, with each $f$ effective. Euclid's algorithm in this dress: states are
tuples $(m, n, r, \text{step-number})$; try writing $f$ out once — it is a
five-minute exercise that makes the definition stick.

---

## 2. Stage-by-stage lab guide

Open `labs/module-01-algorithms/src/lab.rs`. Each stage below has a test file
`labs/module-01-algorithms/tests/stage_NN_*.rs`; run `./grade 1` and the
grader takes the stages in order, stopping at the first failure, exactly like
a CodeCrafters track.

### Stage 1 — `euclid_e` (Algorithm 1.1E)

Implement Algorithm E exactly as stated, keeping the step labels as comments.
Two Rust-specific notes:

- Model "go back to step E1" with `loop`, not recursion — Knuth's algorithms
  are imperative state machines, and the step-faithful style makes the
  correspondence to the book auditable.
- *Definiteness:* `assert!(m > 0 && n > 0, "... positive ...")`. The test
  checks the panic message mentions "positive" — an algorithm's input
  specification is part of the algorithm.

### Stage 2 — `euclid_f` (Algorithm 1.1F, exercise 1.1-3, rating 16)

Knuth asks: the replacement `m <- n, n <- r` in E3 does no computation —
can it be eliminated? Yes: alternate which variable holds the "dividend."

```text
F1. [Remainder m/n.]  m <- m mod n.   F2. If m = 0, answer n.
F3. [Remainder n/m.]  n <- n mod m.   F4. If n = 0, answer m.  Back to F1.
```

The point of this stage is not speed; it is that *two different algorithms*
can compute *the same function* — Knuth distinguishes the function computed
from the method computing it, and so should you. The tests check F against
your E on a 120×120 grid.

### Stage 3 — `extended_euclid` (Algorithm 1.2.1E)

Euclid's algorithm can do more than find $d = \gcd(m, n)$: it can produce
integers $a, b$ with

$$a\,m + b\,n = d \qquad\text{(Bézout's identity)}$$

which *certify* the answer — anyone can multiply and check. Maintain two
coefficient pairs alongside the $(c, d)$ pair that plays the role of $(m, n)$:

$$a_1 m + b_1 n = c \quad\text{and}\quad a\,m + b\,n = d.$$

Initially $(a_1, b_1) = (1, 0)$, $(a, b) = (0, 1)$, $(c, d) = (m, n)$. Each
division step $c = qd + r$ replaces $(c, d)$ by $(d, r)$; restore the invariant
by replacing $(a_1, a)$ with $(a, a_1 - q a)$ and likewise for $b$. When
$r = 0$, return $(d, a, b)$.

*Why this matters:* when $\gcd(m, n) = 1$, the identity reads
$a\,m \equiv 1 \pmod n$ — extended Euclid *is* modular inversion, the workhorse
of §4.5.2 and of all of modern public-key cryptography. The stage tests use it
exactly that way.

*Proof to write in your notebook:* the invariant holds initially and is
preserved by the update; at termination $d = \gcd$ by the Stage-1 lemma. That
is mathematical induction doing the work — §1.2.1's theme.

### Stage 4 — `division_steps` and Lamé's theorem

Let $T(m, n)$ be the number of divisions (executions of **E1**) that Algorithm
E performs. How large can $T$ be? This question, answered by Lamé in 1845, is
often called the first real theorem of *analysis of algorithms*.

**Intuition.** The descent is slowest when every quotient is 1, i.e. when
each number is barely bigger than the next — and "each term is the sum of the
one before" is precisely the Fibonacci recurrence $F_1 = F_2 = 1$,
$F_{k+1} = F_k + F_{k-1}$.

**Theorem (Lamé).** If $n < F_{k+1}$, then Algorithm E applied to any $(m, n)$
performs at most $k - 1$ divisions once $m, n$ are in reduced position; the pair
$(F_{k+1}, F_k)$ attains $T = k - 1$ exactly.

*Proof sketch (run the machine backwards).* Suppose $T(m, n) = t$ with $m > n$.
The last pair before termination is $(x, y)$ with $y \ge 1$ and $x \ge 2y$… in
fact, working backwards from the final division, the smallest inputs that can
force $t$ divisions satisfy the Fibonacci recurrence, so $n \ge F_{t+1}$.
Contrapositive: $n < F_{k+1}$ implies $t \le k - 1$. Knuth gives the full
argument (and much more — the *average*, via continued fractions) in §4.5.3.

**Your job:** implement `division_steps`, then let the tests run the
experiment: exhaustively over all $m, n < F_{16} = 987$, the worst case is 14
divisions and the first pair to achieve it is $(377, 610) = (F_{14}, F_{15})$ —
the swap costs one extra division, then the pure Fibonacci descent does 13.

The last test peeks at the average case: Knuth shows $T(m, n)$ averages about
$(12 \ln 2 / \pi^2) \ln n \approx 0.843 \ln n$. You verify only the order of
magnitude —
the constant took mathematicians (Heilbronn, Dixon, Porter) into the 1970s to
pin down, and Knuth tells that story in §4.5.3.

---

## 3. Check your understanding

Answer before moving on (no code needed):

1. Which of the five properties fails for "S1. Set $x$ to a real number solving
   $x^2 = 2$"? (Effectiveness — and also definiteness: *which* solution?)
2. Why does Algorithm E terminate even when $m < n$ initially?
3. If $T(m, n) = 1$, what can you say about $m$ and $n$? ($n$ divides $m$.)
4. Where exactly does the correctness proof use $m = qn + r$ with
   $0 \le r < n$ — and which Rust operator guarantees it for `u64`?

## 4. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 fifteen minutes to
an hour · 30 hours · 40 term project · 50 open research problem. An arrow ▶
marks especially instructive exercises. Rows tagged *course* are our own
exercises, drawn from §1.1's prose rather than the book's exercise list. Log
your attempts in `course/module-01-algorithms/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 1.1-4 | 10 | Text says $\gcd(2166, 6099)$; compute it by hand with Algorithm E. |
| course (§1.1) | 15 | What is the value of the variables at each step of E on (m, n) = (119, 544)? |
| ▶1.1-3 | 16 | Design Algorithm F (you did — stage 2). |
| course (§1.1) | 12 | Is a recipe from a cookbook an algorithm? Grade it against the five properties. |
| ▶1.1-9 | 30 | Formalize what it means for one computational method to *simulate* another; course variant: define *equivalence* and prove your E and F equivalent under your definition. |

## In the real world

Extended Euclid is not a museum piece — well, it *is* one now: the Museum's
[Fermat's Clock](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-2.3-fermats-clock.html)
shows the RSA trapdoor whose private key is exactly the modular inverse your
stage-3 code computes. It also runs the modular inverses inside
RSA key generation and elliptic-curve signatures, so a version of your stage-3
code runs every time your browser opens a TLS connection. Bignum libraries
(GMP and friends) normalize every rational number with a gcd. And Lamé's
theorem is the ancestor of a promise you now see in every serious API:
*a worst-case bound stated as a theorem, not a hope*. Even the definiteness
drill — panic loudly on inputs the algorithm wasn't defined for — is exactly
Rust's `assert!`/debug-assertion culture: an algorithm's domain is part of
its contract.

## Proof techniques you practiced

- **Invariant + decreasing quantity** — the gcd-preservation lemma plus the
  strictly shrinking remainder: partial correctness and termination,
  separately, then combined. You will reuse this on every loop you ever prove.
- **Certification** — Bézout coefficients make the output *checkable* by
  anyone; stage 3's tests verify the certificate, not your internals.
- **Extremal witness** — Lamé's theorem is proved by *constructing* the worst
  input (consecutive Fibonacci numbers), then arguing nothing beats it.
- **Empirical confirmation of a theorem** — stage 4's exhaustive sweep below
  $F_{16}$ is analysis of algorithms as Knuth practices it: derive, then measure.

## 5. Where this leads

- The **invariant + decreasing quantity** proof pattern returns in every
  module; next in Module 02 as honest-to-goodness induction (§1.2.1).
- $T(m, n)$ is your first *cost function*; Module 02 builds the asymptotic
  toolkit (O-notation, harmonic numbers) that makes such functions tame.
- **Extended Euclid** returns in Module 05 (binary gcd, modular arithmetic,
  primality testing).
