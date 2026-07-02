//! Module 09 — Backtracking and Dancing Links.
//! Source: TAOCP Vol. 4B, §7.2.2 (backtracking) and §7.2.2.1 (dancing links).

// ---------------------------------------------------------------------------
// Stage 1 — Algorithm 7.2.2B (basic backtrack), applied to the n queens problem.
// ---------------------------------------------------------------------------

/// Count the ways to place `n` non-attacking queens, one per row, using the
/// generic backtrack skeleton of Algorithm 7.2.2B.
///
/// State: a partial placement `a[0..l]`, where `a[k]` is the column of the
/// queen in row `k`. We extend row by row (the "level" l), and prune with the
/// property test: no two queens share a column or a diagonal.
pub fn count_queens_solutions(n: usize) -> u64 {
    if n == 0 {
        return 1; // the empty placement is a (degenerate) solution
    }
    let mut a = vec![0usize; n];
    let mut count = 0u64;
    backtrack_queens(0, n, &mut a, &mut count, &mut |_| {});
    count
}

/// Return the first solution found (columns per row, 0-indexed), or None.
pub fn first_queens_solution(n: usize) -> Option<Vec<usize>> {
    if n == 0 {
        return Some(vec![]);
    }
    let mut a = vec![0usize; n];
    let mut found: Option<Vec<usize>> = None;
    let mut count = 0u64;
    backtrack_queens(0, n, &mut a, &mut count, &mut |sol| {
        if found.is_none() {
            found = Some(sol.to_vec());
        }
    });
    found
}

/// Can a queen at (row, col) coexist with the queens already placed in
/// `a[0..row]`?  (The property that Algorithm B's step B3 tests.)
fn queens_ok(a: &[usize], row: usize, col: usize) -> bool {
    for r in 0..row {
        let c = a[r];
        if c == col {
            return false; // same column
        }
        // same diagonal iff row-distance equals column-distance
        if (row - r) == (col.max(c) - col.min(c)) {
            return false;
        }
    }
    true
}

fn backtrack_queens(
    row: usize,
    n: usize,
    a: &mut [usize],
    count: &mut u64,
    visit: &mut impl FnMut(&[usize]),
) {
    // B1/B2. [Enter level.] If all rows are placed, we have a solution.
    if row == n {
        *count += 1;
        visit(a);
        return;
    }
    // B3. [Try a value.] For each candidate column in row `row`...
    for col in 0..n {
        // ...that passes the property test (no attack on earlier queens):
        if queens_ok(a, row, col) {
            a[row] = col;
            // B4. [Advance.] Descend to the next level.
            backtrack_queens(row + 1, n, a, count, visit);
            // B5. [Backtrack.] Implicit: the loop restores by overwriting a[row].
        }
    }
}

// ---------------------------------------------------------------------------
// Stage 2 — Walker's method: backtracking with bitwise domain representation.
// ---------------------------------------------------------------------------

/// Count non-attacking placements of `n` queens using Walker's technique
/// (§7.2.2): the set of columns still available in the current row is carried
/// as a bitmask, so choosing a queen and updating the three attack sets are
/// single machine words. `n` must be <= 31 (we use u32 masks).
pub fn count_queens_bitwise(n: usize) -> u64 {
    assert!(n <= 31, "count_queens_bitwise supports n <= 31");
    if n == 0 {
        return 1;
    }
    let all = (1u32 << n) - 1;
    fn go(all: u32, cols: u32, diag_down: u32, diag_up: u32) -> u64 {
        // `free` = columns attacked by no earlier queen (column or diagonal).
        let mut free = all & !(cols | diag_down | diag_up);
        if cols == all {
            return 1; // every column filled: a full placement
        }
        let mut total = 0;
        while free != 0 {
            let bit = free & free.wrapping_neg(); // lowest set bit (a "ruler" trick)
            free ^= bit;
            // Placing a queen on `bit`: diagonals shift by one as we move to the
            // next row, and the new column set gains `bit`.
            total += go(
                all,
                cols | bit,
                (diag_down | bit) << 1,
                (diag_up | bit) >> 1,
            );
        }
        total
    }
    go(all, 0, 0, 0)
}

