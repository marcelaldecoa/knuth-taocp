//! Stage 3 — Shuffling (Algorithm 3.4.2P) and why the naive shuffle fails.
//!
//! Implement `shuffle` and `naive_shuffle` in src/lab.rs.
//! Lesson: course/module-04-random/README.md (§3.4.2, the counting argument).

use lab_04_random::{chi_square_uniform, naive_shuffle, shuffle, Lcg};
use std::collections::{BTreeMap, BTreeSet};

const MMIX_A: u64 = 6364136223846793005;
const MMIX_C: u64 = 1442695040888963407;

fn scaled(x: u64, bound: u64) -> u64 {
    ((x as u128 * bound as u128) >> 64) as u64
}

/// Rank a permutation of {0,..,n-1} in 0..n! via its Lehmer code.
fn perm_rank(p: &[usize]) -> usize {
    let n = p.len();
    let mut rank = 0usize;
    for i in 0..n {
        let smaller = (i + 1..n).filter(|&j| p[j] < p[i]).count();
        rank = rank * (n - i) + smaller;
    }
    rank
}

#[test]
fn shuffle_always_produces_a_permutation() {
    // Whatever the draws, the multiset of items is preserved: a shuffle is a
    // rearrangement, never a loss or duplication.
    let mut g = Lcg::new(1, MMIX_A, MMIX_C, 0);
    let mut rng = |bound: u64| scaled(g.next(), bound);
    for n in 0..12usize {
        for _ in 0..50 {
            let mut v: Vec<usize> = (0..n).collect();
            shuffle(&mut v, &mut rng);
            let mut sorted = v.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..n).collect::<Vec<_>>(), "n={n}: not a permutation");
        }
    }
}

#[test]
fn tiny_slices_are_left_alone() {
    // Length 0 and 1 need no randomness; the rng must not be consulted.
    let mut rng = |_b: u64| panic!("rng must not be consulted for len < 2");
    let mut empty: [u32; 0] = [];
    shuffle(&mut empty, &mut rng);
    let mut one = [42u32];
    shuffle(&mut one, &mut rng);
    assert_eq!(one, [42]);
}

#[test]
fn algorithm_p_is_exactly_uniform_for_n_3() {
    // On 3 items Algorithm P consumes rng(3) then rng(2): exactly 6 tapes,
    // and each yields a DISTINCT permutation — perfect uniformity.
    let mut seen = BTreeSet::new();
    for k2 in 0..3u64 {
        for k1 in 0..2u64 {
            let tape = [k2, k1];
            let mut i = 0;
            let mut rng = |b: u64| {
                let v = tape[i];
                i += 1;
                assert!(v < b);
                v
            };
            let mut arr = [0u8, 1, 2];
            shuffle(&mut arr, &mut rng);
            assert!(seen.insert(arr), "tape {tape:?} collided on {arr:?}");
        }
    }
    assert_eq!(seen.len(), 6, "all 6 permutations of 3 items must appear");
}

#[test]
fn all_720_permutations_of_six_are_uniform() {
    // Empirically confirm uniformity: shuffle [0..6] many times and apply the
    // stage-2 chi-square test over the 720 possible outcomes. With k = 720
    // categories there are 719 degrees of freedom; the 99% point of the
    // chi-square table is ≈ 810, so a fair shuffle stays well below ~900.
    let mut g = Lcg::new(20260702, MMIX_A, MMIX_C, 0);
    let mut rng = |bound: u64| scaled(g.next(), bound);
    let mut counts = vec![0u64; 720];
    let trials = 720 * 60; // ~60 per bucket
    for _ in 0..trials {
        let mut v: [usize; 6] = [0, 1, 2, 3, 4, 5];
        shuffle(&mut v, &mut rng);
        counts[perm_rank(&v)] += 1;
    }
    // Every permutation must actually occur.
    assert!(counts.iter().all(|&c| c > 0), "some permutation never appeared");
    let vstat = chi_square_uniform(&counts);
    assert!(vstat < 900.0, "shuffle looks biased: V = {vstat} over 719 d.o.f.");
}

#[test]
fn naive_shuffle_is_biased_on_27_tapes() {
    // The broken shuffle draws from the FULL range at every step: 3^3 = 27
    // equally likely tapes. But 27 is not divisible by 3! = 6, so the six
    // permutations cannot come out equally often. Enumerate every tape and
    // read off the exact counts — 4 or 5 out of 27, never the fair 4.5.
    let mut counts = BTreeMap::new();
    for t in 0..27u64 {
        let tape = [t / 9, (t / 3) % 3, t % 3];
        let mut i = 0;
        let mut rng = |_b: u64| {
            let v = tape[i];
            i += 1;
            v
        };
        let mut arr = [0u8, 1, 2];
        naive_shuffle(&mut arr, &mut rng);
        *counts.entry(arr).or_insert(0u32) += 1;
    }
    let got: Vec<([u8; 3], u32)> = counts.into_iter().collect();
    let expect: Vec<([u8; 3], u32)> = vec![
        ([0, 1, 2], 4),
        ([0, 2, 1], 5),
        ([1, 0, 2], 5),
        ([1, 2, 0], 5),
        ([2, 0, 1], 4),
        ([2, 1, 0], 4),
    ];
    assert_eq!(got, expect);
    // Sanity: 27 tapes total, and the spread proves non-uniformity.
    let total: u32 = got.iter().map(|&(_, c)| c).sum();
    assert_eq!(total, 27);
    assert!(got.iter().any(|&(_, c)| c == 4) && got.iter().any(|&(_, c)| c == 5));
}
