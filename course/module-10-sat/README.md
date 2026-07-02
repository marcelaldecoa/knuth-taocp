# Module 10 — Satisfiability

> **Source:** *The Art of Computer Programming*, Vol. 4B, §7.2.2.2
> (*Satisfiability*).
> **Lab:** `labs/module-10-sat` · **Grade it:** `./grade 10`
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 4B, §7.2.2.2 is the longest section Knuth has ever written on
> a single topic — over 300 pages — and this module is a runway onto it.

The **Boolean satisfiability problem** (SAT) asks a deceptively small question:
given a Boolean formula, is there an assignment of `true`/`false` to its
variables that makes the whole thing true? By the end of this module you will
have built a complete SAT solver from scratch — DIMACS parser, unit-propagation
engine, and a DPLL search — and used it to settle real combinatorial questions:
van der Waerden numbers, the pigeonhole principle, n-queens, and graph
colouring. Knuth calls SAT "the *queen* of combinatorial problems," and the
reason is the next paragraph.

---

## 1. SAT, the queen of combinatorial problems

Almost every decision problem you meet in this course — does this graph have a
3-colouring? a Hamiltonian cycle? can these tasks be scheduled? — can be
*rephrased* as "is this Boolean formula satisfiable?" That is not a vague
analogy; it is a theorem.

**Cook–Levin theorem (1971).** SAT is **NP-complete**: it is in NP (a proposed
satisfying assignment can be *checked* in polynomial time — exactly what
`Cnf::evaluate` does), and *every* problem in NP reduces to it in polynomial
time. So a fast algorithm for SAT would be a fast algorithm for thousands of
problems at once, and the existence of one is precisely the P = NP question. We
do not expect a worst-case-polynomial algorithm. And yet modern SAT solvers
routinely dispatch industrial formulas with *millions* of variables. This gap
between worst-case hopelessness and practical triumph is the whole drama of
§7.2.2.2, and this module is where you feel it firsthand.

Because everything reduces to SAT, a good solver is a *universal* combinatorial
engine: you spend your cleverness on the *encoding* (stage 4), and let the
solver do the search.

---

## 2. Conjunctive normal form and DIMACS

A **literal** is a variable or its negation. In this course a literal is a
nonzero `i32`: `+v` means "variable *v* is true", `-v` means "variable *v* is
false", for `1 ≤ v ≤ num_vars`. A **clause** is a disjunction (OR) of literals,
written as a `Vec<i32>`. A formula in **conjunctive normal form** (CNF) is a
conjunction (AND) of clauses: `Vec<Vec<i32>>`.

```text
(x1 ∨ ¬x2) ∧ (x2 ∨ x3)      ≡      vec![vec![1, -2], vec![2, 3]]
```

Every Boolean formula has an equivalent CNF, so CNF costs no generality and
gives solvers one uniform shape to chew on. Two edge cases are worth burning
into memory, because they fall straight out of the definitions:

- The **empty clause** `[]` is a disjunction of *nothing*, hence **false**. A
  formula containing it is unsatisfiable — deriving the empty clause is exactly
  how a solver *proves* UNSAT.
- The **empty formula** `[]` is a conjunction of *nothing*, hence vacuously
  **true**. It is satisfied by every assignment.

### DIMACS `cnf`, the lingua franca

Every SAT benchmark and every solver speaks the DIMACS text format:

```text
c any line starting with c is a comment
p cnf 3 2        <- header: 3 variables, 2 clauses
1 -2 0           <- clause (x1 ∨ ¬x2), terminated by 0
2 3 0            <- clause (x2 ∨ x3)
```

Rules you will enforce in `parse_dimacs`: exactly one `p cnf V C` header before
any clause data; whitespace (spaces, tabs, newlines) separates tokens, so a
clause may wrap across lines or share a line with the next; each clause ends
with a `0`; a bare `0` is a legal *empty* clause. Reject — with a descriptive
`Err(String)` — a missing or duplicated header, clause data before the header,
non-integer garbage, a literal whose variable exceeds `V`, an unterminated final
clause, and a clause count that disagrees with `C`. `to_dimacs` is the inverse,
and the round trip must be lossless: `parse_dimacs(&c.to_dimacs()) == Ok(c)`.

