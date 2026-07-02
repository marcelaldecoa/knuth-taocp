//! Stage 4 — Exact cover with colors (Algorithm 7.2.2.1C).
//!
//! Implement `Xcc` in src/lab.rs: dancing links with primary items
//! (covered exactly once), secondary items (at most once), and colors —
//! two options sharing a secondary item are compatible iff they assign
//! it the same color. The purify/unpurify choreography must restore the
//! structure perfectly: several tests solve twice and compare.
//! The lesson: course/module-17-zdd-xcc/README.md.

use lab_17_zdd_xcc::Xcc;

#[test]
fn plain_exact_cover_reproduces_module_09() {
    // Knuth's §7.2.2.1 exact-cover example (Module 09, stage 3): items
    // a..g = 0..6, all primary, no colors anywhere. Unique solution:
    // options {0, 3, 4} = {c,e} + {a,d,f} + {b,g}.
    let mut x = Xcc::new(7, 0);
    x.add_option(&[2, 4], &[]); // 0: {c, e}
    x.add_option(&[0, 3, 6], &[]); // 1: {a, d, g}
    x.add_option(&[1, 2, 5], &[]); // 2: {b, c, f}
    x.add_option(&[0, 3, 5], &[]); // 3: {a, d, f}
    x.add_option(&[1, 6], &[]); // 4: {b, g}
    x.add_option(&[3, 4, 6], &[]); // 5: {d, e, g}
    assert_eq!(x.solve_all(), vec![vec![0, 3, 4]]);
    // Dancing links restored: same answer the second time.
    assert_eq!(x.solve_all(), vec![vec![0, 3, 4]]);
    assert_eq!(x.count_solutions(), 1);
}

#[test]
fn plain_exact_cover_edge_cases() {
    // No items: the empty selection is the one cover.
    let mut empty = Xcc::new(0, 0);
    assert_eq!(empty.count_solutions(), 1);
    // A primary item no option covers: unsatisfiable.
    let mut unsat = Xcc::new(2, 0);
    unsat.add_option(&[0], &[]);
    assert_eq!(unsat.count_solutions(), 0);
    // Two ways to partition four items.
    let mut part = Xcc::new(4, 0);
    part.add_option(&[0, 1], &[]);
    part.add_option(&[2, 3], &[]);
    part.add_option(&[0, 1, 2, 3], &[]);
    assert_eq!(part.count_solutions(), 2);
}

#[test]
fn color_semantics_unit_cases() {
    // Two options meet at secondary item 0. Same color: compatible.
    let mut same = Xcc::new(2, 1);
    same.add_option(&[0], &[(0, 7)]);
    same.add_option(&[1], &[(0, 7)]);
    assert_eq!(same.solve_all(), vec![vec![0, 1]]);
    assert_eq!(same.count_solutions(), 1, "structure restored after solving");

    // Different colors: incompatible — no solution exists.
    let mut diff = Xcc::new(2, 1);
    diff.add_option(&[0], &[(0, 7)]);
    diff.add_option(&[1], &[(0, 8)]);
    assert_eq!(diff.count_solutions(), 0);
    assert_eq!(diff.count_solutions(), 0);

    // Secondary means AT MOST once: leaving it untouched is fine.
    let mut loose = Xcc::new(2, 1);
    loose.add_option(&[0], &[(0, 3)]);
    loose.add_option(&[1], &[]);
    assert_eq!(loose.solve_all(), vec![vec![0, 1]]);

    // Color choice branches: p1 must pick a color; only one agrees
    // with each choice for p0. Two solutions, one per color.
    let mut branch = Xcc::new(2, 1);
    branch.add_option(&[0], &[(0, 1)]); // 0
    branch.add_option(&[0], &[(0, 2)]); // 1
    branch.add_option(&[1], &[(0, 1)]); // 2
    branch.add_option(&[1], &[(0, 2)]); // 3
    let mut sols = branch.solve_all();
    sols.sort();
    assert_eq!(sols, vec![vec![0, 2], vec![1, 3]]);
}

/// Latin-square completion. Items (all primary): cell(r,c) = 3r + c,
/// row-symbol(r,s) = 9 + 3r + s, col-symbol(c,s) = 18 + 3c + s. The
/// option "put symbol s in cell (r,c)" covers one item of each kind —
/// a Latin square is exactly an exact cover of these 27 items.
fn latin3(givens: &[(usize, usize, usize)]) -> (Xcc, Vec<(usize, usize, usize)>) {
    let mut x = Xcc::new(27, 0);
    let mut decode = Vec::new();
    for r in 0..3 {
        for c in 0..3 {
            for s in 0..3 {
                if let Some(&(_, _, gs)) = givens.iter().find(|&&(gr, gc, _)| gr == r && gc == c) {
                    if gs != s {
                        continue; // a filled cell admits only its symbol
                    }
                }
                x.add_option(&[3 * r + c, 9 + 3 * r + s, 18 + 3 * c + s], &[]);
                decode.push((r, c, s));
            }
        }
    }
    (x, decode)
}

