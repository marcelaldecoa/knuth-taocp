# Walkthrough — Module 23 (read after your stage is green)

Design commentary on the reference implementation
(`reference/src/m23_csp.rs`). Not needed to pass — only to deepen. Each
section is the "compare with Knuth's answer" step made explicit.

## Stage 1: the model, and basic backtracking

**Normalize once, then binary-search forever.** `Csp::new` sorts and dedups
every domain; `add_allowed` sorts and dedups every pair list. That single
decision pays four times over: `check` and `prefix_consistent` test pairs by
`binary_search`; "values in ascending order" is automatically well-defined
(which is what makes the node counts pinnable); `solve_basic` emits
solutions in lexicographic order *by construction*, no sort needed; and
stage 4's encoder can enumerate in-domain pairs in a deterministic order.
Normalizing at the boundary instead of defensively re-checking everywhere is
the same taste as Module 09's "build the structure once, then dance on it."

**The panics are API design, not paranoia.** `add_allowed` rejects `x == y`
(message contains `"distinct"`) with a pointer to the right encoding — a
unary restriction is a domain shrink, not a self-constraint — and rejects
out-of-range variables (`"range"`); `check` rejects a wrong-length
assignment (`"length"`). The grader tests these substrings because a
contract you don't enforce is a contract you don't have; a self-constraint
silently treated as satisfiable would poison every count in the module.

**Where the node counter sits.** `basic_step` increments `nodes` *between*
`push` and the consistency test — a placement is counted whether or not it
survives. Put the increment after the test and queens-5 comes out below 220;
put it in the recursive call and you miss the leaf placements. The counter's
position is part of the algorithm's definition (§7.2.2's cost measure), not
an implementation detail — which is exactly why the tests can pin 220.

**`prefix_consistent` checks less than you'd think.** It tests only
constraints with *both* endpoints $\le l$ — and thanks to the invariant
("the prefix below $l$ was already consistent"), only those *involving* $l$
can newly fail. The reference still loops over all constraints for clarity;
an index from variable to incident constraints is the first optimization
you'd add, and stage 2 effectively builds that access pattern anyway.

## Stage 2: forward checking and MRV

**One engine, one boolean.** `solve_fc` and `solve_fc_mrv` share
`lookahead_step`; the `mrv` flag only changes the variable-selection scan
(first unassigned index vs. smallest live domain, with strict `<` so ties
stay at the lowest index). Sharing the engine means the filtering and undo
logic — the part that can harbor subtle bugs — exists once and is exercised
by both test batteries. The MRV path sorts its solutions before returning
because its emission order is no longer lexicographic; the FC path doesn't
need to, and doesn't.

**The save list, and why not dancing cells.** Placing `x = a` partitions
each unassigned neighbor's live domain into keepers and dropped values
(`std::mem::take` + `partition` keeps the borrow checker happy while a
constraint borrow is alive), and logs `(variable, dropped)` per touched
neighbor. Undo walks the log in reverse, merges each batch back, and
re-sorts that domain. This is deliberately the *plain-Vec* version of
§7.2.2.3's *dancing cells*: a sparse set would make the undo an $O(1)$
fence move instead of an $O(d \log d)$ sort, but it surrenders sorted
iteration order — and the module's determinism (lexicographic solutions,
pinned node counts) leans on ascending values. The reference chooses
observable determinism over constant-factor speed; exercise 5 builds the
sparse-set version and shows how to keep both.

**Undo must also run on the dead-end path.** When a live domain wipes out,
the loop `break`s with the removal log *partially* filled — and the
restoration code after the recursion runs unconditionally. A tempting
refactor is to `continue` straight to the next value on wipeout; do that
and the domains stay filtered, and every branch to the right of the dead
end searches a corrupted world. The stage tests catch it (solution sets
shrink), but the honest defense is structural: exactly one exit path,
through the undo.

**Why FC's counts can be pinned at all.** Only live values are ever placed,
and live domains are subsets of the originals kept in sorted order, so the
whole tree — hence 53 for queens-5, 130 for queens-6, 75 for the $C_5$
coloring — is determined by the contract. The subtree relationship with
basic (exercise 1) is what the random-instance test checks as
`nodes_fc <= nodes_basic`.

## Stage 3: AC-3, and the requeue bug that really happened

**Directed arcs as `(constraint index, bool)`.** Each constraint yields two
arcs; `revise(ci, rev)` prunes the tail against the head, flipping the pair
orientation when `rev` is set. Cloning the head's domain and the allowed
list before the `retain` is the borrow-checker toll — a production solver
would restructure to avoid it, but the clone keeps `revise` eight lines and
visibly equal to its specification.

