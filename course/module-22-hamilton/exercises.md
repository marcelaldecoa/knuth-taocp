# Exercises — Module 22 (Hamiltonian Paths and Cycles)

Self-contained problems on this module's material — counting Hamiltonian cycles,
the Euler/Hamilton asymmetry, backtracking prunes, the knight's-tour parity
obstruction, the Gray-code/hypercube bridge, Held–Karp, and NP-completeness. You
can work every one **without the books**: each states the problem in full, gives
a **hint** to peek at when stuck, and a worked **answer sketch** to check against
after you try. Numeric answers here are reproduced by the code you write in the
lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). These problems track the pre-fascicle material toward §7.2.2.4 and
the earlier modules it ties together (08 Gray codes, 09 backtracking, 13 bitmask
states).

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | $K_n$ has $(n-1)!/2$ Hamiltonian cycles (fixed-start / divide-by-two) | 10 | ⬜ |
| 2 | Eulerian even-degree criterion — prove necessity | 15 | ⬜ |
| 3 | ▶ Add endpoint-reachability pruning to Algorithm H | 25 | ⬜ |
| 4 | $5\times5$ knight-tour colour-parity impossibility | 20 | ⬜ |
| 5 | ▶ Reflected Gray code = Hamiltonian cycle on $Q_d$ (induction) | 25 | ⬜ |
| 6 | Held–Karp with parent pointers (reconstruct the tour) | 20 | ⬜ |
| 7 | ▶ Hamiltonicity is NP-complete via 3-SAT (reduction sketch) | 30 | ⬜ |
| 8 | Held–Karp by popcount layers (space saving) | 30 | ⬜ |

## Problems

### 1. $K_n$ has $(n-1)!/2$ Hamiltonian cycles (rating 10 · cf. Ex. 22-1)

**Problem.** The complete graph $K_n$ has an edge between every pair of vertices.
Prove it has exactly $(n-1)!/2$ (undirected) Hamiltonian cycles, using the
fixed-start / divide-by-two argument the module's `count_hamiltonian_cycles`
embodies. Confirm the value for $n = 5$.

**Hint.** Anchor the cycle at vertex 0 to kill the "where does it start" freedom,
count the *directed* orderings of the remaining vertices, then divide out the
two directions each undirected cycle is walked in.

**Answer sketch.** Every Hamiltonian cycle contains vertex 0, so fix it as the
start. After 0, the other $n-1$ vertices can be visited in **any** order (all
edges exist in $K_n$), giving $(n-1)!$ directed closed walks $0 \to \cdots \to
0$. But each *undirected* cycle is traced by exactly **two** such walks — one
each way around — so the undirected count is

$$
\frac{(n-1)!}{2}.
$$

This is *counting by symmetry*: fix a representative (start at 0) to remove the
rotational over-count, then divide by the orbit size (2 directions). For $n = 5$:
$4!/2 = 12$, matching the value the lab pins for $K_5$. (For $n = 3$ it gives 1 —
the triangle is its own unique cycle.)

### 2. Eulerian even-degree criterion — prove necessity (rating 15 · cf. Ex. 22-2)

**Problem.** State Euler's criterion and prove its **necessity**: if a connected
graph has an Eulerian circuit (a closed trail using every edge exactly once),
then every vertex has even degree. Do the same for an open Eulerian trail (zero
or two odd-degree vertices).

**Hint.** Walk along the circuit and account for the edges at a single vertex.
Every time the walk arrives, what must it do next, and how many edges does one
visit consume?

**Answer sketch.** **Criterion (README §1):** a connected graph has an Eulerian
circuit iff every vertex has even degree, and an Eulerian trail iff exactly zero
or two vertices have odd degree. **Necessity (circuit).** Follow the Eulerian
circuit and consider any vertex $v$. Each time the walk **enters** $v$ it must
**leave** again (it is a closed trail that ends elsewhere until the very end), so
each visit to $v$ uses one incoming and one outgoing edge — a *pair*. Since every
edge incident to $v$ is used exactly once and edges at $v$ are consumed two per
visit, $\deg(v) = 2 \cdot (\text{number of visits to } v)$, which is **even**.
**Open trail.** The two endpoints of an open trail are entered/left an unequal
number of times: the start leaves once more than it enters, the end enters once
more than it leaves, so each has **odd** degree; every other vertex is still
entered-and-left in pairs, hence even. So an open trail forces exactly two
odd-degree vertices. This is the local, per-vertex condition whose *composability*
the module contrasts with Hamiltonicity's global entanglement — the moral of §1.

