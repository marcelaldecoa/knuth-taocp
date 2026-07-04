# Exercises — Module 01 (The Notion of an Algorithm, §1.1)

Self-contained problems on this module's material — the five properties of an
algorithm, Euclid's Algorithm E, its wasteless variant F, and the formal
notion of a computational method. You can work every one **without the book**:
each states the problem in full, gives a **hint** to peek at when stuck, and a
worked **answer sketch** to check against after you try. Computational answers
here are reproducible with pencil and paper (or the code you write in the lab).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 1.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Compute $\gcd(2166, 6099)$ by hand with Algorithm E | 10 | ⬜ |
| 2 | Trace the variables of Algorithm E on $(m, n) = (119, 544)$ | 15 | ⬜ |
| 3 | ▶ Design Algorithm F (eliminate the trivial replacement) | 16 | ⬜ |
| 4 | Grade a cookbook recipe against the five properties | 12 | ⬜ |
| 5 | ▶ Define equivalence of computational methods; prove E $\equiv$ F | 30 | ⬜ |

## Problems

### 1. Compute $\gcd(2166, 6099)$ by hand with Algorithm E (rating 10 · cf. 1.1–1)

**Problem.** Apply Algorithm E (steps E1–E3: divide, test remainder, reduce) to
$m = 2166$, $n = 6099$ and find $\gcd(2166, 6099)$. Show every division.

**Hint.** Since $m < n$ here, the very first division just leaves $r = m$ and step
E3 swaps the operands — no special case needed, exactly as the lesson notes for
$m < n$. After that, each step replaces $(m, n)$ by $(n, m \bmod n)$.

**Answer sketch.** Tabulating each execution of E1 as $m = qn + r$:

| $m$ | $n$ | $q$ | $r = m \bmod n$ |
|---|---|---|---|
| 2166 | 6099 | 0 | 2166 |
| 6099 | 2166 | 2 | 1767 |
| 2166 | 1767 | 1 | 399 |
| 1767 | 399 | 4 | 171 |
| 399 | 171 | 2 | 57 |
| 171 | 57 | 3 | **0** |

The last nonzero remainder / final $n$ is $\boxed{57}$, reached in six divisions.
Check: $2166 = 57 \cdot 38$ and $6099 = 57 \cdot 107$, and $\gcd(38, 107) = 1$, so
$57$ is indeed the greatest common divisor.

### 2. Trace the variables of Algorithm E on $(m, n) = (119, 544)$ (rating 15 · cf. 1.1–2)

**Problem.** List the values held by the variables $m$, $n$, $r$ at each execution
of step E1 when Algorithm E runs on $m = 119$, $n = 544$. How many divisions does
it take, and what is the answer?

**Hint.** Track the triple $(m, n, r)$ each time you reach E1. Because $119 < 544$,
predict what the first remainder must be before you compute anything — the
lesson's "if $m < n$ the first division swaps the operands" remark tells you.

**Answer sketch.** Each row is one arrival at E1, showing $(m, n)$ on entry and the
remainder $r$ computed:

| E1 pass | $m$ | $n$ | $r$ | then |
|---|---|---|---|---|
| 1 | 119 | 544 | 119 | E3 swaps to $(544, 119)$ |
| 2 | 544 | 119 | 68 | E3 to $(119, 68)$ |
| 3 | 119 | 68 | 51 | E3 to $(68, 51)$ |
| 4 | 68 | 51 | 17 | E3 to $(51, 17)$ |
| 5 | 51 | 17 | **0** | E2 halts, answer $n = 17$ |

Five divisions; the answer is $\gcd(119, 544) = 17$. The first pass is the
"wasted" swap-division ($r = 119 = m$), and the remaining four do the real work —
the same four divisions the lesson traces for $(544, 119)$.

### 3. ▶ Design Algorithm F (eliminate the trivial replacement) (rating 16 · cf. 1.1–3)

**Problem.** In Algorithm E, step E3 does the assignment $m \leftarrow n$,
$n \leftarrow r$ — pure data movement, no arithmetic. Design an equivalent
algorithm F that computes $\gcd(m, n)$ *without* this replacement step, and argue
it computes the same function. (You implemented F in Stage 2.)

**Hint.** The replacement only exists to keep "the dividend" in $m$ and "the
divisor" in $n$. Instead of moving values between $m$ and $n$, *alternate which
variable plays the divisor* on successive steps.

**Answer sketch.** Keep both operands in place and take remainders in each
direction, alternating:

