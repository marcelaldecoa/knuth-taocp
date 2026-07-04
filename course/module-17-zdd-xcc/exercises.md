# Exercises — Module 17 (ZDDs and Exact Cover with Colors)

Self-contained problems on this module's material — the zero-suppressed
decision diagram and its canonical form, the family algebra
($\cup$, $\cap$, $\setminus$, $\sqcup$), counting structures in graphs, and Algorithm
7.2.2.1C (dancing links with colors). You can work every one **without the
books**: each states the problem in full, gives a **hint** to peek at when
stuck, and a worked **answer sketch** to check against after you try.
Computational answers here are reproduced by the code you write in the lab (or
a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Exercise IDs are the module's content tags (match by content in
your printing of §7.1.4 / §7.2.2.1).

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | BDD vs. ZDD of $\{\{1\},\{2\},\{3\}\}$: which nodes each elides | 18 | ⬜ |
| 2 | ▶ Canonicity for $n=2$: all 16 families over $\{0,1\}$ | 25 | ⬜ |
| 3 | Member $\leftrightarrow$ $\top$-path bijection, carefully | 20 | ⬜ |
| 4 | ▶ The semiring laws, and a join blowup family | 25 | ⬜ |
| 5 | `restrict_with` / `restrict_without`, proved via the split | 22 | ⬜ |
| 6 | ▶ Simpath-style frontier ZDD for $s$–$t$ paths | 35 | ⬜ |
| 7 | Order-4 Latin squares: pin the count | 22 | ⬜ |
| 8 | ▶ LIFO exactness of purify/unpurify, by induction | 25 | ⬜ |
| 9 | Why branching on a secondary item is *wrong* | 20 | ⬜ |
| 10 | ▶ Emit the XCC solution family as a ZDD | 40 | ⬜ |

## Problems

### 1. BDD vs. ZDD of $\{\{1\},\{2\},\{3\}\}$: which nodes each elides (rating 18 · §7.1.4-(z-vs-b))