### 3. ▶ Add endpoint-reachability pruning to Algorithm H (rating 25 · cf. Ex. 22-3)

**Problem.** Algorithm H (backtracking for Hamiltonicity) explores a search tree
that can explode. Add the **endpoint-reachability** prune of §2 and argue it is
sound (never rejects a completable partial path), then describe how to measure
its effect on the search-tree size for random graphs.

**Hint.** After committing a partial path with endpoint $x_{l-1}$, look at each
still-unused vertex $u$. Under what neighbor condition can $u$ provably never be
reached by any completion, so you may backtrack immediately? Be careful to
exempt the vertex you could still step to next.

**Answer sketch.** **The prune.** At each entry to H2 with partial path $x_0
\cdots x_{l-1}$, scan the unused vertices. An unused vertex $u$ can appear in a
completion only if it is reachable, which requires it to have at least one unused
neighbor to be *entered* from and (unless it becomes the final endpoint) another
to *leave* by. Concretely: if some unused $u$ has **all** its neighbors already
used — and $u$ is not itself the next vertex we could step to from $x_{l-1}$ —
then no completion can ever include $u$, yet every vertex must be included, so
**backtrack now**. (A stronger form for cycles: every unused vertex needs $\ge 2$
unused-or-endpoint neighbors.) **Soundness.** The prune only cuts a branch when a
mandatory vertex is provably unreachable in *every* extension of the current
prefix, so it removes no branch that leads to a valid Hamiltonian completion — it
changes *how fast*, never *what* is computed (README §2). **Measuring it.** Count
the number of times H4 fires (nodes of the search tree entered) with and without
the prune, over a batch of random graphs at fixed $n$ and edge density, and
report the ratio; the prune's value is largest near the Hamiltonicity threshold
density, where naïve search wanders deepest before failing. (Exact shrinkage is
instance-dependent — this is an empirical measurement, not a closed formula.)

### 4. $5\times5$ knight-tour colour-parity impossibility (rating 20 · cf. Ex. 22-4)

**Problem.** Two-colour a $5\times5$ chessboard like a checkerboard. Prove that
a knight's tour (a Hamiltonian path on the knight-move graph) must **begin and
end on the majority colour**, so no tour can start from a minority-colour square.

**Hint.** What does a single knight move do to the colour of the square? Then a
25-square tour is an alternating colour sequence — count how many of each colour
it must contain.

**Answer sketch.** A knight always jumps from a square of one colour to a square
of the **opposite** colour (a $(\pm1,\pm2)$ or $(\pm2,\pm1)$ move changes $r + c$
parity by an odd amount). So along any tour the colours **strictly alternate**.
On the $5\times5$ board the two colours split **13 / 12** (the corner colour,
with $(r+c)$ even, is the majority — verified by direct count). A tour visits all
25 squares in an alternating colour sequence of length 25; such a sequence has
$\lceil 25/2 \rceil = 13$ squares of the colour it *starts* with and
$\lfloor 25/2 \rfloor = 12$ of the other. To use all 13 majority and all 12
minority squares, the sequence must start (and, by the same count, end) on the
**majority** colour. Hence a tour beginning on a minority square is impossible
for *any* algorithm. This is why the module is careful to note that Warnsdorff's
failure from square 2 is the heuristic's fault (square 2 is a majority square, so
a tour from it exists) and *not* one of these parity-forbidden starts — a
local invariant (each move flips colour) forcing a global impossibility.

### 5. ▶ Reflected Gray code = Hamiltonian cycle on $Q_d$ (rating M25 · cf. Ex. 22-5)

**Problem.** The hypercube $Q_d$ has the $2^d$ binary strings as vertices, with
an edge between strings differing in exactly one bit. Prove by induction on $d$
that the reflected binary Gray code $g(k) = k \oplus \lfloor k/2 \rfloor$ (i.e.
`k ^ (k >> 1)`) for $k = 0, \dots, 2^d - 1$ lists every vertex once with
consecutive strings — **including the wrap-around** — differing in a single bit,
so it is a Hamiltonian cycle on $Q_d$. Fill in *both* seams.