// ---------------------------------------------------------------------------
// Stage 3 — Algorithm 7.2.2.1X: exact cover by dancing links (DLX).
// ---------------------------------------------------------------------------

/// An exact-cover problem solved by Knuth's dancing links.
///
/// Items are `0..num_items`. An *option* is a subset of items; a solution is a
/// collection of options that covers every item exactly once. Internally the
/// items and option-cells live in a sparse toroidal doubly linked structure
/// (four link arrays L/R/U/D plus a column pointer), so that removing and
/// *restoring* a node each cost exactly two pointer assignments — the trick
/// that gives the method its name.
pub struct ExactCover {
    left: Vec<usize>,
    right: Vec<usize>,
    up: Vec<usize>,
    down: Vec<usize>,
    col: Vec<usize>,       // node -> its column header
    size: Vec<usize>,      // column header -> number of nodes in the column
    node_option: Vec<usize>, // node -> index of the option it belongs to
    num_items: usize,
    num_options: usize,
    root: usize, // the special header h
}

impl ExactCover {
    /// Create a problem over `num_items` items and no options yet.
    pub fn new(num_items: usize) -> Self {
        // Node 0 is the root header h; nodes 1..=num_items are column headers.
        let n = num_items + 1;
        let mut ec = ExactCover {
            left: vec![0; n],
            right: vec![0; n],
            up: vec![0; n],
            down: vec![0; n],
            col: vec![0; n],
            size: vec![0; n],
            node_option: vec![usize::MAX; n],
            num_items,
            num_options: 0,
            root: 0,
        };
        // Circular header list: h <-> col1 <-> col2 <-> ... <-> h.
        for i in 0..n {
            ec.left[i] = if i == 0 { num_items } else { i - 1 };
            ec.right[i] = if i == num_items { 0 } else { i + 1 };
            // Each column starts empty: up/down point back at the header.
            ec.up[i] = i;
            ec.down[i] = i;
            ec.col[i] = i;
        }
        ec
    }

    /// Add one option (a subset of item indices). Returns the option's index.
    pub fn add_option(&mut self, items: &[usize]) -> usize {
        let opt = self.num_options;
        self.num_options += 1;
        let mut first: Option<usize> = None;
        for &item in items {
            assert!(item < self.num_items, "item {item} out of range");
            let c = item + 1; // column header index
            let node = self.left.len();
            // Grow every parallel array by one node.
            self.left.push(node);
            self.right.push(node);
            self.up.push(0);
            self.down.push(0);
            self.col.push(c);
            self.size.push(0); // unused for non-headers
            self.node_option.push(opt);
            // Splice `node` into the bottom of column c (just above header c).
            let last = self.up[c];
            self.down[last] = node;
            self.up[node] = last;
            self.down[node] = c;
            self.up[c] = node;
            self.size[c] += 1;
            // Link horizontally into this option's circular row.
            match first {
                None => first = Some(node),
                Some(f) => {
                    let l = self.left[f];
                    self.right[l] = node;
                    self.left[node] = l;
                    self.right[node] = f;
                    self.left[f] = node;
                }
            }
        }
        opt
    }

    fn cover(&mut self, c: usize) {
        // Unlink header c from the header list.
        self.right[self.left[c]] = self.right[c];
        self.left[self.right[c]] = self.left[c];
        // For every option touching c, unlink its other cells from their columns.
        let mut i = self.down[c];
        while i != c {
            let mut j = self.right[i];
            while j != i {
                self.down[self.up[j]] = self.down[j];
                self.up[self.down[j]] = self.up[j];
                self.size[self.col[j]] -= 1;
                j = self.right[j];
            }
            i = self.down[i];
        }
    }

