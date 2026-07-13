# Module 23 — Constraint Satisfaction

> **Source:** *The Art of Computer Programming*, Vol. 4 Fascicle 7, §7.2.2.3
> (Constraint Satisfaction; toward Vol. 4C), with threads back to Vol. 4B
> §7.2.2 (backtrack programming, Algorithm 7.2.2B) and §7.2.2.2
> (satisfiability).
> **Lab:** `labs/module-23-csp` · **Grade it:** `./grade 23`
> **Prerequisites:** Module 09 (backtracking) and Module 10 (SAT). Module 13
> (BDDs) and Module 22 (Hamiltonicity) make cameo appearances.
>
> **A note on the source.** Fascicle 7 is a *draft pre-fascicle* — the newest
> material in this course, still being written for its eventual home in
> Vol. 4C. Its section numbers are stable (§7.2.2.3 follows §7.2.2.2's SAT),
> but its exercise numbering is not, so this module cites sections only. The
> lesson is self-contained: you can complete the module without the book.

Every combinatorial puzzle in this course — n-queens in Module 09, SAT in
Module 10, graph coloring, exact cover, Hamiltonicity in Module 22 — has had
the same skeleton under its skin: *variables, a set of possible values for
each, and constraints saying which combinations are acceptable.* §7.2.2.3
names that skeleton the **constraint satisfaction problem** and studies it on
its own terms. The payoff is leverage: one model, one search algorithm, one
lookahead technique, one consistency filter, and one translation to SAT — and
every puzzle above inherits all five at once. This module builds that stack
in four stages, with the numbers pinned at every step.

---

## 1. One model for a thousand puzzles (§7.2.2.3)

A **binary CSP** consists of:

- **variables** $x_0, x_1, \ldots, x_{n-1}$;
- an explicit finite **domain** $D_v$ of values for each variable $v$;
- **constraints**, each naming an ordered pair of *distinct* variables
  $(x, y)$ together with the explicit set of value pairs it **allows**:

$$
R_{xy} \subseteq D_x \times D_y.
$$

A complete assignment $(a_0, \ldots, a_{n-1})$ is a **solution** when every
$a_v \in D_v$ and every constraint's induced pair $(a_x, a_y)$ lies in its
allowed set. That is the whole model — no clause syntax, no graph structure,
no arithmetic: a constraint is just a *list of permitted pairs*, the most
general binary relation there is.

Two conventions run through the lab and the reference, and the tests rely on
both:

1. **Domains and allowed-pair lists are sorted and duplicate-free.** The
   constructors enforce it, so every membership test can be a binary search
   and "values in ascending order" is well-defined.
2. **A search *node* is counted every time a value is tentatively placed on a
   variable, *before* any consistency test.** This is the standard cost
   measure Knuth instruments backtrack programs with in §7.2.2: it counts
   *placements attempted*, not "placements that survived," so different
   algorithms are compared on exactly the work they perform. Every pinned
   node count below depends on this definition — adopt it precisely.

### Everything is a CSP

**n-queens.** Variable $i$ is the queen in column $i$; its value is the row,
domain $\{0, \ldots, n-1\}$. For each column pair $i < j$ at distance
$d = j - i$, the allowed pairs are exactly the $(a, b)$ with $a \ne b$ and
$|a - b| \ne d$ — different rows, different diagonals. That is `queens_csp`:
§7.2.2's running example restated in §7.2.2.3's language, the constraint
being the list of non-attacking row pairs, nothing more.

**Graph coloring.** One variable per vertex, domain $\{0, \ldots, k-1\}$; for
every edge, the allowed pairs are the *unequal* ones. That is `coloring_csp`:
"proper coloring" stops being a special-purpose notion and becomes one
particular allowed-pair set — the disequality relation.

**SAT.** A binary clause $(\ell_x \lor \ell_y)$ is a constraint on its two
variables with domains $\{0, 1\}$, allowing the three of the four pairs that
satisfy it — 2-SAT is *literally* a binary CSP. A $k$-literal clause is a
$k$-ary constraint (all $2^k - 1$ satisfying tuples); §7.2.2.3 shows how
higher arities fold into the binary model when needed. The traffic also runs
the other way, and that is the direction we implement: stage 4 translates any
binary CSP *into* CNF, handing the model to §7.2.2.2's solvers.

The moral, stated up front: **CSP is the meeting point.** Backtracking
(§7.2.2), its refinements — lookahead, consistency, ordering heuristics
(§7.2.2.3) — and translation to SAT (§7.2.2.2) become theorems about *one*
object instead of five ad-hoc tricks. F7's application tour makes the same
point at scale: line labeling in computer vision, graph labeling and
embedding, subgraph isomorphism — all the same triple of variables, domains,
allowed pairs.

---

## 2. Stage 1 — basic backtracking is Algorithm 7.2.2B in new clothes

Since a CSP's constraints are arbitrary pair lists, no formula decides it (we
will be precise about *how* hard in §6); we **search**. Stage 1 is Knuth's
Algorithm 7.2.2B — the basic backtrack skeleton of Module 09 — specialized to
the CSP model. Variables are assigned in index order, values in ascending
order:

