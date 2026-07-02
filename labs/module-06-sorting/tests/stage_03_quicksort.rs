//! Stage 3 — Quicksort (Algorithm 5.2.2Q, partition-exchange).
//!
//! Implement `quicksort` and `quicksort_counting` in src/lab.rs. Lesson:
//! course/module-06-sorting/README.md.
//!
//! The tests do not care about your pivot rule or stack discipline — they test
//! the *contract*: quicksort sorts (a permutation), survives the adversarial
//! shapes that kill the take-the-first-key pivot, and makes O(n log n)
//! comparisons on random data. Median-of-three is recommended so that sorted,
//! reverse, and organ-pipe inputs at n = 100_000 do not blow the stack or the
//! time budget.

use lab_06_sorting::{quicksort, quicksort_counting};

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
fn sorts_knuths_example() {
    let mut v = KNUTH16;
    quicksort(&mut v);
    assert_eq!(v, KNUTH16_SORTED);
}

#[test]
fn correct_on_adversarial_shapes() {
    for &n in &[0usize, 1, 2, 3, 9, 10, 100, 1000] {
        // all equal
        let mut eq = vec![7i64; n];
        quicksort(&mut eq);
        assert!(is_sorted(&eq) && eq.iter().all(|&x| x == 7), "all-equal n={n}");
        // already sorted
        let mut asc: Vec<i64> = (0..n as i64).collect();
        quicksort(&mut asc);
        assert!(is_sorted(&asc), "sorted n={n}");
        // reverse sorted
        let mut desc: Vec<i64> = (0..n as i64).rev().collect();
        let orig = desc.clone();
        quicksort(&mut desc);
        assert!(is_sorted(&desc) && same_multiset(&desc, &orig), "reverse n={n}");
        // organ pipe: 0,1,...,k,...,1,0 — a classic median-of-three trap
        let mut organ: Vec<i64> = (0..n as i64).chain((0..n as i64).rev()).collect();
        let og = organ.clone();
        quicksort(&mut organ);
        assert!(is_sorted(&organ) && same_multiset(&organ, &og), "organ n={n}");
    }
}

#[test]
fn permutation_on_lcg_data() {
    let mut rng = lcg(0xDEAD_BEEF);
    for _ in 0..30 {
        let n = (rng() % 2000) as usize;
        let original: Vec<i64> = (0..n).map(|_| (rng() >> 40) as i64).collect();
        let mut v = original.clone();
        quicksort(&mut v);
        assert!(is_sorted(&v), "unsorted n={n}");
        assert!(same_multiset(&v, &original), "not a permutation n={n}");
    }
}

#[test]
fn heavy_duplicate_keys() {
    // Few distinct values stress the partition's handling of keys == pivot.
    let mut rng = lcg(0x0D15EA5E);
    let n = 20_000;
    let original: Vec<i64> = (0..n).map(|_| (rng() % 4) as i64).collect();
    let mut v = original.clone();
    quicksort(&mut v);
    assert!(is_sorted(&v));
    assert!(same_multiset(&v, &original));
}

#[test]
fn one_hundred_thousand_random_keys_complete() {
    // The headline scalability check: 100k keys must sort well within budget.
    let mut rng = lcg(0x1234_5678_9ABC_DEF0);
    let original: Vec<i64> = (0..100_000).map(|_| (rng() >> 20) as i64).collect();
    let mut v = original.clone();
    quicksort(&mut v);
    assert!(is_sorted(&v));
    assert!(same_multiset(&v, &original));
    // Sorted and reverse at 100k must NOT be quadratic (median-of-three).
    let mut asc: Vec<i64> = (0..100_000).collect();
    quicksort(&mut asc);
    assert!(is_sorted(&asc));
    let mut desc: Vec<i64> = (0..100_000).rev().collect();
    quicksort(&mut desc);
    assert!(is_sorted(&desc));
}

#[test]
fn counting_variant_sorts_and_stays_near_n_ln_n() {
    // Knuth's average is ~ 2 n ln n comparisons; median-of-three beats it.
    // We assert a loose ceiling of 3 n ln n at n = 10_000 on random data.
    let mut rng = lcg(0xFEED_FACE);
    let n = 10_000usize;
    let original: Vec<i64> = (0..n).map(|_| (rng() >> 33) as i64).collect();
    let mut v = original.clone();
    let comps = quicksort_counting(&mut v) as f64;
    assert!(is_sorted(&v), "counting variant must sort");
    assert!(same_multiset(&v, &original));
    let bound = 3.0 * (n as f64) * (n as f64).ln();
    assert!(comps < bound, "quicksort made {comps} comps, expected < {bound:.0}");
    assert!(comps > 0.0);
}
