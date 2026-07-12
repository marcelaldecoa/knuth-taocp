# Module 17 — ZDDs and Exact Covering with Colors

> **Source:** *The Art of Computer Programming*, Vol. 4A, §7.1.4 (zero-suppressed
> decision diagrams) and Vol. 4B, §7.2.2.1 (dancing links with colors,
> Algorithm 7.2.2.1C).
> **Lab:** `labs/module-17-zdd-xcc` · **Grade it:** `./grade 17`
> **Prerequisites:** Module 13 (BDDs — this module *is* the "paragraph on
> ZDDs" from that lesson, grown up) and Module 09 (dancing links — Algorithm
> C is Algorithm X with one new move).
>
> This lesson is self-contained: you can complete the module without the
> books. If you own them, read §7.1.4 from "zero-suppressed BDDs" onward and
> §7.2.2.1 from "color controls" onward.

Module 13 ended with a teaser: change one reduction rule in a BDD and you get
a structure that represents *families of sets* instead of boolean functions,
astonishingly compactly when the families are sparse. This module builds that
structure — the **zero-suppressed decision diagram** — gives it an algebra,
and uses it to count matchings and independent sets by the million without
enumerating a single one. Then we return to dancing links and teach it a new
step: **colors**, which turn "these options conflict" into "these options
must *agree*" — exactly what crossword crossings, graceful labelings, and
Latin squares need. Both halves preach the same sermon: *represent the family
of solutions, not one solution.*

---

## 1. From functions to families

A boolean function $f(x_0, \ldots, x_{n-1})$ and a family of subsets of
$\{0, \ldots, n-1\}$ are interchangeable in principle: the family is
$F = \{ s : f(\chi_s) = 1 \}$ where $\chi_s$ is the bit vector of $s$. Module 13's BDDs
canonically represent the *function*. But the encoding sets up a mismatch.
The families combinatorics cares about are **sparse** twice over: almost all
of the $2^n$ subsets are *not* members, and each member set contains only a few
of the $n$ elements. A BDD is forced to spend nodes saying "…and $x_{17}$ is 0, and
$x_{18}$ is 0, and $x_{19}$ is 0…" for every irrelevant variable, because deleting a
BDD node asserts "this variable doesn't matter," not "this variable is
absent."

Minato's 1993 insight, which Knuth develops in §7.1.4, is to change what
node deletion *means*.

### The definition

A **ZDD** over ordered variables $x_0 < x_1 < \cdots < x_{n-1}$ is a DAG with two
sinks and branch nodes `(v, LO, HI)`, denoting a family of sets by:

```text
F(⊥)            = ∅            the EMPTY FAMILY: no member sets
F(⊤)            = {∅}          ONE member: the empty set
F((v, l, h))    = F(l) ∪ { s ∪ {v} : s ∈ F(h) }
```

LO = "the member sets that do not contain $v$"; HI = "the member sets that
contain $v$, with $v$ deleted." Children only test variables larger than $v$.

Keep $\bot$ and $\top$ straight — they are as different as 0 and 1:

| | $\varnothing = F(\bot)$ | $\{\varnothing\} = F(\top)$ |
|---|---|---|
| number of member sets | 0 | 1 |
| is $\varnothing$ a member? | no | yes |
| role in the algebra | identity of $\cup$, annihilator of $\sqcup$ | identity of $\sqcup$ |

### The two reduction rules

1. **Zero-suppression.** Never create a node with HI = $\bot$. Such a node would
   say "the member sets containing $v$ are: none," which is exactly its LO
   child. `mk(v, lo, ⊥)` returns `lo`.
2. **Uniqueness.** Never create two nodes with the same `(v, LO, HI)`; a
   *unique table* (hash-consing, as in Module 13) shares them.

And one rule conspicuously **absent**: a ZDD node with LO = HI is *kept*. It
says "$v$ is optional: every member set below comes in a with-$v$ and a
without-$v$ version" — real information. Trading the BDD's `lo == hi` rule for
zero-suppression is the whole difference between the two structures, and the
consequence is that a *missing* variable now defaults to "absent from the
set" instead of "irrelevant to the function."

### A worked example

The family $F = \{\{0,1\}, \{2\}\}$ over universe $\{0, 1, 2\}$. Split on $x_0$: members
without 0 are $\{\{2\}\}$; members with 0 (0 removed) are $\{\{1\}\}$. Recurse:

```text
        (x0)
       /    \        LO: sets without 0        HI: sets with 0
     (x2)   (x1)
     /  \   /  \
    ⊥   ⊤  ⊥   ⊤
```