**The requeue direction — a bug story.** When a revision shrinks $D_v$,
the reference requeues the arcs whose **head** is $v$: for each constraint,
`(cj, false)` if `c.y == pruned` (that arc prunes $D_{c.x}$ *against* the
shrunken $D_{c.y}$), `(cj, true)` if `c.x == pruned`. During development the
first draft had those two conditions swapped — requeue by *tail* — and here
is the instructive part: **the wipeout test still passed.** On the
two-variable cycle $X < Y$, $Y < X$, the initial queue alone (every arc,
revised once, some twice) empties a domain before propagation distance
matters. What failed was the *chain* test: on $X < Y < Z$ the buggy version
terminates with $D_Z = \{2, 3\}$ — the deduction $Y \ne 3 \Rightarrow Z \ne
2$ requires re-revising the arc $Z \to Y$ *after* $Y$'s domain shrank, and
only head-directed requeuing does that. Two morals. First, the mnemonic:
*a tail leans on its head*; when a head loses values, its dependents must
re-check. Second, the testing lesson: wipeout detection is a weak oracle for
propagation (many orders reach an empty domain), while a fixpoint with a
known exact value — three singletons — is a strong one. The suite also
cross-checks order-independence (reversed constraint insertion) and
idempotence, which the uniqueness theorem (lesson §4) says any correct
worklist must satisfy — and the buggy version violates both on the very
same chain (a second run prunes further; inserting the constraints in the
other order reaches a different fixpoint). The chain was simply the first
and clearest tripwire.

**Termination is a shrink argument.** Every requeue is justified by a
strict shrink of some domain, and total shrinkage is bounded by
$\sum_v |D_v|$; with $2e$ arcs and an $O(d^2)$ revise, the classic
$O(e \cdot d^3)$ bound falls out. The reference makes no attempt at AC-4/
AC-6-style support bookkeeping ($O(e \cdot d^2)$) — at this module's sizes
the constant-factor simplicity wins, and the uniqueness theorem guarantees
the cheaper algorithm computes the identical fixpoint.

**What mutation testing said.** Seeding 102 bugs into the reference and
re-running the suites left five survivors on the first pass. Two were real
gaps, both now closed by dedicated tests: the MRV tie-break (`<` → `<=`
prefers the *last* minimum; the size-2/2/2 instance pins 5 nodes against the
mutant's 8) and a complement-requeue in AC-3 (caught by the two-wave
instance above plus a run-it-twice fixpoint check). The other three, all in
`count_models`, are **provably equivalent mutants**: flipping the bit-sense
of an assignment or the sign convention of literals composes with the
bijection $b \mapsto \bar b$ over assignments, so every model *count* is
unchanged for every formula — no count-only test can distinguish them. They
are expected survivors of the weekly sweep, not gaps.

## Stage 4: the direct encoding

**The `var` table is the whole design.** Number SAT variables
domain-slot by domain-slot, keep the map `var[v][j]`, and every clause is a
table lookup — ALO rows are literally `var[v].clone()`. The emission order
(ALO block, then AMO, then conflicts) is contractual: the tests index
`clauses[0..n]` as the ALO rows and assert everything after them is a
binary all-negative clause, a shape check that catches sign slips and
misordered emission long before model counting would.

**Conflicts are computed from the *complement* of `allowed`.** The encoder
walks $D_x \times D_y$ and emits a clause exactly when `binary_search`
misses. That inversion is the point where the CSP's "allowed pairs"
convention meets CNF's "forbidden combinations" native tongue — and it is
why the clause count formula has the "forbidden *in-domain* pairs" term:
pairs outside the current domains produce no clause, so pruning domains
first (stage 3) shrinks the encoding. Try it: AC-3 the $X < Y < Z$ chain,
then encode — the singleton domains yield a near-trivial formula.

**`count_models` is an oracle, so it stays stupid.** A brute-force truth
table over $2^{\text{num\_vars}}$ assignments, with a hard panic above 24
variables (message `"truth table"`). No unit propagation, no early exit —
sharing zero reasoning with anything else in the module is precisely what
makes its agreement with `solve_basic` (the bijection test) evidence rather
than tautology. The same closure-over-`bits` literal evaluation appeared in
Module 10; here it certifies a *reduction* instead of a solver.

## Going further

- **dom/wdeg and friends.** MRV uses only domain size; modern solvers
  weight each constraint by how often it caused failures and branch on the
  variable minimizing domain/weighted-degree — a *learning* ordering
  heuristic. §7.2.2.3's discussion of variable ordering is the doorway;
  bolting dom/wdeg onto `lookahead_step` is an afternoon's experiment.
- **MAC — maintaining arc consistency.** Stage 2 filters one constraint
  deep; MAC runs full AC-3 (restricted to arcs touching changed domains)
  after *every* placement, trading more propagation per node for far fewer
  nodes. With `ac3` and the save-list undo already written, MAC is a
  composition exercise — and the standard production baseline.
- **The support encoding.** Stage 4's direct encoding lets a SAT solver's
  unit propagation simulate only forward checking; the support encoding
  (clauses listing each value's supports) makes unit propagation perform
  full arc consistency. Encode the chain both ways and watch propagation
  alone solve one of them — stages 3 and 4 collapsing into each other.
- **Dancing cells at full scale.** Exercise 5's sparse set is the seed of
  Knuth's F7 solver architecture: all-array state, trail-based undo, no
  allocation in the search loop. Compare with Module 17's dancing links to
  see the two reversible-deletion philosophies side by side — pointers that
  remember their neighbors vs. fences that remember nothing because the
  cells never left.
