//! Module 22 — Hamiltonian Paths and Constraint Satisfaction.
//! Source: toward TAOCP Vol. 4C, §7.2.2.4 (Hamiltonian paths and cycles),
//! with threads back to §7.2.1.1 (Gray codes), §7.2.2 (backtracking), and
//! §7.1.3 (bitmask states).
//!
//! This is the course finale. Four stages, each a different lens on the same
//! NP-complete question — *does a route through every vertex exist?* —
//! ranging from raw backtracking, through a greedy heuristic, to an exact
//! exponential dynamic program, with a detour proving that the reflected
//! Gray code you built in Module 08 *is* a Hamiltonian cycle on a hypercube.
//!
//! Graph convention: a graph on `n` vertices `0..n` is given by **neighbor
//! lists** `adj: &[Vec<usize>]`, where `adj[u]` lists the vertices adjacent to
//! `u`. Graphs are assumed **undirected and simple** (so `adj` is symmetric,
//! no self-loops, no repeats). Neighbor lists are the right shape for the
//! sparse graphs backtracking chews on; when constant-time adjacency queries
//! help, transpose to an `n × n` bool matrix with [`adjacency_matrix`].

// ===========================================================================
// Stage 1 — Hamiltonian paths and cycles by backtracking (§7.2.2.4).
// ===========================================================================

/// Build an `n × n` boolean adjacency matrix from neighbor lists.
///
/// `matrix[u][v] == true` iff `v` appears in `adj[u]`. Backtracking asks
/// "is `u` adjacent to `v`?" once per edge examined; the matrix answers in
/// O(1) instead of scanning a neighbor list.
pub fn adjacency_matrix(adj: &[Vec<usize>]) -> Vec<Vec<bool>> {
    let n = adj.len();
    let mut m = vec![vec![false; n]; n];
    for (u, nbrs) in adj.iter().enumerate() {
        for &v in nbrs {
            if v < n {
                m[u][v] = true;
            }
        }
    }
    m
}

/// A **Hamiltonian path**: a permutation of all `n` vertices in which every
/// two consecutive vertices are adjacent. Returns one such path, or `None`.
///
/// We reuse the skeleton of Algorithm 7.2.2B (basic backtrack). The partial
/// solution is a simple path `x_0 x_1 ... x_{l-1}`; the "level" `l` is its
/// length; the property tested at each extension is *unvisited and adjacent
/// to the current endpoint*. A path may begin anywhere, so we launch the
/// search from every vertex and stop at the first success.
///
/// ```text
/// H1. [Initialize.]  Choose a start vertex s; set the path to (s).
/// H2. [Done?]        If the path has all n vertices, report it.
/// H3. [Extend.]      Let v range over vertices adjacent to the endpoint that
///                    are not yet on the path.
/// H4. [Advance.]     Append v; recurse from H2.
/// H5. [Backtrack.]   Remove v; try the next candidate. If none, back up.
/// ```
pub fn hamiltonian_path(adj: &[Vec<usize>]) -> Option<Vec<usize>> {
    let n = adj.len();
    if n == 0 {
        return Some(vec![]);
    }
    if n == 1 {
        return Some(vec![0]);
    }
    let mat = adjacency_matrix(adj);
    let mut path = Vec::with_capacity(n);
    let mut used = vec![false; n];
    // H1. Try each possible start vertex.
    for s in 0..n {
        path.clear();
        used.iter_mut().for_each(|b| *b = false);
        path.push(s);
        used[s] = true;
        if extend(&mat, s, &mut path, &mut used, false) {
            return Some(path);
        }
    }
    None
}

/// A **Hamiltonian cycle**: a Hamiltonian path whose endpoints are also
/// adjacent, so it closes into a loop through every vertex exactly once.
/// Returns the cycle as `n` vertices `x_0 ... x_{n-1}` (the closing edge
/// `x_{n-1} -> x_0` is implicit), or `None`.
///
/// Every Hamiltonian cycle passes through vertex 0, so — unlike the path
/// search — we may fix the start at 0 without loss of generality.
pub fn hamiltonian_cycle(adj: &[Vec<usize>]) -> Option<Vec<usize>> {
    let n = adj.len();
    if n < 3 {
        return None; // a simple cycle needs at least three vertices
    }
    let mat = adjacency_matrix(adj);
    let mut path = Vec::with_capacity(n);
    let mut used = vec![false; n];
    path.push(0);
    used[0] = true;
    if extend(&mat, 0, &mut path, &mut used, true) {
        Some(path)
    } else {
        None
    }
}

