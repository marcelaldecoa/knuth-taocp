# Module 14 — Conflict-Driven Clause Learning

> **Source:** *The Art of Computer Programming*, Vol. 4B, §7.2.2.2
> (*Satisfiability*), Algorithm C.
> **Lab:** `labs/module-14-cdcl` · **Grade it:** `./grade 14`
>
> This lesson is self-contained: you can complete the module without the
> book, though it leans on Module 10 (CNF, unit propagation, DPLL). This is
> the **course capstone**: by the end you will have built the algorithm that
> powers essentially every industrial SAT solver of the last twenty-five
> years.

Module 10 ended with a confession: your DPLL solver *thrashes*. It rediscovers
the same dead end thousands of times, in thousands of disguises. **Conflict-
driven clause learning** (CDCL) is the repair, and it is one of the great
algorithmic stories of our time: three ideas — *learn a clause from every
conflict*, *jump back non-chronologically*, and *propagate lazily with watched
literals* — turned a 1962 algorithm into a tool that settles formulas with
millions of variables. Knuth presents it as Algorithm 7.2.2.2C and calls the
CDCL solvers' success "a technological miracle." This module builds the
miracle from parts, and proves it correct as we go.

---

## 1. From DPLL to CDCL: don't just backtrack — learn

Recall Algorithm D (Module 10). It maintains a partial assignment, propagates
unit clauses, and on a conflict undoes the *most recent* guess and tries the
other value. Two structural weaknesses:

1. **Amnesia.** When a branch dies, DPLL learns nothing. If guesses
   `x1 = true, ..., x9 = true` are irrelevant and the *real* contradiction is
   between $x_{10}$ and $x_{37}$, DPLL will re-derive that contradiction inside
   every one of the $2^9$ subtrees over `x1..x9`.
2. **Chronological backtracking.** DPLL always returns to the most recent
   decision, even when the conflict's actual causes lie many levels lower.

CDCL fixes both with a single mechanism. When propagation hits a conflict,
the solver *analyzes* it: which assignments, exactly, caused this clause to
die? The analysis produces a **learned clause** — a new clause, implied by
the formula, that summarizes the cause and is added to the clause database.
The same analysis reveals the **backjump level**: the deepest decision level
that actually matters. Everything above it is abandoned in one stroke —
subtrees that DPLL would enumerate are skipped because the learned clause
now *propagates* where DPLL would have to *guess*.

The slogan to remember: **clause learning is lazily discovered resolution.**
The learned clause is a resolution consequence of the input (we prove this in
§2) that a resolution prover would have had to find by blind search; CDCL
finds it exactly when, and because, the search needed it.

## 2. Resolution, the logic under the hood

**Definition.** Let $C \lor x$ and $C' \lor \lnot x$ be clauses ($C, C'$ clauses not
containing $x$ or $\lnot x$). Their **resolvent** on $x$ is $C \lor C'$.

**Lemma (soundness of resolution).** Every assignment satisfying both
$C \lor x$ and $C' \lor \lnot x$ satisfies $C \lor C'$.

*Proof.* Fix such an assignment. If it makes $x$ true, then $C' \lor \lnot x$ forces
some literal of $C'$ true. If $x$ is false, $C \lor x$ forces some literal of $C$ true.
Either way $C \lor C'$ has a true literal. ∎

**Corollary (learned clauses are implied).** Conflict analysis (§4) produces
its clause by a chain $R_0, R_1, \ldots, R_t$ where $R_0$ is a clause of the database and
each $R_{i+1}$ is a resolvent of $R_i$ with a database clause. By the lemma and
induction on $i$, every model of the database satisfies $R_t$. Since database
clauses are input clauses or earlier learned clauses, a second induction (on
the order clauses were learned) gives: **every learned clause is implied by
the input formula.** ∎

This corollary is why you can trust a CDCL solver's work — and stage 3 makes
you *check* it: the test oracle `brute_force_implies` verifies each learned
clause semantically against the input formula.

## 3. The trail and the implication graph, formally

CDCL replaces recursion with an explicit log. The **trail** is the sequence
of all literals currently true, in the order they became true. Each literal
is either

