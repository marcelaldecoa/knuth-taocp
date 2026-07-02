//! Stage 1 — Replacement selection: runs twice the memory (Algorithm 5.4.1R).
//!
//! Implement `replacement_selection` in src/lab.rs.
//! The lesson: course/module-15-external/README.md.

use lab_15_external::replacement_selection;

/// Knuth's sixteen keys, the running example of Chapter 5.
const KNUTH16: [i64; 16] = [
    503, 87, 512, 61, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703,
];

/// Deterministic pseudo-random data (hand-rolled LCG; no external crates).
fn lcg_vec(n: usize, seed: u64) -> Vec<i64> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (x >> 16) as i64
        })
        .collect()
}

/// Module-06-style run counting: the number of maximal non-decreasing
/// stretches (0 for the empty array).
fn count_ascending_runs(v: &[i64]) -> usize {
    if v.is_empty() {
        return 0;
    }
    1 + v.windows(2).filter(|w| w[0] > w[1]).count()
}

/// Split into the natural ascending runs (what P = 1 must reproduce).
fn ascending_runs(v: &[i64]) -> Vec<Vec<i64>> {
    let mut out: Vec<Vec<i64>> = Vec::new();
    for &x in v {
        match out.last_mut() {
            Some(run) if *run.last().unwrap() <= x => run.push(x),
            _ => out.push(vec![x]),
        }
    }
    out
}

fn assert_valid_runs(input: &[i64], runs: &[Vec<i64>]) {
    for (i, r) in runs.iter().enumerate() {
        assert!(!r.is_empty(), "run {i} is empty");
        assert!(
            r.windows(2).all(|w| w[0] <= w[1]),
            "run {i} is not non-decreasing"
        );
    }
    // Concatenated multiset must equal the input's.
    let mut all: Vec<i64> = runs.iter().flatten().copied().collect();
    let mut expect = input.to_vec();
    all.sort();
    expect.sort();
    assert_eq!(all, expect, "runs are not a permutation of the input");
}

#[test]
fn worked_example_from_the_text() {
    // §5.4.1: P = 3 on the sixteen keys gives THREE runs — of lengths
    // 4, 6, 6, even though memory holds only three records at a time.
    let runs = replacement_selection(&KNUTH16, 3);
    assert_eq!(
        runs,
        vec![
            vec![87, 503, 512, 908],
            vec![61, 170, 275, 426, 653, 897],
            vec![154, 509, 612, 677, 703, 765],
        ]
    );
}

#[test]
fn runs_are_sorted_and_form_a_permutation() {
    for (n, p, seed) in [
        (1usize, 1usize, 1u64),
        (100, 3, 2),
        (1000, 7, 3),
        (5000, 16, 4),
        (5000, 100, 5),
        (777, 5, 6),
    ] {
        let data = lcg_vec(n, seed);
        let runs = replacement_selection(&data, p);
        assert_valid_runs(&data, &runs);
    }
}

#[test]
fn sorted_input_is_one_run_regardless_of_p() {
    // The snow-plow never meets snow falling behind it: nothing ever
    // freezes, so the whole file is a single run — for ANY memory size.
    let sorted: Vec<i64> = (0..5000).collect();
    for p in [1usize, 2, 7, 64, 1000] {
        let runs = replacement_selection(&sorted, p);
        assert_eq!(runs.len(), 1, "sorted input, p = {p}");
        assert_eq!(runs[0], sorted);
    }
    // Non-decreasing (with duplicates) counts as sorted too.
    let dups = vec![1, 1, 2, 2, 2, 3, 5, 5, 9, 9];
    assert_eq!(replacement_selection(&dups, 3).len(), 1);
}

#[test]
fn reverse_sorted_input_gives_runs_of_exactly_p() {
    // Every arriving record is smaller than the last one output, so all P
    // slots freeze: each run is exactly the P records present at its start.
    let rev: Vec<i64> = (0..1000i64).rev().collect();
    let runs = replacement_selection(&rev, 50);
    assert_eq!(runs.len(), 20);
    assert!(runs.iter().all(|r| r.len() == 50));
    assert_valid_runs(&rev, &runs);

    // When p does not divide n, only the final run is short: ceil(n/p) runs.
    let rev: Vec<i64> = (0..1003i64).rev().collect();
    let runs = replacement_selection(&rev, 50);
    assert_eq!(runs.len(), 21); // ceil(1003 / 50)
    assert!(runs[..20].iter().all(|r| r.len() == 50));
    assert_eq!(runs[20].len(), 3);
    assert_valid_runs(&rev, &runs);
}

#[test]
fn two_p_law_on_random_input() {
    // Knuth's snow-plow argument (§5.4.1): in the steady state the plow
    // travels a full lap while the snowfall replaces all P records, so the
    // expected run length tends to 2P on random input.
    let p = 64usize;
    let n = 100_000usize;
    for seed in [20260702u64, 42] {
        let data = lcg_vec(n, seed);
        let runs = replacement_selection(&data, p);
        assert_valid_runs(&data, &runs);
        let avg = n as f64 / runs.len() as f64;
        assert!(
            avg > 1.7 * p as f64 && avg < 2.3 * p as f64,
            "seed {seed}: average run length {avg:.1}, expected near 2P = {}",
            2 * p
        );
    }
}

#[test]
fn p_equal_one_degenerates_to_natural_ascending_runs() {
    // With one slot there is nothing to select among: a record either
    // extends the current run or starts the next — exactly the natural
    // (module-06 `count_runs`) run structure of the input.
    for (data, label) in [
        (KNUTH16.to_vec(), "knuth16"),
        (lcg_vec(2000, 11), "lcg"),
        ((0..50i64).rev().collect::<Vec<_>>(), "reverse"),
    ] {
        let runs = replacement_selection(&data, 1);
        assert_eq!(runs.len(), count_ascending_runs(&data), "{label}: count");
        assert_eq!(runs, ascending_runs(&data), "{label}: exact split");
    }
    // KNUTH16 famously has 8 ascending runs (§5.1.3).
    assert_eq!(replacement_selection(&KNUTH16, 1).len(), 8);
}

#[test]
fn p_at_least_n_gives_one_sorted_run() {
    // If the whole file fits in memory this is just an internal sort.
    let data = lcg_vec(500, 77);
    let mut expect = data.clone();
    expect.sort();
    for p in [500usize, 501, 10_000] {
        let runs = replacement_selection(&data, p);
        assert_eq!(runs.len(), 1, "p = {p}");
        assert_eq!(runs[0], expect);
    }
}

#[test]
fn empty_input_yields_no_runs() {
    assert!(replacement_selection(&[], 4).is_empty());
}

#[test]
#[should_panic(expected = "at least one")]
fn zero_memory_is_rejected() {
    // Definiteness: the algorithm is defined for P >= 1 only.
    replacement_selection(&[3, 1, 2], 0);
}