    fn uncover(&mut self, c: usize) {
        // Exact reverse of cover: relink in the opposite order.
        let mut i = self.up[c];
        while i != c {
            let mut j = self.left[i];
            while j != i {
                self.size[self.col[j]] += 1;
                self.down[self.up[j]] = j;
                self.up[self.down[j]] = j;
                j = self.left[j];
            }
            i = self.up[i];
        }
        self.right[self.left[c]] = c;
        self.left[self.right[c]] = c;
    }

    /// Choose the active column of minimum size (Knuth's MRV heuristic S).
    fn choose_column(&self) -> usize {
        let mut best = self.right[self.root];
        let mut best_size = self.size[best];
        let mut c = self.right[best];
        while c != self.root {
            if self.size[c] < best_size {
                best = c;
                best_size = self.size[c];
            }
            c = self.right[c];
        }
        best
    }

    fn search(&mut self, partial: &mut Vec<usize>, limit: usize, out: &mut Vec<Vec<usize>>) {
        if out.len() >= limit {
            return;
        }
        if self.right[self.root] == self.root {
            // Every item covered: record the options in the current partial.
            let mut sol: Vec<usize> = partial.iter().map(|&r| self.node_option[r]).collect();
            sol.sort_unstable();
            out.push(sol);
            return;
        }
        let c = self.choose_column();
        if self.size[c] == 0 {
            return; // an uncovered item with no options: dead end
        }
        self.cover(c);
        let mut r = self.down[c];
        while r != c {
            partial.push(r);
            let mut j = self.right[r];
            while j != r {
                self.cover(self.col[j]);
                j = self.right[j];
            }
            self.search(partial, limit, out);
            // Uncover in reverse horizontal order.
            let mut j = self.left[r];
            while j != r {
                self.uncover(self.col[j]);
                j = self.left[j];
            }
            partial.pop();
            if out.len() >= limit {
                break;
            }
            r = self.down[r];
        }
        self.uncover(c);
    }

    /// Every exact cover, each as a sorted list of option indices.
    pub fn solve_all(&mut self) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        let mut partial = Vec::new();
        self.search(&mut partial, usize::MAX, &mut out);
        out
    }

    /// The number of distinct exact covers.
    pub fn count_solutions(&mut self) -> u64 {
        self.solve_all().len() as u64
    }

    /// The first exact cover found (sorted option indices), or None.
    pub fn solve_first(&mut self) -> Option<Vec<usize>> {
        let mut out = Vec::new();
        let mut partial = Vec::new();
        self.search(&mut partial, 1, &mut out);
        out.into_iter().next()
    }
}

// ---------------------------------------------------------------------------
// Stage 4 — Sudoku as an exact-cover problem (§7.2.2.1 application).
// ---------------------------------------------------------------------------

