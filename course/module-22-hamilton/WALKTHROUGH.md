# Walkthrough — Module 22 (read after your stage is green)

Design commentary on the reference implementation
(`reference/src/m22_hamilton.rs`). Not needed to pass — only to deepen. Each
section is the "compare with Knuth's answer" step made explicit.

## Stage 1: Hamiltonian paths by backtracking

**Shape.** One recursive `extend(mat, start, path, used, want_cycle)` serves
*both* `hamiltonian_path` and `hamiltonian_cycle`; the single `want_cycle` flag
decides the acceptance test at the leaf (`path.len() == n`). Reusing the engine
keeps the invariant in one place and means a bug can only be fixed once. The
count uses a separate `count_cycles` recursion because it must exhaust *all*
leaves rather than return at the first.

**The invariant.** At every entry to `extend`, `path` is a simple path in the
graph and `used[v]` is exactly `path.contains(v)`. `push` + `used[v]=true`
before the recursive call and `pop` + `used[v]=false` after it keep the two in
lockstep — *exact* state restoration. This is why backtracking needs no copying
of the whole state: the mutations are perfectly undone. Steal this pattern; it
is the difference between an O(1)-per-node search and an O(n)-per-node one.

**Why fix start 0 for cycles.** Every Hamiltonian cycle passes through every
vertex, so in particular through 0; searching from 0 alone loses no cycle and
saves an `n×` factor over trying all starts. For *paths* we cannot fix the start
(a path need not touch any particular vertex first), so we loop over starts.

**Divide by two, and only by two.** `count_hamiltonian_cycles` counts directed
closed walks from the fixed start 0, then halves. The halving is exact for every
graph with `n ≥ 3` because each undirected cycle is walked in exactly two
directions from 0 — never one (a cycle has a well-defined reverse) and never
more (the start is fixed). That is a counting-by-symmetry argument: fix a
representative, divide by the orbit size.

**Beating the naive version.** The naive approach generates all `n!`
permutations and filters; ours prunes the instant an edge is missing, so on
sparse graphs (Petersen: 3-regular) it visits a tiny fraction of the tree. The
`adjacency_matrix` precomputation trades O(n²) space for O(1) adjacency tests
inside the hot loop — worth it because the loop runs far more often than `n²`.

## Stage 2: Warnsdorff's heuristic — the knight's tour

**Shape.** `knight_moves` lists destinations in a *fixed offset order*; that
order is not cosmetic — it is the deterministic tie-break that makes
`warnsdorff_tour` reproducible. Change the offset order and you change which
starts succeed. Encoding the tie-break in a stable data order (rather than a
scattered `if`) is an idiom worth stealing.

**The greedy choice, as a min over a pair.** The core is
`best = min over unvisited neighbors of (onward_degree, square)`. Folding the
tie-break into the *second* component of the tuple means one comparison handles
both the primary rule (fewest onward moves) and the tie-break (smaller index),
with no special-casing. The `match best { … }` fold expresses "keep the running
minimum" without a sentinel `usize::MAX`.

**The invariant that makes it *fast* (not correct).** Warnsdorff has *no*
correctness invariant — it is a heuristic and can fail. What it has is a
*progress* guarantee: each iteration marks one square, so it terminates in
`board²` steps regardless of success. Returning `None` on a dead end is a
first-class outcome, not an error; the reference treats "stuck" and "done" as
the two normal loop exits.

**Beating a naive tour search.** A backtracking tour solver is *correct* but can
be slow on large boards; Warnsdorff is `O(board²)` and usually wins outright.
The reference's honesty is the point: its doc comment and unit test record
*specific* failures (`5×5` sq 2, `8×8` sq 24) where a tour provably exists — so
you never mistake this fast tool for a guarantee. The right production design
pairs the two: try Warnsdorff, fall back to backtracking on `None`.

## Stage 3: Hamiltonian cycles on the hypercube are Gray codes

**Shape.** Three tiny functions, and that tininess *is* the lesson: because the
hypercube has so much structure, the Hamiltonian cycle needs no search at all —
`gray_code_cycle` is the single expression `k ^ (k >> 1)`. When you can
recognize structure, you replace an exponential search with a closed form.

**The invariant.** `is_hamiltonian_cycle_on_hypercube` verifies two things: a
permutation (all `2^d` vertices once, via a `seen` array) and the *cyclic*
single-bit-step property. The subtlety is the wrap: `cycle[(i+1) % n]` includes
the closing edge from last back to first. Forgetting the `% n` would validate a
Hamiltonian *path* while claiming a *cycle* — a classic off-by-one that the
tests catch by construction.

**Why `count_ones() == 1` is the whole story.** Two hypercube vertices are
adjacent iff their XOR is a power of two, i.e. has exactly one set bit. So the
adjacency test, the Gray-code "changes one bit" property, and "is a hypercube
edge" are *literally the same predicate*. The reference leans on that identity
rather than re-deriving three separate checks.

**Beating the naive version.** You *could* run stage 1's general backtracking on
`Q_d` to find a Hamiltonian cycle — and it would work — but that is exponential
effort to rediscover a formula. Recognizing `Q_d`'s cycle as the reflected Gray
code is the difference between O(2^d)-search and O(2^d)-*write-down*.

## Stage 4: Held–Karp — shortest Hamiltonian path by bitmask DP

**Shape.** A flat `dp[1<<n][n]` table filled in a single forward pass, "push"
style: from each reachable `(S, j)` we *relax* every extension `(S|1<<k, k)`.
The push form reads naturally as "extend this path by one vertex" and avoids the
inner `min` needing to know its predecessors explicitly.

**The invariant / evaluation order.** The correctness of the single increasing
`for s in 1..full` loop rests on one fact: `S | (1<<k) > S` whenever bit `k` was
clear. So when we read `dp[S][j]` it is already final — every state is completed
before any state that depends on it. That is a topological order for free, no
recursion or memo table needed. The `if cur >= INF { continue }` guard skips
unreachable states so `INF + weight` never overflows (and `INF = u64::MAX/4`
leaves headroom even without the guard).

**Why the subset dimension is the whole trick.** Brute force carries the *entire
order* of visited vertices — `n!` of them. Held–Karp observes the future depends
only on the *set* visited and the *current* vertex, collapsing `n!` histories
into `2^n` subsets. The bitmask (Module 13) makes that subset a machine word, so
`S\{j}` and `S∪{k}` are single instructions. The reference cycle variant fixes
`dp[1][0]=0` and forbids stepping onto 0 mid-tour (`for k in 1..n`), then closes
with `+ d[j][0]` — the minimal change from path to TSP.

**Beating the naive version.** The stage-4 tests keep an explicit brute-force
permutation enumerator as an oracle for `n ≤ 9`; the DP agrees with it exactly
while running in `O(2^n n²)` instead of `O(n·n!)`. The `bench.rs` example makes
the `~2×`-per-city growth visible — the honest face of NP-hardness, and the
reason the lesson spends real time on heuristics and approximation.
