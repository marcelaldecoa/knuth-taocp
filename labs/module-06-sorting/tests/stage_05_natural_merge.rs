//! Stage 5 — Natural merge sort (Algorithm 5.2.4N).
//!
//! Implement `count_runs`, `natural_merge_sort`, and
//! `natural_merge_sort_counting` in src/lab.rs. Lesson:
//! course/module-06-sorting/README.md.
//!
//! Natural merge sort exploits the ascending runs already present in the data:
//! each pass merges runs pairwise, so an input with r runs finishes in about
//! lg r passes. The payoff is *adaptivity* — a nearly sorted file is nearly
//! free.

use lab_06_sorting::{count_runs, natural_merge_sort, natural_merge_sort_counting};

const KNUTH16: [i64; 16] = [
    503, 087, 512, 061, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703,
];
const KNUTH16_SORTED: [i64; 16] = [
    061, 087, 154, 170, 275, 426, 503, 509, 512, 612, 653, 677, 703, 765, 897, 908,
];

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

fn same_multiset(a: &[i64], b: &[i64]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

#[test]
fn count_runs_of_knuths_example() {
    // 503 | 087 512 | 061 908 | 170 897 | 275 653 | 426 |
    // 154 509 612 677 765 | 703  ->  eight ascending runs.
    assert_eq!(count_runs(&KNUTH16), 8);
}

#[test]
fn count_runs_extremes() {
    assert_eq!(count_runs(&[]), 0, "empty file has no runs");
    assert_eq!(count_runs(&[5]), 1);
    // A sorted file is a single run.
    assert_eq!(count_runs(&[1, 2, 3, 4, 5]), 1);
    assert_eq!(count_runs(&(0..1000i64).collect::<Vec<_>>()), 1);
    // A strictly decreasing file of length n is n runs.
    for n in [1i64, 2, 5, 50] {
        let desc: Vec<i64> = (0..n).rev().collect();
        assert_eq!(count_runs(&desc), n as usize, "reverse length {n}");
    }
    // Plateaus of equal keys stay within one run (non-decreasing).
    assert_eq!(count_runs(&[2, 2, 2, 1, 1, 3]), 2);
}

#[test]
fn count_runs_matches_a_direct_scan() {
    let mut rng = lcg(0x7777);
    for _ in 0..40 {
        let n = (rng() % 80) as usize;
        let v: Vec<i64> = (0..n).map(|_| (rng() % 10) as i64).collect();
        let expect = if n == 0 {
            0
        } else {
            1 + (1..n).filter(|&i| v[i - 1] > v[i]).count()
        };
        assert_eq!(count_runs(&v), expect, "runs of {v:?}");
    }
}

#[test]
fn sorts_knuths_example() {
    let mut v = KNUTH16;
    natural_merge_sort(&mut v);
    assert_eq!(v, KNUTH16_SORTED);
}

#[test]
fn correctness_battery() {
    let mut rng = lcg(0x5151_5151);
    for &n in &[0usize, 1, 2, 3, 8, 9, 16, 17, 100, 1000, 5000] {
        let original: Vec<i64> = (0..n).map(|_| (rng() >> 40) as i64).collect();
        let mut v = original.clone();
        natural_merge_sort(&mut v);
        assert!(is_sorted(&v), "random n={n}");
        assert!(same_multiset(&v, &original), "permutation n={n}");
        let mut desc: Vec<i64> = (0..n as i64).rev().collect();
        natural_merge_sort(&mut desc);
        assert!(is_sorted(&desc), "reverse n={n}");
        let mut dup: Vec<i64> = (0..n).map(|i| (i % 4) as i64).collect();
        natural_merge_sort(&mut dup);
        assert!(is_sorted(&dup), "dups n={n}");
    }
}

#[test]
fn sorted_input_is_nearly_free() {
    // Adaptivity (§5.2.4): an already-sorted file is one run, so a single
    // detection scan of n - 1 comparisons finishes it. Assert < 2n at 100k.
    let n = 100_000usize;
    let mut v: Vec<i64> = (0..n as i64).collect();
    let comps = natural_merge_sort_counting(&mut v) as f64;
    assert!(is_sorted(&v));
    assert!(
        comps < 2.0 * n as f64,
        "sorted input cost {comps} comparisons, expected < {}",
        2 * n
    );
    // A file that is already one run should cost exactly n - 1 detections.
    assert_eq!(
        natural_merge_sort_counting(&mut (0..1000i64).collect::<Vec<_>>()),
        999
    );
}

#[test]
fn random_input_is_log_linear() {
    let mut rng = lcg(0xABAB_CDCD);
    let n = 20_000usize;
    let original: Vec<i64> = (0..n).map(|_| (rng() >> 33) as i64).collect();
    let mut v = original.clone();
    let comps = natural_merge_sort_counting(&mut v) as f64;
    assert!(is_sorted(&v));
    assert!(same_multiset(&v, &original));
    let nn = n as f64;
    let bound = 2.5 * nn * nn.log2();
    assert!(comps < bound, "merge made {comps} comps, expected < {bound:.0}");
}