### Evaluation

`evaluate(assignment)` takes a **complete** assignment (`assignment[v-1]` = the
value of variable *v*) and returns whether the formula holds: *every* clause
must have *at least one* true literal. A literal `lit` is true when
`lit > 0 && assignment[lit-1]` or `lit < 0 && !assignment[-lit-1]`. This one
function is your ground truth for the rest of the module — every "SAT" answer a
solver produces is validated by feeding its model back through `evaluate`, never
by comparing against a fixed expected model.

---

## 3. The running example: van der Waerden numbers

Knuth threads one family of formulas through all of §7.2.2.2, and so will we.

**Van der Waerden's theorem (1927).** For any *j*, *k* there is a least integer
`W(j, k)` such that *however* you 2-colour the integers `1, 2, …, W(j,k)` (say
red/blue), you are *forced* to create either a red *j*-term arithmetic
progression or a blue *k*-term one. An arithmetic progression (AP) is
`a, a+d, a+2d, …` — equally spaced integers.

The smallest interesting value is **W(3, 3) = 9**. Encode "is `1..n`
2-colourable with no monochromatic 3-term AP?" as `waerden_cnf(3, 3, n)`:
variable *i* means "*i* is red". For every 3-term AP `a, a+d, a+2d` inside
`1..=n` add two clauses,

```text
(¬x_a ∨ ¬x_{a+d} ∨ ¬x_{a+2d})     "not all three red"
( x_a ∨  x_{a+d} ∨  x_{a+2d})     "not all three blue"
```

The formula is satisfiable **iff** `n < W(3,3) = 9`. Your solver will confirm
both halves. For `n = 8` a satisfying colouring is

```text
i:     1 2 3 4 5 6 7 8
colour R R B B R R B B
```

Check it by hand: `(1,2,3)=RRB`, `(2,4,6)=RBR`, `(1,4,7)=RBB`, … no three
equally-spaced positions share a colour. For `n = 9` no such colouring exists —
that is *precisely* the statement `W(3,3) = 9`, and `solve(&waerden_cnf(3,3,9))`
returns `None`. (Knuth's book pushes this to `W(3,3,3,3)` and beyond; the
numbers explode — `W(2,6) = 1132`, and most values are simply unknown.)

---

## 4. Unit propagation — the inference engine

Before searching, a solver *reasons*. The one inference rule that carries
almost all the weight is the **unit clause rule**:

> If under the current partial assignment a clause has no true literal and
> exactly **one** unassigned literal, that literal *must* be set true — there is
> no other way to satisfy the clause.

Setting it may make another clause unit, forcing a chain of implications.
**Unit propagation** applies the rule to a fixed point. Two outcomes:

- a clause becomes **all-false** (0 unassigned, none true) → a **conflict**: the
  current partial assignment cannot be extended to a model;
- otherwise we reach a fixed point, having **forced** some literals.

Our `unit_propagate(cnf, assignment)` returns `Propagation::Conflict` or
`Propagation::Implied(forced)`, where `forced` lists the literals it assigned,
in order. It respects pre-assigned variables, skips satisfied clauses, and is
**idempotent**: run it again at the fixed point and it forces nothing —
`Implied(vec![])`.

### Trace

Formula `(x1) ∧ (¬x1 ∨ x2) ∧ (¬x2 ∨ x3)`, empty assignment:

```text
clause (x1):        unit → x1 := true         forced = [1]
clause (¬x1 ∨ x2):  ¬x1 now false → unit x2   forced = [1, 2]
clause (¬x2 ∨ x3):  ¬x2 now false → unit x3   forced = [1, 2, 3]
fixed point.        Implied([1, 2, 3])
```

Replace the first clause by two clauses `(x1) ∧ (¬x1)` and propagation forces
`x1 := true`, then finds `(¬x1)` all-false → `Conflict`.

### Watched literals (how real solvers do it)

Rescanning every clause on every step, as our lab version does, is fine at lab
scale but quadratic. Knuth's Algorithm D uses **two watched literals** per
clause and maintains this invariant:

> Each clause *watches* two of its literals. As long as both watched literals
> are non-false, the clause cannot be unit or falsified, so it may be ignored.