Count nodes with the rules in force: `single(2) = (x2, ⊥, ⊤)`,
`single(1) = (x1, ⊥, ⊤)`, root `(x0, single(2), single(1))` — three branch
nodes plus two sinks, $Z(F) = 5$, and no node mentions a variable its
subfamily doesn't contain. Read membership by walking: is $\{2\} \in F$? At
$x_0$: $0 \notin \{2\}$, go LO to the $x_2$ node; $2 \in \{2\}$, go HI to $\top$; every wanted element
consumed: yes. Is $\{0\} \in F$? At $x_0$: go HI to the $x_1$ node — but now the wanted
set is exhausted while the node demands a decision about 1: go LO, reach $\bot$:
no. And crucially, is $\{1\} \in F$? At $x_0$ we go LO to the $x_2$ node, which has
*skipped past* $x_1$ — by zero-suppression, no member set on this side contains
1: no, in $O(1)$. That skip-means-absent convention is stage 1's
`contains_set`.

### The canonical-form theorem

**Theorem (§7.1.4, ZDD form).** Fix the variable order. Every finite family
$F$ of subsets of $\{0, \ldots, n-1\}$ is denoted by exactly one reduced,
zero-suppressed diagram.

*Proof sketch (induction on the number of available variables).* Zero
variables: the only families are $\varnothing$ and $\{\varnothing\}$, denoted by the two sinks, which
are distinct. Now suppose the theorem holds for families over $\{v+1, \ldots\}$ and
let $F$ be a family over $\{v, v+1, \ldots\}$. Split $F$ uniquely as

$$F_0 = \{ s \in F : v \notin s \}, \qquad F_1 = \{ s \setminus \{v\} : s \in F,\ v \in s \}.$$

Both are families over $\{v+1, \ldots\}$, so each has a unique diagram by induction.
If $F_1 = \varnothing$ — $v$ absent from every member — then $F = F_0$ and the unique diagram
of $F$ is $F_0$'s: this is *exactly* the case zero-suppression forbids us to
build a node for, so the rule and the uniqueness argument are the same fact.
If $F_1 \ne \varnothing$, the node $(v, D(F_0), D(F_1))$ denotes $F$, and any diagram for $F$
rooted at a $v$-node must have children denoting $F_0$ and $F_1$ (the semantics
forces it), unique by induction; the unique table makes the node itself
unique. Distinct families split into distinct $(F_0, F_1)$ pairs, so distinct
families get distinct diagrams. ∎

Executable corollary, as in Module 13: with hash-consing, **family equality
is `Ref` equality**. Stage 1 tests `==` on families built along wildly
different routes; stage 2 turns algebraic *laws* into `assert_eq!` on two
`Ref`s.

### The attractive nuisance

Here is the trap Knuth warns about. In a BDD, $f$ and $\lnot f$ have identical size —
complement is free, and the diagram for "the function is *false* here" costs
the same as for "true." Families are not like that. The family
$F = \{\{i\} : 0 \le i < 64\}$ has a 66-node ZDD (one chain node per variable plus
the sinks — stage 1 pins this). Its *complement family* — all $2^{64} - 64$ other
subsets — is a different family with a different (also smallish, but
structurally unrelated) diagram, and operations natural for functions
(complement, "for all") are unnatural and potentially explosive for
families. The moral: a ZDD is not "a better BDD." It is the right canonical
form when your objects are *sparse set collections* — matchings, covers,
paths, cliques — and the wrong one when they are dense truth tables. Module
13's ordering lesson ($B(f)$ can swing exponentially with variable order)
still applies verbatim to $Z(F)$; on top of it, ZDD size now also tracks
*sparsity*: variables absent from every member cost literally nothing, which
is why $\{\{0\}\}$ is 3 nodes whether the universe has 3 elements or 3 million.

### Counting members: no $2^{\text{skip}}$ factor

Module 13's Algorithm 7.1.4C had to multiply by $2^{(\text{skipped levels})}$, because
a BDD edge that skips a variable leaves it free. A ZDD edge that skips a
variable *fixes* it: absent. So:

**Lemma.** The member sets of $F(f)$ are in bijection with the paths from $f$ to
$\top$ (at each node, HI $\Leftrightarrow$ "the branch variable is in the set").

*Proof.* Given a member set $s$, the walk used by `contains_set` traces a
unique path to $\top$, by induction on the semantics. Conversely a path to $\top$
determines the set of variables at whose nodes it took HI, and that set is a
member, again by the semantics; the two maps invert each other. ∎

Hence `count_sets` is the naked recursion $c(\bot) = 0$, $c(\top) = 1$,
$c(\mathrm{node}) = c(\mathrm{LO}) + c(\mathrm{HI})$ — path counting — memoized to $O(Z(f))$ additions.
When you implement it, notice how pleasant this is compared to the BDD
version: the structure fits the question.