```text
F1. [Remainder m mod n.]  Set m <- m mod n.
F2. [Is m zero?]          If m = 0, terminate; n is the answer.
F3. [Remainder n mod m.]  Set n <- n mod m.
F4. [Is n zero?]          If n = 0, terminate; m is the answer. Otherwise go to F1.
```

No value is ever copied from one variable to the other. Correctness rides on the
Stage-1 lemma $\gcd(m, n) = \gcd(n, m \bmod n)$: step F1 replaces the pair
$(m, n)$ by $(m \bmod n, n)$ and F3 replaces $(m, n)$ by $(m, n \bmod m)$ — each
preserves the gcd — and the second argument strictly decreases across the pair
F1–F3, so it terminates by the same "decreasing non-negative integer" argument
as E. Two passes of F reproduce one E-reduce-plus-swap, so F and E compute the
identical function $\gcd$; that they are *different methods* for the *same
function* is the whole point (Problem 5 makes "same function" precise). The Stage-2
tests confirm F agrees with E on the full $120 \times 120$ grid.

### 4. Grade a cookbook recipe against the five properties (rating 12 · cf. 1.1–5)

**Problem.** Take a typical cookbook recipe ("season to taste; bake until golden;
add a pinch of salt") and decide, property by property, whether it qualifies as
an *algorithm* under Knuth's five criteria: finiteness, definiteness, input,
output, effectiveness.

**Hint.** Go through the five in order and look for the weakest link. Which
phrases could two cooks reasonably carry out differently? Which step lacks a
precise stopping condition?

**Answer sketch.** A recipe scores well on some criteria and fails others:

- **Input** ✓ — the ingredients are the specified input quantities.
- **Output** ✓ — the finished dish is a well-defined output.
- **Finiteness** — usually ✓ if each step ends, though "bake until golden" only
  terminates if "golden" is reliably reached.
- **Definiteness** ✗ — "season to taste", "a pinch", "until golden" are
  ambiguous: two people executing the same words get different actions. This is
  the criterion cookbooks characteristically violate.
- **Effectiveness** — the individual physical operations are doable, but they are
  not the *exact, mechanical* operations Knuth demands ("a person with pencil and
  paper could do it exactly").

Verdict: a recipe is a **computational method** at best, not an algorithm — it
fails **definiteness** (and arguably effectiveness). The lesson makes exactly this
distinction: a procedure that is not rigorously and unambiguously specified is
not an algorithm, which is why our implementations reject under-specified inputs
loudly rather than guess.

### 5. ▶ Define equivalence of computational methods; prove E $\equiv$ F (rating 30 · cf. 1.1–9)

**Problem.** Give a formal definition of what it means for two computational
methods to be *equivalent*, then prove that your Algorithm E and Algorithm F
(Problem 3) are equivalent under your definition.

**Hint.** The lesson's formal apparatus is the quadruple $(Q, I, \Omega, f)$: a
state set $Q$, inputs $I$, outputs $\Omega$, and a transition $f : Q \to Q$ fixing
each output. A computation is $x, f(x), f(f(x)), \dots$ until it lands in $\Omega$.
Equivalence should compare the *input-to-output relation*, not the intermediate
states, since E and F pass through different states.

**Answer sketch.** Model each method as $(Q, I, \Omega, f)$ per the lesson. Two
methods $(Q, I, \Omega, f)$ and $(Q', I', \Omega', f')$ are **equivalent** when
they share the same input and output sets ($I = I'$, and $\Omega$, $\Omega'$
identified by their output values) and, for every input $x \in I$, either both
computations fail to terminate, or both terminate with the *same output value*.
Equivalently: they compute the same partial function $I \to \Omega$. (This
deliberately ignores the path taken — a stronger "lock-step" notion that required
matching intermediate states would wrongly declare E and F inequivalent even
though they always return the same gcd.)

*Proof that E $\equiv$ F.* Both take $I = \{(m, n) : m, n \ge 1\}$ and output a positive
integer. Both always terminate (each has an invariant $\gcd$ and a strictly
decreasing non-negative second quantity — Problems 3 and the Stage-1 lemma). It
remains to show they yield the same value. By the reduction lemma
$\gcd(m, n) = \gcd(n, m \bmod n)$, every step of E preserves $\gcd(m, n)$, and E
halts returning $n$ when $r = 0$, i.e. returning $\gcd$; the same lemma shows each
of F's two remainder steps preserves $\gcd(m, n)$ and F halts returning the
nonzero operand, again $\gcd$. Hence for every input both terminate with the
identical value $\gcd(m, n)$, so E and F are equivalent. $\blacksquare$ This is the
distinction Knuth draws between the *function computed* and the *method computing
it*: equivalence is sameness of the former.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