When a variable is assigned, only clauses watching the *now-false* literal are
inspected. Such a clause tries to move its watch to another non-false literal;
if none exists, the *other* watched literal is either unassigned (the clause is
now **unit** — force it) or false (**conflict**). The payoff: assigning a
variable touches only the clauses that could actually change status, and — the
elegant part — **backtracking needs no work at all**, because the watched pair
is still a valid pair after we unassign variables. This is the single most
important data-structure idea in practical SAT solving. You will implement the
simple scanning version; understanding the watched-literal invariant is enough.

---

## 5. DPLL — search on top of inference

Inference alone rarely settles a formula; eventually you must **guess**. The
Davis–Putnam–Logemann–Loveland procedure (1962), Knuth's **Algorithm
7.2.2.2D**, alternates guessing with propagation and backtracks on conflict.

```text
D1. [Initialize.]    Start from the empty partial assignment.
D3/D6. [Propagate.]  unit_propagate. If Conflict, this branch is dead → backtrack.
D2. [Success?]       If every clause has a true literal, report the model.
                     Otherwise pick a branching literal ℓ from an unsatisfied
                     clause (which now has ≥ 2 unassigned literals).
D4. [Branch.]        Assign ℓ := true and recurse.
D7. [Backtrack.]     If that failed, undo, assign ℓ := false and recurse.
D8. [Failure.]       If both values of ℓ fail, this subtree is UNSAT.
```

A subtle but crucial engineering point (D7): to backtrack you must undo
*everything* propagation forced after the guess, not just the guess. In the lab
kernel we simply **snapshot** the assignment before propagating and restore it
on failure.

### Completeness

DPLL never returns a wrong answer. Unit propagation only assigns literals that
are *logically forced*, so it never removes a model — the set of models
extending the partial assignment is preserved. And each branch node tries
*both* truth values of ℓ, so the two subtrees together cover *every* completion.
Therefore DPLL returns `None` only when genuinely no assignment satisfies the
formula, and a model only when it has verified one. (Termination: every branch
fixes one more variable, so recursion depth ≤ `num_vars`.)

### Hand trace with the search tree

Take `F = (x1 ∨ x2) ∧ (¬x1 ∨ ¬x2) ∧ (¬x1 ∨ x3) ∧ (¬x1 ∨ ¬x3)`. No clause is
unit at the root (all four have two unassigned literals), so we branch on the
first literal of the first clause, `x1`. Our kernel tries the literal *as it
appears* first — here that is `x1 := true`.

```text
root: no units, branch on x1
│
├── x1 = true                      D4
│     propagate:
│       (¬x1 ∨ ¬x2): unit → x2 = false
│       (¬x1 ∨  x3): unit → x3 = true
│       (¬x1 ∨ ¬x3): ¬x1 false, ¬x3 false  →  ✗ CONFLICT
│     backtrack (D7): undo x2, x3, x1
│
└── x1 = false                     D7
      propagate:
        (x1 ∨ x2): unit → x2 = true
        (¬x1 ∨ ¬x2): ¬x1 true  → satisfied
        (¬x1 ∨  x3): ¬x1 true  → satisfied
        (¬x1 ∨ ¬x3): ¬x1 true  → satisfied
      fixed point, all clauses satisfied  →  ✓ SUCCESS (D2)
      x3 unassigned → complete with false
      model: x1 = F, x2 = T, x3 = F
```

`evaluate([false, true, false])` returns `true` — a genuine model, found after
exactly one backtrack. This is the entire algorithm in miniature.

### Heuristics (why solvers differ)

*Which* literal to branch on (D2) is a free choice that dramatically changes the
tree size. Classic rules: **MOM** (maximum occurrences in minimum-size clauses),
**Jeroslow–Wang** (weight literals by `2^{-|clause|}`), and in modern CDCL
solvers **VSIDS** (bump variables that appear in recent conflicts). Any choice
is *correct*; only speed changes. Our lab branches on the first literal of the
first unsatisfied clause — the simplest rule that still lets propagation shine.

---

## 6. When search explodes: pigeonhole and the road to CDCL