**Problem.** Take the family $F = \{\{1\},\{2\},\{3\}\}$ over the universe
$\{1,2,3,4,5,6\}$, with variable order $x_1 < x_2 < \cdots < x_6$. Draw both the
reduced ordered **BDD** (Module 13's rule: elide any node with LO $=$ HI) and
the **ZDD** (Module 17's rule: never create a node with HI $= \bot$, keep
LO $=$ HI). For each diagram say which nodes are elided and why, and count the
nodes.

**Hint.** As a *function*, $F$ is true only at the three points $\{1\}$,
$\{2\}$, $\{3\}$ — and at each of those points $x_4 = x_5 = x_6 = 0$. Ask, for
each structure: does variable $x_5$ *matter*, and does the structure's deletion
rule mean "$x_5$ is don't-care" or "$x_5$ is absent"?

**Answer sketch.** The ZDD splits $F$ on $x_1$: members without $1$ are
$\{\{2\},\{3\}\}$, members with $1$ (with $1$ removed) are $\{\varnothing\}$.
Recursing gives the chain

$$
(x_1,\ (x_2,\ (x_3,\bot,\top),\ \top),\ \top),
$$

three branch nodes plus the two sinks — $Z(F) = 5$. Variables $x_4, x_5, x_6$
never appear: they are absent from every member, and under zero-suppression a
*skipped* variable defaults to "absent," so they cost literally nothing. The
BDD cannot elide them — flipping $x_5$ from $0$ to $1$ turns a member into a
non-member, so $x_5$ genuinely *matters* and must be branched on to force it to
$0$. Building the reduced BDD of the same function over all six variables gives
**10 nodes** (it elides only genuine LO $=$ HI nodes, none of which are the
$x_4, x_5, x_6$ tests). The moral of §1: for a sparse family the ZDD pays for
support, not universe; the BDD pays for the universe.

### 2. ▶ Canonicity for $n = 2$: all 16 families over $\{0,1\}$ (rating 25 · §7.1.4-(canon-z))

**Problem.** Instantiate §1's canonical-form induction at $n = 2$. There are
exactly $16$ families of subsets of $\{0,1\}$; write each one's unique reduced
zero-suppressed diagram, and confirm the diagrams are pairwise distinct.

**Hint.** The universe $\{0,1\}$ has $4$ subsets ($\varnothing, \{0\}, \{1\},
\{0,1\}$), so a family is any subset of that $4$-element powerset: $2^{4} = 16$
of them. Build each with the split rule $F = F_0 \cup (\{\{0\}\} \sqcup F_1)$,
where $F_0$ = members without $0$ and $F_1$ = members with $0$, $0$ removed.

**Answer sketch.** $2^{2^{2}} = 16$ families, each denoted by exactly one
diagram (write $(v, \mathrm{LO}, \mathrm{HI})$; $\bot, \top$ are the sinks):

| family | diagram |
|---|---|
| $\varnothing$ | $\bot$ |
| $\{\varnothing\}$ | $\top$ |
| $\{\{0\}\}$ | $(0,\bot,\top)$ |
| $\{\{1\}\}$ | $(1,\bot,\top)$ |
| $\{\{0,1\}\}$ | $(0,\bot,(1,\bot,\top))$ |
| $\{\varnothing,\{0\}\}$ | $(0,\top,\top)$ |
| $\{\varnothing,\{1\}\}$ | $(1,\top,\top)$ |
| $\{\varnothing,\{0,1\}\}$ | $(0,\top,(1,\bot,\top))$ |
| $\{\{0\},\{1\}\}$ | $(0,(1,\bot,\top),\top)$ |
| $\{\{0\},\{0,1\}\}$ | $(0,\bot,(1,\top,\top))$ |
| $\{\{1\},\{0,1\}\}$ | $(0,(1,\bot,\top),(1,\bot,\top))$ |
| $\{\varnothing,\{0\},\{1\}\}$ | $(0,(1,\top,\top),\top)$ |
| $\{\varnothing,\{0\},\{0,1\}\}$ | $(0,\top,(1,\top,\top))$ |
| $\{\varnothing,\{1\},\{0,1\}\}$ | $(0,(1,\top,\top),(1,\bot,\top))$ |
| $\{\{0\},\{1\},\{0,1\}\}$ | $(0,(1,\bot,\top),(1,\top,\top))$ |
| $\{\varnothing,\{0\},\{1\},\{0,1\}\}$ | $(0,(1,\top,\top),(1,\top,\top))$ |

The two sinks handle the $n = 0$ base case ($\varnothing$ vs. $\{\varnothing\}$,
distinct); every other family has a root $0$-node whose LO denotes $F_0$ and HI
denotes $F_1$, unique by induction, unique by the hash table. Distinct families
give distinct $(F_0, F_1)$ splits, hence distinct diagrams — exactly the
theorem. Note $\{\{1\},\{0,1\}\}$ keeps a LO $=$ HI node $(0,(1,\bot,\top),
(1,\bot,\top))$: "$0$ is optional," the very information a BDD would throw away.

### 3. Member $\leftrightarrow$ $\top$-path bijection, carefully (rating 20 · §7.1.4-(count-z))

**Problem.** Prove §1's lemma: the member sets of the family $F(f)$ are in
bijection with the directed paths from node $f$ to the $\top$ sink, where a path
is read by recording, at each node it passes through, whether it took the HI
edge (branch variable is *in* the set) or the LO edge (variable *not* in the
set). Include the argument that distinct members give distinct paths, and
conclude that `count_sets` is $c(\bot) = 0$, $c(\top) = 1$, $c(\text{node}) =
c(\mathrm{LO}) + c(\mathrm{HI})$.

**Hint.** Induct on the diagram structure using the defining semantics
$F(v,l,h) = F(l) \cup \{s \cup \{v\} : s \in F(h)\}$. The walk that
`contains_set` performs is the forward map; reading off the HI-choices along a
path is the inverse.

**Answer sketch.** *Forward.* Given $s \in F(f)$, run `contains_set`: at node
$(v,l,h)$, take HI iff $v \in s$. By the semantics this reaches $\top$ (removing
$v$ when present keeps a member of the child family), tracing one path. *Back.*
Given a path to $\top$, let $s$ be the set of branch variables at which it took
HI; by the same semantics, walking that path certifies $s \in F(f)$. *Injective.*
If $s \ne s'$, they differ at some smallest variable $v$; because variables are
tested in increasing order and each is tested at most once on any path, the two
walks agree until the node testing $v$, where one takes HI and the other LO —
so the paths diverge and are distinct. The two maps invert each other, giving a
bijection. Since the paths through a node split disjointly into those through
LO and those through HI, the number of members satisfies $c(\text{node}) =
c(\mathrm{LO}) + c(\mathrm{HI})$ with $c(\bot) = 0$, $c(\top) = 1$; memoized it
is $O(Z(f))$ additions — no $2^{\text{skip}}$ factor, because a skipped variable
is *absent* (one possibility), not free (two).

### 4. ▶ The semiring laws, and a join blowup family (rating M25 · §7.1.4-(join-alg))

**Problem.** (a) Show that families of sets, with union $\cup$ as addition and
join $\sqcup$ as multiplication, form a **commutative semiring**: $\cup$ is
commutative, associative, with identity $\varnothing$; $\sqcup$ is commutative,
associative, with identity $\{\varnothing\}$; $\sqcup$ distributes over $\cup$;
and $\varnothing$ annihilates ($\varnothing \sqcup g = \varnothing$). (b) Show
$|f \sqcup g| = |f| \cdot |g|$ when $f$ and $g$ have disjoint supports. (c)
Exhibit $f, g$ over a *shared* support whose join has vastly more members than
the node counts $Z(f), Z(g)$ would suggest.

**Hint.** For (a) work from the definition $f \sqcup g = \{a \cup b : a \in f,
\ b \in g\}$ directly — distributivity over $\cup$ is immediate from it. For (b)
the disjoint supports let you recover $a$ from $a \cup b$ by restricting to
$f$'s support, so the map $(a,b) \mapsto a \cup b$ is injective. For (c) recall
§3's power-set construction $P = \bigsqcup_e (\{\varnothing\} \cup \{\{e\}\})$.

**Answer sketch.** (a) Commutativity/associativity of $\cup$ are set facts;
$f \cup \varnothing = f$. For $\sqcup$: $a \cup b = b \cup a$ gives commutativity;
$(a \cup b) \cup c$ associativity; $\{\varnothing\} \sqcup g = \{\varnothing \cup
b : b \in g\} = g$ is the identity; $\varnothing \sqcup g = \varnothing$ (no $a$
to choose) annihilates. Distributivity: $f \sqcup (g \cup h) = \{a \cup b : a
\in f,\ b \in g \cup h\} = (f \sqcup g) \cup (f \sqcup h)$. So it is a
commutative semiring. (b) If supports are disjoint, $a \cup b$ determines $a$
(intersect with $f$'s support) and $b$, so $(a,b) \mapsto a \cup b$ is a
bijection onto $f \sqcup g$: $|f \sqcup g| = |f| \cdot |g|$. (c) Take $f = g = P$,
the power set of $\{0,\ldots,n-1\}$. Its ZDD is a chain of $n$ nodes with LO $=$
HI (so $Z(P) = n + 2$ with sinks), yet $|P| = 2^{n}$. And $P \sqcup P = P$ (the
power set is closed under union and contains $\varnothing$), so a join of two
$(n{+}2)$-node diagrams denotes $2^{n}$ members — verified with `count_sets`.
The lesson of §1's "attractive nuisance": member count and node count are
uncoupled; the ZDD is small exactly when the family is *structured*, not when it
is small.

### 5. `restrict_with` / `restrict_without`, proved via the split (rating 22 · §7.1.4-(univ))

**Problem.** Design two one-pass memoized recursions:
`restrict_with(f, v)` returning the subfamily of members that *contain* $v$, and
`restrict_without(f, v)` returning those that do *not*. Give the recursion in
terms of the top-variable split and prove correctness. (Both return families
over the original universe — `restrict_with` keeps $v$ in each surviving member.)

**Hint.** Compare $f$'s top variable $u$ with the target $v$. Three cases:
$u < v$, $u = v$, $u > v$. Under zero-suppression, "$f$'s top is already past
$v$" tells you something definite about whether any member contains $v$.

**Answer sketch.** Write $f = (u, f_0, f_1)$ at its top variable $u$, so $f_0 =
F(\mathrm{LO})$ (members without $u$) and $f_1 = F(\mathrm{HI})$ (members with
$u$, $u$ removed). For **`restrict_without(f, v)`**: if $u < v$, recurse into
both children — $\mathrm{mk}(u, \text{rw}(f_0,v), \text{rw}(f_1,v))$; if $u = v$,
every member through HI contains $v$, so drop it — return $f_0$; if $u > v$ (or
$f \in \{\bot,\top\}$), $v$ is below the top, so *no* member contains it and all
survive — return $f$. For **`restrict_with(f, v)`**: if $u < v$, recurse
$\mathrm{mk}(u, \text{rW}(f_0,v), \text{rW}(f_1,v))$; if $u = v$, keep only the
HI side but reattach $v$ — return $\mathrm{mk}(v, \bot, f_1)$; if $u > v$ or a
sink, *no* member contains $v$ — return $\bot$. Correctness is the split
semantics case by case: at $u = v$ the HI branch is exactly "members containing
$v$." Both are $O(Z(f))$ under memoization. (Sanity: `restrict_with(f,v)` and
`restrict_without(f,v)` are disjoint and union to $f$.)

### 6. ▶ Simpath-style frontier ZDD for $s$–$t$ paths (rating 35 · §7.1.4-(frontier))

**Problem.** The §3 algebra (power set, then filter conflicts) is quadratic in
constraints. Implement instead a **frontier-based** construction: sweep the
edges of a grid graph one at a time, maintaining a *frontier* of partial-state
equivalence classes (each vertex on the boundary tracked by its current degree
and connected-component mate), and build the ZDD of all simple $s$–$t$ paths
directly in one pass. Validate the member count against a brute-force
path-enumeration on small grids.

**Hint.** Process edges in a fixed order. The frontier is the set of vertices
that have an already-processed incident edge and an unprocessed one; for each
you need its path-degree ($0$, $1$, or $2$) and, via a union-find "mate" array,
which frontier endpoint it is connected to. When an edge decision would create a
degree-$3$ vertex, a cycle, or strand $s$/$t$, route that branch to $\bot$; when
a vertex leaves the frontier in a bad state, prune. Merge equivalent frontier
states to share nodes.

**Answer sketch.** This is Knuth's Algorithm S (Simpath) specialized to $s$–$t$
paths. Correctness rests on two facts: (i) the frontier state is a *sufficient
statistic* — two partial edge selections with the same frontier degrees and
mate-structure extend to exactly the same completions, so merging them loses
nothing; (ii) each edge contributes one ZDD level (variable = that edge), and
the canonical-form theorem guarantees that whatever order you build in, the
finished diagram is the *same* one the §3 algebra would produce — canonicity is
route-independent. Validation: for the $2 \times 2$ grid ($s$, $t$ opposite
corners) enumerate all edge subsets by brute force, keep those that form a
simple $s$–$t$ path, and check the total equals `count_sets` of the frontier
ZDD; repeat on $3 \times 3$. Because the frontier width, not the constraint
count, bounds the work, this is the technique that scales to the $10^{60}$-path
grids of §6 — the same finished diagram, a very different route. (This is an
open-ended mini-project; grade yourself by the brute-force agreement on small
grids and by never materializing the power set.)

### 7. Order-4 Latin squares: pin the count (rating 22 · §7.2.2.1-(latin4))

**Problem.** Stage 4 encodes an $n \times n$ Latin square as exact cover with
primary items cell$(r,c)$, row-sym$(r,s)$, col-sym$(c,s)$ — $3n^2$ items — and
options "symbol $s$ into cell $(r,c)$." It pins order 3 to $12$ squares. Extend
the encoding to order 4 and confirm your XCC solver counts exactly **576** Latin
squares.

**Hint.** Nothing about the encoding changes except $n = 4$: $3 \cdot 16 = 48$
primary items and $n^3 = 64$ options (one per $(r,c,s)$), each covering exactly
one cell item, one row-sym item, one col-sym item. `count_solutions` should
return $576$.

**Answer sketch.** With $n = 4$ there are $48$ primary items and $64$ options;
the solver reports **576** completed squares — the known value of $L_4$ (the
number of Latin squares of order 4), verified by an independent
backtracking counter. The point Knuth makes in §4 is that a Latin square *is* an
exact cover: the cell items force one symbol per cell, and the row-sym / col-sym
items force each symbol once per line. That completing a general partial Latin
square is NP-complete, yet MRV + dancing links handles order 4 (and far beyond)
without breaking a sweat, is testimony to how well the choose-minimum-size
heuristic exploits structure. (Cross-check ladder: $L_1 = 1$, $L_2 = 2$,
$L_3 = 12$, $L_4 = 576$.)

### 8. ▶ LIFO exactness of purify/unpurify, by induction (rating 25 · §7.2.2.1-(undo))

**Problem.** Prove fact 3 of §4: in Algorithm 7.2.2.1C, `unhide` restores
*exactly* the nodes that the matching `hide` unlinked, even though `hide`/`unhide`
skip purified ($\text{color} = -1$) nodes under a single `color >= 0` guard.
State the invariant relating a node's mark to the stack of active `purify`
operations, and induct on the recursion tree.

**Hint.** The only way `hide` and `unhide` can disagree is if some node's color
flips between $-1$ and its true color $c$ *in between* them. Which operation is
the only one that writes those marks, and how is its lifetime nested relative to
a `hide`/`unhide` pair by the C5/C6 stack discipline?

**Answer sketch.** *Invariant.* At any point in the search, a secondary node
carries $\text{color} = -1$ iff there is an *active* (not yet undone) `purify` of
its item on the recursion stack that marked it; the per-node state machine is
exactly $c \leftrightarrow -1$, never deeper, because §4-fact-2 shows no item is
purified twice (a second purify would require committing a still-linked node of
that item, but all such are either already $-1$ — commit is a no-op on $-1$ — or
belong to different-color options unreachable after fact 1). *Induction on the
recursion tree.* `commit`/`uncommit` in C5/C6 are strictly LIFO: the `purify`
that a node's mark belongs to either strictly encloses the `hide`/`unhide` pair
in question or is strictly enclosed by it — nesting, never interleaving, by the
stack. So between a `hide` at depth $d$ and its `unhide` on backtrack, no
enclosing/enclosed `purify` boundary is crossed, hence no relevant mark flips.
Therefore `unhide`'s `color >= 0` guard evaluates identically to `hide`'s, and
relinks precisely the set `hide` unlinked — Module 09's dancing argument (a
removed node still points at its neighbors) strengthened by a one-bit invariant.
Base case: a leaf does no hide/unhide. Stage 4 tests this the honest way — solve
twice, demand byte-identical output.

