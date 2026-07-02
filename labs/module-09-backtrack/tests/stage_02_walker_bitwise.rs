//! Stage 2 — Backtracking with bitwise state (Walker's method).

use lab_09_backtrack::{count_queens_bitwise, count_queens_solutions};

#[test]
fn agrees_with_basic_backtrack() {
    for n in 0..=11 {
        assert_eq!(
            count_queens_bitwise(n),
            count_queens_solutions(n),
            "bitwise disagrees with Algorithm B at n={n}"
        );
    }
}

#[test]
fn empty_board() {
    assert_eq!(count_queens_bitwise(0), 1);
}

#[test]
fn reaches_further_than_the_naive_version() {
    // The bitwise representation is fast enough to push past where the
    // string-comparison version bogs down. These are the known counts.
    assert_eq!(count_queens_bitwise(12), 14200);
    assert_eq!(count_queens_bitwise(13), 73712);
}
