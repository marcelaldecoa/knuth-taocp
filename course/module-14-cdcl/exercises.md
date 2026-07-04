# Exercises — Module 14 (Conflict-Driven Clause Learning)

Self-contained problems on this module's material — Algorithm C, the
first-UIP learning rule, the watched-literal invariant, termination, and the
resolution certificate a CDCL solver leaves behind. You can work every one
**without the books**: each states the problem in full, gives a **hint** to
peek at when stuck, and a worked **answer sketch** to check against. Where a
problem asks for a trace or measurement, reproduce it with the solver you
build in the lab.

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` =
especially instructive). Numbers like "cf. 7.2.2.2–207" point at the matching
exercise in Vol. 4B §7.2.2.2 for readers who own it; §7.2.2.2 has over 500
exercises, so the numbers are indicative and the statements here are the ones
to solve.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Trace Algorithm C on $\text{waerden}(3,3;9)$ | 20 | ⬜ |
| 2 | ▶ A1–A3 stops at the *first* UIP; its clause is asserting | 25 | ⬜ |
| 3 | LIFO unassignment preserves the watch invariant | 22 | ⬜ |
| 4 | ▶ Termination without restarts; restarts stay complete | 30 | ⬜ |
| 5 | On-the-fly subsumption | 28 | ⬜ |
| 6 | Clause purging; measure on $\text{php}(9,8)$ — mini-project | 40 | ⬜ |
| 7 | Emit and check a DRAT proof log from stage 4 | 35 | ⬜ |

## Problems

### 1. Trace Algorithm C on $\text{waerden}(3,3;9)$ (rating 20 · cf. 7.2.2.2–207)

**Problem.** The formula $\text{waerden}(3,3;9)$ asks: can the integers
$1,\dots,9$ be 2-colored with no monochromatic 3-term arithmetic progression?
Encode color of element $i$ as a Boolean $x_i$. For each of the $16$ progressions
$(a, a+d, a+2d)$ inside $\{1,\dots,9\}$ — namely $(1,2,3), (1,3,5), (1,4,7),
(1,5,9), \dots, (7,8,9)$ — add two clauses,
$(x_a \lor x_{a+d} \lor x_{a+2d})$ and
$(\lnot x_a \lor \lnot x_{a+d} \lor \lnot x_{a+2d})$, forbidding an all-color-0 or
all-color-1 progression. That is $32$ clauses on $9$ variables. Run
Algorithm C (§6) on it and exhibit, for the first few conflicts, the learned
clause and the backjump level. What verdict does the solver return, and why
must it be that verdict *independent of your decision heuristic*?

**Hint.** You are not asked to predict the exact clauses — those depend on
which variable you decide first (§7: heuristics change speed, never the
answer). You *are* asked to check that every learned clause and backjump obeys
the invariants of §4 and §6, and to say what the final answer is. The van der
Waerden number $W(3,3)$ decides the verdict.

**Answer sketch.** $W(3,3) = 9$: every 2-coloring of $\{1,\dots,9\}$ contains a
monochromatic 3-term progression, while $\{1,\dots,8\}$ still has a good
coloring (brute force confirms the boundary — $8$ satisfiable, $9$ not). So
$\text{waerden}(3,3;9)$ is **unsatisfiable**, and Algorithm C must answer
**UNSAT**. Whatever your heuristic, each trace obeys the same laws:

- every learned clause is **asserting** — exactly one literal at the conflict
  level $d$ (§4), so after backjumping to the second-highest level $b$ of its
  literals it is unit and immediately forces that literal;
- every learned clause is **implied by the formula** (the §2 corollary: it is a
  resolution consequence of the input), so adding it never changes the model
  set;
- the backjump level is the second-highest decision level among the learned
  clause's literals (§4);
- the run ends on a **level-0 conflict**, which by §7 is a resolution
  refutation of the formula — the certificate that the coloring is impossible.

Because the answer is forced by $W(3,3) = 9$ and every learned clause is
sound, two different heuristics produce different clause *sequences* but the
same UNSAT verdict. (Cross-check in the lab: `waerden_cnf(3,3,9)` is UNSAT and
`waerden_cnf(3,3,8)` is SAT; the solver's SAT models are re-verified by
evaluation and its UNSAT verdict by the brute-force cross-check the stage-4
tests apply.)

### 2. ▶ A1–A3 stops at the *first* UIP; its clause is asserting (rating 25 · cf. 7.2.2.2–233)

**Problem.** The conflict-analysis loop A1–A3 (§4) starts with $R$ = the
conflict clause and repeatedly resolves away the most recently assigned
conflict-level literal, stopping when $R$ has exactly one literal at the
conflict level $d$. Prove three things: (a) step A3 is always a legal
resolution while $R$ still has $\ge 2$ conflict-level literals; (b) the loop
terminates; (c) the literal it stops at is the **first UIP** and the resulting
clause is **asserting** (one conflict-level literal). Illustrate on the
eight-variable example of §4, whose database is

$$
C_0 = \lnot x_1 \lor x_2,\quad C_1 = \lnot x_1 \lor x_3 \lor x_7,\quad C_2 = \lnot x_2 \lor \lnot x_3 \lor x_4,
$$

$$
C_3 = \lnot x_4 \lor x_5 \lor x_8,\quad C_4 = \lnot x_4 \lor x_6,\quad C_5 = \lnot x_5 \lor \lnot x_6,
$$

with decisions $\lnot x_7$@1, $\lnot x_8$@2, $x_1$@3, conflict on $C_5$.

**Hint.** For (a): the *latest* conflict-level literal of $R$ cannot be the
decision (the decision is the *earliest* level-$d$ assignment), so it was
forced and has a reason clause containing it; $R$ contains its negation. For
(b): each A3 step deletes $\lnot p$ and can only add literals assigned *earlier*
on the trail. For (c): the level-$d$ frontier of $R$ marches strictly
backwards toward a single dominator.

**Answer sketch.** *(a) Legality.* While $R$ has two or more level-$d$
literals, let $\lnot p$ be the one whose variable was assigned latest. $p$ is not
the decision $\ell_d$ (decisions come first at their level), so $p$ was forced by
a reason clause $\text{reason}(p)$ with $p \in \text{reason}(p)$ and every other
literal of it false and assigned no later than $p$. Since $\lnot p \in R$ and
$p \in \text{reason}(p)$, resolving $R$ with $\text{reason}(p)$ on $p$'s variable is a
valid resolution.

*(b) Termination.* Resolving removes $\lnot p$ from $R$ and adds only literals of
$\text{reason}(p)$ other than $p$ — all assigned *earlier* than $p$. So the
position on the trail of "latest level-$d$ literal of $R$" moves strictly
backwards. It cannot pass the decision $\ell_d$, so within finitely many steps
$R$ has just one level-$d$ literal and A2 halts.

*(c) First UIP and asserting.* At every step $R$ is a clause all of whose
literals are false and which is implied by the database (§2, each resolvent is
sound). When A2 stops, $R$ has exactly one level-$d$ literal $\lnot u$ — that is
the definition of **asserting**. The vertex $u$ dominates the conflict along
the shrinking level-$d$ frontier that $R$ tracks, so it is a UIP, and because
the frontier only ever moved *backwards from the conflict*, $u$ is the UIP
**closest to the conflict** — the first UIP.

*Worked check.* Resolving $C_5 = (\lnot x_5 \lor \lnot x_6)$ against $C_4$ on $x_6$
gives $(\lnot x_4 \lor \lnot x_5)$; resolving that against $C_3$ on $x_5$ gives
$(\lnot x_4 \lor x_8)$ — one level-3 literal ($\lnot x_4$), so we stop. The first UIP
is $x_4$; the learned clause $(\lnot x_4 \lor x_8)$ backjumps to level 2 (where
$x_8$ lives) and forces $\lnot x_4$. Continuing past the first UIP (on $x_4$, $x_3$,
$x_2$) yields the **last-UIP** clause $(\lnot x_1 \lor x_7 \lor x_8)$: same backjump
level 2 but a longer clause — worse, per §4's monotonicity. (Both resolution
chains reproduced exactly by hand-checking each resolvent.)

### 3. LIFO unassignment preserves the watch invariant (rating 22 · cf. 7.2.2.2–236)

**Problem.** Invariant W (§5) says: at every propagation fixed point, for each
clause, if a watched literal is false then either the other watched literal is
true or the clause has at most one non-false literal. Make §5's sketch
rigorous: prove that if Invariant W holds and `backjump` unassigns a **suffix
of the trail** (a set of whole top decision levels, newest first — LIFO
order), then Invariant W still holds afterward, with **no watch list touched**.

**Hint.** Unassigning only turns some values from set to `None`; it never sets
a value. A violation would need a *false* watched literal $w$ whose partner
watched literal $w'$ *was* true and became unassigned while $w$ stayed false.
Compare the decision levels at which $w$ was falsified and $w'$ was satisfied.

**Answer sketch.** Unassignment removes values; it can only *shrink* the set of
false literals. So the only way to break Invariant W is a clause with a false
watch $w$ whose partner $w'$ lost its true value while $w$ remained false. When
propagation last examined this clause and left $w$ false (case P1 — "other
watch true, keep it"), $w'$ was true. Both events — $w$'s falsification and
$w'$'s satisfaction — had happened by that examination, with
$\operatorname{level}(w') \le \operatorname{level}(w)$: within one propagation batch both sit at the
current level, and across batches the satisfying $w'$ was assigned no later
than $w$ was falsified. `backjump(b)` deletes exactly the levels $> b$, a suffix
of the trail. If it unassigns $w'$, then $\operatorname{level}(w') > b$; but
$\operatorname{level}(w) \ge \operatorname{level}(w') > b$, so $w$ is unassigned in the same sweep and is no
longer false — no violation. If $w'$ survives, the clause is unchanged. Either
way Invariant W holds, and since no watched literal *moved*, no watch list
needs repair. The LIFO/whole-levels shape is essential: it guarantees "pop
$w'$'s level $\Rightarrow$ pop $w$'s level too." $\blacksquare$ (This is the "invariant stable
under undo" pattern — the reason backtracking is free.)

### 4. ▶ Termination without restarts, and completeness with them (rating 30 · cf. 7.2.2.2–239)

**Problem.** (a) Prove Algorithm C (§6, no restarts) terminates. (b) Then argue
that adding restarts — periodically backjumping to level 0 while *keeping* the
learned clauses and activities — preserves both termination in practice and
completeness (it still answers SAT iff the formula is satisfiable).

**Hint.** For (a) use the state vector $\sigma = (n_0, n_1, n_2, \dots)$ where
$n_l$ is the number of trail literals at level $l$, and show each learning step
increases $\sigma$ **lexicographically**; then bound how many such vectors
exist. For (b): what makes a SAT answer trustworthy, and what makes an UNSAT
answer trustworthy, and does a restart touch either guarantee?

**Answer sketch.** *(a) Termination.* Take the state right after each learning
step C4 and form $\sigma = (n_0, n_1, \dots)$. Backjumping to level $b$ leaves
$n_0, \dots, n_{b-1}$ unchanged, wipes all higher components to zero, and then
the asserting clause **forces one new literal at level $b$**, so $n_b$ grows by
at least one. The wiped-out higher components are majorized by that growth at
position $b$, so $\sigma$ strictly increases lexicographically. The asserting
literal is genuinely new at level $b$: it was assigned at the conflict level
$d > b$ (its negation was the UIP) and the backjump unassigned it. Each $n_l \le
n$ (the variable count) and at most $n+1$ components are nonzero, so a
strictly lexicographically increasing sequence of such bounded vectors is
finite — **finitely many conflicts**. Between conflicts every C2/C5/C6 pass
grows the trail, bounded by $n$. Hence the whole run is finite.

*(b) Restarts.* Correctness never depended on the decision heuristic (§7), and
a restart is just "decide differently from level 0." A **SAT** answer is
trusted because C5 fires only when all variables are assigned with no conflict,
which — by watched propagation missing no falsified clause (§6) — means every
clause has a true literal; a restart cannot manufacture a false SAT. An
**UNSAT** answer fires only on a level-0 conflict, which is a resolution
refutation of $F$ (§7) and thus sound; restarts keep the learned clauses, so
they keep every refutation fragment. Completeness therefore survives. The one
subtlety is termination: unrestricted restarts could loop, so production
solvers space restarts out (reluctant-doubling / Luby, §8) far enough that the
progress measure of (a) still applies between restarts, or accept a
probabilistic guarantee. The learned clauses and activities carried across the
restart are what turn "re-decide" into genuine progress, not amnesia.

### 5. On-the-fly subsumption (rating 28 · cf. 7.2.2.2–266)

**Problem.** During conflict analysis (A1–A3, §4), suppose the resolvent
produced at some A3 step is a **subset** of the reason clause it was just
resolved against — i.e. the resolvent $R'$ satisfies $R' \subseteq \text{reason}(p)$
after removing the resolved variable. Show that $\text{reason}(p)$ is then
**subsumed**: it can be replaced in the database by the shorter clause $R'$
without changing the set of models, and argue why doing it *on the fly*, mid
analysis, is sound.

**Hint.** A clause $A$ **subsumes** a clause $B$ when $A \subseteq B$ as literal
sets. What does every model of $A$ do to $B$? And recall the §2 corollary:
every clause A3 handles — inputs, earlier learned clauses, and each resolvent
— is implied by the input formula $F$.

**Answer sketch.** If $A \subseteq B$ then any assignment satisfying $A$ makes one
of $A$'s literals true, and that literal is also in $B$, so it satisfies $B$
too: $A$ **subsumes** $B$, and $B$ is redundant in the presence of $A$. Here
$A = R'$ and $B = \text{reason}(p)$. By the §2 corollary $R'$ is implied by $F$
(it is a resolvent along the analysis chain), so it may be added to the
database soundly; and because $R' \subseteq \text{reason}(p)$, the clause
$\text{reason}(p)$ is subsumed and may be deleted. Replacing $B$ by the strictly
shorter $A$ therefore preserves the model set — the database still means the
same formula, and every future propagation over $A$ is at least as strong as
over $B$ (fewer literals to falsify before it becomes unit). Doing it during
analysis is sound because A3 has already established the resolution step that
produces $R'$; the strengthening uses no information the analyzer did not
already compute, so it is free work. (One care: only the *reason* clause is
replaced; the trail's reason pointers that referenced $B$ now reference the
subsuming clause $A$, which still contains the forced literal, so propagation
soundness — §3's "$\ell \in R$, all others false" contract — is preserved.)

### 6. Clause purging; measure on $\text{php}(9,8)$ (rating 40 · cf. 7.2.2.2–284, mini-project)

**Problem.** Learned clauses accumulate without bound. Add **clause purging**
to your stage-4 solver: rank the learned clauses by a quality measure, and
periodically delete the worst half of them. A standard measure is **literal
block distance (LBD)** — the number of distinct decision levels among a
clause's literals — with activity as a tiebreaker; keep every clause currently
serving as a reason on the trail. Measure the effect on the pigeonhole formula
$\text{php}(9,8)$: 9 pigeons, 8 holes, variable $x_{ij}$ = "pigeon $i$ in hole
$j$", with $9$ at-least-one clauses (width $8$) and $8 \cdot \binom{9}{2} = 288$
at-most-one binary clauses — $72$ variables, $297$ input clauses, and provably
UNSAT. Report solve time, conflicts, and peak clause-database size with and
without purging.

**Hint.** Never delete a clause that is the reason of a literal currently on the
trail — that would corrupt the implication graph. Purge only at safe points
(e.g. right after a restart or at level 0). For the measurement, expect the
pigeonhole growth to be brutal: by Haken's theorem (§8) *every* resolution
refutation of $\text{php}(n{+}1, n)$ is exponential in $n$, and a CDCL run yields
a resolution refutation no shorter than itself.

**Answer sketch.** *Design.* Store an LBD with each learned clause (compute it
when the clause is learned: count distinct `level_of` values among its
literals). At a purge point, protect (i) clauses of LBD $\le 2$ and (ii) any
clause that is a current reason; sort the rest by (LBD, then activity) and drop
the worst half. This mirrors §8's scope note — the production Algorithm C
"ranks learned clauses… and the database is periodically halved."

*What the measurement shows.* Purging **lowers peak memory** substantially
(the database no longer grows monotonically) at the cost of possibly re-deriving
some deleted clauses, so **conflict counts rise modestly** while **peak
database size falls sharply**; net wall-clock is usually a wash or a win at
this size, and a large win at industrial scale where memory, not conflicts, is
the binding constraint. But the deeper lesson is the *wall*: $\text{php}(9,8)$
is UNSAT with only $297$ input clauses, yet the refutation is exponentially
long. No heuristic and no purging policy escapes it — Haken's lower bound is a
property of **resolution**, hence of every CDCL solver (§8). Compare
$\text{php}(7,6)$ ($42$ variables, $133$ clauses), dispatched quickly, against
$\text{php}(9,8)$: same shape, merciless growth. The way around is a stronger
proof system or symmetry breaking (Module 13's BDDs count PHP models
effortlessly), not a cleverer clause database. Report your numbers as a table
of (formula, purge on/off) $\times$ (conflicts, peak clauses, time), and let the
exponential speak.

### 7. Emit and check a DRAT proof log from stage 4 (rating 35 · this course's own)

**Problem.** When your solver answers UNSAT, it should be able to hand a
skeptic an object that certifies the answer *without re-running the search*.
Extend stage 4 to emit a **DRAT proof log**: one line per learned clause (the
literals of the clause, in DIMACS integer form, terminated by `0`), plus a `d`
("delete") line whenever a clause is purged (exercise 6). Then check the log
with an external DRAT checker (e.g. `drat-trim`) against the original CNF.
Explain why the sequence of learned clauses *is* a valid proof, and what the
checker verifies at each line.

**Hint.** §7 shows an UNSAT run splices into a **resolution refutation**: the
learned clauses' derivations, chained with the final level-0 conflict, derive
the empty clause. What weaker, purely local property must each logged clause
have for a DRAT checker to accept it — and why does every first-UIP learned
clause have it?

**Answer sketch.** Each learned clause is **implied by the formula** (§2
corollary) — indeed it is a resolution consequence — so appending it preserves
unsatisfiability, and the final level-0 conflict is the empty clause. That
chain of "add an implied clause" steps ending in the empty clause is exactly a
refutation. DRAT checks a *local* condition per added line: the **RAT
property** (Resolution Asymmetric Tautology), which every clause derivable by
propagation-and-resolution — including every first-UIP clause — satisfies. For
each line the checker verifies that adding the negation of the clause's
literals as units and running unit propagation on the current database yields a
conflict (the RUP special case), i.e. the clause is redundant; `d` lines let it
drop clauses so the check stays fast. When the final empty clause is reached
and every intermediate line passed, the checker has an *independent*,
polynomial-time confirmation that the formula is UNSAT — the auditable
certificate §7 promised, and precisely what SAT-competition solvers are
required to output. (Sanity: emit the log for $\text{php}(7,6)$, a compact
UNSAT instance, and confirm `drat-trim` reports the proof verified.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