- a **decision** — a guess, opening a new **decision level**; the d-th
  decision starts level $d$ (level 0 holds what follows from the formula with
  no guesses at all); or
- an **implication** — a literal forced by unit propagation; it is tagged
  with its **reason**, the clause that was unit.

Formally, if literal $\ell$ was forced by reason clause $R$, then $\ell \in R$ and every
other literal of $R$ is false on the trail — that is *what it means* for $R$ to
have been unit. This local fact is the whole soundness interface between the
propagator (stage 1) and the analyzer (stage 3).

**Definition (implication graph).** Given a trail, form a DAG whose vertices
are the trail's literal assignments (write `x4@3` for "x4 true, assigned at
level 3"). For each implied literal $\ell$ with reason $R$, draw an edge from $\lnot m$ to
$\ell$ for every literal $m \in R \setminus \{\ell\}$ (each $m$ is false, so $\lnot m$ is an assignment on
the trail). Decisions have no incoming edges. When a clause $K$ dies — all its
literals false — add a **conflict vertex** $\Lambda$ with an edge from $\lnot m$ for every
$m \in K$.

The trail plus the reason tags *is* this graph, stored as one flat array —
no pointers, no allocation. That is why stage 2's `Trail` has `level_of`,
`reason_of`, and nothing fancier: Knuth's data structures here are arrays
with a discipline, exactly in the spirit of the MIX-era chapters.

## 4. UIPs, and learning the first-UIP clause

Fix a conflict at decision level $d$, with decision literal $\ell_d$.

**Definition (UIP).** A vertex $u$ at level $d$ is a **unique implication
point** if every path from $\ell_d$ to $\Lambda$ passes through $u$. (In graph terms: $u$
dominates $\Lambda$ relative to $\ell_d$, considering only level-$d$ vertices.)

UIPs always exist: $\ell_d$ itself is one (every level-$d$ implication descends
from it). There may be several; order them along the trail. The **first
UIP** is the one closest to the conflict.

### The cut, and why "asserting" matters

Any UIP $u$ defines a *cut* of the graph: on one side $u$ and everything after
it (the "conflict side"), on the other everything else. The clause read off
the cut — the negations of the assignments with an edge crossing into the
conflict side — contains **exactly one level-$d$ literal, $\lnot u$**. Such a clause
is called **asserting**, and assertingness is the payoff:

after backjumping to level $b$ = (highest level among the clause's other
literals), those other literals are still false, $u$ is unassigned, and the
learned clause has become **unit**. It immediately *forces* $\lnot u$ at level $b$.
The solver does not re-guess; it is pushed forward. (With two or more
level-$d$ literals the clause would arrive at level $b$ with two unassigned
literals — silent, useless.)

### Why the *first* UIP

Conflict analysis computes the first-UIP clause by resolution, walking the
trail backwards:

```text
A1. [Start.]    R := the conflict clause. (Every literal of R is false.)
A2. [UIP?]      If exactly one literal of R was assigned at level d, stop:
                R is the learned clause, that literal is ¬(first UIP).
A3. [Resolve.]  Let p = the most recently assigned level-d literal with
                ¬p ∈ R (scan the trail backwards to find it). Replace R by
                the resolvent of R and reason(p) on p's variable. Return
                to A2.
```

Three facts, worth proving before you code stage 3:

- **A3 is always legal.** While $R$ has $\ge 2$ level-$d$ literals, the latest of
  them cannot be the decision (the decision is the *earliest* level-$d$
  assignment), so it has a reason, and that reason contains $p$ while $R$
  contains $\lnot p$ — a valid resolution.
- **Termination and correctness.** Each A3 step removes $\lnot p$ and adds only
  literals assigned *earlier* on the trail (a reason's other literals were
  already false when $p$ was forced). So the "latest level-$d$ literal of $R$"
  moves strictly backwards, and A2 must trigger at the decision at the
  latest. The vertex it stops at is a UIP — indeed the first: every path
  from $\ell_d$ to $\Lambda$ must cross the shrinking level-$d$ frontier that $R$ tracks —
  and $R$ is exactly the first-UIP cut clause.