### 9. Why branching on a secondary item is *wrong* (rating 20 · §7.2.2.1-(mrv))

**Problem.** Algorithm 7.2.2.1C's step C3 chooses an active *primary* item to
branch on; the root ring threads primary headers only. Show that branching on a
*secondary* item can be outright **wrong** — producing a missing or spurious
solution — not merely slower.

**Hint.** Recall the covering contract: primary items must be covered *exactly*
once, secondary items *at most* once. Branching on an item means "try each
option that covers it, and one of them must be chosen." Apply that "must" to a
secondary item that a valid solution simply leaves uncovered.

**Answer sketch.** Branching on item $i$ commits the search to picking *some*
option covering $i$ — it is a "$i$ is covered by exactly one of these" case
split. That is correct for a primary item, whose contract is "exactly once." But
a secondary item's contract is "**at most** once," so a perfectly valid solution
may leave it *uncovered*. Concrete instance: one primary item $p$ and one
secondary item $s$, with a single option $\{p\}$ (covering only $p$, not $s$).
The unique exact cover is $\{\,\{p\}\,\}$: $p$ is covered once, $s$ zero times —
legal. But if C3 branched on $s$, it would demand an option covering $s$; there
is none, so the search would report **no solution** — wrong. (Dually, forcing a
secondary item to be covered can manufacture solutions that violate other
"at most once" items.) Restricting branching to primary items — and, via colors
and `purify`, letting secondary items constrain *reversibly* rather than force —
is what keeps the semantics correct; MRV then only chooses *which* primary item,
never *whether* to cover an optional one.