```text
B1. [Initialize.]  l <- 0 (no variables assigned).
B2. [Done?]        If l = n, visit the solution (a_0, ..., a_{n-1}); back up.
B3. [Try a value.] Let a range over D_l in ascending order.
                   Place x_l = a           <- this is one NODE, counted now
B4. [Test.]        If every constraint whose BOTH endpoints are <= l holds
                   on the partial assignment, set l <- l + 1 and go to B2.
                   Otherwise undo the placement and continue B3.
B5. [Backtrack.]   When D_l is exhausted, set l <- l - 1 and resume B3 one
                   level up; if l < 0, the search is complete.
```

The invariant at every entry to B2: *the placed prefix violates no constraint
among its own variables.* Placing preserves it (B4 tests exactly the
constraints newly completed by $x_l$); backtracking restores the previous
state exactly — Module 09's exact-state-restoration discipline, which never
leaves this module. Every later refinement only strengthens B4's test or
reorders B3's choices. And note the honesty of the counter: B3 counts the
placement *before* B4 tests it — a rejected placement cost real work.

### Hand trace: 4-queens as a CSP

Domains $\{0,1,2,3\}$ per column; constraints as above. Here is the entire
basic search tree — every placement, with ✗ marking those B4 rejects:

```text
x0=0  x1: 0✗ 1✗ 2   x2: 0✗ 1✗ 2✗ 3✗                        (dead end)
              3     x2: 0✗ 1    x3: 0✗ 1✗ 2✗ 3✗             (dead end)
                        2✗ 3✗
x0=1  x1: 0✗ 1✗ 2✗
              3     x2: 0       x3: 0✗ 1✗ 2 SOLUTION (1,3,0,2)  3✗
                        1✗ 2✗ 3✗
x0=2  x1: 0         x2: 0✗ 1✗ 2✗
                        3       x3: 0✗ 1 SOLUTION (2,0,3,1)  2✗ 3✗
          1✗ 2✗ 3✗
x0=3  x1: 0         x2: 0✗ 1✗ 2   x3: 0✗ 1✗ 2✗ 3✗           (dead end)
                        3✗
          1         x2: 0✗ 1✗ 2✗ 3✗                          (dead end)
          2✗ 3✗
```

Count the placements level by level: $4$ at level 0, $16$ at level 1 (all
four roots are consistent, each tries four values), $24$ at level 2 (six
consistent pairs survive), $16$ at level 3 (four consistent triples) — total
**60 nodes** for **2 solutions**, the famous pair of rows $(1,3,0,2)$ and
$(2,0,3,1)$, each the $180^\circ$ rotation of the other. Notice how much of
the tree is spent *discovering* dead ends late: the branch $x_0=0, x_1=3,
x_2=1$ places all four values of $x_3$ before giving up, though the doom was
sealed two levels earlier. Stage 2 exists to notice sooner.

### The pinned facts

Your stage 1 must reproduce, exactly:

- **Solution counts** for queens $n = 4, \ldots, 8$: the classic table
  $2, 10, 4, 40, 92$ (§7.2.2).
- **Node count** for queens-5 under the contract's order: **220**. This
  number is fully determined by the conventions — variables by index, values
  ascending, count every placement — which is why the test can pin it.
- **Coloring counts are chromatic-polynomial evaluations.** The number of
  proper $k$-colorings of a graph $G$ is $P(G, k)$, and for the cycle $C_n$
  a classic deletion–contraction argument (exercise 2) gives

$$
P(C_n, k) = (k-1)^n + (-1)^n (k-1),
$$

  so $C_5$ with 3 colors has $P(C_5, 3) = (3-1)^5 + (-1)^5(3-1) = 30$
  solutions; likewise $P(K_3, 3) = 3! = 6$ and $P(K_4, 3) = 0$. Counting
  anchors like these are the module's certification style: an independent
  closed form your search must hit on the nose.

---

## 3. Stage 2 — lookahead: forward checking, dancing cells, and MRV

