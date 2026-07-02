//! Stage 2 — lexicographic permutations (Algorithm 7.2.1.2L).
//!
//! Implement `next_permutation` and `all_permutations` in src/lab.rs.
//! Lesson: course/module-08-generation/README.md.

use lab_08_generation::{all_permutations, next_permutation};

fn factorial(n: u32) -> usize {
    (1..=n as usize).product()
}

#[test]
fn all_permutations_of_three_is_exact() {
    // Lexicographic order of {1,2,3}.
    assert_eq!(
        all_permutations(3),
        vec![
            vec![1, 2, 3],
            vec![1, 3, 2],
            vec![2, 1, 3],
            vec![2, 3, 1],
            vec![3, 1, 2],
            vec![3, 2, 1],
        ]
    );
}

#[test]
fn base_cases() {
    // 0! = 1! = 1: one permutation each.
    assert_eq!(all_permutations(0), vec![Vec::<u32>::new()]);
    assert_eq!(all_permutations(1), vec![vec![1]]);
}

#[test]
fn count_is_n_factorial() {
    for n in 0..=7u32 {
        assert_eq!(all_permutations(n).len(), factorial(n), "n! for n={n}");
    }
}

#[test]
fn output_is_strictly_increasing_lexicographically() {
    // Algorithm L visits permutations in increasing lex order, so the list is
    // sorted and duplicate-free.
    for n in 0..=7u32 {
        let perms = all_permutations(n);
        assert!(perms.windows(2).all(|w| w[0] < w[1]), "n={n}: lex increasing");
    }
}

#[test]
fn first_is_sorted_last_is_reversed() {
    for n in 1..=7u32 {
        let perms = all_permutations(n);
        let first: Vec<u32> = (1..=n).collect();
        let last: Vec<u32> = (1..=n).rev().collect();
        assert_eq!(perms[0], first, "n={n}: first is the identity");
        assert_eq!(*perms.last().unwrap(), last, "n={n}: last is reversed");
    }
}

#[test]
fn every_permutation_is_a_rearrangement() {
    // Each visited tuple is a permutation of 1..=n (same multiset).
    for n in 1..=7u32 {
        let sorted: Vec<u32> = (1..=n).collect();
        for p in all_permutations(n) {
            let mut q = p.clone();
            q.sort_unstable();
            assert_eq!(q, sorted, "n={n}: {p:?} is a rearrangement");
        }
    }
}

#[test]
fn last_permutation_returns_false_and_is_unchanged() {
    // Knuth's convention: at the last permutation L2 terminates, leaving the
    // array sorted in non-increasing order.
    let mut a = vec![3, 2, 1];
    assert!(!next_permutation(&mut a), "no successor to the last permutation");
    assert_eq!(a, vec![3, 2, 1], "array left unchanged");

    // Trivial arrays never have a successor.
    let mut empty: Vec<u32> = vec![];
    assert!(!next_permutation(&mut empty));
    let mut single = vec![7];
    assert!(!next_permutation(&mut single));
}

#[test]
fn stepping_walks_the_lex_order() {
    // Repeatedly calling next_permutation reproduces all_permutations exactly.
    for n in 1..=6u32 {
        let expect = all_permutations(n);
        let mut a: Vec<u32> = (1..=n).collect();
        let mut seen = vec![a.clone()];
        while next_permutation(&mut a) {
            seen.push(a.clone());
        }
        assert_eq!(seen, expect, "stepping matches all_permutations({n})");
    }
}

#[test]
fn multiset_support_visits_distinct_arrangements_once() {
    // [1,1,2] has exactly 3 distinct arrangements because L2/L3 use `>=`.
    let mut a = vec![1u32, 1, 2];
    let mut seen = vec![a.clone()];
    while next_permutation(&mut a) {
        seen.push(a.clone());
    }
    assert_eq!(
        seen,
        vec![vec![1, 1, 2], vec![1, 2, 1], vec![2, 1, 1]],
        "3 distinct arrangements of the multiset {{1,1,2}}"
    );

    // §7.2.1.2 multiset {1,2,2,3}: twelve arrangements, lex-increasing.
    let mut b = vec![1u32, 2, 2, 3];
    let mut m = vec![b.clone()];
    while next_permutation(&mut b) {
        m.push(b.clone());
    }
    assert_eq!(m.len(), 12, "12 = 4! / 2! distinct arrangements");
    assert_eq!(m[0], vec![1, 2, 2, 3]);
    assert_eq!(m[1], vec![1, 2, 3, 2]);
    assert_eq!(*m.last().unwrap(), vec![3, 2, 2, 1]);
    assert!(m.windows(2).all(|w| w[0] < w[1]), "lex increasing, no repeats");
}
