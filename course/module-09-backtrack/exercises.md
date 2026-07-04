# Exercises — Module 09 (Backtracking and Dancing Links, §7.2.2, §7.2.2.1)

Self-contained problems on this module's material — the backtrack tree and its
size (Algorithm B), the all-solutions vs first-solution distinction, the
cover/uncover couplet of dancing links, secondary items (the XC variant), and
Knuth's random-probe tree-size estimator. You can work every one **without the
books**: each states the problem in full, gives a **hint** to peek at when
stuck, and a worked **answer sketch** to check against after you try.
Computational answers here are reproduced by the code you write in the lab (or a
few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise, its number is noted for
readers who own Volume 4B.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Draw the $n=4$ queens tree and count its nodes | 10 | ⬜ |
| 2 | ▶ All-solutions vs first-solution in Algorithm B | 22 | ⬜ |
| 3 | Prove `cover`/`uncover` are exact inverses | 26 | ⬜ |
| 4 | ▶ Add secondary items to `ExactCover` (the XC variant) | 30 | ⬜ |
| 5 | Knuth's random-probe tree-size estimator | 35 | ⬜ |

## Problems

### 1. Draw the $n=4$ queens tree and count its nodes (rating 10 · cf. 7.2.2–1)

**Problem.** In the row-by-row backtracker (Algorithm B), $x_l$ is the column of
the queen in row $l$, and a partial placement $x_1\ldots x_l$ is legal when every
pair of placed queens satisfies $x_r \ne x_l$ and $|x_l - x_r| \ne |l - r|$.
Count the **nodes** of the search tree for $n = 4$, where a node is a *legal
partial placement* — the empty placement at the root, through complete solutions
at the leaves. Give the count level by level.

**Hint.** Level $0$ is the single empty placement. Level $1$ has all four columns
(no earlier queen to conflict). For each deeper level, extend each legal
placement of the level above by every column that survives the property test.

**Answer sketch.** Counting legal partial placements by level (row just filled):

| level $l$ | legal placements | count |
|---|---|---|
| 0 | (empty) | 1 |
| 1 | $x_1 \in \{0,1,2,3\}$ | 4 |
| 2 | $02,03,\ 13,\ 20,\ 30,31$ | 6 |
| 3 | $031,\ 130,\ 203,\ 302$ | 4 |
| 4 | $1302,\ 2031$ (the two solutions) | 2 |

Total $= 1 + 4 + 6 + 4 + 2 = \mathbf{17}$ nodes. For example, from $x_1 = 0$ only
columns $2,3$ survive at row 1 (column 1 shares a diagonal); the two solutions
are $(1,3,0,2)$ and $(2,0,3,1)$, matching the README's $n=4$ count of two. (The
solution counts $1,0,0,2,10,4,40,92,\dots$ are OEIS A000170; here we count *tree
nodes*, not just the leaves.)

### 2. ▶ All-solutions vs first-solution in Algorithm B (rating 22 · cf. 7.2.2–5)

**Problem.** Algorithm B's step B2 says: "If $l > n$, VISIT $x_1\ldots x_n$ and go
to B5." The lab exposes both `count_queens_solutions` (find *all*) and
`first_queens_solution` (find *one*). Explain precisely where in B1–B5 the two
behaviours diverge, and quantify the savings for $n = 4$.

**Hint.** The two searches build the identical tree; they differ only in what
happens *after* a complete solution is visited. One keeps backtracking; the other
stops the whole search. So the first-solution search explores only the part of
the tree to the *left* of (and including the path to) the first leaf.

**Answer sketch.** The trees are identical up to the moment of the first VISIT.
All-solutions: after VISIT it falls through to B5 and keeps backtracking,
eventually enumerating every leaf — for $n = 4$ it walks all $\mathbf{17}$ nodes
and reports both solutions $(1,3,0,2)$ and $(2,0,3,1)$. First-solution: the
instant B2 reaches $l > n$ it *returns* instead of continuing to B5, so it stops
at the leftmost complete leaf. For $n = 4$ the leftmost solution is $(1,3,0,2)$,
reached after exploring only the $x_1 = 0$ subtree (which yields no solution) and
the path $x_1{=}1 \to x_2{=}3 \to x_3{=}0 \to x_4{=}2$: **9 nodes** versus 17.
The saving grows with $n$ (e.g. first-solution for $n = 8$ touches 114 nodes
against 2057 for the full tree). The distinction is purely control-flow — "return
on VISIT" vs "record and continue" — with no change to the property test or the
tree's shape.