### 10. ▶ Emit the XCC solution family as a ZDD (rating 40 · §7.2.2.1-(zdd-xcc))

**Problem.** Build §4's bridge between the two halves of the module. Modify the
XCC search so that instead of visiting solutions one at a time it **memoizes on
the set of still-active items** and emits the entire family of solutions (each
solution = its set of chosen option indices) as a ZDD. Validate `count_sets` of
that diagram against `count_solutions` on the module's instances (order-3 Latin
squares, the word-pair grid).

**Hint.** The subproblem faced at any search node is fully determined by which
items are still active — two different partial choices that leave the same items
uncovered have identical solution *sub*-families. Cache on that key; a cache hit
becomes a *shared* ZDD node, exactly as hash-consing shares family nodes in
stages 1–2. The ZDD variables are option indices.

**Answer sketch.** Give the search a return value: the ZDD of all completions of
the current subproblem, over option-index variables. At C2 (root ring empty)
return $\top$ (the one completion is "add nothing more"). At a branch on primary
item $i$, for each option $r$ in $i$'s list form $\{\{r\}\} \sqcup (\text{ZDD
returned by the recursive call after committing } r)$, and $\cup$ these across
$r$; that is the family "choose $r$, then any completion." Memoize the whole call
on the canonical key = the set of active items (a bitmask or sorted item list),
so identical subproblems return the *same* `Ref` — turning the search DAG into a
shared ZDD. By §1's canonicity, the result is the unique diagram of the solution
family, and `count_sets` on it equals the ordinary `count_solutions` — validate:
$12$ for order-3 Latin squares, $7$ for the word-pair grid, matching stage 4's
counts. This is *Dancing with ZDDs*: where the plain solver visits solutions one
by one, the memoized variant hands back all of them as one diagram, ready for
`count_sets`, uniform sampling, or optimization in $O(Z)$ — the sense in which
dancing links and ZDDs are two faces of one idea. (Open-ended mini-project;
grade by exact agreement of `count_sets` with `count_solutions` on every module
instance.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
