//! Stage 1 — Comparison lower bounds by decision trees (§5.3.1).
//!
//! Implement `min_comparisons_lower_bound`, `sort_and_count`, and `is_sorted`
//! in src/lab.rs. Lesson: course/module-20-networks/README.md.

use lab_20_networks::{is_sorted, min_comparisons_lower_bound, sort_and_count};

/// A hand-rolled LCG so the property tests are deterministic (no rand crate).
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn ceil_lg_factorial_is_pinned_for_small_n() {
    // ceil(lg n!) for n = 1..12 (§5.3.1). Each is the least h with 2^h >= n!.
    let want = [0, 1, 3, 5, 7, 10, 13, 16, 19, 22, 26, 29];
    for n in 1..=12usize {
        assert_eq!(
            min_comparisons_lower_bound(n),
            want[n - 1],
            "ceil(lg {n}!) should be {}",
            want[n - 1]
        );
    }
}

#[test]
fn the_bound_is_a_valid_information_bound() {
    // It must equal the least h with 2^h >= n!, i.e. 2^h >= n! and
    // 2^(h-1) < n! (for n >= 2).
    let mut fact: u128 = 1;
    for n in 1..=15u128 {
        fact *= n;
        let h = min_comparisons_lower_bound(n as usize) as u32;
        assert!((1u128 << h) >= fact, "2^h must reach n! for n={n}");
        if n >= 2 {
            assert!((1u128 << (h - 1)) < fact, "h must be minimal for n={n}");
        }
    }
}

#[test]
fn the_famous_gap_at_twelve() {
    // The information bound gives 29 for n = 12, but the true minimum number of
    // comparisons is S(12) = 30. The counting argument is not always tight.
    assert_eq!(min_comparisons_lower_bound(12), 29);
    // S(n) = ceil(lg n!) holds for n <= 11; check the bound agrees there.
    let s = [0, 1, 3, 5, 7, 10, 13, 16, 19, 22, 26];
    for n in 1..=11usize {
        assert_eq!(min_comparisons_lower_bound(n), s[n - 1]);
    }
}

#[test]
fn n_zero_and_one_need_no_comparisons() {
    assert_eq!(min_comparisons_lower_bound(0), 0);
    assert_eq!(min_comparisons_lower_bound(1), 0);
}

#[test]
fn sort_and_count_actually_sorts() {
    let mut state = 0xdead_beefu64;
    for n in 0..60usize {
        let mut a: Vec<i64> = (0..n).map(|_| (lcg(&mut state) >> 40) as i64 % 200 - 100).collect();
        let mut expected = a.clone();
        expected.sort();
        let comps = sort_and_count(&mut a);
        assert_eq!(a, expected, "sort_and_count must sort (n={n})");
        assert!(is_sorted(&a));
        // A nonempty, non-singleton input forces at least one comparison.
        if n >= 2 {
            assert!(comps >= 1, "expected a comparison count for n={n}");
        }
    }
}

#[test]
fn is_sorted_detects_order() {
    assert!(is_sorted(&[]));
    assert!(is_sorted(&[42]));
    assert!(is_sorted(&[1, 1, 2, 3, 3, 9]));
    assert!(!is_sorted(&[1, 3, 2]));
    assert!(!is_sorted(&[9, 8, 7]));
}

#[test]
fn a_sort_never_beats_the_worst_case_bound_on_reversed_input() {
    // On the fully reversed input, binary insertion's comparison count is a
    // reasonable proxy that comparison sorting is not free: it should be at
    // least the number of adjacent swaps a naive count would need to notice,
    // and certainly positive for n >= 2. (The information bound is a *worst
    // case over inputs*; this checks the counter is wired up.)
    for n in [2usize, 4, 8, 16] {
        let mut a: Vec<i64> = (0..n as i64).rev().collect();
        let comps = sort_and_count(&mut a);
        assert!(is_sorted(&a));
        assert!(comps >= min_comparisons_lower_bound(n) as u64 / 2);
    }
}