Some formulas are UNSAT for a reason so simple a child sees it, yet drive DPLL
to its knees. The **pigeonhole principle** `PHP(m, n)` says *m* pigeons cannot
occupy *n* holes if `m > n` with no two pigeons sharing a hole. Encode it
(`pigeonhole_cnf`) with `x[p][h]` = "pigeon *p* in hole *h*": each pigeon sits
somewhere (`x[p][0] ∨ … ∨ x[p][n-1]`), and no hole is shared
(`¬x[p][h] ∨ ¬x[q][h]` for `p < q`). It is unsatisfiable exactly when `m > n`.

**Haken's theorem (1985).** Every *resolution* refutation of `PHP(n+1, n)` has
size **exponential** in *n*. Since DPLL's reasoning is a form of resolution,
DPLL provably needs exponential time on the pigeonhole family — no branching
heuristic can save it. This is not a defect of one implementation; it is a
lower bound on the whole proof system. `solve(&pigeonhole_cnf(4,3))` still
returns `None` quickly enough for the lab, but the growth is brutal.

Haken's result is exactly what motivates **CDCL** (conflict-driven clause
learning), the engine of every modern solver and Knuth's **Algorithm C**: when
search hits a conflict, *analyse* it, derive a brand-new clause that explains
the dead end, add it to the formula, and *non-chronologically* backjump past the
irrelevant guesses. Learned clauses let the solver reason in ways plain DPLL
cannot, escaping some (not all — pigeonhole stays hard) exponential traps. CDCL
is where this module points; you are building the DPLL skeleton it is layered on.

---

## 7. Encoding problems into SAT

The art of applying a SAT solver is the *encoding*. The recurring subproblem is
constraining *how many* of a set of literals are true.

### At-most-one and exactly-one

"At most one of `ℓ1, …, ℓn`" in the **pairwise** (binomial) encoding is one
clause per pair: `(¬ℓi ∨ ¬ℓj)` for all `i < j` — `C(n,2) = O(n²)` clauses and
*zero* new variables. Add the single clause `(ℓ1 ∨ … ∨ ℓn)` for at-*least*-one
and you have **exactly-one** (`exactly_one`). Note the edge case: exactly-one of
*nothing* yields the empty clause — correctly unsatisfiable.

The `O(n²)` blow-up matters at scale. §7.2.2.2 gives cleverer encodings — the
**sequential (ladder)** and **commander** encodings introduce `O(n)` auxiliary
variables to get only `O(n)` clauses. At lab sizes the pairwise encoding wins on
simplicity, but knowing the trade-off is the point: encoding choice can decide
whether a solver finishes in a second or a century.

### Three applications

- **n-queens** (`queens_cnf`). `x[r][c]` = "queen on row *r*, column *c*."
  Exactly one queen per row; at most one per column; at most one per diagonal
  (both directions). Satisfiable for `n = 1` and `n ≥ 4`, unsatisfiable for
  `n = 2, 3`. `decode_queens` reads a model into `cols[r]` = the queen's column.
- **Graph *k*-colouring** (`coloring_cnf`). `x[v][c]` = "vertex *v* has colour
  *c*." Exactly one colour per vertex; for every edge `(u,w)` and colour *c*,
  `(¬x[u][c] ∨ ¬x[w][c])` forbids equal colours on adjacent vertices. The
  Petersen graph is 3-colourable but the complete graph `K₄` is not
  2-colourable. `decode_coloring` reads out `colours[v]`.

Always validate the *decoded* solution against the problem's own rules — a
legal queen placement, a proper colouring — not against a fixed model. Many
distinct models can be equally correct.

### Symmetry