---

## 2. The family algebra

Combinatorics builds families from families; the ZDD's API is that algebra.
Every operation is a memoized top-variable recursion in the style of Module
13's `apply`, but the base cases speak set-language now. Throughout, write
the split of $f$ at its top variable $v$ as $f = f_0 \cup (\{\{v\}\} \sqcup f_1)$, i.e.
$f_0 = F(\mathrm{LO})$, $f_1 = F(\mathrm{HI})$.

**Union, intersection, difference** are what you expect; the only novelty
is what happens when the two arguments test *different* top variables — if
only $f$ tests $v$ ($v$ smaller than $g$'s top), then **no member of $g$ contains
$v$**, and each operation reacts in its own way:

| op | only $f$ tests $v$ | only $g$ tests $v$ | both test $v$ |
|---|---|---|---|
| $f \cup g$ | $\mathrm{mk}(v, f_0 \cup g, f_1)$ | $\mathrm{mk}(v, f \cup g_0, g_1)$ | $\mathrm{mk}(v, f_0 \cup g_0, f_1 \cup g_1)$ |
| $f \cap g$ | $f_0 \cap g$ — $f$'s HI dies wholesale | $f \cap g_0$ | $\mathrm{mk}(v, f_0 \cap g_0, f_1 \cap g_1)$ |
| $f \setminus g$ | $\mathrm{mk}(v, f_0 \setminus g, f_1)$ — $g$ can't cancel $v$-sets | $f \setminus g_0$ — $g$'s HI is idle | $\mathrm{mk}(v, f_0 \setminus g_0, f_1 \setminus g_1)$ |

with trivial cases $\varnothing \cup g = g$, $\varnothing \cap g = \varnothing$, $f \setminus f = \varnothing$, $f \cup f = f \cap f = f$, and
so on. Each `mk` may zero-suppress its result — that is the algebra
maintaining canonicity for free.

**Join** is the interesting one — Knuth's $f \sqcup g$ (Minato's product):

$$f \sqcup g = \{ a \cup b : a \in f,\ b \in g \}.$$

It is the Minkowski-sum idea transplanted to set systems: every way of
combining one choice from $f$ with one from $g$. $\{\varnothing\}$ is its identity (join with
"choose nothing" changes nothing) and $\varnothing$ annihilates (no way to choose from
an empty menu). Derive the recursion by distributing joins over the top-
variable splits — join distributes over $\cup$ directly from its definition:

$$
\begin{aligned}
f \sqcup g &= (f_0 \cup v\cdot f_1) \sqcup (g_0 \cup v\cdot g_1) && \text{writing } v\cdot h \text{ for } \{\{v\}\} \sqcup h \\
&= f_0 \sqcup g_0 \;\cup\; v\cdot(f_1 \sqcup g_0) \;\cup\; v\cdot(f_0 \sqcup g_1) \;\cup\; v\cdot v\cdot(f_1 \sqcup g_1)
\end{aligned}
$$