Basic backtracking tests only *backward* — constraints among the variables
already placed. **Forward checking** looks the other way. The solver keeps a
**live domain** per variable, initially a copy of $D_v$; placing $x = a$:

1. For each constraint incident to $x$ whose other endpoint $y$ is
   unassigned, delete from $y$'s live domain every $b$ with $(a, b)$ not
   allowed.
2. If some live domain **wipes out** — empties — the placement is a dead
   end: abandon it *now*, not three levels deeper.
3. On backtrack, restore every deleted value — exactly.

Step 3 is where the engineering lives. The lab's honest baseline is a **save
list**: log each deletion as (variable, values removed) while filtering, and
merge the log back on the way out. Simple, visibly correct, and it keeps the
live domains sorted, so solutions still stream out lexicographically.

### Dancing cells: the sparse set (§7.2.2.3's signature trick)

Fascicle 7's production answer is prettier. Represent each domain as a
**sparse set**: an array `dom` holding the values in *some* order, an inverse
index `pos` (where does value $b$ sit in `dom`?), and a fence `size`. The
live values are exactly `dom[0..size]`.

```text
   dom:  [ 5 | 9 | 2 | 7 | 4 | 8 ]      size = 6      pos[2] = 2, pos[7] = 3, ...
                                  ^ fence

   delete 9:  swap dom[1] <-> dom[5], update pos, size <- 5

   dom:  [ 5 | 8 | 2 | 7 | 4 | 9 ]      size = 5
                              ^ fence     (9 is parked beyond the fence, intact)

   undo:      size <- 6   — that's the whole undo
```

Deletion is a swap-to-the-fence plus a decrement: $O(1)$. Membership is
`pos[b] < size`: $O(1)$. The undo is the beautiful part: *deleted values are
never destroyed*, only parked beyond the fence, so restoring a batch of
deletions is just moving the fence back — $O(1)$, provided restores happen in
last-in-first-out order, which a backtracking search guarantees by
construction. Knuth calls this **dancing cells**: the same philosophy as
Module 09's dancing links — *a removed element retains exactly the
information needed to restore it* — flattened from pointers into array cells,
friendlier to caches and to reasoning. The price is that `dom[0..size]` is no
longer sorted; the lab's save-list keeps the sorted-order guarantee instead,
and exercise 5 has you build the sparse-set version and confront the trade
honestly.

### MRV: branch on the tightest spot

Forward checking maintains, as a free by-product, every unassigned variable's
*current live domain size* — a running measure of how constrained it is. The
**minimum-remaining-values** heuristic (MRV, "most constrained variable
first") uses it: always branch next on the unassigned variable with the
*smallest* live domain, ties to the smallest index. The logic is Knuth's
advice from §7.2.2, sharpened in §7.2.2.3's treatment of variable-ordering
heuristics: a variable with 2 live values splits the search two ways, one
with 5 splits it five — and if a variable is *doomed* (live domain about to
empty), you want to find out at depth 1, not depth 9. "Fail first" is the
slogan: the cheapest subtree to search is the one that isn't there. One
contract consequence: with MRV the variable order varies down different
branches, so solutions no longer emerge lexicographically — `solve_fc_mrv`
sorts before returning.

### The worked numbers

Node counts under the contract's definition (placements of *live* values,
counted before testing); the **bold** rows are pinned by the stage 2 tests,
the rest are equally determined by the conventions:

| Instance | basic | forward checking | FC + MRV |
|---|---|---|---|
| queens-4 | 60 | 16 | 16 |
| **queens-5** | **220** | **53** | **53** |
| **queens-6** | **894** | **130** | **118** |
| queens-7 | 3584 | 463 | 393 |
| queens-8 | 15720 | 1724 | 1360 |
| **$C_5$, 3 colors** | **138** | **75** | — |

Read the queens-6 row aloud: $894 \to 130 \to 118$. Forward checking alone
cuts the tree almost sevenfold — every value it places has already survived
filtering by all placed neighbors, so entire doomed subtrees of the basic
tree are never entered. MRV shaves further by *reordering* which tree gets
searched at all. A claim you will prove in exercise 1: under the same static
variable order, forward checking's tree is a **subtree** of basic
backtracking's — every placement FC makes, basic makes too — so
$\text{nodes}_{FC} \le \text{nodes}_{basic}$ on *every* instance, not just
these. Lookahead prunes work, never solutions: both searches return
identical solution sets, which the tests check instance by instance.

---

## 4. Stage 3 — arc consistency (AC-3)

Forward checking filters the future against the *placed* variables. §7.2.2.3
asks the natural next question: how much filtering can we do against the
*unplaced* ones — before search even starts?

### Support, and the revise step