Both problems are riddled with **symmetry**: any colouring stays proper if you
permute the colour names; the queens board has 8 reflections/rotations. A solver
blindly re-explores all of them. **Symmetry-breaking** clauses (e.g. "vertex 0
gets colour 0", or lexicographic ordering constraints) prune these redundant
branches and are, in practice, the difference between tractable and hopeless on
symmetric instances — another major theme of §7.2.2.2.

---

## 8. Stage-by-stage lab guide

Open `labs/module-10-sat/src/lab.rs`. Run `./grade 10`; the grader takes the
four stages in order, stopping at the first failure.

### Stage 1 — `Cnf::parse_dimacs`, `to_dimacs`, `evaluate`
Build the representation. Parse defensively (every malformed input in §2 must
return `Err`), keep the round trip lossless, and make `evaluate` treat the empty
clause as false and the empty formula as true. `evaluate` panics if the
assignment is shorter than `num_vars` — an incomplete assignment has no truth
value.

### Stage 2 — `unit_propagate`
Loop over clauses classifying each as satisfied / unit / conflict / neither,
forcing units and repeating until a full pass forces nothing. Return
`Conflict` the instant a clause is all-false; otherwise `Implied(forced)`.
Respect pre-assigned variables and record forced literals in assignment order.
Match the `Propagation` enum exactly — the tests construct and match its
variants.

### Stage 3 — `solve`, `solve_brute`, `pigeonhole_cnf`, `waerden_cnf`
`solve` is Algorithm D: snapshot, propagate, detect success, else branch two
ways with backtracking. `solve_brute` enumerates all `2^num_vars` assignments as
the honest cross-check (cap at 25 vars). Then build the two formula families;
the tests confirm `PHP(4,3)`/`PHP(5,4)` UNSAT, `waerden(3,3;8)` SAT and
`waerden(3,3;9)` UNSAT, and cross-check random 3-SAT against `solve_brute`.

### Stage 4 — `exactly_one`, `queens_cnf`, `decode_queens`, `coloring_cnf`, `decode_coloring`
Assemble the encodings from `exactly_one` plus at-most-one pair clauses, and
write the decoders. Every SAT result is validated *semantically*: a decoded
queens placement has no two queens attacking, a decoded colouring has no
monochromatic edge.

---

## 9. Check your understanding

1. Why is a formula containing the empty clause unsatisfiable, while the empty
   formula is satisfiable? (Disjunction of nothing is false; conjunction of
   nothing is true.)
2. After `unit_propagate` returns `Implied(forced)`, what does a second call
   return, and why? (`Implied(vec![])` — a fixed point forces nothing;
   idempotence.)
3. In the DPLL trace of §5, why must backtracking undo `x2` and `x3` and not
   just `x1`? (They were *forced by* the guess `x1 = true`; a different guess
   invalidates them.)
4. `PHP(n+1, n)` is "obviously" false. Why does that not make it easy for DPLL?
   (Haken: every resolution refutation is exponentially large.)
5. The pairwise at-most-one encoding uses `C(n,2)` clauses. For `n = 1000`
   variables in a row of a Sudoku, how many clauses is that, and why might you
   prefer a sequential encoding? (≈ 500 000; `O(n)` vs `O(n²)`.)

## 10. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 fifteen minutes to an
hour · 30 hours · 40 term project · 50 open research problem. An arrow ▶ marks
especially instructive exercises. Log your attempts in
`course/module-10-sat/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 7.2.2.2-1 | 10 | Show that the empty clause is unsatisfiable and the empty formula is valid; relate to `evaluate`. |
| 7.2.2.2-4 | 15 | Prove the unit-clause rule preserves the set of satisfying assignments. |
| ▶7.2.2.2-13 | 22 | Verify `W(3,3) = 9` by exhibiting an 8-element colouring and arguing no 9-element one exists. |
| 7.2.2.2-17 | 20 | Give the pairwise at-most-one encoding and count its clauses; compare with the sequential encoding's variable/clause counts. |
| ▶7.2.2.2-99 | 30 | Prove DPLL is sound and complete (the argument sketched in §5). |
| 7.2.2.2-176 | 40 | Read Haken's exponential lower bound for pigeonhole resolution and reproduce its outline. |

(Numbers are indicative; consult Vol. 4B for the exact statements — the section
has over 500 exercises.)

## 11. Where this leads

- **CDCL / Algorithm C.** Conflict analysis, clause learning, and
  non-chronological backjumping turn the DPLL skeleton you built into a
  state-of-the-art solver. Watched literals (§4) are the data structure that
  makes it fly.
- **Encodings everywhere.** Once you can encode into SAT, planning, scheduling,
  hardware verification, and puzzle-solving are all one `solve` call away — the
  practical face of Cook–Levin.
- **Proof complexity.** Haken's theorem is the doorway to a rich theory of *why*
  certain formulas are hard, connecting SAT solving to the deepest questions in
  complexity — ultimately P vs NP.