### 3. Prove `cover`/`uncover` are exact inverses (rating 26 · cf. 7.2.2.1–8)

**Problem.** In dancing links, removing a node $x$ from its doubly linked list is
$$\text{right}[\text{left}[x]] \leftarrow \text{right}[x], \qquad
\text{left}[\text{right}[x]] \leftarrow \text{left}[x],$$
and restoring it is
$$\text{right}[\text{left}[x]] \leftarrow x, \qquad
\text{left}[\text{right}[x]] \leftarrow x.$$
`cover(c)` removes column header $c$ and, for every row using $c$, removes that
row's other nodes; `uncover(c)` reverses this, walking rows and nodes in the
**opposite order**. Prove that `cover(c)` immediately followed by `uncover(c)`
restores the entire structure bit for bit.

**Hint.** Two ingredients. (a) A single remove/restore pair on one node is an
identity *provided* $x$'s own `left[x]` and `right[x]` are untouched between them.
(b) When several nodes are removed, restoring must be in the exact reverse (LIFO)
order, because a later removal may overwrite a neighbour link that an earlier
node depends on.

**Answer sketch.** *Single node.* Remove sets its two neighbours to point past
$x$; it does **not** modify `left[x]` or `right[x]`. So after removal $x$ still
records its original neighbours. Restore reads those intact links and reinstates
$\text{right}[\text{left}[x]] = x$, $\text{left}[\text{right}[x]] = x$, returning
both neighbours' links to their pre-removal values. If no other operation touched
`left[x]`/`right[x]` in between, the pair is the identity on all four link fields.

*Many nodes — why reverse order.* `cover(c)` removes nodes in some sequence
$x_1, x_2, \dots, x_m$. Two consecutive removals in the *same* list interact: when
$x_i$ and later $x_j$ are neighbours, removing $x_j$ rewrites a link that points
around the already-removed $x_i$. Restoring $x_i$ before $x_j$ would splice $x_i$
into a list from which $x_j$ is still absent, corrupting the pointers. Restoring
in reverse order $x_m, \dots, x_1$ guarantees that when $x_i$ is restored, every
node removed *after* it has already been put back, so the list around $x_i$ is
exactly as it was the instant before $x_i$ left — and part (a) applies to make
each restore an identity. By induction over the $m$ operations, `uncover(c)`
(which visits rows bottom-to-top and nodes right-to-left, the mirror of `cover`)
returns every link field to its original value. Hence solving the same instance
twice yields identical results — the property Stage 3 checks, and the reason no
copy of the matrix is ever needed. $\blacksquare$

### 4. ▶ Add secondary items to `ExactCover` (the XC variant) (rating 30 · cf. 7.2.2.1)

**Problem.** Exact cover requires every item covered *exactly once*. In the **XC**
variant, items split into **primary** (must be covered exactly once) and
**secondary** (may be covered *at most* once — zero or one time). Describe the
minimal change to the dancing-links `ExactCover` of Stage 3 that supports
secondary items, and argue it is correct. (Concrete use: the $n$-queens diagonals
are naturally secondary — a diagonal need not be used, but no two queens may
share one.)

**Hint.** Two places touch the "which items remain" logic: the MRV column-choice
scan and the termination test. Secondary items must still be *covered* when an
option uses them (so conflicts are enforced), but they must never be *chosen* as
the branching column, and leftover uncovered secondaries must not block a
solution. The classic trick: link the secondary headers among themselves but
**not** into the root's active header list.