/// Steps H2–H5: extend the current path, honoring the cycle constraint when
/// `want_cycle` is set. Returns `true` (leaving a full solution in `path`) as
/// soon as one is found.
fn extend(
    mat: &[Vec<bool>],
    start: usize,
    path: &mut Vec<usize>,
    used: &mut [bool],
    want_cycle: bool,
) -> bool {
    let n = mat.len();
    let last = *path.last().unwrap();
    // H2. [Done?] All vertices placed — accept iff the cycle can close.
    if path.len() == n {
        return !want_cycle || mat[last][start];
    }
    // H3. [Extend.] Candidates: unvisited neighbors of the endpoint.
    for v in 0..n {
        if !used[v] && mat[last][v] {
            // H4. [Advance.]
            used[v] = true;
            path.push(v);
            if extend(mat, start, path, used, want_cycle) {
                return true;
            }
            // H5. [Backtrack.]
            path.pop();
            used[v] = false;
        }
    }
    false
}

/// Count the **distinct undirected Hamiltonian cycles** of a graph.
///
/// We fix vertex 0 as the start (every Hamiltonian cycle contains it) and
/// count *directed* closed walks `0 -> ... -> 0` visiting each vertex once.
/// Each undirected cycle is then counted exactly twice — once in each
/// direction — so we divide by 2. For a graph with fewer than three vertices
/// there are no simple cycles, and the count is 0.
///
/// Sanity checks the tests pin: the complete graph `K_n` has `(n-1)!/2`
/// Hamiltonian cycles; the cycle graph `C_n` has exactly 1; the Petersen
/// graph has 0.
pub fn count_hamiltonian_cycles(adj: &[Vec<usize>]) -> u64 {
    let n = adj.len();
    if n < 3 {
        return 0;
    }
    let mat = adjacency_matrix(adj);
    let mut used = vec![false; n];
    used[0] = true;
    let mut count = 0u64;
    count_cycles(&mat, 0, 0, 1, &mut used, &mut count);
    count / 2
}

fn count_cycles(
    mat: &[Vec<bool>],
    start: usize,
    last: usize,
    depth: usize,
    used: &mut [bool],
    count: &mut u64,
) {
    let n = mat.len();
    if depth == n {
        if mat[last][start] {
            *count += 1;
        }
        return;
    }
    for v in 0..n {
        if !used[v] && mat[last][v] {
            used[v] = true;
            count_cycles(mat, start, v, depth + 1, used, count);
            used[v] = false;
        }
    }
}

// ===========================================================================
// Stage 2 — Warnsdorff's rule: the knight's tour (§7.2.2.4's heuristic).
// ===========================================================================

/// The legal knight moves from square `sq` on a `board × board` chessboard.
///
/// Squares are numbered `row * board + col`, `0 <= row, col < board`. The
/// eight `(±1, ±2)` / `(±2, ±1)` offsets are tried in a fixed order, and the
/// on-board destinations are returned in that order (this fixed order is what
/// makes the tie-breaking in [`warnsdorff_tour`] deterministic).
pub fn knight_moves(board: usize, sq: usize) -> Vec<usize> {
    let b = board as i64;
    let (r, c) = ((sq / board) as i64, (sq % board) as i64);
    const DELTAS: [(i64, i64); 8] = [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ];
    let mut out = Vec::with_capacity(8);
    for (dr, dc) in DELTAS {
        let (nr, nc) = (r + dr, c + dc);
        if nr >= 0 && nr < b && nc >= 0 && nc < b {
            out.push((nr as usize) * board + nc as usize);
        }
    }
    out
}