**Hint.** Build $G_d$ from $G_{d-1}$ by the reflect-and-prefix rule: prefix 0 to
$G_{d-1}$ forward, then prefix 1 to $G_{d-1}$ reversed. Check the middle seam and
the wrap-around separately.

**Answer sketch.** **Base $d = 1$:** $G_1 = 0, 1$; the step $0 \to 1$ and the
wrap $1 \to 0$ each flip one bit. **Step:** assume $G_{d-1}$ is a Hamiltonian
cycle on $Q_{d-1}$. Form $G_d$ as (i) each string of $G_{d-1}$ prefixed with 0,
in order, then (ii) each string of $G_{d-1}$ prefixed with 1, in **reverse**
order. Three checks: **All vertices once** — the first half is exactly the
$2^{d-1}$ strings beginning $0\ldots$ (once each, by hypothesis), the second half
the $2^{d-1}$ beginning $1\ldots$; together all $2^d$, no repeats. **Steps inside
a half** flip a single low-order bit (the hypothesis, read forward in half one
and backward — still single-bit — in half two) and never touch the new top bit.
**The two seams:** at the *middle*, half one ends at $0w$ and half two begins at
$1w$ (same $w = $ last string of $G_{d-1}$, since the reflection restarts from
it), differing only in the top bit; at the *wrap-around*, $G_d$ ends at $1u$ and
restarts at $0u$ (same $u = $ first string of $G_{d-1}$), again a lone top-bit
flip. All three hold, so $G_d$ is a Hamiltonian cycle on $Q_d$. $\blacksquare$
The reflection *doubles* a $Q_{d-1}$ cycle into a $Q_d$ cycle, using the single
new bit to stitch the two copies at both ends — the very "reflected" step of
Module 08's Gray code, now read as a hypercube tour. (Verify computationally:
each successive XOR $g(k)\oplus g(k+1)$ is a single power of two — a hypercube
edge.)

### 6. Held–Karp with parent pointers (reconstruct the tour) (rating 20 · cf. Ex. 22-6)

**Problem.** The Held–Karp DP computes the *cost* $C(S, j)$ of the cheapest path
visiting set $S$ and ending at $j$, but returns only a number. Modify it to also
return the optimal **tour** (the vertex sequence), by storing parent pointers.

**Hint.** When you take the $\min$ over predecessors $i$ in
$C(S, j) = \min_i [C(S \setminus \{j\}, i) + d(i, j)]$, remember *which* $i$ won.
Then walk backward from the optimal final state.

**Answer sketch.** Alongside `dp[S][j]` keep `par[S][j] = ` the argmin $i^*$ that
achieved $C(S, j)$ — the vertex visited just before $j$ on the best path to state
$(S, j)$. After the table is filled, pick the optimal final state: for a **path**,
$j^* = \arg\min_j \mathrm{dp}[V][j]$ (full set $V$); for a **cycle**, $j^* =
\arg\min_{j \ne 0} \big(\mathrm{dp}[V][j] + d(j, 0)\big)$. Then **reconstruct by
walking parents backward**: start at $(V, j^*)$, emit $j^*$, move to
$(V \setminus \{j^*\}, \mathrm{par}[V][j^*])$, emit that vertex, and repeat —
each step removes the current endpoint's bit from $S$ — until $S$ is a singleton.
Reversing the emitted list gives the tour from start to $j^*$ (append the closing
edge to 0 for a cycle). This adds only $O(2^n n)$ extra memory (one parent per
state) and $O(n)$ reconstruction time, and turns Held–Karp from a *decision/cost*
routine into one that returns the witnessing route — the same "store the choice,
replay it" pattern used to recover any DP's optimal object.

### 7. ▶ Hamiltonicity is NP-complete via 3-SAT (rating 30 · cf. Ex. 22-7)

**Problem.** Sketch a proof that "does $G$ have a Hamiltonian cycle?" is
NP-complete, via a reduction from 3-SAT (Karp, 1972). Cover both obligations:
membership in NP, and hardness by a polynomial-time reduction with variable and
clause gadgets.