- **First UIP minimizes the backjump level.** Continuing to resolve past
  the first UIP never *removes* lower-level literals — resolution on
  level-$d$ variables only adds side literals. So among all UIP cuts, the
  first UIP's clause has the smallest set of lower-level literals, hence
  the lowest (or equal) second-highest level, hence the deepest backjump —
  and it is the shortest such clause too. Stopping early is not laziness;
  it is optimal within the chain.

One more rule: literals assigned at **level 0** are dropped from $R$
entirely. They are false in every future state of the search (level 0 is
never undone), and since the formula implies them false, dropping them
preserves implication: if $F \models C \lor m$ and $F \models \lnot m$ then $F \models C$.

### A worked example: eight variables, three levels

This is the lesson's centerpiece — and it is **machine-checked**: the same
scenario, literal for literal, is the test `eight_variable_worked_example`
in `tests/stage_03_clause_learning.rs` and `first_uip_worked_example` in the
reference. The clause database:

```text
C0 = (¬x1 ∨ x2)         C3 = (¬x4 ∨ x5 ∨ x8)
C1 = (¬x1 ∨ x3 ∨ x7)    C4 = (¬x4 ∨ x6)
C2 = (¬x2 ∨ ¬x3 ∨ x4)   C5 = (¬x5 ∨ ¬x6)
```

Decisions: $\lnot x_7$ at level 1, $\lnot x_8$ at level 2, $x_1$ at level 3. Propagation at
level 3 then runs C0, C1, C2, C3, C4 in turn and crashes into C5:

| t | literal | level | reason |
|---|---------|-------|--------|
| 0 | $\lnot x_7$ | 1 | (decision) |
| 1 | $\lnot x_8$ | 2 | (decision) |
| 2 | $x_1$ | 3 | (decision) |
| 3 | $x_2$ | 3 | C0 |
| 4 | $x_3$ | 3 | C1 |
| 5 | $x_4$ | 3 | C2 |
| 6 | $x_5$ | 3 | C3 |
| 7 | $x_6$ | 3 | C4 |
|   | $\Lambda$ | | C5 all false — conflict |

The implication graph (`@d` = assigned at level $d$; $x_1$, $\lnot x_7$, $\lnot x_8$ are the
decisions):

```text
 x1@3 ────C0────> x2@3 ───C2───┐
   │                           ├──> x4@3 ───C4───> x6@3 ──C5──┐
   └──────C1────> x3@3 ───C2───┘      │                       ├──> Λ
 ¬x7@1 ───C1───────^                  └───C3───> x5@3 ───C5───┘
                                ¬x8@2 ───C3────────^
```

Every path from the decision `x1@3` to $\Lambda$ passes through `x4@3` — and through
nothing else at level 3 beyond it. So $x_4$ is the **first UIP** ($x_1$, the
decision, is the *last*). Now run A1–A3, resolving backwards along the trail
(on $x_6$, then $x_5$):

```text
R = C5                        = (¬x5 ∨ ¬x6)      two level-3 literals
R = resolve(R, C4, on x6)     = (¬x4 ∨ ¬x5)      still two
R = resolve(R, C3, on x5)     = (¬x4 ∨ x8)       one level-3 literal: stop
```

Learned clause **C6** $= (\lnot x_4 \lor x_8)$; its other literal $x_8$ lives at level 2,
so the solver **backjumps to level 2** — the level-3 suffix `x1..x6`
disappears wholesale, and level 2's trail grows by the forced literal:
`¬x8@2 (decision), ¬x4@2 (reason C6)`. No re-guessing of $x_1$ happened, and
none was needed: with $\lnot x_4$ asserted, the old dead end is unreachable.

Had we kept resolving to the *last* UIP (through C2, C1, C0) we would have
learned $(\lnot x_1 \lor x_7 \lor x_8)$ — a longer clause asserting the same level. Trace
it yourself; it is the check-your-understanding question 2.

## 5. Two watched literals: propagation that never looks back

Unit propagation eats most of a SAT solver's cycles, so Algorithm C refuses
to touch a clause until it might matter. The device — introduced by the
Chaff solver (2001), adopted by Knuth as the heart of Algorithm C's inner
loop — is the **two watched literals** scheme of stage 1.