/// Warnsdorff's rule (1823): a greedy heuristic for the open knight's tour.
///
/// From the current square, always step to the **unvisited neighbor with the
/// fewest onward unvisited moves** — head for the most cramped corner first,
/// while you still can. Ties are broken deterministically by choosing the
/// smaller square index. Returns the full tour (all `board²` squares, in
/// visiting order) if the greedy walk completes, or `None` if it paints
/// itself into a corner.
///
/// This is a *heuristic*, not an algorithm in Knuth's strict sense: it never
/// backtracks, so on some boards and start squares it fails. With this
/// tie-break the greedy walk completes a `5 × 5` tour from a corner and from
/// the center, yet it gets stuck when started from square 2 of that same
/// board — even though a full tour from square 2 provably exists (backtracking
/// finds one). Heuristics are not guarantees; that honesty is the lesson.
/// See the module README.
///
/// ```text
/// W1. [Initialize.]  Mark the start visited; it is the current square.
/// W2. [Done?]        If every square is visited, report the tour.
/// W3. [Score.]       For each unvisited neighbor v of the current square,
///                    let d(v) = number of unvisited neighbors of v.
/// W4. [Choose.]      Move to the v minimizing (d(v), v). If none, fail.
/// ```
pub fn warnsdorff_tour(board: usize, start: usize) -> Option<Vec<usize>> {
    let n2 = board * board;
    if n2 == 0 || start >= n2 {
        return None;
    }
    let mut visited = vec![false; n2];
    let mut tour = Vec::with_capacity(n2);
    // W1.
    let mut cur = start;
    visited[cur] = true;
    tour.push(cur);
    // W2.
    while tour.len() < n2 {
        // W3 / W4. Pick the unvisited neighbor with fewest onward moves,
        // breaking ties toward the smaller square index.
        let mut best: Option<(usize, usize)> = None; // (onward_degree, square)
        for nb in knight_moves(board, cur) {
            if visited[nb] {
                continue;
            }
            let deg = knight_moves(board, nb)
                .into_iter()
                .filter(|&s| !visited[s])
                .count();
            best = match best {
                None => Some((deg, nb)),
                Some((bd, bs)) if deg < bd || (deg == bd && nb < bs) => Some((deg, nb)),
                keep => keep,
            };
        }
        match best {
            Some((_, nb)) => {
                visited[nb] = true;
                tour.push(nb);
                cur = nb;
            }
            None => return None, // stuck: the heuristic failed
        }
    }
    Some(tour)
}

/// Is `tour` a valid **open** knight's tour of a `board × board` board?
///
/// It must list all `board²` squares exactly once (a permutation of
/// `0..board²`) with every consecutive pair a legal knight move. It does
/// **not** require the tour to close back to the start.
pub fn is_valid_tour(board: usize, tour: &[usize]) -> bool {
    let n2 = board * board;
    if tour.len() != n2 {
        return false;
    }
    let mut seen = vec![false; n2];
    for &s in tour {
        if s >= n2 || seen[s] {
            return false; // out of range or repeated square
        }
        seen[s] = true;
    }
    for w in tour.windows(2) {
        if !knight_moves(board, w[0]).contains(&w[1]) {
            return false; // consecutive squares not a knight move apart
        }
    }
    true
}

// ===========================================================================
// Stage 3 — Hamiltonian cycles on the hypercube ARE Gray codes (§7.2.1.1).
// ===========================================================================

/// The neighbors of vertex `v` in the `d`-dimensional hypercube `Q_d`.
///
/// Vertices of `Q_d` are the `2^d` bit strings of length `d` (packed into a
/// `u32`); two are adjacent iff they differ in exactly one bit. So `v` has
/// exactly `d` neighbors: flip each of bits `0..d`.
pub fn hypercube_neighbors(d: usize, v: u32) -> Vec<u32> {
    (0..d).map(|i| v ^ (1u32 << i)).collect()
}

/// A Hamiltonian cycle on `Q_d`, produced as the **reflected binary Gray
/// code**: the `k`-th vertex is `g(k) = k XOR (k >> 1)`, for `k = 0 ..< 2^d`.
///
/// Successive codewords `g(k)` and `g(k+1)` differ in exactly one bit, and so
/// do the last and first (the code is *cyclic*) — that is precisely the
/// statement that this ordering is a Hamiltonian cycle on the hypercube.
/// See the module README for the induction that proves it.
pub fn gray_code_cycle(d: usize) -> Vec<u32> {
    (0..(1u32 << d)).map(|k| k ^ (k >> 1)).collect()
}

/// Is `cycle` a Hamiltonian cycle on `Q_d`? It must list all `2^d` vertices
/// exactly once, and every consecutive pair — *including the wrap-around from
/// last to first* — must differ in exactly one bit.
pub fn is_hamiltonian_cycle_on_hypercube(d: usize, cycle: &[u32]) -> bool {
    let n = 1usize << d;
    if cycle.len() != n {
        return false;
    }
    let mut seen = vec![false; n];
    for &v in cycle {
        if (v as usize) >= n || seen[v as usize] {
            return false; // out of range or a repeated vertex
        }
        seen[v as usize] = true;
    }
    for i in 0..n {
        let a = cycle[i];
        let b = cycle[(i + 1) % n];
        if (a ^ b).count_ones() != 1 {
            return false; // consecutive vertices not adjacent in Q_d
        }
    }
    true
}

