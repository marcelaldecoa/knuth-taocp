# Module 22 — Hamiltonian Paths and Cycles

> **Source:** *The Art of Computer Programming*, toward Vol. 4C, §7.2.2.4
> (Hamiltonian paths and cycles; pre-fascicle material), with threads back to
> Vol. 4A §7.2.1.1 (Gray codes), Vol. 4B §7.2.2 (backtracking), and Vol. 4A
> §7.1.3 (bitwise tricks).
> **Lab:** `labs/module-22-hamilton` · **Grade it:** `./grade 22`
>
> This lesson is self-contained: you can complete the module without the book.
> It is also the **course finale**, so it deliberately reaches back and ties
> together three earlier modules — 08 (Gray codes), 09 (backtracking), and
> 13 (bitmask states). Watch for the ↩ marks.

A single innocent question runs through this whole module:

> *Given a graph, is there a route that visits every vertex exactly once?*

That route is a **Hamiltonian path** (and if it returns to its start, a
**Hamiltonian cycle**). The question sounds almost identical to one you can
answer in a blink — *is there a route using every edge exactly once?* — yet the
two live on opposite sides of the deepest divide in computer science. By the
end of the module you will have attacked the Hamiltonian question four ways:
exhaustively (backtracking), greedily (Warnsdorff's rule), structurally (the
hypercube-Gray-code bridge), and optimally-but-exponentially (Held–Karp), and
you will understand *why* no fifth, fast, always-correct way is known.

---

## 1. Two questions that look the same and aren't

Let $G$ be a graph on $n$ vertices.

