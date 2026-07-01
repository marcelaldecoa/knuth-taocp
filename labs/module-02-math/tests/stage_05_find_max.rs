//! Stage 5 — Analysis of an algorithm: finding the maximum
//! (Algorithm 1.2.10M).
//!
//! Implement `find_max` and `find_max_counting` in src/lab.rs. The lesson:
//! course/module-02-math/README.md, part 7 — this stage reproduces the
//! first complete algorithm analysis in the book: E[A] = H_n - 1.

use lab_02_math::{find_max, find_max_counting};

/// Deterministic pseudo-random numbers: the course's standard LCG.
struct Lcg(u64);

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        (self.next_u64() >> 33) % bound
    }
}

/// A Fisher–Yates-shuffled permutation of 1..=n, driven by the LCG.
fn shuffled_permutation(n: usize, rng: &mut Lcg) -> Vec<i64> {
    let mut v: Vec<i64> = (1..=n as i64).collect();
    for i in (1..n).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        v.swap(i, j);
    }
    v
}

/// Visit every permutation of `v` (Heap's algorithm), calling `f` on each.
fn for_each_permutation(v: &mut Vec<i64>, k: usize, f: &mut impl FnMut(&[i64])) {
    if k <= 1 {
        f(v);
        return;
    }
    for i in 0..k {
        for_each_permutation(v, k - 1, f);
        if k % 2 == 0 {
            v.swap(i, k - 1);
        } else {
            v.swap(0, k - 1);
        }
    }
}

#[test]
fn lesson_worked_trace() {
    // The hand trace from the lesson: X = (7, 2, 9, 4, 8, 3).
    // Scanning from the right, m runs 3 -> 8 -> 9, so A = 2; the maximum 9
    // sits at (0-based) index 2.
    assert_eq!(find_max(&[7, 2, 9, 4, 8, 3]), (2, 9));
    assert_eq!(find_max_counting(&[7, 2, 9, 4, 8, 3]), (2, 9, 2));
}

#[test]
fn single_element() {
    // n = 1: step M2 terminates immediately; no comparison, no change.
    assert_eq!(find_max_counting(&[42]), (0, 42, 0));
    assert_eq!(find_max(&[-7]), (0, -7));
}

#[test]
fn extreme_arrangements() {
    // Increasing input: the initial m = X[n] is already the maximum; A = 0.
    let inc: Vec<i64> = (1..=50).collect();
    assert_eq!(find_max_counting(&inc), (49, 50, 0));
    // Decreasing input: every one of the n - 1 comparisons changes m.
    let dec: Vec<i64> = (1..=50).rev().collect();
    assert_eq!(find_max_counting(&dec), (0, 50, 49));
}

#[test]
fn ties_keep_the_rightmost_maximum() {
    // M3 tests "X[k] <= m" — equal elements do NOT displace the current
    // maximum, so scanning right-to-left keeps the largest index j.
    assert_eq!(find_max(&[5, 5, 5]), (2, 5));
    assert_eq!(find_max(&[3, 7, 7, 1]), (2, 7));
    assert_eq!(find_max_counting(&[2, 2]), (1, 2, 0));
    assert_eq!(find_max_counting(&[9, 1, 9]), (2, 9, 0));
}

#[test]
fn negatives_and_agreement_between_the_two_entry_points() {
    assert_eq!(find_max(&[-5, -2, -9]), (1, -2));
    let mut rng = Lcg(20260701);
    for len in 1..=60usize {
        let v: Vec<i64> = (0..len).map(|_| (rng.next_u64() % 1000) as i64 - 500).collect();
        let (j, m) = find_max(&v);
        let (j2, m2, _a) = find_max_counting(&v);
        assert_eq!((j, m), (j2, m2), "entry points disagree on {v:?}");
        assert_eq!(m, *v.iter().max().unwrap());
        assert_eq!(v[j], m);
        assert!(v[j + 1..].iter().all(|&x| x < m), "j must be the last maximum");
    }
}

#[test]
fn exact_distribution_of_a_over_all_permutations_of_six() {
    // §1.2.10: the number of permutations of {1..n} on which A = k is the
    // Stirling cycle number [n, k+1]. For n = 6 the histogram is
    // (120, 274, 225, 85, 15, 1), and the total of A over all 720
    // permutations is 720 * (H_6 - 1) = 720 * 29/20 = 1044.
    let mut histogram = [0u64; 6];
    let mut total = 0u64;
    let mut v: Vec<i64> = (1..=6).collect();
    for_each_permutation(&mut v, 6, &mut |p| {
        let (_, m, a) = find_max_counting(p);
        assert_eq!(m, 6);
        histogram[a as usize] += 1;
        total += a;
    });
    assert_eq!(histogram, [120, 274, 225, 85, 15, 1]);
    assert_eq!(total, 1044); // exactly H_6 - 1 = 29/20 on average
}

#[test]
fn average_of_a_matches_h_n_minus_one() {
    // THE result of §1.2.10: on a random permutation of n distinct values,
    // E[A] = H_n - 1. Monte Carlo with the fixed LCG, n = 20:
    // H_20 - 1 = 2.5977...; 10_000 trials put the sample mean within a few
    // thousandths of a sigma-scaled tolerance.
    let n = 20usize;
    let trials = 10_000u64;
    let mut rng = Lcg(112358);
    let mut total = 0u64;
    for _ in 0..trials {
        let p = shuffled_permutation(n, &mut rng);
        let (_, m, a) = find_max_counting(&p);
        assert_eq!(m, n as i64);
        total += a;
    }
    let mean = total as f64 / trials as f64;
    let h_n_minus_1: f64 = (2..=n).map(|k| 1.0 / k as f64).sum();
    assert!(
        (mean - h_n_minus_1).abs() < 0.08,
        "mean A = {mean}, expected about H_{n} - 1 = {h_n_minus_1}"
    );
}

#[test]
#[should_panic(expected = "n >= 1")]
fn empty_input_is_rejected() {
    // Algorithm M is stated for n >= 1 — definiteness, as always.
    find_max(&[]);
}
