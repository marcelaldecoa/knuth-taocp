//! Stage 4 — Reservoir sampling (Algorithm 3.4.2R).
//!
//! Implement `reservoir_sample` in src/lab.rs.
//! Lesson: course/module-04-random/README.md (§3.4.2, the k/t induction).

use lab_04_random::{chi_square_uniform, reservoir_sample, Lcg};
use std::collections::BTreeMap;

const MMIX_A: u64 = 6364136223846793005;
const MMIX_C: u64 = 1442695040888963407;

fn scaled(x: u64, bound: u64) -> u64 {
    ((x as u128 * bound as u128) >> 64) as u64
}

#[test]
fn short_stream_returns_everything_in_order() {
    // Fewer than k records: return them all, in stream order, without ever
    // consulting the rng.
    let mut rng = |_b: u64| panic!("rng must not be consulted for a short stream");
    assert_eq!(reservoir_sample(0..3u32, 10, &mut rng), vec![0, 1, 2]);
}

#[test]
fn zero_k_is_empty_and_never_draws() {
    let mut rng = |_b: u64| panic!("rng must not be consulted when k = 0");
    assert_eq!(reservoir_sample(0..5u32, 0, &mut rng), Vec::<u32>::new());
}

#[test]
fn exact_uniformity_n4_k2() {
    // n = 4, k = 2 consumes rng(3) then rng(4): 12 equally likely tapes. Each
    // of the C(4,2) = 6 two-element subsets must arise from exactly 2 tapes,
    // so every subset is sampled with probability 2/12 = 1/6 — uniform.
    let mut counts = BTreeMap::new();
    for m3 in 0..3u64 {
        for m4 in 0..4u64 {
            let tape = [m3, m4];
            let mut i = 0;
            let mut rng = |b: u64| {
                let v = tape[i];
                i += 1;
                assert!(v < b);
                v
            };
            let mut s = reservoir_sample(0..4u32, 2, &mut rng);
            s.sort_unstable();
            *counts.entry(s).or_insert(0u32) += 1;
        }
    }
    assert_eq!(counts.len(), 6, "all six 2-subsets must occur");
    assert!(counts.values().all(|&c| c == 2), "each subset from exactly 2 tapes");
}

#[test]
fn sample_is_a_distinct_k_subset_of_the_stream() {
    // Structural contract on real (LCG-driven) draws: length k, all elements
    // come from the stream, no duplicates.
    let mut g = Lcg::new(99, MMIX_A, MMIX_C, 0);
    let mut rng = |bound: u64| scaled(g.next(), bound);
    let n = 50u32;
    let k = 7usize;
    for _ in 0..200 {
        let s = reservoir_sample(0..n, k, &mut rng);
        assert_eq!(s.len(), k);
        assert!(s.iter().all(|&x| x < n), "element outside the stream");
        let mut sorted = s.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), k, "sample has a duplicate: {s:?}");
    }
}

#[test]
fn every_item_is_chosen_with_frequency_about_k_over_n() {
    // The invariant proved in the lesson: after the whole stream is seen, each
    // of the n items is in the reservoir with probability exactly k/n. Over
    // many trials the appearance counts should look uniform across items — so
    // the stage-2 chi-square test (n - 1 = 9 d.o.f., 99% point ≈ 21.67) passes.
    let n = 10u32;
    let k = 3usize;
    let trials = 30_000u64;
    let mut g = Lcg::new(7, MMIX_A, MMIX_C, 0);
    let mut rng = |bound: u64| scaled(g.next(), bound);
    let mut appear = [0u64; 10];
    for _ in 0..trials {
        for x in reservoir_sample(0..n, k, &mut rng) {
            appear[x as usize] += 1;
        }
    }
    // Total appearances = trials·k; each item expects trials·k/n.
    let total: u64 = appear.iter().sum();
    assert_eq!(total, trials * k as u64);
    let expected = trials as f64 * k as f64 / n as f64;
    for (i, &c) in appear.iter().enumerate() {
        let rel = c as f64 / expected;
        assert!((0.9..1.1).contains(&rel), "item {i}: freq {c} vs expected {expected}");
    }
    let v = chi_square_uniform(&appear);
    assert!(v < 21.67, "appearance counts look non-uniform: V = {v}");
}

#[test]
fn reservoir_equal_to_stream_length_returns_all() {
    // Exactly k records: reservoir fills and the loop body never runs, so the
    // sample is the whole stream unchanged.
    let mut rng = |_b: u64| panic!("no draw needed when stream length == k");
    assert_eq!(reservoir_sample(0..4u32, 4, &mut rng), vec![0, 1, 2, 3]);
}