**Hint.** NP membership is a one-liner (guess and check). For hardness, design a
graph where a Hamiltonian cycle is *forced* to encode a truth assignment: one
gadget per variable that can be traversed in two directions (= true / false), and
one node per clause that can be "picked up" only by a literal that satisfies it.

**Answer sketch.** **In NP.** A Hamiltonian cycle is a permutation of the $n$
vertices; given one as a certificate, checking it visits each vertex once and
every consecutive pair (and the closing pair) is an edge takes $O(n)$ time. So
the problem is in NP. **NP-hardness (3-SAT $\le_p$ Ham. cycle).** Given a 3-CNF
formula with variables $u_1, \dots, u_m$ and clauses $c_1, \dots, c_k$, build a
graph in polynomial size:

- **Variable gadget.** For each variable $u_i$ include a "chain" that a
  Hamiltonian cycle must traverse either **left-to-right** or
  **right-to-left**; the two directions encode $u_i = \text{true}$ /
  $u_i = \text{false}$. Chaining the gadgets in series forces the cycle to pick a
  direction for *every* variable — i.e. a full truth assignment.
- **Clause gadget.** For each clause $c_j$ add a vertex, and wire it so the cycle
  can detour through $c_j$ **only from a variable chain traversed in the
  direction that makes one of $c_j$'s three literals true.** A clause vertex must
  be visited exactly once, so it is coverable iff at least one literal satisfies
  the clause.

A Hamiltonian cycle then exists **iff** there is a direction-choice (assignment)
under which every clause vertex can be attached — i.e. iff the formula is
satisfiable. The construction has $O(m + k)$ vertices per gadget and is built in
polynomial time, so it is a valid reduction. Since 3-SAT is NP-complete, so is
Hamiltonian cycle. **Consequence (README §1).** This is why no polynomial-time
Hamiltonicity test is known — one would put every NP problem in P — and why the
module answers the question by *search* (Algorithm H) and *exponential-but-tamed*
DP (Held–Karp) rather than a formula. (This is the standard textbook reduction
sketched at the gadget level; the fiddly wiring that makes each gadget behave
exactly as claimed is Karp's, and is where the full proof spends its effort.)

### 8. Held–Karp by popcount layers (space saving) (rating 30 · cf. Ex. 22-8)

**Problem.** Held–Karp as written stores `dp[S][j]` for all $2^n$ subsets at once
— $O(2^n n)$ memory. Show how to compute the optimal *cost* while keeping only
**two "layers"** of subsets in memory, by processing subsets in order of
**popcount**, and quantify the memory saved.

**Hint.** In $C(S, j) = \min_i [C(S \setminus \{j\}, i) + d(i, j)]$, how does the
popcount of $S \setminus \{j\}$ compare to that of $S$? Which subsets does a
size-$s$ subset depend on? Then ask how large one popcount layer can be.

**Answer sketch.** **Why two layers suffice.** A state $(S, j)$ with
$|S| = s$ depends only on states $(S \setminus \{j\}, i)$ with $|S \setminus
\{j\}| = s - 1$. So if we process subsets **in increasing popcount order** — a
valid topological order, since removing a set bit strictly lowers popcount, just
as the module's "increasing numeric order" works because adding a bit raises the
value — then computing layer $s$ needs only layer $s - 1$. Keep exactly those two
layers; discard layer $s - 2$ before starting layer $s$. **Memory saved.** Peak
storage becomes proportional to the two largest adjacent layers, i.e.
$O\!\big(\binom{n}{\lfloor n/2 \rfloor} \cdot n\big)$ instead of $O(2^n n)$.
Since $\binom{n}{\lfloor n/2 \rfloor} \approx 2^n / \sqrt{n}$, this trims peak
memory by a factor of roughly $\sqrt{n}$ (asymptotically); at the lab's $n = 13$
the full table holds $2^{13}\cdot 13 = 106496$ entries while the two-layer peak
is only about $2\binom{13}{6}\cdot 13 \approx 44616$ — a measured $\approx 2.4\times$
reduction, growing with $n$. **Caveat.** This saves memory only for the *cost*;
reconstructing the tour (Problem 6) needs the parent of each state, so full
tour-recovery still wants the whole `par` table (or a recompute pass). The time
stays $O(2^n n^2)$ — only the space footprint shrinks.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