- An **Eulerian** trail uses **every edge** exactly once; an Eulerian circuit
  is a closed one. (Königsberg's bridges, 1736 — the birth of graph theory.)
- A **Hamiltonian** path uses **every vertex** exactly once; a Hamiltonian
  cycle is a closed one. (Named for W. R. Hamilton's 1857 "Icosian game.")

Euler settled his question completely and cheaply:

> **Theorem (Euler).** A connected graph has an Eulerian circuit iff every
> vertex has even degree; it has an Eulerian trail iff exactly zero or two
> vertices have odd degree.

You can *check* that condition in one pass over the vertices — $O(n + m)$
time — and Hierholzer's algorithm even *constructs* the trail in linear time.
Eulerianness is a local, countable property.

The Hamiltonian question has **no such characterization**. Deciding whether a
graph has a Hamiltonian cycle is **NP-complete** (Karp, 1972): it is exactly as
hard as SAT (Module 10/14), graph coloring, and every other problem in that
class. No polynomial-time algorithm is known, and finding one — or proving none
exists — *is* the P versus NP problem, the most famous open question in the
field.

**State the asymmetry plainly, because it is the moral of the module:**
"every edge once" is easy; "every vertex once" is (as far as anyone knows)
exponentially hard. Two problems can be one word apart in English and a
universe apart in complexity. Everything that follows is a response to that
hardness — exhaustive search made as smart as we can, heuristics that are fast
but fallible, special structures where the answer is free, and an exponential
algorithm that is nonetheless the best we know.

> **Why "vertices once" is hard, intuitively.** For an Eulerian trail the
> constraints are *independent per vertex* (just parity of degree), so they
> compose. For a Hamiltonian path the choice you make at one vertex reaches
> across the whole graph — using a vertex now forbids it forever, and whether
> that dooms you may not surface until many steps later. Local conditions do
> not suffice; you seem forced to search.

---

## 2. Backtracking for Hamiltonicity (§7.2.2.4)

Since no formula decides Hamiltonicity, we **search** — and search with
backtracking, the workhorse of Module 09 ↩. We grow a partial path one vertex
at a time and abandon (backtrack) the moment it cannot be completed. We reuse
the skeleton of **Algorithm 7.2.2B** almost verbatim; only the *property test*
changes.

### Algorithm H (Hamiltonian path/cycle by backtracking)

State: a partial simple path $x_0 x_1 \ldots x_{l-1}$ and a `used[]` flag per vertex.
The "level" $l$ is the path's length.

```text
H1. [Initialize.]  Pick a start s (for a path, try each s in turn; for a
                   cycle, s = 0 suffices — every cycle contains vertex 0).
                   Set the path to (s), used[s] <- true, l <- 1.
H2. [Done?]        If l = n:  for a path, succeed; for a cycle, succeed iff
                   x_{l-1} is adjacent to s. Otherwise go to H3.
H3. [Try a value.] Let v range over vertices adjacent to the endpoint x_{l-1}
                   with used[v] = false.
H4. [Advance.]     Append v, used[v] <- true, l <- l+1; go to H2.
H5. [Backtrack.]   When the candidates at level l are exhausted, remove the
                   last vertex (used <- false, l <- l-1) and resume the loop
                   one level up. If we back up past level 1, report failure.
```

The bracketed **assertion** that makes it correct: *at every entry to H2,
$x_0 \ldots x_{l-1}$ is a simple path in $G$.* Advancing preserves it (H3 only admits an
unused neighbor); backtracking restores the earlier state exactly (H5 undoes
what H4 did). That "state restoration is exact" invariant is the beating heart
of all backtracking ↩ Module 09.

### A hand trace: does the 4-cycle $C_4$ have a Hamiltonian cycle?

Vertices `0,1,2,3`; edges `0–1, 1–2, 2–3, 3–0`. Fix $s = 0$.

| l | path so far | endpoint | unused neighbors | action |
|---|---|---|---|---|
| 1 | 0 | 0 | 1, 3 | try 1 |
| 2 | 0 1 | 1 | 2 | try 2 |
| 3 | 0 1 2 | 2 | 3 | try 3 |
| 4 | 0 1 2 3 | 3 | — | l=n; 3 adjacent to 0? **yes** → cycle! |

Answer: `0 1 2 3` (closing `3–0`). Had the last vertex *not* been adjacent to 0,
H2 would reject and H5 would backtrack to try the other branch (`0 3 2 1`).

### Counting, and why we divide by two

`count_hamiltonian_cycles` fixes the start at vertex 0 and counts *directed*
closed walks $0 \to \ldots \to 0$. Every undirected cycle is walked in **two**
directions, so the undirected count is (directed count)/2. Two anchors the
tests pin down:

- **Complete graph $K_n$.** From the fixed start, the remaining $n-1$ vertices
  can appear in any order, all edges present, so there are $(n-1)!$ directed
  cycles and **$(n-1)!/2$** undirected ones. ($K_5$: $4!/2 = 12$.)
- **Cycle graph $C_n$.** Only two directed walks (clockwise, counter-clockwise),
  so exactly **1** undirected cycle — itself.

### Pruning: how real solvers survive

Plain Algorithm H already works, but its search tree can explode. The classic
prunes (state them; you will implement stronger ones in the exercises):

1. **Degree $\ge 2$ for cycles.** A vertex with fewer than two neighbors cannot lie
   on any cycle — reject immediately.
2. **Endpoint reachability.** After committing a partial path, if some unused
   vertex has *all* its neighbors already used (and it is not the vertex we
   could still step to), no completion exists — backtrack.
3. **Connectivity of the remainder.** If deleting the used interior vertices
   disconnects the unused ones, we are stuck.

These do not change *what* is computed — only how fast. That distinction
(function vs. method ↩ Module 01) recurs constantly.

### The Petersen graph: a famous "no"

The **Petersen graph** (10 vertices, 15 edges, 3-regular: an outer pentagon,
an inner pentagram, and five spokes) is the textbook example of a graph that is
**not Hamiltonian** — it has *no* Hamiltonian cycle at all — yet it *does* have
a Hamiltonian **path**. Your stage-1 code confirms both facts by search:
`count_hamiltonian_cycles` returns 0, `hamiltonian_cycle` returns `None`, but
`hamiltonian_path` returns `Some(...)`. It is a small graph, so backtracking
settles it in microseconds — a reminder that "NP-complete" is a statement about
worst-case *asymptotics*, not about any particular instance.

---

## 3. Warnsdorff's rule: a fast, fallible heuristic

Exhaustive search is correct but can be slow. Often we would trade the
guarantee for speed. **Warnsdorff's rule** (1823) is the archetype: a *greedy
heuristic* for the **knight's tour** — a Hamiltonian path on the graph whose
vertices are the squares of a chessboard and whose edges are legal knight
moves.

> **Warnsdorff's rule.** From the current square, always move to the unvisited
> square that has the **fewest unvisited onward moves.** Head for the most
> cramped part of the board first, while you still can.

### Algorithm W (Warnsdorff)

```text
W1. [Initialize.]  Mark the start visited; it is the current square.
W2. [Done?]        If all board² squares are visited, report the tour.
W3. [Score.]       For each unvisited neighbor v of the current square, let
                   d(v) = number of unvisited neighbors of v.
W4. [Choose.]      Move to the v minimizing (d(v), v)  [ties -> smaller
                   square index]. If there is no unvisited neighbor, FAIL.
```

Note what is **missing**: there is no backtracking. Algorithm W commits to each
greedy step forever. That makes it blazingly fast — $O(\text{board}^2)$ moves, each
looking at $\le 8$ neighbors — but it is a *heuristic*, not an algorithm in Knuth's
strict sense (§1.1's finiteness-and-definiteness bar ↩ Module 01): it can walk
into a dead end and simply give up.

### Why "fewest onward moves" usually works

The danger in any tour is orphaning a square — leaving a square whose only
access routes you have already used up. Low-degree squares (corners, edges) are
the ones most easily orphaned. Warnsdorff services them **first**, when they
still have an entrance *and* an exit, deferring the flexible high-degree
central squares to the end where flexibility is cheap. Empirically this
succeeds astonishingly often, and on boards up to modest sizes it completes
full tours from almost every start.

### Why it sometimes fails — the honest story

"Usually" is not "always." With the deterministic tie-break above, the greedy
walk completes a $5 \times 5$ tour from a corner and even from the center — yet it gets
**stuck starting from square 2** of that same board, *even though a full tour
from square 2 provably exists* (a backtracking search finds one). The same
happens on $8 \times 8$ from square 24. The heuristic made a locally sensible choice
that was globally fatal, and, having no backtracking, it cannot recover.

That is the lesson, stated bluntly: **a heuristic is a bet, not a proof.** When
Warnsdorff succeeds it is wonderful; when it fails it tells you nothing about
whether a tour exists. Real systems pair heuristics like this with a fallback
(backtracking, or a smarter tie-break) precisely because of this gap.

> **A rigorous aside — parity forbids some starts entirely.** Colour the $5 \times 5$
> board like a checkerboard: 13 squares of one colour, 12 of the other. A
> knight always steps to the opposite colour, so a 25-square tour alternates
> colours and must *begin and end on the majority colour* (13 of them). The 12
> minority squares can be the endpoint of **no** tour whatsoever — no algorithm,
> however clever, can start a tour there. Square 2 is *not* one of those (it is
> a majority square), so its failure is genuinely Warnsdorff's fault, not the
> board's. Distinguishing "impossible" from "the heuristic blinked" is exactly
> the kind of care this course trains.

---

## 4. Hamiltonian cycles on the hypercube ARE Gray codes ↩ Module 08

Sometimes the graph has so much structure that a Hamiltonian cycle can be
written down with no search at all. The cleanest example ties this finale
straight back to where combinatorial generation began.

The **$d$-dimensional hypercube** $Q_d$ has the $2^d$ binary strings of length
$d$ as vertices, with an edge between two strings iff they **differ in exactly
one bit**. A Hamiltonian cycle on $Q_d$ is therefore a cyclic listing of all
$d$-bit strings in which *each string differs from the next in a single bit* —
which is precisely the definition of a **cyclic Gray code**. The two ideas are
not analogous; they are *the same object* seen from two angles:

> "Generate all $n$-bit strings, changing one bit at a time" (Module 08's
> reflected Gray code) **=** "walk a Hamiltonian cycle on the hypercube $Q_n$"
> (this module).

The reflected binary Gray code you built in Module 08 is $g(k) = k \oplus \lfloor k/2 \rfloor$
(i.e. `k ^ (k >> 1)`), for $k = 0, 1, \ldots, 2^d - 1$. `gray_code_cycle(d)` emits
exactly that sequence, and it *is* our Hamiltonian cycle.

### Theorem and proof: $g$ is a Hamiltonian cycle on $Q_d$

> **Theorem.** The sequence $G_d = g(0), g(1), \ldots, g(2^d - 1)$ lists every vertex
> of $Q_d$ exactly once, and consecutive vertices — *including the wrap-around
> from the last back to the first* — differ in exactly one bit. Hence $G_d$ is a
> Hamiltonian cycle on $Q_d$.

*Proof (induction on $d$, mirroring the "reflect and prefix" construction).*

**Base $d = 1$.** $G_1 = 0, 1$. Two vertices; $0 \to 1$ flips one bit and the wrap
$1 \to 0$ flips one bit. ✓

**Step.** Assume $G_{d-1}$ is a Hamiltonian cycle on $Q_{d-1}$ listing all
$2^{d-1}$ shorter strings with single-bit steps. The reflected construction
builds $G_d$ in two halves:

1. **First half:** prefix a $0$ to each string of $G_{d-1}$, in order.
2. **Second half:** prefix a $1$ to each string of $G_{d-1}$ in **reverse**
   order (the "reflection").

Check the three claims.

- *All vertices once.* The first half is exactly the $2^{d-1}$ strings starting
  $0\ldots$, each once (by the hypothesis); the second half is the $2^{d-1}$ strings
  starting $1\ldots$, each once. Together: all $2^d$ strings, no repeats. ✓
- *Steps within a half* flip a single bit among the low $d-1$ positions (that is
  the hypothesis, applied forward in the first half and backward — still
  single-bit — in the second) and never touch the new top bit. ✓
- *The two seams.* At the **middle seam**, the first half ends at $0 \cdot w$ where $w$
  is $G_{d-1}$'s last string, and the second half begins at $1 \cdot w$ (same $w$,
  because the reflection starts from $G_{d-1}$'s last string). These differ only
  in the top bit. ✓ At the **wrap-around**, $G_d$ ends at $1 \cdot u$ where $u$ is
  $G_{d-1}$'s *first* string, and $G_d$ begins at $0 \cdot u$. Again a single top-bit
  difference. ✓

All three hold, so $G_d$ is a Hamiltonian cycle on $Q_d$. ∎

The reflection is doing something beautiful: it *doubles* a cycle on $Q_{d-1}$
into a cycle on $Q_d$, using the single new bit to stitch the two copies at both
their ends. That is the same doubling that makes the code "reflected."

`hypercube_neighbors(d, v)` (flip each of the $d$ low bits) and
`is_hamiltonian_cycle_on_hypercube` (all $2^d$ vertices once, cyclic single-bit
steps) let you *verify* the theorem computationally in stage 3 — and confirm
that the Module-08 formula and the Module-22 hypercube tell one story.

---

## 5. Held–Karp: the shortest Hamiltonian path by bitmask DP ↩ Module 13

The existence question ("is there a Hamiltonian path?") has an *optimization*
sibling that matters even more in practice: among all Hamiltonian paths (or
cycles) of a **weighted** graph, find the **cheapest**. The cyclic version is
the **Traveling Salesman Problem** (TSP), perhaps the most-studied optimization
problem in existence.

Brute force enumerates all $n!$ orders — hopeless past $n \approx 12$. The
**Held–Karp** dynamic program (1962) does far better by a classic move: notice
that a partial tour's future depends only on *which* vertices remain and *where*
we currently stand — not on the order in which the visited ones were visited.

### The recurrence

Let $d(i, j)$ be the weight of edge `i–j`. Define

> $C(S, j)$ = the least weight of a path that starts somewhere, visits **exactly
> the set of vertices $S$**, and **ends at $j \in S$**.

Then, peeling off the last vertex $j$:

> **$C(S, j) = \min_{i \in S \setminus \{j\}} \bigl[ C(S \setminus \{j\}, i) + d(i, j) \bigr]$**

with base case $C(\{j\}, j) = 0$. The shortest Hamiltonian **path** is
$\min_j C(V, j)$ over the full set $V$. For the shortest **cycle**, anchor the
start at vertex 0 and close the best path:
$\min_{j \ne 0} C(V, j) + d(j, 0)$.

### Why the subset makes it $2^n$ ↩ Module 13

The state is a pair $(S, j)$: a **subset** $S$ of the $n$ vertices and a last
vertex $j$. There are $2^n$ subsets and $n$ choices of $j$, so $2^n n$ states,
each computed by a $\min$ over $\le n$ predecessors — total **$O(2^n n^2)$** time
and **$O(2^n n)$** space. We encode $S$ as a bitmask in a machine word, exactly
the bitmask-state technique of Module 13 ↩: "vertex $k$ is in $S$" is bit $k$;
$S \setminus \{j\}$ is `S & !(1<<j)`; "adding $k$" is `S | (1<<k)`. Iterating subsets in
**increasing numeric order** is a valid evaluation order, because adding a
vertex only *sets* a bit, so `S | (1<<k) > S` — every state is computed after
all the states it depends on.

### A hand trace: the triangle

$n = 3$, weights $d(0,1)=1, d(1,2)=2, d(0,2)=3$.

| S | j | $C(S,j)$ | from |
|---|---|---|---|
| {0} | 0 | 0 | base |
| {1} | 1 | 0 | base |
| {2} | 2 | 0 | base |
| {0,1} | 1 | 1 | C({0},0)+d(0,1) |
| {0,1} | 0 | 1 | C({1},1)+d(1,0) |
| {1,2} | 1 | 2 | C({2},2)+d(2,1) |
| {1,2} | 2 | 2 | C({1},1)+d(1,2) |
| {0,2} | 2 | 3 | C({0},0)+d(0,2) |
| {0,1,2} | 2 | 3 | C({0,1},1)+d(1,2) = 1+2 |
| {0,1,2} | 0 | 3 | C({1,2},1)+d(1,0) = 2+1 |

Shortest **path** = $\min_j C(\{0,1,2\}, j) = 3$ (the route 0–1–2, skipping the
expensive edge 0–2). Shortest **cycle** = $C(\{0,1,2\},2) + d(2,0) = 3 + 3 = 6$
(the only triangle). Your stage-4 tests check exactly these numbers, then
cross-check the DP against brute-force permutation enumeration on random
instances up to $n = 9$, and confirm $n = 13$ runs in milliseconds.

### The punchline: this is still the best we know

Held–Karp turns $n!$ into $2^n n^2$ — the difference between "impossible at
$n = 15$" and "instant at $n = 20$." And yet **$O(2^n n^2)$ remains, sixty
years later, essentially the best known worst-case bound for general TSP.** No
polynomial algorithm has been found, and (Section 1's asymmetry again) finding
one would collapse P and NP. Held–Karp is the honest state of the art:
exponential, but *tamed* — a subset dimension instead of a factorial one. When
even $2^n$ is too much (routing thousands of stops), practice turns to
*approximation* and *heuristics* — the Warnsdorff spirit, scaled up — which is
why Section 3 was not a detour but a preview.

---

## 6. Stage-by-stage lab guide

Open `labs/module-22-hamilton/src/lab.rs`. Graphs are neighbor lists
`&[Vec<usize>]`, assumed undirected and simple; `adjacency_matrix` transposes
them to an $n \times n$ bool grid for $O(1)$ adjacency checks.

### Stage 1 — `hamiltonian_path`, `hamiltonian_cycle`, `count_hamiltonian_cycles`

Implement Algorithm H. Build the adjacency matrix once; recurse with a `path`
vector and a `used` flag array, restoring state exactly on backtrack (H5). For
the path, launch from every start; for the cycle, fix start 0 and require the
last vertex adjacent to 0. For the count, tally directed closed walks from 0 and
divide by 2. The tests *validate* every returned route (permutation +
consecutive adjacency + closure) rather than comparing to a fixed answer, and
they pin $K_n$ → $(n-1)!/2$, $C_n$ → 1, $P_n$ → path but no cycle, and the Petersen
graph → no cycle but a path.

### Stage 2 — `knight_moves`, `warnsdorff_tour`, `is_valid_tour`

`knight_moves(board, sq)` returns the on-board $(\pm 1, \pm 2)/(\pm 2, \pm 1)$ destinations in
a fixed order — that fixed order is what makes your tie-breaking deterministic.
`warnsdorff_tour` follows Algorithm W: at each step pick the unvisited neighbor
minimizing `(onward-degree, square index)`, and return `None` if you get stuck.
`is_valid_tour` checks that a sequence is a permutation of all squares with
every consecutive pair a knight move. Expect success from corners on `5–8`
boards and from every start on $6 \times 6$; expect the documented **failures** from
$5 \times 5$ square 2 and $8 \times 8$ square 24 — the heuristic's honest limits.

### Stage 3 — `hypercube_neighbors`, `gray_code_cycle`, `is_hamiltonian_cycle_on_hypercube`

`hypercube_neighbors` flips each of the $d$ low bits. `gray_code_cycle` is a
one-liner: `k ^ (k >> 1)` for $k$ in `0..2^d`. The validator checks all $2^d$
vertices appear once and consecutive (cyclically!) vertices differ in one bit.
The tests confirm the sequence *is* a Hamiltonian cycle on $Q_d$, matches the
Module-08 formula bit-for-bit, and that each successive XOR is a single power of
two — a hypercube edge.

### Stage 4 — `shortest_hamiltonian_path`, `shortest_hamiltonian_cycle`

Fill `dp[S][j]` $= C(S, j)$ in increasing subset order using the Held–Karp
recurrence, with a large `INF` sentinel that leaves headroom against overflow.
Answer the path query with `min_j dp[full][j]`; the cycle query anchors at 0 and
closes with $+\, d(j, 0)$. Handle the degenerate $n \le 1$ (weight 0). The tests
hand-check tiny instances, cross-check against brute force for $n \le 9$, accept
arbitrary (triangle-inequality-free, even asymmetric) weights, and time $n = 13$.

Run the growth curve any time with
`cargo run -p lab-22-hamilton --example bench --features solutions --release`.

---

## 7. Check your understanding

1. A connected graph has every vertex of even degree. Which route are you
   *guaranteed*, and which are you *not*? (Eulerian circuit yes; Hamiltonian
   cycle — no guarantee at all.)
2. Why may `count_hamiltonian_cycles` divide by exactly 2, never some other
   number, for every graph with $n \ge 3$? (Each undirected cycle is traversed in
   precisely two directions from the fixed start.)
3. On a $5 \times 5$ board, name a square from which *no* knight's tour exists for
   *any* algorithm, and explain why. (Any minority-colour square — a tour must
   start and end on the majority colour.)
4. In Held–Karp, why is iterating subsets in increasing integer order a valid
   evaluation (topological) order? (Adding a vertex only sets a bit, so a state
   is always numerically larger than the states it depends on.)
5. If someone hands you a polynomial-time algorithm that always decides
   Hamiltonicity correctly, what famous conjecture have they just settled?
   (P = NP.)

## 8. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 up to an hour ·
30 hours · 40 term project · 50 open research. ▶ marks especially instructive
ones. Log attempts in `course/module-22-hamilton/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 22-1 | 10 | Prove $K_n$ has $(n-1)!/2$ Hamiltonian cycles by the fixed-start / divide-by-two argument. |
| 22-2 | 15 | Give the even/odd-degree Eulerian criterion and prove necessity. |
| ▶22-3 | 25 | Add the "endpoint reachability" prune to Algorithm H and measure the search-tree shrinkage on random graphs. |
| 22-4 | 20 | Prove the $5 \times 5$ colour-parity fact: a knight's tour must begin and end on the majority colour. |
| ▶22-5 | 25 | Prove the reflected Gray code is a Hamiltonian cycle on $Q_d$ by induction (fill in every seam of §4). |
| 22-6 | 20 | Modify Held–Karp to also *return the optimal tour*, not just its cost (store parent pointers). |
| ▶22-7 | 30 | Show the decision problem "does $G$ have a Hamiltonian cycle?" is NP-complete via a reduction from 3-SAT (sketch Karp's gadget). |
| 22-8 | 30 | Implement the Held–Karp *space*-saving trick: process subsets by popcount so only two "layers" live in memory. |

## In the real world

- **Logistics and routing.** TSP and its vehicle-routing cousins decide how
  UPS/FedEx trucks, Amazon vans, and airline crews sequence stops; a 1% route
  improvement is millions of dollars and tonnes of CO₂. Held–Karp is the exact
  core; at real scale, solvers switch to cutting planes and heuristics.
- **The Concorde TSP solver** has found *provably optimal* tours through tens of
  thousands of cities (including an 85,900-city VLSI instance) using
  branch-and-cut — exact optimization that leans on exactly the "exponential but
  tamed" mindset of Held–Karp, plus decades of polyhedral theory.
- **PCB drilling and chip manufacture.** A drill or laser visiting thousands of
  holes on a circuit board is a literal TSP; minimizing head travel is minimizing
  Hamiltonian-path length.
- **DNA sequencing and genome assembly.** Reconstructing a sequence from
  overlapping fragments is a Hamiltonian-path problem on an overlap graph — and
  the switch to *Eulerian* de-Bruijn-graph formulations (which are poly-time!)
  was a genuine algorithmic breakthrough, the §1 asymmetry paying a Nobel-adjacent
  dividend.
- **Puzzle and game design.** Knight's tours, Gray-code encoders on rotary
  shafts (physically single-bit-change so a misread never jumps far), and maze
  and level generation all trade on Hamiltonicity.
- **Why NP-completeness matters practically.** It is not a counsel of despair —
  it is a *design signal*. When a problem is NP-complete you stop hunting for a
  fast exact algorithm and instead reach for exponential-but-tamed exact methods
  (Held–Karp, branch-and-cut), approximation algorithms with proven ratios, or
  heuristics (Warnsdorff, local search) with empirical muscle. Knowing *which*
  situation you are in is half the engineering.

## Why it's done this way

- **Backtracking, not a formula**, because Hamiltonicity has no local
  characterization — the honest response to NP-completeness is a smart
  exhaustive search with prunes that cut the tree without changing the answer.
- **Warnsdorff trades the guarantee for speed** on purpose: when a fast bet is
  worth more than a slow certainty, a heuristic is the right tool — *provided*
  you remember it can fail and plan a fallback.
- **The Gray-code bridge** shows that recognizing structure can replace search
  entirely: on the hypercube the answer is a closed-form formula, no
  backtracking needed.
- **Held–Karp reshapes the state space** — a subset dimension ($2^n$) instead of
  a sequence dimension ($n!$) — the single idea that turns a factorial into a
  merely exponential cost, and the bitmask makes that subset a machine word.

## Proof techniques you practiced

- **Invariant + exact state restoration** — Algorithm H's "the prefix is always
  a simple path," undone precisely on backtrack; the master pattern from Module
  01, now guarding a search tree.
- **Induction with a construction** — the reflected Gray code is a Hamiltonian
  cycle on $Q_d$, proved by *building* $G_d$ from $G_{d-1}$ and checking both
  seams. Structural induction that mirrors the algorithm.
- **Parity / colouring arguments** — the $5 \times 5$ knight's-tour impossibility from
  a two-colouring; a local invariant (each move flips colour) forcing a global
  conclusion.
- **Dynamic programming via optimal substructure** — Held–Karp's recurrence
  earns its correctness from "the future depends only on the visited set and the
  current vertex," and its efficiency from evaluating states in a valid order.
- **Counting by symmetry (divide-out the over-count)** — $(n-1)!/2$ cycles in
  $K_n$: fix a representative (start at 0), then divide by the size of the orbit
  (two directions).
- **Complexity as classification** — recognizing Hamiltonicity as NP-complete,
  and *using* that classification to choose an approach rather than to give up.

## You've finished the course

That is the last stage of the last module. Look back at what these twenty-two
modules built: not twenty-two tricks, but *a way of working* — state the
problem precisely, prove what you claim, count what you build, and only then
trust it. This finale made the connective tissue explicit on purpose:
backtracking (09), bitmask state (13), and Gray-code generation (08) all
reappeared here as facets of one NP-complete question, and the invariant-plus-
measure pattern from Euclid in Module 01 was still doing the correctness work at
the very end.

Now turn to **[`docs/toolkit.md`](../../docs/toolkit.md)** and read it as a
mirror. Every proof technique in that table you have now practiced at least
twice — induction, counting, extremal witnesses, potential arguments,
reductions, certification, and the rest — which is the difference between having
*seen* a tool and *owning* it. The books go further (Vol. 4C is still being
written, and §7.2.2.4 is where these Hamiltonian ideas will land in full), but
you leave with the thing Knuth most wants you to have: the habit of turning a
vague "it works" into a precise, proved, counted, and measured claim. Go build
something, and prove it correct.

## 9. Where this leads

- **Vol. 4B/4C, §7.2.2.3–7.2.2.4** develop constraint satisfaction and
  Hamiltonian paths in depth, with far stronger pruning than we used here.
- **Approximation algorithms** (Christofides' $3/2$-approximation for metric TSP)
  and **local search** (2-opt, Lin–Kernighan) are the practical sequel to
  Held–Karp when $2^n$ is too much.
- **Exact exponential algorithms** study exactly *how* exponential these
  problems must be — for general weighted TSP no algorithm beating Held–Karp's
  $2^n \cdot \mathrm{poly}(n)$ is known, and improving it is a long-standing
  open problem (though special cases, such as unweighted Hamiltonicity, can be
  solved faster).
- **The whole course** now lives in [`docs/toolkit.md`](../../docs/toolkit.md):
  read it, and see how far you've come.