Each clause of length $\ge 2$ *watches* two of its literals (keep them in
slots 0 and 1). Per literal $\ell$ we keep the **watch list**: the clauses
currently watching $\ell$.

**Invariant W.** At every propagation fixed point, for every clause: if a
watched literal is false, then the other watched literal is true, or the
clause has at most one non-false literal (it is unit or conflicting).

Contrapositive: a clause whose two watches are both non-false is neither
unit nor false — it can safely be *ignored*. So when literal $p$ becomes true,
only the clauses watching $\lnot p$ are examined; for each, in order:

```text
P1. [Satisfied?]  Other watch true → keep the watch, next clause.
P2. [Move.]       Some unwatched literal is non-false → swap it into the
                  watch slot, move the clause to that literal's watch list.
                  (The clause again watches two non-false literals.)
P3. [Unit?]       Every literal but the other watch is false, other watch
                  unassigned → the clause is unit: force the other watch,
                  log (literal, clause) as an implication.
P4. [Conflict.]   Other watch false too → the clause is falsified: report it.
```

Process forced literals **in assignment order** (a FIFO queue — in Knuth's
formulation, a pointer $G$ chasing the trail). The stage tests pin this order
down: it is the canonical one, and it makes conflicts deterministic.

**Theorem (backtracking is free).** If Invariant W holds and variables are
unassigned in LIFO order (a suffix of the trail — exactly what `backjump`
produces), Invariant W still holds. No watch list needs repair.

*Proof sketch.* Unassignment turns values into `None`; a violation would
need a *false* watch $w$ whose partner $w'$ *was* true and became unassigned
while $w$ stayed false. When propagation last examined $w$ and kept it false
(case P1), $w'$ was true, and both $w$'s falsification and $w'$'s assignment had
happened at levels $\le$ the level current at that examination — with
$\operatorname{level}(w') \le \operatorname{level}(w\text{-falsified})$: within a batch both sit at the same level,
across batches the satisfying $w'$ was assigned no later. Backjumping removes
whole levels from the top, so it cannot unassign $w'$ while keeping $\lnot w$: if
$\operatorname{level}(w')$ is popped, then $\operatorname{level}(\lnot w) \ge \operatorname{level}(w')$ is popped too, and the watch
is no longer false at all. Cases P3/P4 leave the clause unit/conflicting,
which Invariant W tolerates and the driver resolves immediately. ∎

**Amortized cost sketch.** Between two conflicts, the work of `propagate` is
(number of clauses examined) = (watch moves) + (kept watches). Every kept
watch is charged to case P1/P3/P4 — $O(1)$ each, at most once per (clause,
assignment). Every watch move strictly advances that clause's watch along
its non-false literals for the current trail prefix. And backtracking is
$O(\text{popped literals})$, touching no clause at all. Compare Module 10's
propagator, which rescans *every* clause after *every* assignment — and
which had to *undo* nothing only because it kept no per-clause state. Here
we keep state and *still* undo nothing; that asymmetry (pay on the way down,
free on the way up) is what makes millions of backtracks per second
possible. The price: `ClauseDb::check_watch_invariant` exists because a
subtly wrong P2 produces a solver that is merely *slow and wrong quietly* —
it misses units and conflicts. Implement the checker honestly; the tests
call it relentlessly.

## 6. Algorithm C, simplified: the CDCL loop

Everything assembles into stage 4's driver. (This is Knuth's Algorithm
7.2.2.2C with its production refinements stripped — see the scope note in
§8.)

```text
C1. [Initialize.]  Load the clauses into the ClauseDb (merge duplicate
                   literals, drop tautologies); empty trail, level 0.
C2. [Propagate.]   Run watched propagation to a fixed point; move each
                   implication onto the trail with its reason.
C3. [Conflict?]    No conflict → go to C5.
C4. [Learn.]       Conflict at level 0 → terminate, UNSAT. Otherwise
                   (learned, b) := analyze(conflict); backjump the trail to
                   level b, unassigning popped variables in LIFO order;
                   add the learned clause — asserting, so it immediately
                   forces its literal at level b. Return to C2.
C5. [Done?]        All variables assigned → return the assignment: a model.
C6. [Decide.]      Choose an unassigned variable and a polarity by the
                   heuristic (§8); open a new level; assign. Return to C2.
```

