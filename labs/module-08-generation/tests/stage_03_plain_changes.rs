//! Stage 3 — plain changes / Steinhaus–Johnson–Trotter (Algorithm 7.2.1.2P).
//!
//! Implement `plain_changes` in src/lab.rs.
//! Lesson: course/module-08-generation/README.md.

use lab_08_generation::plain_changes;

fn factorial(n: u32) -> usize {
    (1..=n as usize).product()
}

#[test]
fn n_equals_three_is_exact() {
    // §7.2.1.2: for n = 3 plain changes runs 123, 132, 312, 321, 231, 213.
    assert_eq!(
        plain_changes(3),
        vec![
            vec![1, 2, 3],
            vec![1, 3, 2],
            vec![3, 1, 2],
            vec![3, 2, 1],
            vec![2, 3, 1],
            vec![2, 1, 3],
        ]
    );
}

#[test]
fn base_cases() {
    assert_eq!(plain_changes(0), vec![Vec::<u32>::new()]);
    assert_eq!(plain_changes(1), vec![vec![1]]);
    assert_eq!(plain_changes(2), vec![vec![1, 2], vec![2, 1]]);
}

#[test]
fn count_is_n_factorial() {
    for n in 0..=7u32 {
        assert_eq!(plain_changes(n).len(), factorial(n), "n! for n={n}");
        if n == 4 {
            assert_eq!(plain_changes(4).len(), 24, "explicitly, 4! = 24");
        }
    }
}

#[test]
fn successive_pairs_differ_by_one_adjacent_transposition() {
    // The defining property: neighbours differ by swapping two ADJACENT
    // positions (and nothing else).
    for n in 1..=6u32 {
        let seq = plain_changes(n);
        for w in seq.windows(2) {
            let diffs: Vec<usize> = (0..n as usize).filter(|&i| w[0][i] != w[1][i]).collect();
            assert_eq!(diffs.len(), 2, "n={n}: exactly two positions change");
            assert_eq!(diffs[1], diffs[0] + 1, "n={n}: the two positions are adjacent");
            // ...and the two changed entries are swapped, not arbitrary.
            assert_eq!(w[0][diffs[0]], w[1][diffs[1]], "n={n}: it is a swap");
            assert_eq!(w[0][diffs[1]], w[1][diffs[0]], "n={n}: it is a swap");
        }
    }
}

#[test]
fn all_permutations_distinct_and_complete() {
    // Plain changes visits every permutation of 1..=n exactly once.
    for n in 1..=6u32 {
        let seq = plain_changes(n);
        let sorted_id: Vec<u32> = (1..=n).collect();
        let mut as_sorted: Vec<Vec<u32>> = seq
            .iter()
            .map(|p| {
                let mut q = p.clone();
                q.sort_unstable();
                assert_eq!(q, sorted_id, "n={n}: {p:?} is a rearrangement");
                p.clone()
            })
            .collect();
        as_sorted.sort();
        as_sorted.dedup();
        assert_eq!(as_sorted.len(), factorial(n), "n={n}: all distinct");
    }
}

#[test]
fn ends_at_two_one_three_four() {
    // For n >= 2 plain changes ends at 2 1 3 4 ... n — one adjacent swap from
    // the identity, making the whole listing a Hamiltonian cycle.
    for n in 2..=6u32 {
        let seq = plain_changes(n);
        let mut expect: Vec<u32> = (1..=n).collect();
        expect.swap(0, 1);
        assert_eq!(*seq.last().unwrap(), expect, "n={n}: ends at 2 1 3 ...");
    }
}
