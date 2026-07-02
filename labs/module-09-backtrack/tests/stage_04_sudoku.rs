//! Stage 4 — Sudoku as an exact-cover problem (§7.2.2.1 application).

use lab_09_backtrack::solve_sudoku;

fn is_valid_solution(g: &[[u8; 9]; 9]) -> bool {
    // Every cell 1..=9; every row, column, and box a permutation of 1..=9.
    let full = |vals: &[u8]| {
        let mut seen = [false; 10];
        for &v in vals {
            if !(1..=9).contains(&v) || seen[v as usize] {
                return false;
            }
            seen[v as usize] = true;
        }
        true
    };
    for r in 0..9 {
        if !full(&g[r]) {
            return false;
        }
    }
    for c in 0..9 {
        let col: Vec<u8> = (0..9).map(|r| g[r][c]).collect();
        if !full(&col) {
            return false;
        }
    }
    for br in 0..3 {
        for bc in 0..3 {
            let mut box_vals = Vec::new();
            for r in 0..3 {
                for c in 0..3 {
                    box_vals.push(g[br * 3 + r][bc * 3 + c]);
                }
            }
            if !full(&box_vals) {
                return false;
            }
        }
    }
    true
}

/// Does `sol` agree with every given (non-zero) clue of `puzzle`?
fn respects_givens(puzzle: &[[u8; 9]; 9], sol: &[[u8; 9]; 9]) -> bool {
    for r in 0..9 {
        for c in 0..9 {
            if puzzle[r][c] != 0 && puzzle[r][c] != sol[r][c] {
                return false;
            }
        }
    }
    true
}

const PUZZLE: [[u8; 9]; 9] = [
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

const SOLUTION: [[u8; 9]; 9] = [
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

#[test]
fn solves_a_known_puzzle_exactly() {
    // This puzzle has a unique solution, so we can compare grids directly.
    assert_eq!(solve_sudoku(&PUZZLE), Some(SOLUTION));
}

#[test]
fn returns_a_valid_grid_respecting_the_clues() {
    let sol = solve_sudoku(&PUZZLE).expect("puzzle is solvable");
    assert!(is_valid_solution(&sol));
    assert!(respects_givens(&PUZZLE, &sol));
}

#[test]
fn complete_grid_returns_itself() {
    assert_eq!(solve_sudoku(&SOLUTION), Some(SOLUTION));
}

#[test]
fn unsolvable_puzzle_returns_none() {
    // Two 5s in the top row.
    let mut bad = PUZZLE;
    bad[0][2] = 5;
    assert_eq!(solve_sudoku(&bad), None);
}

#[test]
fn empty_grid_yields_some_valid_solution() {
    let empty = [[0u8; 9]; 9];
    let sol = solve_sudoku(&empty).expect("empty grid is solvable");
    assert!(is_valid_solution(&sol));
}
