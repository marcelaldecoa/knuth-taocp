# Exercises — Module 10 (Satisfiability, §7.2.2.2)

Self-contained problems on this module's material — the empty-clause/empty-formula
edge cases, soundness of the unit rule, the van der Waerden number $W(3,3)$,
at-most-one encodings and their clause counts, the soundness and completeness of
DPLL, and Haken's exponential lower bound for pigeonhole. You can work every one
**without the books**: each states the problem in full, gives a **hint** to peek
at when stuck, and a worked **answer sketch** to check against after you try.
Computational answers here are reproduced by the code you write in the lab (or a
few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 4B.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Empty clause is UNSAT, empty formula is valid | 10 | ⬜ |
| 2 | The unit-clause rule preserves the set of models | 15 | ⬜ |
| 3 | ▶ $W(3,3) = 9$: an 8-witness, and why 9 is impossible | 22 | ⬜ |
| 4 | Pairwise vs sequential at-most-one: count clauses | 20 | ⬜ |
| 5 | ▶ DPLL is sound and complete | M30 | ⬜ |
| 6 | Haken's pigeonhole lower bound: read and outline | 40 | ⬜ |

## Problems

### 1. Empty clause is UNSAT, empty formula is valid (rating 10 · cf. 7.2.2.2–1)

**Problem.** In CNF, a clause is a disjunction (OR) of literals and a formula is
a conjunction (AND) of clauses. Show that the **empty clause** `[]` is
unsatisfiable (so any formula containing it is UNSAT), while the **empty formula**
`[]` (no clauses) is valid — satisfied by every assignment. Relate both to how
`evaluate` must behave.

**Hint.** A disjunction is true iff *at least one* disjunct is true; a
conjunction is true iff *every* conjunct is true. Apply each definition to a
collection of size zero.

**Answer sketch.** The empty clause is an OR over *no* literals; an OR is true iff
some disjunct is true, and with no disjuncts there is none, so it is **false**
under every assignment — unsatisfiable. Since a formula is an AND of its clauses
and an AND is false as soon as one conjunct is false, any formula containing `[]`
is unsatisfiable; indeed *deriving* the empty clause is exactly how a solver
proves UNSAT. Conversely the empty formula is an AND over *no* clauses; an AND is
true iff every conjunct is true, and vacuously there are none to fail, so it is
**true** under every assignment — valid. Accordingly `evaluate` must require
*every* clause to have at least one true literal: with an empty clause present
that clause has no true literal, so `evaluate` returns `false`; with no clauses
at all the "every clause" condition holds vacuously, so `evaluate` returns
`true`. These two conventions are what the Stage 1 tests pin down.

### 2. The unit-clause rule preserves the set of models (rating 15 · cf. 7.2.2.2–4)

**Problem.** The unit rule says: if, under the current partial assignment, a
clause $C$ has no true literal and exactly one unassigned literal $\ell$, then
$\ell$ must be set true. Prove that forcing $\ell$ true does not change the set of
*full* satisfying assignments that extend the current partial one — so unit
propagation never loses (or gains) a model. This is the soundness that makes
Stage 2 safe to run before search.

**Hint.** Take any full assignment $A$ that (i) extends the current partial one
and (ii) satisfies the whole formula. What value must $A$ give $\ell$ in order to
satisfy the clause $C$?

**Answer sketch.** Let $A$ be any model extending the current partial assignment.
$A$ must satisfy $C$. By hypothesis every literal of $C$ other than $\ell$ is
already *false* under the partial assignment (no true literal, and $\ell$ is the
only unassigned one), and $A$ extends that partial assignment, so those literals
are still false under $A$. The only way $A$ can make $C$ true is therefore
$\ell = \text{true}$. Hence *every* model extending the current partial assignment
already sets $\ell$ true, so adding the constraint "$\ell := \text{true}$" removes
no models. And it obviously adds none (it only restricts). Thus the set of
extending models is unchanged. Two consequences the README relies on: if the rule
ever forces a variable both true and false along a chain, the clause that becomes
all-false is a genuine **conflict** (no extending model exists); and running
propagation again at the fixed point forces nothing (**idempotence**,
`Implied([])`), since every unit has already been resolved. $\blacksquare$

### 3. ▶ $W(3,3) = 9$: an 8-witness, and why 9 is impossible (rating 22 · cf. 7.2.2.2–13)

**Problem.** $W(3,3)$ is the least $n$ such that *every* red/blue colouring of
$1,\dots,n$ contains a monochromatic 3-term arithmetic progression (AP)
$a, a+d, a+2d$. Show $W(3,3) = 9$ by (a) exhibiting a colouring of $1,\dots,8$
with no monochromatic 3-AP, and (b) arguing that no colouring of $1,\dots,9$
avoids one. The `waerden_cnf(3,3,n)` formula is SAT iff $n < W(3,3)$.

**Hint.** For (a) try the period-4 pattern from the README. For (b), you do not
need to check all $2^9$ colourings by hand: fix the colour of $5$ (the centre)
and chase the forced consequences through the APs through $5$, or simply appeal
to the exhaustive check your solver performs — `solve(&waerden_cnf(3,3,9))`
returns `None`.

**Answer sketch.** *(a) Witness.* The colouring
$$\begin{array}{c|cccccccc} i & 1 & 2 & 3 & 4 & 5 & 6 & 7 & 8 \\ \hline
\text{colour} & R & R & B & B & R & R & B & B \end{array}$$
(RRBBRRBB) has *no* monochromatic 3-AP. There are exactly twelve 3-APs inside
$1..8$; listing their colour triples:
$(1,2,3){=}RRB$, $(2,3,4){=}RBB$, $(3,4,5){=}BBR$, $(4,5,6){=}BRR$,
$(5,6,7){=}RRB$, $(6,7,8){=}RBB$ (step $1$); $(1,3,5){=}RBR$, $(2,4,6){=}RBR$,
$(3,5,7){=}BRB$, $(4,6,8){=}BRB$ (step $2$); $(1,4,7){=}RBB$, $(2,5,8){=}RRB$
(step $3$). None is $RRR$ or $BBB$, so $n = 8$ is colourable — hence
$W(3,3) > 8$.

*(b) No 9-colouring.* An exhaustive search over all $2^9 = 512$ colourings of
$1,\dots,9$ finds that every one contains a monochromatic 3-AP; equivalently
`waerden_cnf(3,3,9)` is UNSAT, so `solve` returns `None`. Therefore $n = 9$
forces a monochromatic 3-AP while $n = 8$ does not, giving $W(3,3) = 9$ exactly —
the smallest interesting van der Waerden number, and the Stage 3 SAT/UNSAT pair
(`waerden(3,3;8)` SAT, `waerden(3,3;9)` UNSAT). $\blacksquare$

### 4. Pairwise vs sequential at-most-one: count clauses (rating 20 · cf. 7.2.2.2–17)

**Problem.** "At most one of $\ell_1, \dots, \ell_n$ is true" can be encoded two
ways. The **pairwise** (binomial) encoding adds a clause
$(\lnot\ell_i \lor \lnot\ell_j)$ for each pair $i < j$ and no new variables. The
**sequential** (ladder / Sinz counter) encoding introduces auxiliary variables to
stay linear. Give the exact clause count of the pairwise encoding, and compare it
with the sequential encoding's variable and clause counts. Why does the
difference matter at scale?

**Hint.** The number of pairs of $n$ items is $\binom{n}{2}$. For the sequential
counter, introduce one "carry" variable $s_i$ per position (bar the last) and
write the three implications that propagate a used slot forward.

**Answer sketch.** *Pairwise.* One clause per unordered pair, so exactly
$$\binom{n}{2} = \frac{n(n-1)}{2} = O(n^2)$$
clauses and **zero** auxiliary variables. E.g. $n=3 \to 3$, $n=4 \to 6$,
$n=5 \to 10$, and $n = 1000 \to \binom{1000}{2} = 499{,}500$ clauses — the
$\approx 500{,}000$ figure the README cites for one Sudoku row.

*Sequential (Sinz counter).* Introduce carries $s_1, \dots, s_{n-1}$ ($n-1$
auxiliary variables) meaning "one of $\ell_1..\ell_i$ is already used," with
clauses $(\lnot\ell_1 \lor s_1)$, $(\lnot\ell_n \lor \lnot s_{n-1})$, and for
each $1 < i < n$ the three clauses $(\lnot\ell_i \lor s_i)$,
$(\lnot s_{i-1} \lor s_i)$, $(\lnot\ell_i \lor \lnot s_{i-1})$. That totals
$2 + 3(n-2) = 3n - 4$ clauses and $n - 1$ new variables — both $O(n)$
(e.g. $n = 1000 \to 999$ variables, $2996$ clauses).

*Why it matters.* Pairwise is $O(n^2)$ clauses, sequential is $O(n)$ clauses at
the cost of $O(n)$ extra variables. At lab sizes the pairwise encoding wins on
simplicity (no auxiliaries, and both give the same models on the original
variables), but when $n$ is large — many at-most-one constraints, each over
hundreds of literals — the quadratic blow-up dominates the formula size and can
be the difference between a solver finishing in a second and choking. Encoding
choice is a first-class performance decision, a recurring theme of §7.2.2.2.

### 5. ▶ DPLL is sound and complete (rating M30 · cf. 7.2.2.2–99)

**Problem.** Prove that the DPLL procedure (Algorithm D: propagate to a fixed
point, report a model if all clauses are satisfied, otherwise branch on a literal
$\ell$ trying both $\ell = \text{true}$ and $\ell = \text{false}$, backtracking on
conflict) is **sound** (any model it reports really satisfies the formula) and
**complete** (it reports `None` only when the formula is genuinely
unsatisfiable). Include termination.

**Hint.** Three pieces. Soundness: when does D report a model, and what has it
verified? Completeness: unit propagation preserves models (Problem 2), and
branching on both values of $\ell$ partitions the remaining search space with no
gaps. Termination: what strictly decreases down every branch?

**Answer sketch.** *Soundness.* D reports a model only at a node where every
clause has a true literal under the current (then completed) assignment — i.e. it
has *verified* satisfaction directly, exactly as `evaluate` would. So a reported
model is genuine; D never reports a wrong "SAT."

*Completeness.* Argue that at every node, the models of the formula that extend
the current partial assignment are exactly covered by the subtree below.
Propagation (Problem 2's unit rule) only assigns *logically forced* literals, so
it neither adds nor removes extending models; if it reaches a conflict
(all-false clause), there is provably *no* extending model, and pruning that
branch loses nothing. At a branch node the two children assign
$\ell = \text{true}$ and $\ell = \text{false}$; every completion of the current
partial assignment sets $\ell$ to one value or the other, so the union of the two
subtrees covers *all* completions — no model can slip between them. By induction
from the leaves up, a subtree returns `None` only when it contains no model, and
returns a model whenever one exists in its range. At the root this says: D
returns `None` iff the formula is unsatisfiable, and otherwise returns a genuine
model. Crucially, on backtrack D must undo *everything* propagation forced after
the guess (the README's snapshot/restore), else the "covers all completions"
invariant breaks.

*Termination.* Each branch node fixes one previously-unassigned variable, and
propagation only ever assigns variables, never unassigns; so the number of
unassigned variables strictly decreases down every path and the recursion depth
is at most `num_vars`. The tree is finite, so D halts. $\blacksquare$ (This is the
argument the README sketches in §5; the toy trace there finds the model
$x_1{=}F, x_2{=}T, x_3{=}F$ after exactly one backtrack.)

### 6. Haken's pigeonhole lower bound: read and outline (rating 40 · cf. 7.2.2.2–176)

**Problem.** The pigeonhole formula $\text{PHP}(n{+}1, n)$ — variables
$x_{p,h}$ = "pigeon $p$ in hole $h$," clauses "each pigeon sits somewhere"
$(x_{p,1} \lor \cdots \lor x_{p,n})$ and "no hole is shared"
$(\lnot x_{p,h} \lor \lnot x_{q,h})$ for $p < q$ — is unsatisfiable exactly
because $n+1 > n$. **Haken's theorem (1985)** states that every *resolution*
refutation of $\text{PHP}(n{+}1, n)$ has size exponential in $n$. Outline why this
implies DPLL runs in exponential time on the pigeonhole family, and sketch the
structure of the lower-bound argument. (Rating 40 — this is a "read the source and
reproduce the outline" exercise, not a from-scratch proof.)

**Hint.** First connect DPLL to resolution: the trace of a DPLL refutation can be
turned into a resolution proof of comparable size, so a lower bound on *all*
resolution proofs is a lower bound on DPLL. Then recall the two-move shape of
resolution lower bounds: show any short proof must contain a "wide" clause, then
count how many wide clauses a proof must contain.

**Answer sketch — the outline one reconstructs.**

*Step 1: DPLL reasoning is resolution.* A DPLL search that proves UNSAT can be
read as a *tree-like resolution refutation*: each conflict corresponds to
resolving the clauses that became all-false, and the branching structure records
the resolution steps. So the size of DPLL's search is lower-bounded by the size
of the smallest resolution refutation. Any statement "every resolution refutation
of $F$ is huge" therefore forces DPLL to be huge on $F$ — no branching heuristic
can escape it, because the bound is on the whole *proof system*, not one search
order. This is precisely why the README says the pigeonhole family drives DPLL
"to its knees" as a matter of principle, not implementation.

*Step 2: the shape of Haken's argument.* Lower bounds of this kind follow a
**width / bottleneck** pattern:
- **Every short refutation must contain a wide clause.** Using a restriction
  (random or adversarial fixing of some variables) one shows that any resolution
  refutation of $\text{PHP}(n{+}1, n)$ must, at some point, derive a clause
  mentioning a large number — order $n^2$ — of the variables. Intuitively the
  pigeonhole contradiction is "global": no small set of variables carries it, so
  the proof cannot stay narrow.
- **Wide clauses are expensive to accumulate.** A counting ("bottleneck")
  argument then shows a refutation must pass through *exponentially many*
  distinct such wide clauses, because each one constrains only a small fraction
  of the exponentially many pigeon-to-hole matchings, and covering them all
  requires exponentially many clauses.

Multiplying the two gives size $2^{\Omega(n)}$. The takeaway grounded in the
README: $\text{PHP}(n{+}1,n)$ is "obviously" false to a human counting argument, yet
provably hard for resolution and hence for DPLL — an *unconditional* lower bound
that motivates conflict-driven clause learning (CDCL, Module 14). Even CDCL does
not fully escape pigeonhole, but learned clauses let it reason in ways plain
resolution search cannot on many other families. (The lab confirms only the
qualitative face of this: `solve(&pigeonhole_cnf(4,3))` and `pigeonhole_cnf(5,4)`
return UNSAT, and the effort grows sharply with $n$.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
