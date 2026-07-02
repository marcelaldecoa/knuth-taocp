//! Stage 1 — Binary search (Algorithm 6.2.1B).
//!
//! Implement `binary_search` and `binary_search_comparisons` in src/lab.rs.
//! Lesson: course/module-07-searching/README.md.

use lab_07_searching::{binary_search, binary_search_comparisons};

/// Knuth's running example table in §6.2.1: 16 sorted keys.
const SIXTEEN: [i64; 16] = [
    61, 87, 154, 170, 275, 426, 503, 509, 512, 612, 653, 677, 703, 765, 897, 908,
];

/// A slow-but-obviously-correct reference: linear scan of a *distinct* sorted
/// slice. `Ok(i)` if `a[i] == key`, else `Err(p)` with p = #elements < key.
fn linear_search(a: &[i64], key: i64) -> Result<usize, usize> {
    for (i, &x) in a.iter().enumerate() {
        if x == key {
            return Ok(i);
        }
        if x > key {
            return Err(i);
        }
    }
    Err(a.len())
}

/// floor(lg n) for n >= 1; the comparison bound is floor(lg n) + 1.
fn flg(n: usize) -> u32 {
    assert!(n >= 1);
    usize::BITS - 1 - (n.leading_zeros())
}

#[test]
fn worked_example_6_2_1() {
    // §6.2.1 traces the search for K = 503: probes 509, 170, 426, 503.
    let (res, c) = binary_search_comparisons(&SIXTEEN, 503);
    assert_eq!(res, Ok(6));
    assert_eq!(c, 4);
    // Unsuccessful search for 400: probes 509, 170, 426, 275, then u < l.
    let (res, c) = binary_search_comparisons(&SIXTEEN, 400);
    assert_eq!(res, Err(5)); // five keys are < 400
    assert_eq!(c, 4);
}

#[test]
fn empty_and_singleton() {
    assert_eq!(binary_search(&[], 42), Err(0));
    let (_, c) = binary_search_comparisons(&[], 42);
    assert_eq!(c, 0, "an empty table needs zero comparisons");

    let one = [7i64];
    assert_eq!(binary_search(&one, 7), Ok(0));
    assert_eq!(binary_search(&one, 6), Err(0));
    assert_eq!(binary_search(&one, 8), Err(1));
    for key in 5..=9 {
        let (_, c) = binary_search_comparisons(&one, key);
        assert!(c <= 1, "floor(lg 1)+1 = 1");
    }
}

#[test]
fn exhaustive_against_linear_scan() {
    // Distinct sorted table of the 100 even numbers 0, 2, ..., 198. Query
    // every integer from -2 to 201: even ones are present, odd ones absent,
    // and the endpoints exercise "before the first" / "after the last".
    let a: Vec<i64> = (0..100).map(|i| 2 * i).collect();
    for key in -2..=201 {
        let got = binary_search(&a, key);
        let want = linear_search(&a, key);
        // Insertion points must match exactly (the table is distinct); for a
        // hit we only require that the returned index actually holds the key.
        match (got, want) {
            (Ok(i), Ok(_)) => assert_eq!(a[i], key, "Ok index must hold the key"),
            (Err(p), Err(q)) => assert_eq!(p, q, "insertion point for {key}"),
            _ => panic!("membership disagreement for key {key}: {got:?} vs {want:?}"),
        }
    }
}

#[test]
fn exhaustive_over_small_sizes() {
    // For every table size 0..=64 built from distinct evens, and every key in
    // a covering range, agree with the linear scan on membership + insertion.
    for n in 0..=64usize {
        let a: Vec<i64> = (0..n as i64).map(|i| 2 * i).collect();
        for key in -2..=(2 * n as i64 + 2) {
            let got = binary_search(&a, key);
            match (got, linear_search(&a, key)) {
                (Ok(i), Ok(_)) => assert_eq!(a[i], key),
                (Err(p), Err(q)) => assert_eq!(p, q, "n={n} key={key}"),
                (g, w) => panic!("n={n} key={key}: {g:?} vs {w:?}"),
            }
        }
    }
}

#[test]
fn duplicates_membership_only() {
    // With repeated keys Knuth's algorithm may return *any* matching index.
    // Contract: a hit returns an index that holds the key; a miss's insertion
    // point keeps the slice sorted.
    let a = [1i64, 1, 1, 2, 2, 5, 5, 5, 5, 9];
    for &k in &[1, 2, 5, 9] {
        match binary_search(&a, k) {
            Ok(i) => assert_eq!(a[i], k, "must land on a slot equal to {k}"),
            Err(_) => panic!("{k} is present"),
        }
    }
    // Absent keys land where insertion preserves sortedness.
    for &k in &[0, 3, 4, 6, 10] {
        match binary_search(&a, k) {
            Err(p) => {
                assert!(p == 0 || a[p - 1] < k, "left neighbour < key");
                assert!(p == a.len() || a[p] > k, "right neighbour > key");
            }
            Ok(_) => panic!("{k} is absent"),
        }
    }
}

#[test]
fn theorem_6_2_1b_comparison_bound() {
    // Theorem B: C <= floor(lg N) + 1 for every search, successful or not.
    for n in 1..=257usize {
        let a: Vec<i64> = (0..n as i64).map(|i| 3 * i).collect();
        let bound = flg(n) + 1;
        for key in -1..=(3 * n as i64) {
            let (res, c) = binary_search_comparisons(&a, key);
            assert!(c <= bound, "n={n} key={key}: C={c} > floor(lg n)+1={bound}");
            assert_eq!(res, binary_search(&a, key), "the two entry points agree");
        }
    }
}

#[test]
fn comparison_count_is_tight_on_powers_of_two() {
    // On a table of size 2^k - 1 the tree is perfect: the deepest key costs
    // exactly floor(lg N)+1 comparisons, and some key attains the bound.
    for k in 1..=10u32 {
        let n = (1usize << k) - 1;
        let a: Vec<i64> = (0..n as i64).collect();
        let bound = flg(n) + 1;
        let max_c = (0..n as i64)
            .map(|key| binary_search_comparisons(&a, key).1)
            .max()
            .unwrap();
        assert_eq!(max_c, bound, "worst successful search on N={n} should hit the bound");
    }
}
