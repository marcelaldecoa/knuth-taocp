//! Stage 2 — Merge insertion / Ford-Johnson (Algorithm 5.3.1M).
//!
//! Implement `ford_johnson_sort` and `ford_johnson_comparisons` in src/lab.rs.
//! Lesson: course/module-20-networks/README.md.

use lab_20_networks::{ford_johnson_comparisons, ford_johnson_sort};

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn is_perm_of(a: &[i64], b: &[i64]) -> bool {
    let mut x = a.to_vec();
    let mut y = b.to_vec();
    x.sort();
    y.sort();
    x == y
}

/// F(n), the worst-case comparison count of merge insertion for n = 0..12.
const FJ_WORST: [u64; 13] = [0, 0, 1, 3, 5, 7, 10, 13, 16, 19, 22, 26, 30];

#[test]
fn sorts_every_size_up_to_fifty() {
    let mut state = 0x0123_4567_89ab_cdefu64;
    for n in 0..=50usize {
        let original: Vec<i64> = (0..n).map(|_| (lcg(&mut state) >> 32) as i64 % 500 - 250).collect();
        let mut a = original.clone();
        ford_johnson_sort(&mut a);
        let mut expected = original.clone();
        expected.sort();
        assert_eq!(a, expected, "merge insertion must sort (n={n})");
        assert!(is_perm_of(&a, &original), "must be a permutation (n={n})");
    }
}

#[test]
fn count_variant_also_sorts() {
    let mut state = 0xfeed_face_dead_beefu64;
    for n in 0..=40usize {
        let original: Vec<i64> = (0..n).map(|_| (lcg(&mut state) >> 33) as i64 % 1000).collect();
        let mut a = original.clone();
        let _ = ford_johnson_comparisons(&mut a);
        let mut expected = original.clone();
        expected.sort();
        assert_eq!(a, expected, "count variant must also sort (n={n})");
    }
}

#[test]
fn never_exceeds_the_worst_case_bound() {
    // Over many random inputs, merge insertion never uses more than F(n).
    let mut state = 0xabcd_1234_5678_9999u64;
    for n in 1..=12usize {
        for _ in 0..200 {
            let mut a: Vec<i64> = (0..n).map(|_| (lcg(&mut state) >> 20) as i64 % 10_000).collect();
            let c = ford_johnson_comparisons(&mut a);
            assert!(c <= FJ_WORST[n], "n={n}: used {c} > F(n)={}", FJ_WORST[n]);
        }
    }
}

#[test]
fn meets_the_bound_exactly_on_small_worst_cases() {
    // Exhaustively over all permutations, the maximum equals F(n): merge
    // insertion is comparison-optimal for these small n (S(n) = F(n), n <= 11;
    // and it still achieves S(12) = 30).
    for n in 1..=7usize {
        let mut perm: Vec<i64> = (1..=n as i64).collect();
        let mut worst = 0u64;
        loop {
            let mut a = perm.clone();
            let c = ford_johnson_comparisons(&mut a);
            let mut sorted = perm.clone();
            sorted.sort();
            assert_eq!(a, sorted);
            worst = worst.max(c);
            if !next_permutation(&mut perm) {
                break;
            }
        }
        assert_eq!(worst, FJ_WORST[n], "worst case over all permutations, n={n}");
    }
}

#[test]
fn handles_duplicates() {
    let mut a = vec![3i64, 1, 3, 1, 2, 2, 3, 1];
    ford_johnson_sort(&mut a);
    assert_eq!(a, vec![1, 1, 1, 2, 2, 3, 3, 3]);
}

#[test]
fn beats_the_naive_bound_for_larger_n() {
    // Merge insertion stays strictly below n*(n-1)/2 (the bubble-sort worst
    // case) and close to the information bound. Just check it is far below the
    // quadratic bound for a moderate n.
    let mut a: Vec<i64> = (0..40i64).rev().collect();
    let c = ford_johnson_comparisons(&mut a);
    assert!(c < 40 * 39 / 2, "merge insertion should be sub-quadratic");
}

fn next_permutation(a: &mut [i64]) -> bool {
    if a.len() < 2 {
        return false;
    }
    let mut i = a.len() - 1;
    while i > 0 && a[i - 1] >= a[i] {
        i -= 1;
    }
    if i == 0 {
        return false;
    }
    let mut j = a.len() - 1;
    while a[j] <= a[i - 1] {
        j -= 1;
    }
    a.swap(i - 1, j);
    a[i..].reverse();
    true
}