Why is the C5 answer a model? Because watched propagation misses no
falsified clause: a clause dies only when its last non-false literal is
falsified, that literal was watched or the falsified watch was examined
(Invariant W), and the driver propagates every assignment before deciding
again. All variables assigned + no conflict reported = every clause has a
true literal. The stage-4 tests refuse to take this on faith: every SAT
verdict is re-checked by evaluating the formula under the returned model.

## 7. Termination and completeness

Neither is obvious: CDCL has no recursion tree, and backjumping *throws
away* work. The classical argument (Knuth's exercise territory; also
Marques-Silva & Sakallah 1999):

**Theorem.** Algorithm C terminates, answering SAT iff the formula is
satisfiable.

*Proof sketch.* For the state after each C4, consider the vector
$\sigma = (n_0, n_1, \ldots)$ where $n_l$ = number of trail literals at level $l$ (pad with
zeros). Claim: each learning step increases $\sigma$ **lexicographically**. The
backjump to level $b$ leaves $n_0 \ldots n_{b-1}$ as they were and then the asserting
literal is forced *at level $b$*, so $n_b$ grows by at least one; higher
components — wiped to zero — are majorized by the growth at position $b$.
Could the asserting literal already have been assigned at some level $\le b$?
No: it was assigned at the conflict level $d > b$ (its negation was the UIP),
so the backjump unassigned it; it is genuinely new at level $b$.

Each $n_l$ is bounded by $n$ (the variable count) and at most $n + 1$ components
can be nonzero, so a lexicographically strictly increasing sequence of such
vectors is finite: **finitely many conflicts**. Between conflicts, every C2/
C5/C6 pass grows the trail, which is bounded by $n$ — so the whole run is
finite.

Partial correctness: a **SAT** answer is verified by §6's argument. An
**UNSAT** answer happens only on a level-0 conflict. Everything at level 0
is either a consequence of unit clauses of F or of learned clauses, all
implied by F (§2 corollary, plus induction along level 0's propagation
order); a clause with all literals false at level 0 therefore witnesses that
$F$ implies a contradiction — $F$ is unsatisfiable. Equivalently: splice the
reason chain of the level-0 conflict into the learned-clause resolutions and
you have a *resolution refutation* of F. CDCL is, literally, a resolution
proof search engine. ∎

Note what the proof did *not* use: the decision heuristic. Any complete
choice of decisions works — heuristics affect speed, never correctness.
That separation is a design principle worth stealing.

## 8. Heuristics: where the magic speed lives

Correct CDCL is the skeleton; these three (all in Knuth's Algorithm C in
refined forms) are the muscles. Stage 4 asks you to implement *some*
deterministic heuristic and document it; the reference implements the first
and third.

- **VSIDS** (Variable State Independent Decaying Sum, from Chaff; Knuth's
  `ACT` array with damping factor $\rho$). Every variable has an *activity*.
  When a clause is learned, bump the activity of its variables; periodically
  *decay* all activities (Knuth multiplies by $\rho \approx 0.95$; the reference, being
  integer-only, halves everything every 128 conflicts). Decide on the
  unassigned variable of maximal activity. Effect: the search clusters
  around the variables involved in *recent* conflicts, digging into one
  contradiction instead of wandering. It costs $O(\text{learned clause})$ per
  conflict plus a scan (production solvers keep a priority heap).