#[test]
fn latin_squares_of_order_3_number_12() {
    // 3 × 3 Latin squares: 12 in all (= 3! · 2! · 1 reduced count 1).
    let (mut x, _) = latin3(&[]);
    assert_eq!(x.count_solutions(), 12);
    assert_eq!(x.count_solutions(), 12, "structure restored");
}

#[test]
fn latin_square_unique_completion() {
    // Givens: first row 0 1 2 and cell (1,0) = 1. Forced all the way:
    // (1,1) can be neither 1 (row) nor 1 (col) and if it were 0 then
    // (1,2) = 2 would clash with (0,2); so row 1 = 1 2 0, and row 2
    // fills in as 2 0 1. Unique completion, pinned.
    let givens = [(0, 0, 0), (0, 1, 1), (0, 2, 2), (1, 0, 1)];
    let (mut x, decode) = latin3(&givens);
    let sols = x.solve_all();
    assert_eq!(sols.len(), 1, "completion must be unique");
    let mut grid = [[9usize; 3]; 3];
    for &o in &sols[0] {
        let (r, c, s) = decode[o];
        grid[r][c] = s;
    }
    assert_eq!(grid, [[0, 1, 2], [1, 2, 0], [2, 0, 1]]);
}

#[test]
fn word_pair_grid_with_color_matched_crossings() {
    // A miniature of §7.2.2.1's word-packing spirit: fill a 2×2 grid so
    // that both rows and both columns spell words from the dictionary
    // D = {AA, AB, BA} (letters A = color 0, B = color 1).
    //
    // Primary items: the four slots across0, across1, down0, down1 —
    // each must receive exactly one word. Secondary items: the four
    // cells, cell(r,c) = 2r + c — where an across word and a down word
    // CROSS, and the colors force them to agree on the letter.
    //
    // Hand derivation of the answer: rows (r0, r1) ∈ D² gives 9 grids;
    // the columns must also lie in D. Checking all nine: AA/AA ✓,
    // AA/AB ✓, AA/BA ✓, AB/AA ✓, AB/AB ✗ (col1 = BB), AB/BA ✓,
    // BA/AA ✓, BA/AB ✓, BA/BA ✗ (col0 = BB). So exactly 7 solutions.
    let dict: [(u32, u32); 3] = [(0, 0), (0, 1), (1, 0)];
    let mut x = Xcc::new(4, 4);
    let mut decode: Vec<(&str, usize, (u32, u32))> = Vec::new();
    for &(l0, l1) in &dict {
        for r in 0..2usize {
            // across word in row r: cells (r,0) and (r,1)
            x.add_option(&[r], &[(2 * r, l0), (2 * r + 1, l1)]);
            decode.push(("across", r, (l0, l1)));
        }
    }
    for &(l0, l1) in &dict {
        for c in 0..2usize {
            // down word in column c: cells (0,c) and (1,c)
            x.add_option(&[2 + c], &[(c, l0), (2 + c, l1)]);
            decode.push(("down", c, (l0, l1)));
        }
    }
    let sols = x.solve_all();
    assert_eq!(sols.len(), 7);

    // Every solution really is a consistent grid: one word per slot,
    // and the letters implied by across and down words agree cell-wise.
    for sol in &sols {
        assert_eq!(sol.len(), 4, "one option per slot");
        let mut cell = [[None::<u32>; 2]; 2];
        for &o in sol {
            let (dir, idx, (l0, l1)) = decode[o];
            let coords = if dir == "across" {
                [(idx, 0), (idx, 1)]
            } else {
                [(0, idx), (1, idx)]
            };
            for ((r, c), l) in coords.into_iter().zip([l0, l1]) {
                match cell[r][c] {
                    None => cell[r][c] = Some(l),
                    Some(prev) => assert_eq!(prev, l, "crossing letters must agree"),
                }
            }
        }
        // All four cells written, and rows/columns are dictionary words.
        let g = |r: usize, c: usize| cell[r][c].expect("cell filled");
        for r in 0..2 {
            assert!(dict.contains(&(g(r, 0), g(r, 1))), "row {r} spells a word");
        }
        for c in 0..2 {
            assert!(dict.contains(&(g(0, c), g(1, c))), "column {c} spells a word");
        }
    }

    // And the whole dance is reversible.
    assert_eq!(x.count_solutions(), 7);
}
