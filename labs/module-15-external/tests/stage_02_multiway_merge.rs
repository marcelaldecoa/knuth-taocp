//! Stage 2 — k-way merging with a tree of losers (§5.4.1).
//!
//! Implement `merge_runs` and `merge_runs_counting` in src/lab.rs — with a
//! loser tree, not a BinaryHeap. The comparison-bound test can tell the
//! difference: a loser tree pays one comparison per level per record
//! (⌈lg k⌉ total); a heap pays up to two.
//! The lesson: course/module-15-external/README.md.

use lab_15_external::{merge_runs, merge_runs_counting, Run};

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

fn sorted_run(n: usize, seed: u64) -> Run {
    let mut r = lcg_vec(n, seed);
    r.sort();
    r
}

/// ⌈lg k⌉ (0 for k <= 1).
fn ceil_lg(k: usize) -> u64 {
    if k <= 1 {
        0
    } else {
        (usize::BITS - (k - 1).leading_zeros()) as u64
    }
}

/// The contract: sorted output, same multiset as flatten + sort.
fn check_merge(runs: &[Run]) -> Vec<i64> {
    let merged = merge_runs(runs);
    let mut expect: Vec<i64> = runs.iter().flatten().copied().collect();
    expect.sort();
    assert_eq!(merged, expect, "merge != flatten + sort");
    merged
}

#[test]
fn merges_the_worked_example_runs() {
    // The three runs replacement selection produced from Knuth's sixteen
    // keys (stage 1) merge back to the fully sorted file.
    let runs: Vec<Run> = vec![
        vec![87, 503, 512, 908],
        vec![61, 170, 275, 426, 653, 897],
        vec![154, 509, 612, 677, 703, 765],
    ];
    let merged = check_merge(&runs);
    assert_eq!(merged[0], 61);
    assert_eq!(merged[15], 908);
}

#[test]
fn equal_length_runs() {
    for k in [2usize, 3, 4, 8, 16] {
        let runs: Vec<Run> = (0..k).map(|i| sorted_run(200, 100 + i as u64)).collect();
        check_merge(&runs);
    }
}

#[test]
fn wildly_skewed_run_lengths() {
    let runs: Vec<Run> = vec![
        sorted_run(10_000, 1),
        sorted_run(1, 2),
        sorted_run(3, 3),
        sorted_run(0, 4),
        sorted_run(5, 5),
        sorted_run(2, 6),
    ];
    check_merge(&runs);
}

#[test]
fn empty_runs_are_harmless() {
    let runs: Vec<Run> = vec![vec![], sorted_run(50, 9), vec![], vec![], sorted_run(7, 10), vec![]];
    check_merge(&runs);
    // All runs empty.
    let runs: Vec<Run> = vec![vec![], vec![], vec![]];
    assert!(merge_runs(&runs).is_empty());
    // No runs at all.
    assert!(merge_runs(&[]).is_empty());
}

#[test]
fn duplicates_do_not_lose_elements() {
    // Ties everywhere: the multiset must survive intact.
    let runs: Vec<Run> = vec![
        vec![1, 1, 1, 2, 2, 3],
        vec![1, 2, 2, 2, 3, 3],
        vec![1, 1, 3, 3, 3, 3],
        vec![2; 10],
    ];
    let merged = check_merge(&runs);
    assert_eq!(merged.iter().filter(|&&x| x == 2).count(), 15);
    // All keys identical.
    let runs: Vec<Run> = (0..5).map(|_| vec![7i64; 40]).collect();
    assert_eq!(check_merge(&runs), vec![7i64; 200]);
}

#[test]
fn k_equals_one_and_k_equals_one_hundred() {
    let one = vec![sorted_run(333, 12)];
    assert_eq!(merge_runs(&one), one[0]);

    let runs: Vec<Run> = (0..100)
        .map(|i| sorted_run((i * 7) % 23, 500 + i as u64))
        .collect();
    check_merge(&runs);
}

#[test]
fn counting_variant_agrees_with_plain_merge() {
    let runs: Vec<Run> = (0..9).map(|i| sorted_run(100 + i * 13, 40 + i as u64)).collect();
    let (out, _comps) = merge_runs_counting(&runs);
    assert_eq!(out, merge_runs(&runs));
}

#[test]
fn comparison_bound_n_ceil_lg_k_plus_k() {
    // §5.4.1: initial tournament <= k - 1 comparisons, then one comparison
    // per level per output record — comparisons <= n·⌈lg k⌉ + k.
    // (Small additive slack allowed; a factor-2 miss is not.)
    for (k, len) in [(2usize, 1000usize), (5, 1000), (16, 500), (64, 100), (100, 200)] {
        let runs: Vec<Run> = (0..k).map(|i| sorted_run(len, 900 + i as u64)).collect();
        let n = (k * len) as u64;
        let (out, comps) = merge_runs_counting(&runs);
        assert_eq!(out.len(), n as usize);
        assert!(out.windows(2).all(|w| w[0] <= w[1]));
        let bound = n * ceil_lg(k) + 2 * k as u64 + 16;
        assert!(
            comps <= bound,
            "k = {k}: {comps} comparisons > n*ceil(lg k) + O(k) = {bound}"
        );
    }
}

#[test]
fn comparison_bound_survives_exhausted_runs() {
    // Once a run is exhausted its slot compares as +infinity — a flag
    // check, not a key comparison. Skewed lengths must not inflate the bill.
    let runs: Vec<Run> = vec![
        sorted_run(5000, 1),
        sorted_run(10, 2),
        sorted_run(10, 3),
        sorted_run(10, 4),
    ];
    let n: u64 = runs.iter().map(|r| r.len() as u64).sum();
    let (out, comps) = merge_runs_counting(&runs);
    assert_eq!(out.len(), n as usize);
    let bound = n * ceil_lg(4) + 2 * 4 + 16;
    assert!(comps <= bound, "{comps} > {bound}");
}