Fix a constraint on $(x, y)$ and read it in both directions: the **arc**
$x \to y$ and the arc $y \to x$. A value $a \in D_x$ is **supported** along
$x \to y$ if some $b \in D_y$ has $(a, b)$ allowed. An unsupported value is
dead weight — a solution must give $y$ *some* value, and every value of $y$
refuses this $a$ — so we may delete it. That is the **revise** operation:

$$
\mathrm{revise}(x \to y): \quad
D_x \leftarrow \{\, a \in D_x : \exists\, b \in D_y \text{ with } (a,b) \in R_{xy} \,\}.
$$

A CSP is **arc consistent** when no revise, on any arc, would delete
anything: every value of every domain has support along every arc leaving its
variable.

### The worklist, and the direction that matters

One pass of revising every arc is not enough, because deletions cascade:
shrinking $D_y$ can strip the support out from under values of *other*
variables that leaned on the deleted values. **AC-3** handles the cascade
with a worklist:

```text
A1. [Initialize.]  Put every directed arc (two per constraint) on the queue.
A2. [Loop.]        While the queue is nonempty, pop an arc (tail -> head)
                   and revise it.
A3. [Cascade.]     If the revision deleted something from D_tail:
                     - if D_tail is now empty, report WIPEOUT (no solutions);
                     - otherwise requeue every arc whose HEAD is tail.
A4. [Done.]        Queue empty: every arc is consistent; report the domains.
```

Step A3's direction deserves a hard stare, because it is the step everyone
gets wrong once. When $D_v$ shrinks, which arcs might newly fail? An arc
$u \to v$ prunes $D_u$ *against* $D_v$ — its tail's values claim support **in
$v$'s domain** — so deleting values of $v$ can invalidate it: requeue it. An
arc $v \to w$ prunes $D_v$ against $D_w$; deleting *tail* values only removes
claimants and can never un-support the survivors. So: **requeue exactly the
arcs whose head is the pruned variable.** Get this backwards and the
algorithm still terminates, still detects many wipeouts — and quietly returns
under-filtered domains. The walkthrough tells that bug's story.

### The demonstration: an ordering chain collapses

Take three variables $X, Y, Z$, each with domain $\{1, 2, 3\}$, and the
constraints $X < Y$ and $Y < Z$ (as allowed-pair lists: the pairs with
$a < b$). Watch AC-3 work:

```text
revise (Z -> Y):  z = 1 has no y < z ................. D_Z = {2, 3}
revise (Y -> X):  y = 1 has no x < y ................. D_Y = {2, 3}
revise (Y -> Z):  y = 3 has no z > y in {2, 3} ....... D_Y = {2}
revise (X -> Y):  x needs some y > x in {2}: x = 1 ... D_X = {1}
revise (Z -> Y):  z needs some y < z in {2}: z = 3 ... D_Z = {3}
```

Fixpoint: $D_X = \{1\}$, $D_Y = \{2\}$, $D_Z = \{3\}$. Consistency alone — no
search — deduced the unique solution $1 < 2 < 3$: the textbook demonstration
that filtering *propagates*. (Add $Z < X$ and the cascade instead empties a
domain: AC-3 proves insolubility without placing a single value. The stage
tests pin both behaviors.) But do not over-conclude: arc consistency is a
*filter*, not a solver. On queens-6 it deletes nothing at all — every row
value of every column has support in every other column — and the search
still has all the work to do. AC shines on tightly-coupled chains and as
preprocessing; §6 shows one family where it is, provably, the whole story.

### The fixpoint is unique — proof

AC-3 makes its deletions in whatever order the worklist serves them. Why must
every order reach the *same* final domains?

Call a family of subdomains $E = (E_v \subseteq D_v)$ **arc-consistent** if
every value of every $E_v$ has support along every arc, *with the supports
drawn from the $E$-domains themselves*.

> **Lemma (closure under union).** If $E$ and $F$ are arc-consistent
> families, so is their componentwise union $E \cup F$.

*Proof.* Take $a \in E_x \cup F_x$; say $a \in E_x$. Along any arc
$x \to y$, $a$ has a support $b \in E_y$ because $E$ is arc-consistent, and
$b \in E_y \subseteq E_y \cup F_y$. So every value of the union has support
inside the union. ∎

> **Theorem (unique maximal fixpoint).** There is a unique *largest*
> arc-consistent family $M \subseteq D$; and AC-3 computes exactly $M$,
> regardless of worklist order.

*Proof.* Let $M$ be the union of **all** arc-consistent families contained
in the original domains. By the lemma (the same one-line argument works for
arbitrary unions), $M$ is itself arc-consistent, and by construction it
contains every other one: it is the unique maximal fixpoint.