// ===========================================================================
// Stage 4 — Held–Karp: shortest Hamiltonian path/cycle by bitmask DP.
// ===========================================================================
//
// The Held–Karp dynamic program (1962). State C(S, j) is the least weight of
// a path that starts somewhere, visits exactly the vertex set S, and ends at
// j in S. The recurrence
//
//     C(S, j) = min over i in S\{j} of  C(S\{j}, i) + d(i, j)
//
// fills a table of 2^n * n entries in O(2^n * n^2) time — still, sixty years
// on, the best known worst-case bound for the traveling salesman problem.

/// A sentinel "infinity" that leaves headroom so `INF + weight` never wraps.
const INF: u64 = u64::MAX / 4;

/// The minimum total edge weight of a Hamiltonian **path** in a complete
/// weighted graph, over all start and end vertices. `dist[i][j]` is the
/// weight of edge `i—j`; weights need not satisfy the triangle inequality.
///
/// Held–Karp, "push" form: subsets are processed in increasing numeric order,
/// which is a valid topological order because adding a vertex `k` to `S`
/// yields `S | (1<<k) > S`. A single vertex has path weight 0.
pub fn shortest_hamiltonian_path(dist: &[Vec<u64>]) -> u64 {
    let n = dist.len();
    if n <= 1 {
        return 0;
    }
    let full = 1usize << n;
    // dp[S][j] = C(S, j).
    let mut dp = vec![vec![INF; n]; full];
    // Base: the one-vertex path {j} ending at j has weight 0.
    for j in 0..n {
        dp[1 << j][j] = 0;
    }
    for s in 1..full {
        for j in 0..n {
            if s & (1 << j) == 0 {
                continue;
            }
            let cur = dp[s][j];
            if cur >= INF {
                continue;
            }
            // Extend the path ending at j by an unvisited vertex k.
            for k in 0..n {
                if s & (1 << k) != 0 {
                    continue;
                }
                let ns = s | (1 << k);
                let cand = cur + dist[j][k];
                if cand < dp[ns][k] {
                    dp[ns][k] = cand;
                }
            }
        }
    }
    (0..n).map(|j| dp[full - 1][j]).min().unwrap()
}