- **Phase saving** (Knuth's `OVAL` polarities). When a variable is
  unassigned by backjumping, remember its last polarity; when it is next
  decided, reuse that polarity. Backjumps stop *un-doing* satisfied
  subproblems: the solver returns to where it was working with its partial
  solution intact.
- **Restarts.** Every so often (Knuth uses the "reluctant doubling"
  sequence of Luby et al.), throw the whole trail away — backjump to level
  0 — and start deciding afresh. This looks insane until you notice what is
  *kept*: the learned clauses and the activities. A restart is not
  forgetting; it is re-deciding in the light of everything learned, escaping
  an early bad decision prefix that no backjump will ever revisit (heavy-
  tailed runtime distributions are the formal story). Restarts complicate
  the §7 termination proof — production solvers space them out so progress
  measures still apply, or just accept probabilistic guarantees; our lab
  omits them, so §7 applies verbatim.

And a limit to the magic, from Module 10's other direction: learned clauses
are resolution consequences, so a CDCL run refuting F yields a resolution
refutation of F no shorter than the run. **Haken's theorem** (every
resolution refutation of $\text{PHP}(n+1, n)$ is exponential in $n$) therefore lower-
bounds *every* CDCL solver on pigeonhole, VSIDS or not. Stage 4 makes you
feel this: `php(7,6)` is dispatched quickly at this size, but the growth is
merciless — try `php(9,8)` and watch. Symmetry breaking or stronger proof
systems (Module 13's BDDs count models of PHP effortlessly!), not better
heuristics, are the way around that wall.

**Honest scope note (what Knuth's full Algorithm C has that ours doesn't).**
Knuth's C interleaves: restarts with reluctant doubling; **clause purging**
(learned clauses are ranked — by literal block distance among other things —
and the database is periodically halved, essential at industrial scale);
lazy *literal-block* data structure details (`MEM`, the `2k+sign` literal
coding, blocked clause lists); on-the-fly subsumption; and a trail-repair
optimization for chronological details we simplified. Nothing in it changes
the four ideas you implement — ClauseDb, Trail, first-UIP, driver — which
are the load-bearing walls. Read §7.2.2.2's Algorithm C after this module
and every exotic step will slot into a place you already built.

---

## Why it's done this way

- **Learning over re-deriving:** the same clause that explains one conflict
  *prevents* its whole equivalence class of restagings; that is the entire
  gap between Module 10's DPLL and this module. It is the memoization idea
  in its most hostile environment (exponential state space, no obvious keys).
- **First UIP, not last:** asserting is non-negotiable (the clause must
  propagate right after the backjump); *first* is optimal within the
  resolution chain — deepest backjump, shortest clause (§4). Nothing about
  it is heuristic, which is why every serious solver made the same choice.
- **Arrays and a discipline, not a graph library:** the implication graph is
  never materialized; trail + reasons encode it exactly as long as analysis
  only ever walks backwards. Knuth's MIX-era habit — flat arrays, index
  arithmetic, invariants instead of pointers — carries a thoroughly modern algorithm
  unchanged.
- **Laziness with a checkable invariant:** watched literals buy speed by
  *not* looking at clauses; the cost is that bugs hide. The module forces
  the invariant into a first-class, tested function because that is how you
  keep a lazy data structure honest — a lesson that transfers far beyond SAT.
- **Verdicts you can audit:** models are checked by evaluation; learned
  clauses by `brute_force_implies`; UNSAT by brute-force cross-checks and
  known theorems ($W(3,3) = 9$, pigeonhole). Certification over trust, as in
  every module since Bézout coefficients in Module 01.

## In the real world

- **Hardware and software verification.** CDCL is the execution engine of
  bounded model checking (turn "does this circuit/program violate the
  property within k steps?" into CNF, ask a solver), of CBMC and its kin for
  C code, and — through CDCL(T), where a theory solver for arithmetic or
  arrays sits inside the CDCL loop — of SMT solvers like Z3 that back
  program verifiers at Microsoft, AWS (s2n, IAM policy reasoning), and every
  serious chip house. When your CPU shipped, CDCL had argued for its
  correctness.
- **Package dependency resolution.** Installing software is SAT: package =
  variable, "A needs B or C", "B conflicts with D" = clauses. SUSE/openSUSE's
  libsolv (zypper, DNF/Fedora) is literally a SAT solver; Conda's resolver
  is SAT-based; Dart's PubGrub — also adopted by the `uv` Python installer —
  is CDCL under another name: its "incompatibilities" are learned clauses
  and its error messages are printouts of the resolution proof. When a
  resolver tells you *why* your install is impossible, you are reading a
  §2-style certificate.
- **Knuth's SAT13.** Knuth published his own literate CDCL implementation,
  `SAT13`, on his *Programs* page (alongside SAT0..SAT11 for the section's
  other algorithms) — Algorithm C exactly as §7.2.2.2 states it, with
  restarts, purging, and all. Diffing your stage 4 against SAT13 is the
  recommended graduation exercise: the skeleton will be eerily familiar.
- **Competition-driven engineering.** MiniSat (2003) fit CDCL in ~600 lines
  and became the reference implementation of the ideas you just built;
  descendants (Glucose, CaDiCaL, Kissat) win the annual SAT Competition with
  refinements of the same trio: watched literals, first-UIP, VSIDS-family
  heuristics.

## Proof techniques you practiced

- **Soundness by induction along a derivation** (§2): one small semantic
  lemma (resolution) lifted over a chain, then over the history of learned
  clauses — the standard shape of proof-system arguments.
- **Dominator/cut reasoning on DAGs** (§4): UIP existence and the
  assertingness of cut clauses; plus a **monotonicity argument** (side
  literals only grow) to prove first-UIP optimal.
- **Invariant stability under undo** (§5): proving an invariant is preserved
  not by an operation, but by its *reversal* — the key to all "backtrack for
  free" data structures, and a cousin of Module 09's dancing links argument.
- **Termination by lexicographic progress measure** (§7): when no single
  quantity decreases, a well-founded order on state vectors still nails
  termination — the same move as Module 01's decreasing remainders, grown up.
- **Adversarial self-checking** (labs): semantic verification of models,
  brute-force implication oracles for learned clauses, exhaustive small-case
  agreement — certifying a complex artifact with simple, obviously-correct
  ones.

---

## 9. Stage-by-stage lab guide

Open `labs/module-14-cdcl/src/lab.rs`. Run `./grade 14`; stages in order,
first failure stops. Everything is std-only; the LCG in the tests is the
course's usual one.

### Stage 1 — `ClauseDb`: two watched literals

Get the representation right first: watches in clause slots 0 and 1, one
watch list per literal (`code(±v) = 2(v−1) + [v<0]` — Knuth's own literal
coding), values per variable, a FIFO queue of pending true literals, an
implication log. `add_clause` must detect unit and falsified arrivals
(that's how learned clauses assert after a backjump — and how a `[]` clause
poisons the formula). `propagate` is the P1–P4 loop, processing the queue
**in order**. `unassign` must do *nothing* but clear the value — the tests
unassign and re-propagate in new directions to prove your watches survived.
Write `check_watch_invariant` early and honestly; it is your best debugger
here (structural part first: each long clause in exactly the two lists its
slots say).

### Stage 2 — `Trail`: levels, reasons, backjump

Mostly bookkeeping, all of it load-bearing for stage 3: `decide` opens a
level, `enqueue` records an implication at the current level, `level_of` /
`reason_of` answer per-variable queries, `assignments_at_level` slices by
the `level_start` boundaries, and `backjump(b)` pops everything above b and
returns it **newest first** so the caller can `unassign` in LIFO order. The
integrated test drives `ClauseDb` and `Trail` in lockstep, step by step —
exactly the choreography your stage-4 driver will perform.

### Stage 3 — `analyze`: first-UIP learning

Algorithm A1–A3 with the standard implementation trick: a `seen` bit per
variable and a counter of unresolved current-level literals; one index
walking `trail.literals()` backwards. Collect lower-level literals as you
go (skipping level 0), decrement the counter as current-level literals are
consumed, and stop at counter zero — the literal you just passed is the
UIP. Put the asserting literal in slot 0 and a backjump-level literal in
slot 1 (your own `add_clause` will thank you). The tests hand-build
implication graphs (no propagation involved), so your analysis is checked
against exact expected clauses — and against `brute_force_implies`, which
you also write here.

### Stage 4 — `solve`: the driver, and the payoff

Loop C1–C6. The subtle points: enqueue **all** implications (with reasons)
after every propagate, even the one produced by `add_clause`'s assertion;
check for level-0 conflict *before* analyzing; unassign backjumped variables
in the order `backjump` returns them; dedup/detautologize input clauses.
Then pick a heuristic, document it, and enjoy the scale tests: `php(7,6)`
(UNSAT, beyond comfortable DPLL) and a 60-variable random 3-SAT instance
(hopeless for brute force at $2^{60}$, milliseconds for you). Also here:
`solve_brute`, `pigeonhole_cnf`, `waerden_cnf` — module 10 muscle memory.

---

## 10. Check your understanding

1. Why must a learned clause contain *exactly one* literal of the conflict
   level before it is worth backjumping with? (One → unit at the backjump
   level, forces immediately. Two or more → arrives with two unassigned
   literals, silent; the solver would have to rediscover the conflict.)
2. In the §4 example, finish the resolution past the first UIP (through C2,
   then C1, then C0) and confirm you get $(\lnot x_1 \lor x_7 \lor x_8)$. Why does its
   backjump level equal the first-UIP clause's here, and why is it worse
   anyway? (Same second-highest level 2, but longer — and in general the
   level can only get worse, by §4's monotonicity argument.)
3. When $p$ becomes true, why is it enough to examine clauses watching $\lnot p$ —
   what about clauses *containing* $\lnot p$ unwatched? (Invariant W: their two
   watches are non-false, so falsifying an unwatched literal leaves $\ge 2$
   non-false literals — not unit, not conflicting. They can wait.)
4. Exactly where does the termination proof (§7) use assertingness? (The
   forced literal at level $b$ makes $n_b$ grow — without it the lexicographic
   measure could stall, and a solver that backjumps without learning can
   loop forever.)
5. Your solver answers UNSAT. What object *could* it print to let a
   skeptic check that answer without re-running the search? (The resolution
   refutation assembled from the learned clauses' derivations plus the
   final level-0 conflict — in practice, a DRAT proof log; see exercises.)

## 11. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 fifteen minutes
to an hour · 30 hours · 40 term project · 50 open research problem. An
arrow ▶ marks especially instructive exercises. Log your work in
`course/module-14-cdcl/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 7.2.2.2-207 | 20 | Trace Algorithm C on waerden(3,3;9) for a few conflicts: exhibit each learned clause and backjump level. |
| ▶7.2.2.2-233 | 25 | Prove the trail-walking analysis (A1–A3) always stops at the *first* UIP, and that its clause is asserting. |
| 7.2.2.2-236 | 22 | Show unassignment in LIFO order preserves the watched-literal invariant (make §5's sketch rigorous). |
| ▶7.2.2.2-239 | 30 | Prove CDCL terminates without restarts (the lexicographic measure of §7), then show how reluctant-doubling restarts preserve completeness. |
| 7.2.2.2-266 | 28 | On-the-fly subsumption: when a resolvent in A3 is a subset of its reason clause, the reason can be strengthened in place. Work out the details. |
| 7.2.2.2-284 | 40 | Add clause purging: rank learned clauses (activity, literal block distance), halve the database periodically, and measure on php(9,8). |
| — | 35 | Emit a DRAT proof log from your stage-4 solver (one line per learned clause, `d` lines for purged ones) and check it with an external checker. |

(Numbers are indicative — Vol. 4B's §7.2.2.2 has over 500 exercises;
consult the book for exact statements. The last row is this course's own.)

## 12. Where this leads

- **Backwards, through the whole course:** CDCL is Module 09's backtracking
  + Module 10's propagation + learning; its data structures are Module 03's
  arenas; its heuristics are decayed counters (Module 04's arithmetic);
  its hard instances were charted in Modules 10 and 13. The capstone is a
  reunion.
- **Proof complexity.** CDCL $\equiv$ resolution (with restarts, polynomially so —
  Pipatsrisawat–Darwiche/Atserias et al.). Stronger systems (extended
  resolution, cutting planes, algebraic provers) power the next generation
  of solvers; Haken's wall is a property of resolution, not of SAT.
- **CDCL(T) and SMT.** Wrap a theory solver (linear arithmetic, arrays,
  bit-vectors) around the propagation loop and you get Z3/CVC5 — the
  engines of modern program verification. The interface is exactly the
  reason/conflict-clause contract you built in stages 1–3.
- **#SAT and beyond.** Model *counting* and knowledge compilation reuse
  component analysis atop CDCL; Module 13's BDDs are the other classical
  route — Knuth devotes §7.1.4 vs §7.2.2.2 to precisely that comparison.
- **And Knuth himself:** read §7.2.2.2 end to end now — the 300 pages that
  this course has been quietly training you for. Then run `SAT13`.