Now the two inclusions for AC-3's result $A$. *($A \supseteq M$.)* By
induction on the deletions: the current domains start as $D \supseteq M$,
and a revise deletes $a \in D_x$ only when $a$ has no support in the
*current* $D_y \supseteq M_y$; a value of $M_x$ always has a support in
$M_y$, hence in $D_y$ — so no value of $M$ is ever deleted.
*($A \subseteq M$.)* When the worklist empties, every arc has been revised
since the last time its head's domain changed (precisely what the requeue
rule maintains), so no revise would delete anything: $A$ is an
arc-consistent family, hence contained in the maximal one. So $A = M$. ∎

The stage tests check the theorem behaviorally: feed the constraints in
reversed order, get identical fixpoints; run AC-3 twice, the second run
changes nothing (idempotence).

> **Corollary (solutions are preserved exactly).** If $s$ is a solution, the
> singleton family $E_v = \{s_v\}$ is arc-consistent — each value's support
> along every arc is the solution's own value at the other end. So
> $s_v \in M_v$ for every $v$: no solution loses a value; and since domains
> only shrink, no new solutions appear. The solution set is untouched.

### Cost

Termination is immediate: every requeue follows a strict shrink of some
finite domain, and total shrinkage is bounded by $\sum_v |D_v|$. For the
bound, let $e$ be the number of constraints and $d = \max_v |D_v|$. Each of
the $2e$ arcs re-enters the queue only when its head loses a value — at most
$d$ times — and one revise costs $O(d^2)$ ($\le d$ tail values each scanning
$\le d$ head values). Total: $O(e \cdot d^3)$, the classic AC-3 bound.
(Remembering *which* support each value used, so a deletion wakes only its
actual dependents, gets the optimal $O(e \cdot d^2)$ — AC-4/AC-6 territory;
§7.2.2.3's discussion of consistency and efficiency is the doorway.)

---

## 5. Stage 4 — translating CSP to SAT (meeting §7.2.2.2)

Module 10 built SAT solvers; §7.2.2.2 builds industrial ones. The **direct
encoding** hands them every binary CSP.

**SAT variables.** For each CSP variable $v$ and each domain slot $j$,
allocate one boolean $p_{v,j}$ meaning "$v$ takes its $j$-th domain value."
Number them $1, 2, 3, \ldots$ domain-by-domain (the DIMACS convention of
Module 10), so

$$
\text{num\_vars} = \sum_v |D_v|.
$$

**Clauses**, in three families (and in this order — the tests index into it):

1. **At-least-one (ALO).** For each $v$: the clause
   $p_{v,0} \lor p_{v,1} \lor \cdots$ — one clause per CSP variable, $n$ in
   all.
2. **At-most-one (AMO).** For each $v$ and each slot pair $i < j$: the binary
   clause $\lnot p_{v,i} \lor \lnot p_{v,j}$ — that is $\binom{|D_v|}{2}$
   clauses per variable.
3. **Conflict.** For each constraint on $(x, y)$ and each in-domain pair
   $(a, b) \in D_x \times D_y$ that the constraint does **not** allow: the
   binary clause $\lnot p_{x,i} \lor \lnot p_{y,j}$ (where $i, j$ are the
   slots of $a, b$). Forbid exactly what the CSP forbids.

So the exact clause count — pinned by the tests — is

$$
n \;+\; \sum_v \binom{|D_v|}{2} \;+\; \sum_{c} \bigl|\{(a,b) \in D_x \times D_y : (a,b) \notin R_{xy}\}\bigr|.
$$

**Worked example: $C_5$ with 3 colors.** Five variables of domain size 3:
$15$ SAT variables. Clauses: $5$ ALO $+\; 5\binom{3}{2} = 15$ AMO $+\; 5
\cdot 3 = 15$ conflicts (each edge forbids its three equal-color pairs)
$= 35$. A brute-force truth table over the $2^{15}$ assignments counts
exactly **30 models** — $P(C_5, 3)$ again, certified through an entirely
different formalism.

**Worked example: queens-4.** $16$ SAT variables. Clauses: $4$ ALO
$+\; 4\binom{4}{2} = 24$ AMO, plus conflicts: a column pair at distance $d$
forbids $4$ equal-row pairs and $2(4-d)$ diagonal pairs, so summing
$4 + 2(4-d)$ over the six pairs (three at $d=1$, two at $d=2$, one at
$d=3$):

$$
3 \cdot (4 + 6) + 2 \cdot (4 + 4) + 1 \cdot (4 + 2) = 30 + 16 + 6 = 52
$$

conflict clauses; $4 + 24 + 52 = 80$ clauses in all, and exactly **2**
models — the two 4-queens solutions of §2's hand trace, found again on the
other side of the bridge. (Exercise 4 turns this tally into a closed form
for all $n$.)

### The bijection theorem

> **Theorem.** The models of the direct encoding correspond one-to-one with
> the solutions of the CSP.

*Proof.* Given a model, look at one CSP variable $v$: the ALO clause forces
at least one $p_{v,j}$ true, the AMO clauses force at most one — so exactly
one, and the model *is* a complete assignment $s$ (read off the chosen
slots). Every constraint of the CSP holds on $s$: if some induced pair
$(s_x, s_y)$ were forbidden, its conflict clause $\lnot p_{x,i} \lor \lnot
p_{y,j}$ would have both literals false. So the map model $\mapsto$
assignment lands in the solution set. It is injective (different models
differ in some $p_{v,j}$, hence choose different values somewhere) and
surjective (a solution $s$ defines the assignment "set exactly the chosen
slots true," which satisfies ALO and AMO by construction and every conflict
clause because $s$ violates no constraint). ∎

The stage tests certify the bijection the honest way: on random CSPs, count
models by brute-force truth table (an oracle that shares no code with the
search) and demand it equal the stage 1 solution count, instance by
instance.

The direct encoding is the simplest of several; §7.2.2.3's treatment of
translating CSP to SAT also develops the **log encoding** (the *slot index*
of $v$'s value in $\lceil \lg |D_v| \rceil$ bits — fewer variables, weaker
propagation) and the **support encoding** (clauses stating each value's
supports, so a SAT solver's unit propagation performs arc consistency for
free — stage 3 smuggled into stage 4). Picking an encoding is choosing
*which reasoning the SAT solver can do cheaply* — a design space you now
hold both vocabularies for.

---

## 6. Tractable islands, and the dichotomy frontier

Everything so far treats CSP as hard, and in general it is: graph coloring
is a CSP, so CSP is NP-complete. But §7.2.2.3 ends by mapping where the
hardness actually lives, and two landmarks belong in this course.

**Trees are easy.** Suppose the *constraint graph* (variables as vertices,
one edge per constraint) is a tree. Root it, run arc consistency, and — if
no domain emptied — assign values root-to-leaves, each variable choosing any
live value supported by its parent's chosen value. Arc consistency
guarantees such a value exists at every step: the assignment is
**backtrack-free**, and the whole solve is polynomial ($O(e \cdot d^2)$ with
the directional variant of AC). The moral: stage 3 is not merely a
preprocessing heuristic; on tree-structured problems *it is the complete
algorithm*. More generally, how tree-like the constraint graph is (its
treewidth — cousin of Module 17's frontier idea) calibrates the cost on
non-trees.

**The dichotomy theorem.** Fix a *constraint language* $\Gamma$ — a set of
allowed relation types — and consider $\mathrm{CSP}(\Gamma)$, the problems
whose constraints all come from $\Gamma$. Feder and Vardi conjectured in
1993, and Bulatov and Zhuk independently proved in 2017, that **every** such
family is either solvable in polynomial time or NP-complete — nothing
strictly in between. (Compare: 2-coloring easy, 3-coloring NP-complete;
2-SAT easy, 3-SAT NP-complete. The theorem says that pattern is the law.)
The classifying tool is the language's *polymorphisms* — operations under
which all its relations are closed — the algebraic theory F7 sketches as the
subject's frontier. This is the newest mathematics in the course: a complete
map of an infinite family of problems, finished within the last decade, and
part of why §7.2.2.3 is still a draft — the territory is still settling.

---

## 7. Stage-by-stage lab guide

Open `labs/module-23-csp/src/lab.rs`. Each stage has a test file
`tests/stage_NN_*.rs`, and `./grade 23` runs them in order. The panics are
contracts: the grader checks the exact substrings (`"distinct"`, `"range"`,
`"length"`, `"truth table"`).

**Stage 1 — `Csp`, `queens_csp`, `coloring_csp`, `solve_basic`.** Normalize
at the boundaries (sort + dedup domains in `new`, pair lists in
`add_allowed`) so everything downstream can binary-search. `solve_basic` is
the B1–B5 skeleton: recurse on the next unassigned index, count each
placement *before* testing it, test only the constraints with both endpoints
placed. The tests pin the queens table, the chromatic anchors, queens-5's
220 nodes, lexicographic output, and cross-check twenty random CSPs against
a product-enumeration oracle.

**Stage 2 — `solve_fc`, `solve_fc_mrv`.** Live domains plus a removal log
per placement; abandon a placement the moment a live domain empties; restore
exactly on the way out. MRV changes only the variable-selection line
(smallest live domain, ties to smallest index) — and remember to sort the
solutions before returning. The tests pin §3's node-count table and check
FC $\le$ basic on random instances.

**Stage 3 — `ac3`.** A worklist of directed arcs; `revise(tail, head)`
deletes unsupported tail values; on any deletion, requeue **the arcs whose
head is the pruned variable** — reread §4 if that phrase does not yet feel
inevitable. Return `false` the moment a domain empties. The tests pin the
$X < Y < Z$ collapse, the wipeout, queens-6 invariance, exact solution
preservation, and — §4's theorem made executable — order-independence and
idempotence of the fixpoint.

**Stage 4 — `encode_direct`, `count_models`.** Number SAT variables
domain-by-domain from 1; emit ALO, then AMO, then conflict clauses, in that
order. `count_models` is a truth-table sweep with the 24-variable guard. The
tests pin the $C_5$ and queens-4 sizes and model counts, the clause shapes
(everything after the ALO block is binary and negative), and the
model↔solution bijection on random instances.

---

## 8. Check your understanding

1. Why is a node counted *before* the consistency test rather than after?
   (It measures placements attempted — the actual work — so basic, FC, and
   MRV are compared on one scale; a rejected placement was not free.)
2. Forward checking placed $x = a$ and no live domain emptied. Can the
   placement still be doomed? (Yes — wipeout is one-constraint-deep; two
   future variables can each keep nonempty domains that are jointly
   unsatisfiable. Closing that gap is what fuller consistency, and
   ultimately search, is for.)
3. In AC-3, after $D_v$ shrinks, why is it pointless to requeue an arc
   $v \to w$? (It prunes its tail $D_v$ against $D_w$; removing tail values
   removes claimants but cannot un-support the survivors. Only arcs whose
   *head* is $v$ can newly fail.)
4. The direct encoding of queens-8 has $64$ variables. How many models does
   it have, without running anything? (92 — the bijection theorem plus the
   queens-8 solution count.)
5. Arc consistency solved $X < Y < Z$ outright but did nothing to queens-6.
   Why? (The chain is a tree with tight constraints — §6's easy case, where
   AC is complete; on queens every value has support everywhere, so the
   filter finds nothing local to remove.)

---

## 9. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 up to an hour ·
30 hours · 40 term project · 50 open research; ▶ marks especially
instructive ones. Fascicle 7 is a draft whose exercise numbers may still
shift, so these are **content tags** cited to §7.2.2.3 by topic; full
statements, hints, and answer sketches live in
[`exercises.md`](./exercises.md).

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| ▶7.2.2.3-(fc-subtree) | 25 | Prove forward checking visits a subtree of basic backtracking's tree under the same static order. |
| 7.2.2.3-(del-con) | 22 | Prove deletion–contraction, derive $P(C_n,k)$, and verify $P(\text{Petersen}, 3) = 120$ with the lab. |
| ▶7.2.2.3-(ac-fixpoint) | 25 | The AC-3 fixpoint uniqueness proof, in full. |
| 7.2.2.3-(enc-size) | 20 | Closed form for the size of the direct encoding of queens-$n$. |
| ▶7.2.2.3-(dancing-cells) | 32 | Mini-project: a dancing-cells sparse-set domain with $O(1)$ delete/undo, swapped into forward checking. |

---

## Why it's done this way

- **Explicit allowed-pair lists, not predicates.** A constraint you can
  *enumerate* is a constraint you can revise, count, encode, and test
  against an oracle. Every stage — FC's filtering, AC-3's support search,
  the conflict clauses — walks those lists; a black-box predicate would have
  forced four different interrogation APIs. The representation *is* the
  algorithm's diet (the Module 09 lesson, again).
- **Count nodes before testing, and pin the counts.** Search heuristics are
  notorious for folklore claims; a fixed placement order plus a fixed node
  definition makes the tree a mathematical object, so "FC beats basic
  $894 \to 130$" is a theorem about this instance, checked by the grader,
  not an anecdote.
- **A save list before dancing cells.** The lab's undo log is $O(d \log d)$
  where the sparse set is $O(1)$ — deliberately. Correct-and-visible first,
  clever second (exercise 5): the failure mode of clever undo structures is
  silent corruption three levels up the tree, and you can only debug that
  against a boring baseline you trust.
- **Requeue by head, and prove the fixpoint unique.** The requeue direction
  is a one-line decision that testing alone barely constrains — the wrong
  direction passes most obvious tests. The uniqueness theorem turns "my
  worklist happens to work" into "any worklist works"; that is why the
  lesson proves it rather than gesturing at it.
- **Encode to SAT rather than extend the solver.** When you already own an
  industrial hammer (Module 10 / §7.2.2.2), a faithful translation beats a
  bespoke feature: the bijection theorem transfers all the SAT solver's
  guarantees to the CSP at zero marginal cost. Choosing the encoding —
  direct, log, support — is choosing what the hammer can reach.

## In the real world

- **Scheduling and rostering.** Timetables, exam schedules, airline crew
  pairings, and hospital rosters are CSPs almost verbatim — slots as
  variables, resources as domains, the rulebook as constraints. Industrial
  constraint-programming systems (Google's **CP-SAT**, IBM's **CP
  Optimizer**, the open **Gecode** and **Choco**) run exactly this module's
  stack: propagation (stage 3 on steroids), ordering heuristics (stage 2),
  and clause learning borrowed through the SAT bridge (stage 4).
- **MiniZinc** is the lingua franca: a modeling language whose compiler *is*
  a CSP-to-solver translator — `encode_direct`'s reduction discipline with a
  menu of backends. Writing one encoding by hand, sizes pinned, is the
  fastest way to understand what such compilers do all day.
- **Register allocation** in compilers is graph coloring — live ranges as
  vertices, interference edges demanding different registers. Chaitin's
  classic allocator and its descendants are coloring heuristics with
  spilling as the fallback when $k$ colors won't do.
- **Verification and configuration.** Hardware model checkers and product
  configurators reduce to CSP/SAT; the line-labeling problem from computer
  vision (Huffman–Clowes, one of F7's opening applications) was among the
  first CSPs ever formalized, and **subgraph isomorphism** (another F7
  application) powers substructure search in chemical databases.
- **Puzzle solvers.** Sudoku, KenKen, futoshiki, nonograms — real solvers
  are propagation loops (stage 3) wrapped around MRV search (stage 2), often
  with an XCC or SAT backend (Modules 17 and 10) for the heavy cases. You
  have now built every layer of that stack yourself.

## Proof techniques you practiced

- **Fixpoint uniqueness via closure under union** — arc-consistent families
  are closed under union, so a unique maximal one exists and any correct
  worklist computes it: the standard shape for proving order-independence of
  any saturation procedure (dataflow analyses, closure computations, unit
  propagation).
- **Bijection between two formalisms** — models of the direct encoding ↔
  CSP solutions, proved by exhibiting inverse maps, then *certified by
  counting both sides* — Module 17's reductions-validated-by-counts
  discipline.
- **Subtree-pruning arguments** — "FC's placements are a subset of basic's"
  (exercise 1), by induction on the search tree: the filtered domain is a
  subset of what basic tries, and FC descends only where basic descends.
  Soundness of a prune = solutions preserved; effectiveness = strict
  subtree.
- **Counting anchors** — chromatic-polynomial evaluations ($P(C_5,3) = 30$,
  $P(K_3,3) = 6$, $P(\text{Petersen},3) = 120$), the queens table, and exact
  clause-count formulas give every stage an independent closed form to hit.
  When two utterly different computations agree on 30, both are probably
  right.
- **Invariant + exact state restoration** — the placed prefix is always
  internally consistent, and every mutation (placement, filtering, fence
  move) is undone precisely. Module 01's oldest lesson, still doing the
  correctness work in the newest fascicle.

## 10. Where this leads

- **Module 09 (backtracking)** is where Algorithm 7.2.2B came from; here it
  met a general model instead of one puzzle.
- **Module 10 (SAT)** receives stage 4's output: run your DPLL/CDCL solver
  on `encode_direct`'s clauses and watch the two modules close the loop.
- **Module 13 (BDDs)** offers the other representation of a solution set —
  and §7.2.2.3's dancing cells sit beside §7.2.2.1's dancing links (Module
  17) as the two great reversible-deletion structures.
- **Module 22 (Hamiltonicity)** is a CSP too coarse for binary constraints
  to capture cheaply — a good meditation on why modeling choices matter.
- **Fascicle 7 itself** continues into subgraph isomorphism, stronger
  consistency, sharper ordering heuristics, and the polymorphism theory
  behind the dichotomy theorem — the living edge of Vol. 4C.
- **[`docs/toolkit.md`](../../docs/toolkit.md)** — the fixpoint-uniqueness
  and bijection patterns practiced here are the toolkit's newest entries;
  read it again and notice how few tools the whole course actually needed.