/// The minimum total edge weight of a Hamiltonian **cycle** — the traveling
/// salesman tour. `dist[i][j]` is the weight of edge `i—j`.
///
/// Fix the start at vertex 0 (every cycle contains it); build up paths from 0
/// that visit `S`, then close the best one back to 0:
///
/// ```text
/// answer = min over j != 0 of  C(full, j) + d(j, 0).
/// ```
///
/// Trivial sizes: `n <= 1` gives 0; `n == 2` gives `d(0,1) + d(1,0)`.
pub fn shortest_hamiltonian_cycle(dist: &[Vec<u64>]) -> u64 {
    let n = dist.len();
    if n <= 1 {
        return 0;
    }
    let full = 1usize << n;
    let mut dp = vec![vec![INF; n]; full];
    // Base: the trivial path {0} ending at 0.
    dp[1][0] = 0;
    for s in 1..full {
        if s & 1 == 0 {
            continue; // every partial tour includes the start vertex 0
        }
        for j in 0..n {
            if s & (1 << j) == 0 {
                continue;
            }
            let cur = dp[s][j];
            if cur >= INF {
                continue;
            }
            for k in 1..n {
                // never step back onto 0 mid-tour
                if s & (1 << k) != 0 {
                    continue;
                }
                let ns = s | (1 << k);
                let cand = cur + dist[j][k];
                if cand < dp[ns][k] {
                    dp[ns][k] = cand;
                }
            }
        }
    }
    let mut ans = INF;
    for j in 1..n {
        let c = dp[full - 1][j];
        if c < INF {
            ans = ans.min(c + dist[j][0]);
        }
    }
    ans
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- graph builders reused across the reference tests ----------------

    fn complete_graph(n: usize) -> Vec<Vec<usize>> {
        (0..n)
            .map(|u| (0..n).filter(|&v| v != u).collect())
            .collect()
    }

    fn cycle_graph(n: usize) -> Vec<Vec<usize>> {
        (0..n)
            .map(|u| vec![(u + n - 1) % n, (u + 1) % n])
            .collect()
    }

    fn path_graph(n: usize) -> Vec<Vec<usize>> {
        (0..n)
            .map(|u| {
                let mut v = Vec::new();
                if u > 0 {
                    v.push(u - 1);
                }
                if u + 1 < n {
                    v.push(u + 1);
                }
                v
            })
            .collect()
    }

    // The Petersen graph: outer 5-cycle, inner pentagram, five spokes.
    fn petersen() -> Vec<Vec<usize>> {
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 4), (4, 0), // outer
            (0, 5), (1, 6), (2, 7), (3, 8), (4, 9), // spokes
            (5, 7), (7, 9), (9, 6), (6, 8), (8, 5), // inner pentagram
        ];
        let mut adj = vec![Vec::new(); 10];
        for (u, v) in edges {
            adj[u].push(v);
            adj[v].push(u);
        }
        adj
    }

    fn factorial(n: u64) -> u64 {
        (1..=n).product()
    }

    fn is_ham_path(adj: &[Vec<usize>], path: &[usize]) -> bool {
        let n = adj.len();
        if path.len() != n {
            return false;
        }
        let mut seen = vec![false; n];
        for &v in path {
            if v >= n || seen[v] {
                return false;
            }
            seen[v] = true;
        }
        path.windows(2).all(|w| adj[w[0]].contains(&w[1]))
    }

    #[test]
    fn complete_graph_cycle_counts() {
        // K_n has (n-1)!/2 distinct undirected Hamiltonian cycles.
        for n in 3..=7 {
            let expected = factorial(n as u64 - 1) / 2;
            assert_eq!(
                count_hamiltonian_cycles(&complete_graph(n)),
                expected,
                "K_{n}"
            );
        }
    }

    #[test]
    fn cycle_graph_has_one_cycle() {
        for n in 3..=8 {
            assert_eq!(count_hamiltonian_cycles(&cycle_graph(n)), 1, "C_{n}");
        }
    }

    #[test]
    fn path_graph_has_path_but_no_cycle() {
        for n in 3..=8 {
            let g = path_graph(n);
            assert!(hamiltonian_path(&g).is_some(), "P_{n} path");
            assert!(hamiltonian_cycle(&g).is_none(), "P_{n} no cycle");
            assert_eq!(count_hamiltonian_cycles(&g), 0);
        }
    }

    #[test]
    fn petersen_is_non_hamiltonian_but_traceable() {
        let g = petersen();
        // Famous: the Petersen graph has no Hamiltonian cycle...
        assert!(hamiltonian_cycle(&g).is_none());
        assert_eq!(count_hamiltonian_cycles(&g), 0);
        // ...but it does have a Hamiltonian path.
        let p = hamiltonian_path(&g).expect("Petersen has a Hamiltonian path");
        assert!(is_ham_path(&g, &p));
    }

    #[test]
    fn returned_solutions_are_validated() {
        for n in 3..=7 {
            let g = complete_graph(n);
            let c = hamiltonian_cycle(&g).unwrap();
            // Valid cycle: permutation + consecutive adjacency + closes up.
            assert!(is_ham_path(&g, &c));
            assert!(g[*c.last().unwrap()].contains(&c[0]));
        }
    }

    #[test]
    fn disconnected_graph_has_no_path() {
        // Two disjoint edges: 0-1 and 2-3.
        let g = vec![vec![1], vec![0], vec![3], vec![2]];
        assert!(hamiltonian_path(&g).is_none());
        assert!(hamiltonian_cycle(&g).is_none());
    }

    #[test]
    fn knight_moves_from_corner_and_center() {
        // On 8x8, a corner (0,0) has exactly two knight moves.
        assert_eq!(knight_moves(8, 0).len(), 2);
        // A central square has all eight.
        let center = 3 * 8 + 3;
        assert_eq!(knight_moves(8, center).len(), 8);
    }

    #[test]
    fn warnsdorff_finds_tours() {
        // Warnsdorff completes an open tour on these boards from a corner.
        for &board in &[5usize, 6, 7, 8] {
            let tour = warnsdorff_tour(board, 0).expect("corner tour");
            assert!(is_valid_tour(board, &tour), "{board}x{board} from corner");
            assert_eq!(tour.len(), board * board);
        }
    }

    #[test]
    fn warnsdorff_can_fail_where_a_tour_exists() {
        // The honest heuristic story: from square 2 of a 5x5 board the greedy
        // walk gets stuck, even though a full tour from square 2 exists (a
        // backtracking search finds one). The same happens on 8x8 from
        // square 24. Warnsdorff is a heuristic, not a guarantee.
        assert!(warnsdorff_tour(5, 2).is_none());
        assert!(warnsdorff_tour(8, 24).is_none());
        // ...but it does succeed from the 5x5 center with this tie-break.
        assert!(warnsdorff_tour(5, 2 * 5 + 2).is_some());
    }

    #[test]
    fn is_valid_tour_rejects_corruption() {
        let mut tour = warnsdorff_tour(6, 0).unwrap();
        assert!(is_valid_tour(6, &tour));
        // Swap two entries to break the knight-move chain.
        tour.swap(3, 20);
        assert!(!is_valid_tour(6, &tour));
    }

    #[test]
    fn gray_cycle_is_hamiltonian_on_hypercube() {
        for d in 1..=8 {
            let cyc = gray_code_cycle(d);
            assert_eq!(cyc.len(), 1 << d);
            assert!(is_hamiltonian_cycle_on_hypercube(d, &cyc), "Q_{d}");
        }
    }

    #[test]
    fn gray_cycle_matches_reflected_code() {
        // Each successive XOR is a single power of two, and equals g(k)^g(k+1).
        for d in 1..=8 {
            let cyc = gray_code_cycle(d);
            for k in 0..cyc.len() {
                let g = (k as u32) ^ (k as u32 >> 1);
                assert_eq!(cyc[k], g, "g({k})");
            }
        }
    }

    #[test]
    fn held_karp_tiny_instances() {
        // n = 3, symmetric: the only tour is 0-1-2-0 = 1+2+3 = 6.
        let d = vec![
            vec![0, 1, 3],
            vec![1, 0, 2],
            vec![3, 2, 0],
        ];
        assert_eq!(shortest_hamiltonian_cycle(&d), 6);
        // Shortest path can skip the most expensive edge (0-2, weight 3):
        // path 0-1-2 has weight 1+2 = 3.
        assert_eq!(shortest_hamiltonian_path(&d), 3);
    }

    #[test]
    fn held_karp_single_vertex() {
        assert_eq!(shortest_hamiltonian_path(&[vec![0]]), 0);
        assert_eq!(shortest_hamiltonian_cycle(&[vec![0]]), 0);
    }

    #[test]
    fn held_karp_matches_brute_force() {
        // Deterministic LCG for symmetric random distances; cross-check the DP
        // against exhaustive permutation enumeration for small n.
        let mut x: u64 = 0x1234_5678_9abc_def0;
        let mut next = || {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (x >> 33) % 100
        };
        for n in 2..=7 {
            let mut d = vec![vec![0u64; n]; n];
            for i in 0..n {
                for j in (i + 1)..n {
                    let w = next() + 1;
                    d[i][j] = w;
                    d[j][i] = w;
                }
            }
            assert_eq!(
                shortest_hamiltonian_path(&d),
                brute_path(&d),
                "path n={n}"
            );
            assert_eq!(
                shortest_hamiltonian_cycle(&d),
                brute_cycle(&d),
                "cycle n={n}"
            );
        }
    }

    fn brute_path(d: &[Vec<u64>]) -> u64 {
        let n = d.len();
        let mut perm: Vec<usize> = (0..n).collect();
        let mut best = INF;
        permute(&mut perm, 0, &mut |p: &[usize]| {
            let cost: u64 = p.windows(2).map(|w| d[w[0]][w[1]]).sum();
            best = best.min(cost);
        });
        best
    }

    fn brute_cycle(d: &[Vec<u64>]) -> u64 {
        let n = d.len();
        let mut perm: Vec<usize> = (0..n).collect();
        let mut best = INF;
        permute(&mut perm, 0, &mut |p: &[usize]| {
            let mut cost: u64 = p.windows(2).map(|w| d[w[0]][w[1]]).sum();
            cost += d[p[n - 1]][p[0]];
            best = best.min(cost);
        });
        best
    }

    fn permute(a: &mut [usize], k: usize, visit: &mut impl FnMut(&[usize])) {
        if k == a.len() {
            visit(a);
            return;
        }
        for i in k..a.len() {
            a.swap(k, i);
            permute(a, k + 1, visit);
            a.swap(k, i);
        }
    }
}
