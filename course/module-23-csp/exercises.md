# Exercises — Module 23 (Constraint Satisfaction)

Self-contained problems on this module's material — the pruning power of
forward checking, chromatic-polynomial counting anchors, the AC-3 fixpoint,
the size of the direct encoding, and Fascicle 7's dancing-cells sparse set.
You can work every one **without the book**: each states the problem in full,
gives a **hint** to peek at when stuck, and a worked **answer sketch** to
check against after you try. Numeric answers here are reproduced by the code
you write in the lab (or a few lines against it).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` =
especially instructive). Fascicle 7 is a *draft pre-fascicle*, so these carry
content tags cited to §7.2.2.3 by topic rather than exercise numbers — match
by content, not by number, if you own a printing.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | ▶ Forward checking visits a subtree of basic backtracking | 25 | ⬜ |
| 2 | Deletion–contraction; $P(C_n, k)$; $P(\text{Petersen}, 3) = 120$ | 22 | ⬜ |
| 3 | ▶ The AC-3 fixpoint is unique (full proof) | M25 | ⬜ |
| 4 | Size of the direct encoding of queens-$n$ in closed form | M20 | ⬜ |
| 5 | ▶ Mini-project: a dancing-cells sparse-set domain | 32 | ⬜ |

## Problems

### 1. ▶ Forward checking visits a subtree of basic backtracking (rating 25 · cf. §7.2.2.3)

**Problem.** Fix a CSP and the module's static order: variables by index,
values ascending, one **node** per tentative placement (counted before any
test). Prove that every node forward checking visits, basic backtracking also
visits — so the FC tree is a subtree of the basic tree and
$\text{nodes}_{FC} \le \text{nodes}_{basic}$ on *every* instance (the
inequality the stage 2 tests check on random CSPs). Then explain why the
argument breaks for MRV.

**Hint.** Identify a node with the pair (consistent prefix it extends, value
placed). Strengthen the induction: whenever FC *descends into* a partial
assignment, (a) basic descends into it too, and (b) each unassigned
variable's live domain is a subset of its full domain containing only values
compatible with every assigned neighbor.

**Answer sketch.** Induct on the depth of the partial assignment $P$ that a
search descends into, proving: *if FC descends into $P$, then basic descends
into $P$, and at $P$ every unassigned $v$'s live domain satisfies
$\text{live}(v) \subseteq D_v$.* Base: both searches descend into the empty
prefix, and live domains start as the full domains. Step: suppose the claim
holds at $P$ and FC places $x = a$ (the next variable is the same for both —
the order is static). Two observations:

- **Every FC placement is a basic placement.** FC draws $a$ from
  $\text{live}(x) \subseteq D_x$; basic at the same prefix $P$ places *every*
  value of $D_x$, so it places $a$ too — the node is shared.
- **Every prefix FC descends into, basic descends into.** FC descends into
  $P \cup \{x = a\}$ only if no live domain wiped out; in particular $a$
  itself survived filtering by every assigned neighbor of $x$, so every
  constraint with both endpoints assigned holds (those among $P$'s variables
  hold by induction). That is exactly basic's descent test, so basic
  descends too. And FC's subsequent filtering only *removes* values from
  live domains, preserving (b).

Hence the set of FC nodes is a subset of the set of basic nodes: a subtree,
and the node-count inequality follows. Note the two directions of the
economy: FC may *place fewer values* at a shared prefix (filtered domain)
and may *descend into fewer prefixes* (wipeout detected where basic would
march on for several more levels). **Why MRV escapes the theorem:** with
dynamic variable ordering the two searches no longer branch on the same
variable at the same prefix, so their trees are not nested — MRV usually
wins (queens-6: $130 \to 118$) but nothing forbids an instance where its
reordering explores more placements than static FC. Subtree arguments need a
shared skeleton; heuristic *reordering* changes the skeleton itself.

### 2. Deletion–contraction; $P(C_n, k)$; $P(\text{Petersen}, 3) = 120$ (rating 22 · cf. §7.2.2.3)

**Problem.** Let $P(G, k)$ count the proper $k$-colorings of $G$. Prove the
**deletion–contraction** identity, for any edge $e = uv$:

$$
P(G, k) = P(G - e, k) - P(G / e, k),
$$

where $G - e$ deletes the edge and $G / e$ contracts it (merges $u$ and
$v$). Use it to derive the lesson's cycle anchor
$P(C_n, k) = (k-1)^n + (-1)^n(k-1)$, and confirm $P(C_5, 3) = 30$. Then
determine $P(\text{Petersen}, 3)$ with the lab.

**Hint.** Classify the proper colorings of $G - e$ by whether $u$ and $v$
received equal or unequal colors. For the cycle, delete one edge of $C_n$
(what graph remains?) and contract it (what smaller cycle appears?), then
induct with base $C_3 = K_3$.

**Answer sketch.** **Identity.** A proper coloring of $G - e$ either gives
$u, v$ different colors — these are exactly the proper colorings of $G$ —
or the same color — these correspond bijectively to proper colorings of
$G / e$ (color the merged vertex with the common color; every other
adjacency is inherited). So $P(G-e) = P(G) + P(G/e)$; rearrange.
**Cycles.** Deleting one edge of $C_n$ leaves the path on $n$ vertices, with
$P(\text{path}_n, k) = k(k-1)^{n-1}$ (color along the path: $k$ choices,
then $k-1$ each). Contracting that edge yields $C_{n-1}$. So

$$
P(C_n, k) = k(k-1)^{n-1} - P(C_{n-1}, k).
$$

Induct: base $P(C_3, k) = k(k-1)(k-2) = (k-1)^3 - (k-1)$ ✓; step
$k(k-1)^{n-1} - \bigl[(k-1)^{n-1} + (-1)^{n-1}(k-1)\bigr] = (k-1)^n +
(-1)^n(k-1)$ ✓. At $n = 5$, $k = 3$: $2^5 - 2 = 30$, the lesson's anchor.
**Petersen.** Running deletion–contraction by hand on 15 edges is a
$2^{15}$-leaf recursion in the worst case — this is where the machine earns
its keep. Build the edge list (outer 5-cycle $i \to i+1 \bmod 5$, inner
pentagram $5{+}i \to 5{+}(i{+}2 \bmod 5)$, spokes $i \to 5{+}i$) and run
`coloring_csp(10, &edges, 3).solve_fc()`: exactly **120** solutions, the
classic count. Sanity-check it against the identity's *parity*: every proper
3-coloring pairs with $3! = 6$ recolorings under permuting the colors, and
indeed $120 = 6 \cdot 20$ — twenty essentially different colorings. (The
same lab call with the stage 4 encoding — `count_models(&encode_direct(...))`
— would recount the 120 through SAT, but note the encoding has 30 variables,
beyond `count_models`' 24-variable truth-table guard: choose your oracle to
fit its budget.)

### 3. ▶ The AC-3 fixpoint is unique — full proof (rating M25 · cf. §7.2.2.3)

**Problem.** Call a family of subdomains $E = (E_v \subseteq D_v)$
*arc-consistent* if for every constraint and both of its arcs, every value in
the tail's $E$-domain has a support in the head's $E$-domain. Prove, in
full: (i) arc-consistent families are closed under arbitrary unions, so a
unique maximal arc-consistent family $M$ exists; (ii) AC-3 never deletes a
value of $M$; (iii) when AC-3's worklist empties, the surviving family is
arc-consistent; conclude AC-3 returns exactly $M$ for *every* worklist
order, and that the solution set is preserved exactly. State the worklist
invariant (iii) needs precisely.

**Hint.** For (i), a support inside $E$ is still a support inside anything
containing $E$. For (ii), induct on deletions: current domains always
contain $M$. For (iii), find a property of "arcs *not* currently on the
queue" that A1 establishes vacuously, each revise establishes for its own
arc, and the requeue rule repairs whenever it is broken.

**Answer sketch.** **(i)** Let $\{E^{(i)}\}$ be arc-consistent families and
$U_v = \bigcup_i E^{(i)}_v$. A value $a \in U_x$ lies in some $E^{(i)}_x$;
along any arc $x \to y$ it has a support $b \in E^{(i)}_y \subseteq U_y$. So
$U$ is arc-consistent. Taking $U = M$ over *all* arc-consistent families
contained in the original domains gives the unique maximal one (it contains
each by construction). **(ii)** Induction on AC-3's deletions, with
invariant "current domains $\supseteq M$". True initially. A revise of arc
$x \to y$ deletes $a \in D_x$ only if $a$ has no support in the *current*
$D_y$; but if $a \in M_x$, its $M$-support $b \in M_y$ lies in the current
$D_y$ by the invariant — contradiction. So only non-$M$ values die, and the
invariant persists. **(iii)** The worklist invariant: *every arc not on the
queue has been revised since the last time its head's domain changed* —
equivalently, every value in its tail's current domain has support in its
head's current domain. A1 makes it vacuous (all arcs queued); popping and
revising an arc establishes its property; the only event that can destroy
the property of an off-queue arc is a shrink of its **head's** domain, and
A3 requeues exactly the arcs whose head shrank. (This is also why the
requeue direction is forced: shrinking a *tail* removes claimants but
cannot un-support survivors.) At termination the queue is empty, so every
arc is consistent: the result $A$ is an arc-consistent family, hence
$A \subseteq M$ by maximality; with (ii)'s $A \supseteq M$, $A = M$ — no
worklist order was ever mentioned, so none can matter. **Solutions.** For a
solution $s$, the singleton family $E_v = \{s_v\}$ is arc-consistent (each
arc's support is the solution's own value at the other end), so $s_v \in
M_v$ for all $v$: pruning to $M$ deletes no solution, and shrinking domains
creates none. The stage 3 tests exercise every clause of this theorem:
reversed constraint order, idempotence, exact solution preservation.

### 4. Size of the direct encoding of queens-$n$ in closed form (rating M20 · cf. §7.2.2.3)

**Problem.** The direct encoding of `queens_csp(n)` has $n^2$ SAT variables.
Count its clauses in closed form: derive the conflict-clause total

$$
\sum_{d=1}^{n-1} (n-d)\bigl(n + 2(n-d)\bigr)
$$

and reduce it to a polynomial in $n$; then give the full clause count.
Verify against the pinned queens-4 numbers (52 conflicts, 80 clauses) and
predict queens-5 and queens-6 before checking with `encode_direct`.

**Hint.** For a column pair at distance $d = j - i$: how many equal-row
pairs are forbidden, and how many diagonal pairs (count $b = a + d$ and
$b = a - d$ separately)? How many column pairs sit at distance $d$? Then
$\sum k = \binom{n}{2}$ and $\sum k^2 = \frac{(n-1)n(2n-1)}{6}$ finish it.

**Answer sketch.** Variables: $n$ queens with domain size $n$ each, so
$n^2$. **ALO:** $n$ clauses. **AMO:** $n \binom{n}{2} = \frac{n^2(n-1)}{2}$
clauses. **Conflicts:** a column pair at distance $d$ forbids the $n$
equal-row pairs $(a, a)$, plus the ascending diagonal pairs $(a, a+d)$ —
there are $n - d$ of them — plus the $n - d$ descending pairs $(a, a-d)$:
$n + 2(n-d)$ forbidden pairs. There are $n - d$ column pairs at distance
$d$, so

$$
\text{conflicts} = \sum_{d=1}^{n-1} (n-d)\bigl(n + 2(n-d)\bigr)
= n \sum_{k=1}^{n-1} k + 2 \sum_{k=1}^{n-1} k^2
= \frac{n^2(n-1)}{2} + \frac{n(n-1)(2n-1)}{3},
$$

substituting $k = n - d$. **Total:**

$$
\text{clauses} = n + \frac{n^2(n-1)}{2} + \frac{n^2(n-1)}{2} + \frac{n(n-1)(2n-1)}{3}
= n + n^2(n-1) + \frac{n(n-1)(2n-1)}{3}.
$$

Checks: $n = 4$: conflicts $\frac{16 \cdot 3}{2} + \frac{4 \cdot 3 \cdot
7}{3} = 24 + 28 = 52$ ✓ and total $4 + 48 + 28 = 80$ ✓ — the pinned stage 4
numbers. Predictions: $n = 5$: conflicts $50 + 60 = 110$, total $5 + 100 +
60 = 165$; $n = 6$: conflicts $90 + 110 = 200$, total $6 + 180 + 110 =
296$. Both confirmed by `encode_direct(&queens_csp(n)).clauses.len()` (the
*encoder* has no size guard — only `count_models`' truth table does). The
shape of the answer is the lesson: the encoding is $\Theta(n^3)$ clauses on
$n^2$ variables — polynomial, faithful, and exactly the kind of size
accounting you should demand of any reduction before trusting it at scale.

### 5. ▶ Mini-project: a dancing-cells sparse-set domain (rating 32 · cf. §7.2.2.3)

**Problem.** Replace stage 2's save-list undo with Fascicle 7's **dancing
cells**: implement a sparse-set domain

```text
SparseDomain { dom: Vec<u32>,  pos: index of each value in dom,  size: usize }
```

with $O(1)$ `contains`, $O(1)$ `delete`, and $O(1)$ *batch undo* (restore a
saved `size`), then swap it into forward checking. Requirements: (a) state
the representation invariant and prove delete and undo preserve it; (b)
preserve the lab's observable behavior — same solution sets *and the same
pinned node counts* (queens-6 FC = 130); (c) measure the win over the
save-list on queens-8.

**Hint.** Invariant: `dom` is a permutation of the original domain, `pos` is
its inverse, and the live values are exactly `dom[0..size]`. Delete by
swapping the victim to `dom[size-1]` and decrementing `size` — deleted
values are *parked*, never destroyed. For (b), think hard about iteration
order: `dom[0..size]` is scrambled, but the node-count pins assume values
are tried **ascending**.

**Answer sketch.** **(a)** Representation invariant: `dom` is a permutation
of the initial (sorted, deduped) domain; `pos[dom[i]] = i` for all `i`; the
live set is $\{ \mathtt{dom}[i] : i < \mathtt{size} \}$. `contains(b)` is
`pos[b] < size`. `delete(b)` swaps `dom[pos[b]]` with `dom[size-1]`, updates
the two `pos` entries, decrements `size`: still a permutation, `pos` still
inverse, live set = old live set minus $\{b\}$ — invariant preserved, $O(1)$.
`undo(saved)` sets `size <- saved`: the values in `dom[size..saved]` are
*precisely* the ones deleted since the checkpoint (deletes park victims
immediately beyond the fence, and nothing else writes there), so the live
set is restored exactly — provided undos happen in LIFO order, which the
backtracking recursion guarantees. That proviso is the same stack discipline
that makes Module 09's dancing links dance; here the "links" are array
indices, which is why F7 calls them dancing *cells*. **(b)** Two traps.
*Iteration while deleting:* when filtering a neighbor's domain, walk index
`i` over `dom[0..size]` but do **not** advance `i` after a delete — the swap
just moved an unexamined value into slot `i`. *Branching order:* the pins
(queens-5 FC 53, queens-6 FC 130) assume candidate values are placed
ascending, but `dom[0..size]` is scrambled by past swaps; iterate the
*original sorted domain* and skip non-live values via `contains` — $O(|D|)$
per branch instead of $O(\text{size})$, a price worth paying for determinism
(the reference makes the same trade with its sorted save-list). With that,
solution sets, order, and node counts all match the save-list version
exactly, and the stage tests pass unmodified. **(c)** The save-list undo
re-sorts each touched domain ($O(d \log d)$ per restore, plus allocation for
the removal log); the sparse set undoes a whole placement in $O(\#\text{touched
vars})$ fence moves and allocates nothing after setup. On queens-8
(15720-node basic tree, 1724 FC) expect the domain-maintenance share of the
runtime to drop by a small integer factor — measure with
`std::time::Instant` around `solve_fc`, best of several runs. The
engineering lesson: same asymptotic search tree, same answers, and the
constant factor lives entirely in the undo structure — which is why
Fascicle 7 spends its pages on cells, not just on trees.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and
how it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
