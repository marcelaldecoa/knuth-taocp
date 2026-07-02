//! Stage 1 — Straight insertion (Algorithm 5.2.1S) and inversions (§5.1.1).
//!
//! Implement `insertion_sort`, `insertion_sort_counting`, and
//! `count_inversions` in src/lab.rs. Lesson: course/module-06-sorting/README.md.
//!
//! The headline theorem (§5.1.1): straight insertion moves a record in step S4
//! exactly once per inversion it must cross, so the S4 move count equals the
//! number of inversions of the input permutation.

use lab_06_sorting::{count_inversions, insertion_sort, insertion_sort_counting};

/// Knuth's standard 16-key example, used throughout Ch. 5 (§5.2).
const KNUTH16: [i64; 16] = [
    503, 087, 512, 061, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703,
];
const KNUTH16_SORTED: [i64; 16] = [
    061, 087, 154, 170, 275, 426, 503, 509, 512, 612, 653, 677, 703, 765, 897, 908,
];

/// x_{n+1} = x_n * 6364136223846793005 + 1442695040888963407  (Knuth's MMIX LCG).
fn lcg(seed: u64) -> impl FnMut() -> u64 {
    let mut x = seed;
    move || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        x
    }
}

fn is_sorted(v: &[i64]) -> bool {
    v.windows(2).all(|w| w[0] <= w[1])
}

/// A sort must be a *permutation* of its input: same multiset of keys.
fn same_multiset(a: &[i64], b: &[i64]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

#[test]
fn sorts_knuths_sixteen_numbers() {
    let mut v = KNUTH16;
    insertion_sort(&mut v);
    assert_eq!(v, KNUTH16_SORTED);
}

#[test]
fn tiny_and_degenerate_inputs() {
    let mut e: [i64; 0] = [];
    insertion_sort(&mut e);
    assert_eq!(e, []);
    let mut one = [42];
    insertion_sort(&mut one);
    assert_eq!(one, [42]);
    let mut two = [2, 1];
    insertion_sort(&mut two);
    assert_eq!(two, [1, 2]);
    let mut dups = [3, 1, 3, 1, 2, 2];
    insertion_sort(&mut dups);
    assert_eq!(dups, [1, 1, 2, 2, 3, 3]);
}

#[test]
fn sortedness_and_permutation_on_lcg_data() {
    let mut rng = lcg(0x51ED_C0DE);
    for &n in &[0usize, 1, 2, 3, 7, 50, 200, 999] {
        let original: Vec<i64> = (0..n).map(|_| (rng() >> 40) as i64).collect();
        let mut v = original.clone();
        insertion_sort(&mut v);
        assert!(is_sorted(&v), "not sorted at n={n}");
        assert!(same_multiset(&v, &original), "not a permutation at n={n}");
    }
}

#[test]
fn inversions_of_knuths_example() {
    // Verified by brute force over all 120 pairs: KNUTH16 has 41 inversions.
    assert_eq!(count_inversions(&KNUTH16), 41);
}

#[test]
fn inversion_extremes() {
    assert_eq!(count_inversions(&[]), 0);
    assert_eq!(count_inversions(&[7]), 0);
    assert_eq!(count_inversions(&[1, 2, 3, 4, 5]), 0, "sorted has no inversions");
    // A strictly decreasing file has the maximum, every pair: n(n-1)/2.
    for n in [2i64, 5, 10, 100] {
        let rev: Vec<i64> = (0..n).rev().collect();
        assert_eq!(
            count_inversions(&rev),
            (n * (n - 1) / 2) as u64,
            "reverse of length {n}"
        );
    }
}

#[test]
fn inversions_match_a_brute_force_double_loop() {
    let mut rng = lcg(0xB0A7);
    for _ in 0..40 {
        let n = (rng() % 60) as usize;
        let v: Vec<i64> = (0..n).map(|_| (rng() % 20) as i64).collect();
        let mut brute = 0u64;
        for i in 0..n {
            for j in (i + 1)..n {
                if v[i] > v[j] {
                    brute += 1;
                }
            }
        }
        assert_eq!(count_inversions(&v), brute, "count mismatch for {v:?}");
    }
}

#[test]
fn move_count_equals_inversions() {
    // Theorem §5.1.1: S4 moves == inversions. Check on the worked example
    // and on a battery of random files.
    let mut v = KNUTH16;
    assert_eq!(insertion_sort_counting(&mut v), 41);
    assert_eq!(v, KNUTH16_SORTED, "counting variant must still sort");

    let mut rng = lcg(0xC0FFEE);
    for _ in 0..40 {
        let n = (rng() % 120) as usize;
        let original: Vec<i64> = (0..n).map(|_| (rng() % 50) as i64).collect();
        let inv = count_inversions(&original);
        let mut v = original.clone();
        let moves = insertion_sort_counting(&mut v);
        assert_eq!(moves, inv, "moves != inversions for {original:?}");
        assert!(is_sorted(&v));
    }
}
