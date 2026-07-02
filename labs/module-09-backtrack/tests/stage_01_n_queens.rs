//! Stage 1 — Basic backtrack: n queens (Algorithm 7.2.2B).

use lab_09_backtrack::{count_queens_solutions, first_queens_solution};

#[test]
fn known_counts() {
    // OEIS A000170. Index n gives the number of n-queens solutions.
    let known = [1u64, 1, 0, 0, 2, 10, 4, 40, 92, 352, 724];
    for (n, &want) in known.iter().enumerate() {
        assert_eq!(count_queens_solutions(n), want, "n={n}");
    }
}

#[test]
fn empty_board_convention() {
    assert_eq!(count_queens_solutions(0), 1);
    assert_eq!(first_queens_solution(0), Some(vec![]));
}

#[test]
fn no_solution_for_two_and_three() {
    assert!(first_queens_solution(2).is_none());
    assert!(first_queens_solution(3).is_none());
}

fn is_valid_placement(sol: &[usize]) -> bool {
    let n = sol.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if sol[i] == sol[j] {
                return false;
            }
            let dc = sol[i].max(sol[j]) - sol[i].min(sol[j]);
            if dc == j - i {
                return false;
            }
        }
    }
    true
}

#[test]
fn first_solution_is_valid() {
    for n in [1usize, 4, 5, 6, 7, 8, 9, 10] {
        let sol = first_queens_solution(n).unwrap_or_else(|| panic!("no solution for n={n}"));
        assert_eq!(sol.len(), n);
        assert!(sol.iter().all(|&c| c < n), "column out of range, n={n}");
        assert!(is_valid_placement(&sol), "invalid placement for n={n}: {sol:?}");
    }
}
