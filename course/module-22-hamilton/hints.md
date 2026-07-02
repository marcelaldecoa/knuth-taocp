# Hints — Module 22

Graduated hints. Read only as far as you need. `./grade 22 --stage K --hint J`.

## Stage 1: Hamiltonian paths by backtracking

1. This is Algorithm 7.2.2B (basic backtrack) with one change: the "property
   test" that admits a candidate is *unvisited AND adjacent to the current
   endpoint*. The partial solution is a simple path; the level is its length.
2. Build the adjacency matrix once so "is u adjacent to v?" is O(1). Recurse
   with a `path: Vec<usize>` and a `used: Vec<bool>`; on backtrack, `path.pop()`
   and clear the `used` flag — restore the earlier state *exactly*. A path may
   start anywhere (loop over all starts); a cycle may fix start 0.
3. For the count, tally *directed* closed walks from vertex 0 and divide by 2
   (each undirected cycle runs in two directions). Cycle acceptance at level n:
   `matrix[last][start]`. Return `None`/`0` when `n < 3` for cycles.
   ```text
   fn extend(mat, start, path, used, want_cycle) -> bool:
     if path.len()==n: return !want_cycle || mat[last][start]
     for v in 0..n where !used[v] && mat[last][v]: try v, recurse, undo
   ```

## Stage 2: Warnsdorff's heuristic — the knight's tour

1. The board graph: square `sq = row*board + col`; edges are the eight
   `(±1,±2)/(±2,±1)` knight offsets that land on the board. Warnsdorff is a
   *greedy* rule — no backtracking — so it may return `None`.
2. `knight_moves` should emit destinations in a fixed offset order; that fixed
   order is your deterministic tie-break. At each step, score each unvisited
   neighbor `v` by how many unvisited neighbors *it* has, and move to the
   minimizer.
3. Break ties by smaller square index: pick the `v` minimizing the pair
   `(onward_degree, v)`. If there is no unvisited neighbor, the walk is stuck →
   return `None`. `is_valid_tour`: length `board²`, a permutation, and every
   `windows(2)` pair is a legal knight move.
   ```text
   deg(v) = knight_moves(board,v).filter(|s| !visited[s]).count()
   next   = argmin over unvisited neighbors of (deg(v), v)
   ```

## Stage 3: Hamiltonian cycles on the hypercube are Gray codes

1. `Q_d` has the `2^d` `d`-bit strings as vertices; two are adjacent iff they
   differ in one bit. A Hamiltonian cycle on `Q_d` is exactly a cyclic Gray
   code — the reflected code `g(k) = k ⊕ ⌊k/2⌋` from Module 08.
2. `hypercube_neighbors(d, v)`: XOR `v` with each `1<<i` for `i` in `0..d`.
   `gray_code_cycle(d)`: map `k -> k ^ (k>>1)` over `k` in `0..(1<<d)`.
3. The validator must check the *cyclic* single-bit property, i.e. include the
   wrap-around from the last vertex to the first:
   ```text
   all 2^d vertices distinct and in range, AND
   for i in 0..n:  (cycle[i] ^ cycle[(i+1) % n]).count_ones() == 1
   ```

## Stage 4: Held–Karp — shortest Hamiltonian path by bitmask DP

1. State `C(S, j)` = least-weight path visiting exactly vertex set `S` and
   ending at `j ∈ S`. Recurrence: `C(S,j) = min_{i∈S\{j}} C(S\{j}, i) + d(i,j)`,
   base `C({j}, j) = 0`. The subset `S` is a bitmask (Module 13).
2. Use a `dp[1<<n][n]` table filled by iterating subsets `S` in *increasing
   integer order* (valid because adding a bit only grows `S`). Use an `INF`
   sentinel like `u64::MAX/4` so `INF + weight` never overflows.
3. Push form: from a reachable `(S, j)`, extend to each `k ∉ S` via
   `dp[S|1<<k][k] = min(…, dp[S][j] + d[j][k])`. Path answer:
   `min_j dp[full][j]`. Cycle: fix start 0 (`dp[1][0]=0`, never revisit 0
   mid-tour), then `min_{j≠0} dp[full][j] + d[j][0]`. Return 0 for `n ≤ 1`.