**Answer sketch.** Keep the identical node mesh and the identical `cover`/`uncover`
couplet. The only structural change is the header list: primary headers form the
circular list hanging off the root $h$ (as before), while each secondary header
is given a self-loop (its left/right point to itself) and is **left out of** the
root list. Consequences, all correct:

- *Branching.* The MRV scan walks only the root's active headers, so it only ever
  chooses a *primary* column — exactly right, since we must satisfy every primary
  item but are free to leave secondaries.
- *Conflict enforcement.* When an option that uses a secondary item is selected,
  `cover` still removes that secondary's column and all rows touching it, so no
  second option can also use it — enforcing "at most once."
- *Termination.* The search succeeds when the root's header list is empty, i.e.
  all *primary* items are covered. Secondary items still uncovered are simply
  never required, which is the "at most once (possibly zero)" semantics.

Because `cover`/`uncover` are unchanged, the reversibility proof of Problem 3
still holds verbatim; only the set of headers eligible for selection shrank. This
one-field change (self-loop the secondaries, omit them from the root list) is why
the README says your `ExactCover` is "one field away from XC." For $n$-queens as
exact cover: the $2n$ row/column items are primary, the $4n-2$ diagonal items are
secondary — and the same engine that solved the toy example now enforces the
diagonal constraints without demanding every diagonal be occupied.

### 5. Knuth's random-probe tree-size estimator (rating 35 · cf. 7.2.2)

**Problem.** Knuth's Monte Carlo estimator predicts the size of a backtrack tree
without traversing it: take a single **random** walk from the root; at each node
count its children $c$, pick one child uniformly, and accumulate. Formally, if
$c_0, c_1, c_2, \dots$ are the child-counts along the random path (with a uniform
choice at each step), the estimator is
$$\hat{C} = 1 + c_0 + c_0 c_1 + c_0 c_1 c_2 + \cdots
= 1 + \sum_{d \ge 1} \prod_{i=0}^{d-1} c_i.$$
Prove that $\mathbb{E}[\hat{C}]$ equals the exact number of nodes in the tree,
and confirm it empirically against the true $n$-queens tree sizes for $n \le 12$.

**Hint.** Use linearity of expectation over nodes, not over paths. Fix a node $v$
at depth $d$ with ancestors having child-counts $c_0, \dots, c_{d-1}$. What is the
probability the random walk *reaches* $v$? What does $\hat{C}$ add when it does?
Multiply.

**Answer sketch.** Write $\hat{C} = \sum_v [\,v \text{ is on the random path}\,]
\cdot w(v)$, where the term contributed when the walk reaches node $v$ at depth
$d$ is $w(v) = \prod_{i=0}^{d-1} c_i$ (the product of branching factors of $v$'s
ancestors; for the root $d = 0$ the empty product is $1$). The walk reaches a
specific node $v$ iff it makes the correct uniform choice at each of $v$'s $d$
ancestors, so
$$\Pr[\text{reach } v] = \prod_{i=0}^{d-1} \frac{1}{c_i}
= \frac{1}{w(v)}.$$
By linearity of expectation,
$$\mathbb{E}[\hat{C}] = \sum_v \Pr[\text{reach } v]\cdot w(v)
= \sum_v \frac{1}{w(v)}\cdot w(v) = \sum_v 1 = (\text{number of nodes}).$$
So $\hat{C}$ is an *unbiased* estimator of the tree size — each node contributes
expected value exactly $1$, regardless of the tree's shape. (Its *variance* can be
large for very irregular trees, which is why one averages several probes.)

*Empirical confirmation.* The exact node counts of the legal-partial-placement
$n$-queens tree (Problem 1's convention) are

| $n$ | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|
| nodes | 17 | 54 | 153 | 552 | 2057 | 8394 | 35539 | 166926 | 856189 |

Averaging $\hat{C}$ over many random probes converges to these: e.g. $2\times10^5$
probes give $\approx 2058$ for $n = 8$ (true $2057$) and $\approx 8.5\times10^5$
for $n = 12$ (true $856189$) — matching to within sampling error, as the
unbiasedness proof predicts. A handful of probes suffices to predict, within a
small factor, the cost of a search that would take far longer to run in full.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
