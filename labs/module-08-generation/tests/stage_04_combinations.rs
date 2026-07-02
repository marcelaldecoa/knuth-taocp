//! Stage 4 — lexicographic/colex combinations (Algorithm 7.2.1.3T).
//!
//! Implement `combinations` in src/lab.rs.
//! Lesson: course/module-08-generation/README.md.

use lab_08_generation::combinations;

#[test]
fn four_choose_two_is_exact() {
    // Hand trace of Algorithm T for (n, k) = (4, 2): the strings c_2 c_1 run
    // 10, 20, 21, 30, 31, 32 — i.e. the ascending sets below.
    assert_eq!(
        combinations(4, 2),
        vec![
            vec![0, 1],
            vec![0, 2],
            vec![1, 2],
            vec![0, 3],
            vec![1, 3],
            vec![2, 3],
        ]
    );
}

#[test]
fn edge_cases_k_zero_and_k_equals_n() {
    // k = 0: the single empty combination.
    assert_eq!(combinations(0, 0), vec![Vec::<u32>::new()]);
    assert_eq!(combinations(5, 0), vec![Vec::<u32>::new()]);
    // k = n: the single full set {0,1,...,n-1}.
    assert_eq!(combinations(4, 4), vec![vec![0, 1, 2, 3]]);
    assert_eq!(combinations(1, 1), vec![vec![0]]);
}

#[test]
fn count_matches_pascals_triangle() {
    // |combinations(n,k)| = C(n,k) on a grid, checked against Pascal's rule.
    let mut binom = [[0u64; 11]; 11];
    for n in 0..=10usize {
        binom[n][0] = 1;
        for k in 1..=n {
            binom[n][k] = binom[n - 1][k - 1] + if k < n { binom[n - 1][k] } else { 0 };
        }
        for k in 0..=n {
            assert_eq!(
                combinations(n as u32, k as u32).len() as u64,
                binom[n][k],
                "C({n},{k})"
            );
        }
    }
}

#[test]
fn each_combination_is_sorted_and_distinct() {
    for n in 0..=10u32 {
        for k in 0..=n {
            for c in combinations(n, k) {
                assert_eq!(c.len(), k as usize, "C({n},{k}): right size");
                assert!(
                    c.windows(2).all(|w| w[0] < w[1]),
                    "C({n},{k}): {c:?} strictly ascending"
                );
                assert!(c.iter().all(|&x| x < n), "C({n},{k}): elements in range");
            }
        }
    }
}

#[test]
fn all_combinations_distinct_and_complete() {
    // The multiset of visited combinations is exactly all k-subsets, once each.
    for n in 0..=9u32 {
        for k in 0..=n {
            let mut all = combinations(n, k);
            let count = all.len();
            all.sort();
            all.dedup();
            assert_eq!(all.len(), count, "C({n},{k}): no duplicates");
        }
    }
}

#[test]
fn colex_order_is_stable_under_growing_n() {
    // A property of colexicographic order: the first C(m,k) combinations of
    // {0..n-1} choose-k use only {0..m-1}, so the (n,k) list extends the
    // (m,k) list as a prefix. Check (5,2) is a prefix of (7,2).
    let small = combinations(5, 2);
    let big = combinations(7, 2);
    assert_eq!(&big[..small.len()], &small[..], "colex is a prefix order");
}

#[test]
#[should_panic(expected = "k <= n")]
fn k_greater_than_n_panics() {
    combinations(3, 5);
}
