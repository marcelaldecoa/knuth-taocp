//! Stage 3 — Exact cover via dancing links (Algorithm 7.2.2.1X).

use lab_09_backtrack::ExactCover;

/// Build Knuth's §7.2.2.1 running example. Items a..g = 0..6; options:
///   0:{c,e} 1:{a,d,g} 2:{b,c,f} 3:{a,d,f} 4:{b,g} 5:{d,e,g}
/// The unique exact cover is options {0, 3, 4} = {c,e}+{a,d,f}+{b,g}.
fn knuth_example() -> ExactCover {
    let mut ec = ExactCover::new(7);
    ec.add_option(&[2, 4]); // 0
    ec.add_option(&[0, 3, 6]); // 1
    ec.add_option(&[1, 2, 5]); // 2
    ec.add_option(&[0, 3, 5]); // 3
    ec.add_option(&[1, 6]); // 4
    ec.add_option(&[3, 4, 6]); // 5
    ec
}

#[test]
fn knuth_running_example_has_the_unique_cover() {
    let mut ec = knuth_example();
    let sols = ec.solve_all();
    assert_eq!(sols.len(), 1, "expected exactly one exact cover");
    assert_eq!(sols[0], vec![0, 3, 4]);
}

#[test]
fn cover_uncover_restores_structure() {
    // Solving twice must give identical results — cover/uncover leaves the
    // toroidal structure exactly as it was.
    let mut ec = knuth_example();
    let first = ec.solve_all();
    let second = ec.solve_all();
    assert_eq!(first, second);
    assert_eq!(ec.count_solutions(), 1);
    assert_eq!(ec.solve_first(), Some(vec![0, 3, 4]));
}

#[test]
fn no_items_has_the_empty_cover() {
    let mut ec = ExactCover::new(0);
    assert_eq!(ec.count_solutions(), 1);
    assert_eq!(ec.solve_first(), Some(vec![]));
}

#[test]
fn item_with_no_option_is_unsatisfiable() {
    let mut ec = ExactCover::new(2);
    ec.add_option(&[0]); // item 1 is never covered
    assert_eq!(ec.count_solutions(), 0);
    assert_eq!(ec.solve_first(), None);
}

#[test]
fn multiple_solutions_counted_once_each() {
    // Items {0,1,2,3}. Options {0,1}, {2,3}, {0,1,2,3}. Two covers:
    // {opt0, opt1} and {opt2}.
    let mut ec = ExactCover::new(4);
    ec.add_option(&[0, 1]);
    ec.add_option(&[2, 3]);
    ec.add_option(&[0, 1, 2, 3]);
    let mut sols = ec.solve_all();
    sols.sort();
    assert_eq!(sols, vec![vec![0, 1], vec![2]]);
}

#[test]
fn larger_partition_problem() {
    // Cover {0,1,2,3,4,5} with pairwise blocks {2k,2k+1} — exactly one cover.
    let mut ec = ExactCover::new(6);
    ec.add_option(&[0, 1]);
    ec.add_option(&[2, 3]);
    ec.add_option(&[4, 5]);
    ec.add_option(&[1, 2]); // decoy that can't be completed
    assert_eq!(ec.count_solutions(), 1);
}