/// Solve a 9x9 Sudoku (0 = empty) by reduction to exact cover with 324 items:
/// 81 cell constraints, 81 row-digit, 81 column-digit, 81 box-digit.
/// Returns the completed grid, or None if unsolvable.
pub fn solve_sudoku(grid: &[[u8; 9]; 9]) -> Option<[[u8; 9]; 9]> {
    // Item layout:
    //   cell(r,c)      = 9*r + c                     (0..81)
    //   row(r,d)       = 81  + 9*r + (d-1)           (81..162)
    //   col(c,d)       = 162 + 9*c + (d-1)           (162..243)
    //   box(b,d)       = 243 + 9*b + (d-1)           (243..324)
    let mut ec = ExactCover::new(324);
    let mut option_rcd: Vec<(usize, usize, usize)> = Vec::new();
    for r in 0..9 {
        for c in 0..9 {
            let given = grid[r][c];
            let digits: Vec<usize> = if given == 0 {
                (1..=9).collect()
            } else {
                vec![given as usize]
            };
            let b = (r / 3) * 3 + c / 3;
            for d in digits {
                let items = [
                    9 * r + c,
                    81 + 9 * r + (d - 1),
                    162 + 9 * c + (d - 1),
                    243 + 9 * b + (d - 1),
                ];
                ec.add_option(&items);
                option_rcd.push((r, c, d));
            }
        }
    }
    let sol = ec.solve_first()?;
    let mut out = [[0u8; 9]; 9];
    for opt in sol {
        let (r, c, d) = option_rcd[opt];
        out[r][c] = d as u8;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queens_counts_match_known_sequence() {
        // OEIS A000170: number of solutions to the n queens problem.
        let known = [1u64, 1, 0, 0, 2, 10, 4, 40, 92, 352, 724, 2680];
        for (n, &want) in known.iter().enumerate() {
            assert_eq!(count_queens_solutions(n), want, "queens n={n}");
        }
    }

    #[test]
    fn queens_first_solution_is_valid() {
        for n in [1usize, 4, 5, 6, 8, 10] {
            let sol = first_queens_solution(n).unwrap();
            assert_eq!(sol.len(), n);
            for i in 0..n {
                for j in (i + 1)..n {
                    assert_ne!(sol[i], sol[j], "same column");
                    assert_ne!(
                        j - i,
                        sol[i].max(sol[j]) - sol[i].min(sol[j]),
                        "same diagonal"
                    );
                }
            }
        }
        assert!(first_queens_solution(3).is_none());
    }

    #[test]
    fn bitwise_agrees_and_reaches_further() {
        for n in 0..=12 {
            assert_eq!(count_queens_bitwise(n), count_queens_solutions(n), "n={n}");
        }
        assert_eq!(count_queens_bitwise(13), 73712);
        assert_eq!(count_queens_bitwise(14), 365596);
    }

    #[test]
    fn dlx_knuth_example() {
        // Knuth's §7.2.2.1 example. Items a..g = 0..6, options:
        //   0: {c, e}         2,4
        //   1: {a, d, g}      0,3,6
        //   2: {b, c, f}      1,2,5
        //   3: {a, d, f}      0,3,5
        //   4: {b, g}         1,6
        //   5: {d, e, g}      3,4,6
        // The unique exact cover is options {3, 4, 0} = {a,d,f}+{b,g}+{c,e}.
        let mut ec = ExactCover::new(7);
        ec.add_option(&[2, 4]);
        ec.add_option(&[0, 3, 6]);
        ec.add_option(&[1, 2, 5]);
        ec.add_option(&[0, 3, 5]);
        ec.add_option(&[1, 6]);
        ec.add_option(&[3, 4, 6]);
        let sols = ec.solve_all();
        assert_eq!(sols.len(), 1);
        assert_eq!(sols[0], vec![0, 3, 4]);
        // Structure is fully restored, so solving again gives the same answer.
        assert_eq!(ec.count_solutions(), 1);
    }

    #[test]
    fn dlx_edge_cases() {
        // No items: the empty selection is the unique cover.
        let mut empty = ExactCover::new(0);
        assert_eq!(empty.count_solutions(), 1);
        // An item with no option covering it: unsatisfiable.
        let mut unsat = ExactCover::new(2);
        unsat.add_option(&[0]);
        assert_eq!(unsat.count_solutions(), 0);
        // Two disjoint options that partition the items: exactly one cover.
        let mut part = ExactCover::new(4);
        part.add_option(&[0, 1]);
        part.add_option(&[2, 3]);
        part.add_option(&[0, 1, 2, 3]);
        assert_eq!(part.count_solutions(), 2); // {opt0,opt1} and {opt2}
    }

    #[test]
    fn sudoku_solves_known_puzzle() {
        // A puzzle with a unique solution (from Wikipedia's Sudoku article).
        let puzzle = [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ];
        let solution = [
            [5, 3, 4, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ];
        assert_eq!(solve_sudoku(&puzzle), Some(solution));
        // An already-complete grid returns itself.
        assert_eq!(solve_sudoku(&solution), Some(solution));
        // A row with two 5s is unsolvable.
        let mut bad = puzzle;
        bad[0][2] = 5;
        assert_eq!(solve_sudoku(&bad), None);
    }
}
