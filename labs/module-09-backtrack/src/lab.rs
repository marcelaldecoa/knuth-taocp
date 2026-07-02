//! Module 09 — Backtracking and Dancing Links (TAOCP Vol. 4B, §7.2.2–7.2.2.1).
//!
//! **Scaffolding tier — Module 05 and up:** the stub states the algorithm and
//! the contract and trusts you to translate it to Rust; the guided-tour aids of
//! Modules 01–04 are gone by design. The nets remain for every stage — the
//! lesson, three graduated hints (`--hint`), the reference, and the walkthrough.
//! (The taper is described in docs/for-newcomers.md §5.)
//!
//! YOUR WORKSPACE. Replace each `todo!()`. Run `./grade 9`. The lesson is in
//! course/module-09-backtrack/README.md.

// ---------------------------------------------------------------------------
// Stage 1 — Algorithm 7.2.2B (basic backtrack), applied to n queens.
// ---------------------------------------------------------------------------

/// Count placements of `n` non-attacking queens (one per row) with the generic
/// backtrack skeleton. Represent a partial placement as `a[0..l]` where `a[k]`
/// is the column of the queen in row `k`; extend row by row, pruning any
/// candidate that shares a column or diagonal with an earlier queen.
///
/// ```text
/// B1. [Initialize.]  Set level l <- 0.
/// B2. [Enter level l.] If l = n, visit the solution (count it); else pick
///                      the first candidate value for a[l].
/// B3. [Try a[l].]    If a[l] is compatible with a[0..l], advance (B4);
///                    otherwise try the next value.
/// B4. [Increase l.]  l <- l + 1; go to B2.
/// B5. [Backtrack.]   Decrease l; resume trying values at the old level.
/// ```
/// Convention: `count_queens_solutions(0) == 1` (the empty placement).
pub fn count_queens_solutions(n: usize) -> u64 {
    let _ = n;
    todo!("Algorithm 7.2.2B applied to n queens")
}

/// The first solution found, as columns per row (0-indexed), or None if there
/// is none. `first_queens_solution(0) == Some(vec![])`.
pub fn first_queens_solution(n: usize) -> Option<Vec<usize>> {
    let _ = n;
    todo!("return the first n-queens placement")
}

// ---------------------------------------------------------------------------
// Stage 2 — Walker's method: bitwise backtracking.
// ---------------------------------------------------------------------------

/// Count non-attacking placements of `n` queens with Walker's bitwise domains:
/// carry the still-free columns of the current row as a bitmask, and update the
/// column set and the two diagonal sets with shifts as you descend a row.
/// Requires `n <= 31` (u32 masks). `count_queens_bitwise(0) == 1`.
///
/// Hint: `free = all & !(cols | diag_down | diag_up)`; isolate the lowest set
/// bit with `free & free.wrapping_neg()`; descending a row shifts the diagonal
/// masks left and right by one.
pub fn count_queens_bitwise(n: usize) -> u64 {
    let _ = n;
    todo!("Walker's bitwise n-queens count")
}

// ---------------------------------------------------------------------------
// Stage 3 — Algorithm 7.2.2.1X: exact cover by dancing links.
// ---------------------------------------------------------------------------

/// An exact-cover problem solved by dancing links. Items are `0..num_items`;
/// an *option* is a subset of items; a solution is a set of options covering
/// each item exactly once.
///
/// Implement the four-way-linked (L/R/U/D) sparse-matrix structure with column
/// headers and a root header, plus `cover`/`uncover`:
///
/// ```text
/// cover(c):   unlink header c from the header list; for each option row i in
///             column c, unlink every other cell j of that row from its column
///             (down[up[j]] <- down[j]; up[down[j]] <- up[j]; size[col j] -= 1).
/// uncover(c): the exact reverse, in reverse order — because each removed node
///             still points at its old neighbours, restoring costs 2 writes.
/// ```
///
/// `search`: if the header list is empty, record the solution; else choose the
/// active column of minimum size (MRV), cover it, and for each option in it,
/// cover the option's other columns, recurse, then uncover.
pub struct ExactCover {
    // Design your own fields (parallel `Vec<usize>` link arrays are the classic
    // choice). This placeholder just lets the stub compile.
    _num_items: usize,
}

impl ExactCover {
    /// Create a problem over `num_items` items and no options.
    pub fn new(num_items: usize) -> Self {
        let _ = num_items;
        todo!("build the root + column headers")
    }

    /// Add one option (subset of item indices). Returns its option index.
    pub fn add_option(&mut self, items: &[usize]) -> usize {
        let _ = items;
        todo!("splice a new option row into the structure")
    }

    /// Every exact cover, each a sorted list of option indices.
    pub fn solve_all(&mut self) -> Vec<Vec<usize>> {
        todo!("Algorithm X via dancing links")
    }

    /// The number of exact covers.
    pub fn count_solutions(&mut self) -> u64 {
        todo!("count exact covers")
    }

    /// The first exact cover found (sorted option indices), or None.
    pub fn solve_first(&mut self) -> Option<Vec<usize>> {
        todo!("stop at the first exact cover")
    }
}

// ---------------------------------------------------------------------------
// Stage 4 — Sudoku as exact cover.
// ---------------------------------------------------------------------------

/// Solve a 9x9 Sudoku (0 = empty) by reduction to a 324-item exact cover
/// (81 cell + 81 row-digit + 81 column-digit + 81 box-digit constraints),
/// then decoding the chosen options. Return the completed grid, or None.
pub fn solve_sudoku(grid: &[[u8; 9]; 9]) -> Option<[[u8; 9]; 9]> {
    let _ = grid;
    todo!("encode Sudoku as exact cover and solve with ExactCover")
}