and $v\cdot v = v$ ($\{v\} \cup \{v\} = \{v\}$: elements don't double), so

$$\mathrm{LO} = f_0 \sqcup g_0, \qquad \mathrm{HI} = (f_1 \sqcup g_1) \cup (f_1 \sqcup g_0) \cup (f_0 \sqcup g_1).$$

When only $f$ tests $v$, the split of $g$ is $(g, \varnothing)$ and this collapses to
$\mathrm{mk}(v, f_0 \sqcup g, f_1 \sqcup g)$. Note that join *calls union* — the memo table is
shared across operations, keyed by an op tag.

Two sanity anchors you will pin in stage 2: $\{\{a\},\{b\}\} \sqcup \{\{c\}\} =
\{\{a,c\},\{b,c\}\}$, and when $f$ and $g$ have disjoint supports, $|f \sqcup g| = |f| \cdot |g|$
(distinct pairs give distinct unions — restrict a union to $f$'s support to
recover $a$). The algebra ($\cup$ as addition, $\sqcup$ as multiplication) is a
commutative semiring with additive identity $\varnothing$ and multiplicative identity
$\{\varnothing\}$; stage 2 verifies the semiring laws *as Ref equalities*, plus the
difference identities $f \setminus (g \cup h) = (f \setminus g) \cap (f \setminus h)$ and
$f \setminus (g \cap h) = (f \setminus g) \cup (f \setminus h)$ — De Morgan with $\complement$ replaced by $\setminus$, which is
the family-friendly way to say "not."

---

## 3. Counting structures in graphs

Now the payoff. To count the matchings of a graph $G = (V, E)$: let the ZDD
variables be the *edge indices* $0, \ldots, m-1$, and build the family of all
matchings — then `count_sets` is the answer. Stage 3 uses a construction
that is nothing but stage-2 algebra:

```text
G1. [Power set.]  P <- ⨆_{e} ({∅} ∪ {{e}}).
                  Each factor says "edge e out, or edge e in"; the join of
                  all m factors is the family of ALL 2^m edge subsets — and
                  its ZDD is a chain of m nodes with LO = HI. (The rule we
                  did NOT adopt earns its keep: a BDD would collapse this
                  chain to ⊤; the ZDD keeps "every variable optional".)
G2. [Filter.]     F <- P. For each pair of edges e ≠ e' sharing a vertex:
                  F <- F \ (P ⊔ {{e, e'}}).
                  P ⊔ {{e,e'}} is every subset containing both e and e', so
                  the difference removes exactly the members violating that
                  one conflict. After all pairs: F = the matchings.
G3. [Count.]      count_sets(F).
```

Independent sets are the same program with vertices as variables and the
*edges themselves* as the conflicting pairs — and the counts must agree
with Module 13's BDD version (same family, different diagram): $P_n$ gives
$F_{n+2}$, $C_n$ gives the Lucas number $L_n$.

### Hand trace: matchings of $P_3$

$P_3$ is 0—1—2 with edges $e_0 = \{0,1\}$, $e_1 = \{1,2\}$; the only conflict pair is
$(e_0, e_1)$ (they share vertex 1).

```text
G1:  P = {∅,{e0}} ⊔ {∅,{e1}} = {∅, {e0}, {e1}, {e0,e1}}
     as a ZDD:  p1 = (e0, p2, p2),  p2 = (e1, ⊤, ⊤)          Z(P) = 3
     (node_count is a reachability sweep: p1, p2, ⊤ — ⊥ is unreachable)
G2:  bad = P ⊔ {{e0,e1}} = {{e0,e1}}   (only one superset exists here)
     F = P \ bad = {∅, {e0}, {e1}}
     as a ZDD:  f = (e0, (e1, ⊤, ⊤), ⊤)
       — check the split: members without e0: {∅,{e1}}; with e0: {∅}. ✓
G3:  c(⊤) = 1;  c((e1,⊤,⊤)) = 1 + 1 = 2;  c(f) = 2 + 1 = 3.
```

Three matchings of $P_3$ — and $3 = F_4$. In general, condition on the last edge
of $P_n$: unused $\Rightarrow$ a matching of $P_{n-1}$; used $\Rightarrow$ its neighbor edge is banned,
leaving a matching of $P_{n-2}$. So $m(P_n) = m(P_{n-1}) + m(P_{n-2})$ with
$m(P_1) = 1$, $m(P_2) = 2$: the Fibonacci numbers, $m(P_n) = F_{n+1}$. For cycles,
condition on one fixed edge to break the ring: $m(C_n) = m(P_{n-1}) +
m(P_{n-3}) \cdot 1 + \cdots$ — the tidy result is $m(C_n) = L_n$, the Lucas numbers, which
stage 3 pins to $n = 18$. For complete graphs, $m(K_n)$ is the **telephone
number** $T(n)$ (= number of involutions of $n$ elements: a matching of $K_n$ is
exactly a way to pair up some subscribers): 1, 1, 2, 4, 10, 26, 76, 232 for
$n \le 7$, with the lovely recurrence $T(n) = T(n-1) + (n-1) \cdot T(n-2)$ — either the
new subscriber stays silent or picks one of $n-1$ partners.

A remark on method: the G1–G3 construction is quadratic in conflicts and
fine for this module's sizes. The industrial-strength technique — Knuth's
*Simpath* and its descendants — sweeps the graph edge by edge keeping a
*frontier* of partial-state equivalence classes, building the ZDD directly
in one pass; that is how people count $10^{60}$ paths through grid graphs. Same
diagram at the end (canonicity!), very different route. Exercise
7.1.4-(frontier) points you there.

---

## 4. Exact cover with colors (Algorithm 7.2.2.1C)

Module 09's Algorithm X solves exact cover: choose options so every item is
covered exactly once. Two upgrades turn it into Knuth's XCC solver, the
workhorse of Vol. 4B §7.2.2.1:

- **Secondary items** are covered *at most* once. (Primary items: exactly
  once, as before; the solver only ever branches on primary items.)
- **Colors.** Each appearance of a secondary item in an option carries a
  color. Two options sharing a secondary item are compatible **iff they
  assign it the same color**. Uncolored sharing (Module 09 semantics) is
  the special case "every appearance a fresh color"; full agreement is the
  case "one color per option pair" — colors interpolate between "conflict"
  and "agree."

Why this is the right primitive: think of a crossword. "Word w goes into
across-slot a" is an option; the cells are shared between across and down
slots. Two words crossing at a cell don't *conflict* — they must *agree on
the letter*. Make cells secondary items and letters colors, and the puzzle
is XCC verbatim (stage 4's word-pair test is a $2 \times 2$ miniature). Without
colors you would need an item per (cell, letter) pair and a blowup of
options; with colors the structure does the agreeing.

### The data structure

Module 09's toroidal links, plus one array. Nodes carry `L/R/U/D`, their
item header, and now `color`:

- primary node: `color = 0`;
- secondary node with user color c: `color = c + 1 > 0`;
- **purified** node: `color = −1` — "my item has already been fixed to my
  color by an option in the current partial solution; I cost nothing now."

The root's horizontal ring threads the *primary* headers only, so the
choose-an-item loop never proposes a secondary item (Knuth keeps secondary
items beyond a boundary $N_1$ in his item array — same idea, different
representation). Secondary headers still own vertical lists.

### The choreography

`hide`/`unhide` are Module 09's row removal with one new guard — skip
purified nodes:

```text
hide(p):   for q ≠ p in p's option row (rightward):
               if color[q] >= 0:  unlink q vertically; size[item(q)] -= 1
unhide(p): the same walk leftward, relinking under the same guard.
```

`cover`/`uncover` are unchanged (they call hide/unhide). The new operations
fix a secondary item's color:

```text
purify(p):       c <- color[p]; i <- item(p)          (c > 0)
                 for q in i's vertical list (downward):
                     if color[q] = c:  if q ≠ p: color[q] <- -1   (mark)
                     else:             hide(q)                    (kill)
unpurify(p):     the same walk upward: marks (-1) become c again;
                 others (≠ p) are unhidden.
commit(j):       color[j] = 0 -> cover(item(j));  > 0 -> purify(j);
                 -1 -> DO NOTHING (already purified: nothing to do,
                 and — the subtle half — nothing to undo later).
uncommit(j):     the mirror image.
```

And the search is Algorithm X with commit in place of cover:

```text
C1. [Initialize.]   Build the linked structure from the options.
C2. [Solved?]       If the root ring is empty, visit the solution
                    (the options of the current partial), then back up.
C3. [Choose i.]     An active primary item, minimum size (MRV).
C4. [Cover i.]      cover(i).
C5. [Try r.]        For each option r in i's list, top to bottom:
                    commit(j) for each other node j of r's row, L to R;
                    recurse;
C6. [Retract r.]    uncommit(j) for each j, R to L; then
C7. [Next r.]       advance r; when the list is exhausted,
C8. [Backtrack.]    uncover(i) and back up.
```

### Why purify is exactly right — the pointer discipline

Three facts carry the correctness proof; each is worth checking against
your own code line by line.

1. **Different colors really die.** For every option q in the item's list
   whose color differs, `purify` calls hide(q), which unlinks all of q's
   option's *other* nodes from their lists — in particular from every
   primary item's list — so step C5 can never choose that option again.
   (Its node in the purified item's own list stays linked, harmlessly:
   that list is never branched on, and only walked again by the matching
   unpurify.) Compatible options survive untouched except for the −1 mark.
2. **No item is purified twice.** A second purify of item i could only be
   triggered by committing a node j in i's list with color[j] > 0. After
   the first purify, every node still linked in i's list is either marked
   −1 (same color — and commit does nothing on −1) or belongs to a
   different-color option that is unreachable by fact 1. So the marks set
   by a purify are erased by exactly one unpurify: the state machine per
   node is $c \leftrightarrow -1$, never deeper.
3. **LIFO makes undo exact.** hide skips −1 nodes; between that hide and
   its matching unhide, the node's mark cannot change (its column's
   unpurify strictly encloses or is strictly enclosed by the hide/unhide
   pair, by the stack discipline of C5/C6). So unhide's guard sees exactly
   what hide's guard saw, and relinks exactly what was unlinked — Module
   09's "dancing" argument (a removed node still points at its neighbors)
   plus a one-bit invariant. Stage 4 tests this the honest way: solve
   twice, demand identical output.

Hand trace of the unit case (stage 4's first color test). Options
A = {$p_0$; s:R}, B = {$p_1$; s:R}: C3 picks $p_0$; C4 covers it; C5 takes A and
commits its s-node → purify: B's s-node has color R → marked −1. Recurse:
C3 picks $p_1$, whose list still contains B; committing B's s-node is a no-op
(−1). Root ring empty: solution $\{A, B\}$. Now replay with B' = {$p_1$; s:G}:
purify sees $G \ne R$ → hide(B'), which unlinks B' from *$p_1$'s* list; at the
next level $p_1$ has an empty list — dead end, zero solutions. Same-color
compatible, different-color not: the semantics fell out of the pointer
dance.

### Latin squares, the canonical XCC citizen

An $n \times n$ **Latin square** fills each cell with one of $n$ symbols so every row
and every column contains each symbol exactly once. As exact cover: items
cell(r,c), row-sym(r,s), col-sym(c,s) — all primary, $3n^2$ of them; the
option "s into (r,c)" covers one of each. A Latin square *is* an exact
cover: cell items force one symbol per cell, row/col items force the Latin
conditions. Order 3 has exactly 12 squares (stage 4 pins it), and *partial*
squares — some cells given — just drop the options that contradict a given,
which is how stage 4's uniqueness-of-completion test works. (Completing a
general partial Latin square is NP-complete, so the fact that XCC handles
instances so casually says something about how good MRV + dancing links
is on structured problems. Sudoku, Module 09's finale, is the case $n = 9$
with box constraints added — and with colors you can go further: KenKen,
word rectangles, graceful labelings; §7.2.2.1 is a bestiary.)

### The ZDD ↔ XCC bridge

The two halves of this module are one subject. The set of *solutions* to an
exact cover problem is a family of sets (each solution = a set of option
indices) — and Knuth points out that this family is naturally a ZDD: run
the XCC search but *memoize on the subproblem* (the set of still-active
items), and identical subproblems become shared ZDD nodes; the diagram of
all solutions comes out, ready for `count_sets`, uniform sampling, or "best
solution under linear weights" in time $O(Z)$. Where Module 09's solver
visits solutions one at a time, the ZDD-building variant can return $10^{15}$ of
them as a million-node diagram. That is the sense in which dancing links
and ZDDs are two faces of the same idea, and it is why they share a chapter
of this course.

---

## 5. Why it's done this way

- **Why a new reduction rule instead of a sparse encoding on BDDs?**
  Because canonical forms only pay off when the *default* matches the
  *data*. BDD deletion defaults a variable to "free" — right for functions,
  catastrophic for families where almost everything is absent. ZDD deletion
  defaults to "absent," so cost tracks the support of the family, not the
  size of the universe. One changed rule, same canonicity proof shape, same
  hash-consing machinery — maximal reuse, minimal concept.
- **Why an algebra instead of bespoke constructions?** union/diff/join are
  memoized O(size-product) primitives with *proved* semantics; anything you
  bolt together from them is correct by construction (stage 3 builds a
  graph counter out of four algebra calls). This is the BDD-apply lesson
  from Module 13 again: implement recursion + memo once, get a whole
  calculus.
- **Why colors instead of more items?** You could encode "agree on the
  letter" by one secondary item per (cell, letter) pair; the option table
  multiplies, and the solver can't see that the choices are exclusive.
  Colors keep the *item* structure (one item per cell) and move agreement
  into a constant-time check inside operations the solver already performs.
  It is the same taste that chose doubly linked lists over copying in
  Module 09: don't re-derive state, annotate it reversibly.
- **Why exactly the −1 marker?** Purify must make same-color nodes free
  *without unlinking them* (their options remain live!), and undo must know
  precisely which nodes to restore. One reserved value in a field that
  already exists gives both, and keeps hide/unhide symmetric under a single
  `>= 0` guard. Count the writes: commit and uncommit touch exactly the
  same pointers in opposite orders — the O(1)-undo discipline that makes
  dancing links dance is preserved bit for bit.

## 6. In the real world

- **Graphillion** (Inoue–Iwashita–Kawahara–Minato) is a production Python
  library whose entire data model is ZDDs over edge sets — "GraphSet"
  objects holding astronomically many subgraphs, built by frontier-based
  search (§3's industrial cousin). Its flagship applications are the ones
  in this module's spirit: **power-grid switching** — enumerating all
  feasible switch configurations of a distribution network (each
  configuration a spanning structure; the ZDD holds every loss-minimizing
  or restoration candidate, so operators optimize over *all* of them, with
  guarantees, instead of heuristically sampling), deployed in research with
  Japanese utilities — and **rail-route enumeration**, the famous
  demonstrations counting every simple path through Japan's railway network
  (and the viral "counting paths on a grid" video that made $10^{60}$ a pop-science
  number). Same `count_sets`, `join`, `diff` you wrote, at scale.
- **Knuth's own XCC solvers** (`DLX2`, `SSXCC`, `XCC` with sharp
  heuristics, all on his site as literate CWEB programs) are used by
  researchers in **combinatorial design theory** — packing and covering
  designs, Latin-square and orthogonal-array completion, polyomino and
  polycube packing, Costas-array style placement problems — because for
  these structured, exact problems a tuned XCC search still beats generic
  SAT encodings often enough to matter. A noticeable slice of recent
  design-theory papers report "found by Knuth's dancing-links solver."
- **EDA (electronic design automation)** is where ZDDs were born: Minato
  introduced them for **cube-set representation** in two-level logic
  minimization — a sum-of-products is a sparse family of cubes, exactly the
  ZDD sweet spot — and Coudert & Madre's implicit prime-cover machinery
  (the *scherzo* line of covering solvers) made "compute ALL prime
  implicants, then solve the covering problem, without listing either"
  practical. CUDD ships a full ZDD API alongside BDDs for this reason, and
  unate-covering/irredundant-cover steps inside synthesis flows still speak
  ZDD today.

## 7. Stage-by-stage lab guide

Open `labs/module-17-zdd-xcc/src/lab.rs`; each stage has a test file
`tests/stage_NN_*.rs`, and `./grade 17` runs them in order.

### Stage 1 — the `Zdd` arena: sinks, `single`, queries, `union`

Copy Module 13's skeleton (arena `Vec<Node>`, sinks first with sentinel
var `u32::MAX`, unique table) and change **one line**: `mk` returns `lo`
when `hi == ⊥`, and does *not* collapse `lo == hi`. Get the two sinks'
meanings straight before anything else (the first test is merciless about
$\varnothing$ vs $\{\varnothing\}$). `count_sets` is the LO+HI recursion; `contains_set` is one walk
with the skip-means-absent rule; `sets` is a prefix DFS (sort the output);
`node_count` a reachability sweep. You also implement `union` here — the
tests need it to assemble any family with more than one member, and it is
the gentlest of the four recursions. The arena audit sweeps *every* node
you ever created and checks `hi != ⊥`: if it fails, your `mk` has a leak
(commonly: building nodes directly somewhere instead of through `mk`).

### Stage 2 — the family algebra

`intersect`, `diff`, `join`, memoized (one shared `HashMap` keyed by an op
tag is fine; only `diff`'s key is order-sensitive). Derive each recursion
from the top-variable split *before* coding — §2's table is the answer
key, but the vf < vg cases are where hand-derivation pays. The tests check
semiring laws as Ref equality on LCG-random families and mirror every op
against `HashSet<BTreeSet<u32>>` brute force for $n \le 8$; if a law fails but
the mirror agrees, your memo key is conflating something (classic bug:
using a commutative key for `diff`).

### Stage 3 — `matchings_zdd`, `independent_sets_zdd`

Pure clients of stages 1–2: build the power set with `join` (G1), filter
conflicts with `diff` of a `join` (G2), count (G3) — trace $P_3$ by hand
first (§3). Get the conflict list right: for matchings the *variables are
edges* and conflicts are edge pairs sharing an endpoint; for independent
sets variables are vertices and conflicts are the edges. The tests pin
Fibonacci, Lucas, telephone numbers, brute-force tiny graphs, and the $4 \times 4$
grid (1234 independent sets, verified against all 65536 subsets).

### Stage 4 — `Xcc`

Start from your Module 09 `ExactCover` and make four changes in order:
(1) constructor takes `(n_primary, n_secondary)` and only rings primary
headers with the root; (2) nodes get a `color` field (0 / c+1 / −1) and
`add_option` takes `(&[usize], &[(usize, u32)])`; (3) `hide`/`unhide` get
the `color >= 0` guard; (4) write `purify`/`unpurify`/`commit`/`uncommit`
and substitute commit/uncommit into the search's C5/C6. Respect the
walking directions religiously — down/up, right/left — reversibility *is*
the algorithm. The tests replay Module 09's example (no colors), probe the
color semantics in isolation, count Latin squares (12), complete a partial
one uniquely, and solve the word-pair grid (7 solutions); nearly every
test solves twice to catch broken restoration.

## 8. Check your understanding

1. What family does the one-node diagram `(x3, ⊤, ⊤)` denote, and why
   would this node be illegal in Module 13's BDD? ($\{\varnothing, \{3\}\}$; a BDD elides
   any node with lo = hi.)
2. `mk(5, f, empty())` returns… what, and which theorem needs that
   behavior? (Returns $f$; uniqueness of the canonical form — the $F_1 = \varnothing$
   case of the induction.)
3. Why does `count_sets` have no $2^{\text{skip}}$ factor where Module 13's
   `count_models` did? (ZDD paths to $\top$ biject with member sets; a skipped
   variable is *absent*, one possibility, not free, two.)
4. In `purify`, same-color nodes are marked −1 but **stay linked** while
   different-color options are hidden. What breaks if you unlink the
   same-color nodes too? (Their options must remain choosable — they're
   compatible! You would also have nothing sensible for unpurify to
   restore, since commit on them later is a no-op precisely because of
   the mark.)
5. An option consisting only of secondary items can never be part of a
   reported solution. Where in the algorithm is that visible? (C3 branches
   only on primary items, so such an option is never chosen; it can still
   *constrain* others through purify/hide.)

## 9. Exercises from the text

Ratings are Knuth's scale (00 immediate · 10 a minute · 20 up to an hour ·
30 hours · 40 term project). ▶ marks especially instructive ones. Exercise
numbers are content tags (match by content in your printing, as in Module
13). Log attempts in `course/module-17-zdd-xcc/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 7.1.4-(z-vs-b) | 18 | Draw both the BDD and the ZDD of the family $\{\{1\},\{2\},\{3\}\}$ over universe $\{1,\ldots,6\}$. Which nodes does each elide, and why? |
| ▶7.1.4-(canon-z) | 25 | Write out §1's canonicity induction fully for $n = 2$: list all 16 families over $\{0,1\}$ and their unique diagrams. |
| 7.1.4-(count-z) | 20 | Prove the path–member bijection lemma carefully, including why distinct members give distinct paths. |
| ▶7.1.4-(join-alg) | 25 | Prove that (families, $\cup$, $\sqcup$) is a commutative semiring. Then investigate size: show $\lvert f \sqcup g\rvert = \lvert f\rvert \cdot \lvert g\rvert$ for disjoint supports, and construct $f$, $g$ over a shared support whose join has far more members than $Z(f) + Z(g)$ nodes would suggest. |
| 7.1.4-(univ) | 22 | Design `restrict_with(f, v)` / `restrict_without(f, v)` (members containing / not containing v) as one-pass recursions and prove them via the split. |
| ▶7.1.4-(frontier) | 35 | Implement Simpath-style frontier construction for s–t simple paths in a grid and check small counts against brute force. |
| 7.2.2.1-(latin4) | 22 | Extend stage 4's encoding to order 4 and confirm there are 576 Latin squares. |
| ▶7.2.2.1-(undo) | 25 | Prove fact 3 of §4 (LIFO exactness) by induction on the recursion tree: state the invariant relating marks to the stack of active purifies. |
| 7.2.2.1-(mrv) | 20 | Knuth branches on primary items only. Show that branching on a secondary item can be *wrong* (a solution may leave it uncovered), not merely slow. |
| ▶7.2.2.1-(zdd-xcc) | 40 | Build the §4 bridge: modify your XCC search to memoize on the set of active items and emit the solution family as a ZDD; validate `count_sets` against `count_solutions` on the module's instances. |

## 10. Proof techniques you practiced

- **Canonicity by structural induction** — the ZDD canonical-form theorem
  is Module 13's proof with one case swapped; seeing which case changes
  ($F_1 = \varnothing$ ↔ the reduction rule) is the technique: *a canonical form is a
  bijection theorem, and each reduction rule discharges exactly one
  non-uniqueness*.
- **Bijective counting** — members ↔ $\top$-paths turned `count_sets` into path
  counting and *deleted* a factor the BDD version needed; stage 3 then
  counted matchings by recurrence (condition on the last edge) and had the
  diagram confirm Fibonacci/Lucas/telephone numbers exactly.
- **Algebraic law as executable identity** — canonicity converts semiring
  laws into `Ref` equalities checked over random families: proof by
  structure plus adversarial testing, each covering the other's blind spot.
- **Reversibility invariants** — XCC correctness rests on "undo sees what
  do saw": a one-bit state machine per node ($c \leftrightarrow -1$) nested inside a LIFO
  discipline. This is Module 09's dancing argument strengthened with state,
  and it is how you prove any incremental search structure honest.
- **Reduction between formalisms** — Latin squares → exact cover,
  crosswords → XCC with colors, and solution-sets → ZDDs: each reduction
  was validated by counting the same thing in two systems (12 squares,
  7 word grids, F/L/T numbers vs Module 13).

## 11. Where this leads

- **Vol. 4B, §7.2.2.1 at full depth:** Knuth's production XCC solvers add
  option ordering, sharp preference heuristics, and *Dancing with ZDDs* —
  the bridge of §4 taken seriously.
- **§7.2.2.2 (SAT):** when constraints stop being "exactly once,"
  cover-style search hands over to CDCL — Modules 10 and 14 — and the
  trade-off between structured (XCC) and clausal (SAT) encodings is a live
  research topic you now have both vocabularies for.
- **Frontier-based construction** (Simpath and Graphillion's engine) is
  the scalable path from stage 3's algebra to $10^{60}$-object families.
- **Module 18 (MMIX)** closes the course at the other end of the
  abstraction stack: the machine all of these pointer dances ultimately
  run on.
