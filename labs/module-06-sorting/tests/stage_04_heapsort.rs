//! Stage 4 — Heapsort (Algorithm 5.2.3H).
//!
//! Implement `is_heap`, `sift_up`, `make_heap`, `heapsort`, and
//! `heapsort_counting` in src/lab.rs. Lesson: course/module-06-sorting/README.md.
//!
//! A max-heap stores keys in an array so that each parent dominates its two
//! children: v[(j-1)/2] >= v[j]. `make_heap` builds one in O(n); heapsort then
//! repeatedly extracts the maximum, giving an in-place O(n log n) *worst-case*
//! sort.

use lab_06_sorting::{heapsort, heapsort_counting, is_heap, make_heap, sift_up};

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
fn is_heap_recognizes_the_condition() {
    assert!(is_heap(&[]));
    assert!(is_heap(&[5]));
    assert!(is_heap(&[3, 2, 1]));
    assert!(is_heap(&[9, 5, 8, 1, 4, 7, 3]));
    assert!(!is_heap(&[1, 2, 3]), "ascending is not a max-heap");
    assert!(!is_heap(&[9, 5, 8, 6, 4]), "child 6 exceeds parent 5");
}

#[test]
fn make_heap_produces_a_heap_with_the_max_on_top() {
    let mut v = KNUTH16;
    make_heap(&mut v);
    assert!(is_heap(&v), "make_heap must yield a heap");
    // §5.2.3: the largest key of KNUTH16, 908, rises to the root.
    assert_eq!(v[0], 908);
    assert!(same_multiset(&v, &KNUTH16), "heap-building must permute in place");
}

#[test]
fn make_heap_on_lcg_batteries() {
    let mut rng = lcg(0x9E37_79B9);
    for &n in &[0usize, 1, 2, 3, 4, 15, 16, 17, 256, 1000] {
        let original: Vec<i64> = (0..n).map(|_| (rng() >> 40) as i64).collect();
        let mut v = original.clone();
        make_heap(&mut v);
        assert!(is_heap(&v), "not a heap at n={n}");
        assert_eq!(v.len(), n);
        assert!(same_multiset(&v, &original), "not a permutation at n={n}");
        if n > 0 {
            let max = *original.iter().max().unwrap();
            assert_eq!(v[0], max, "root must be the maximum at n={n}");
        }
    }
}

#[test]
fn sift_up_fixes_a_single_violation() {
    // Two child-heaps [.,3,2,1] under a small root: siftup the root into place.
    let mut v = vec![0i64, 8, 5, 7, 2, 3, 4]; // subtrees of root 0 are heaps
    let end = v.len() - 1;
    sift_up(&mut v, 0, end);
    assert!(is_heap(&v), "siftup must restore the heap");
    assert_eq!(v[0], 8, "the dominating key must reach the root");
}

#[test]
fn heapsort_sorts_knuths_example() {
    let mut v = KNUTH16;
    heapsort(&mut v);
    assert_eq!(v, KNUTH16_SORTED);
}

#[test]
fn heapsort_battery_including_tiny_sizes() {
    // n = 0, 1, 2 are the fiddly boundary cases for the selection loop.
    let mut e: [i64; 0] = [];
    heapsort(&mut e);
    assert_eq!(e, []);
    let mut one = [42];
    heapsort(&mut one);
    assert_eq!(one, [42]);
    let mut two = [2, 1];
    heapsort(&mut two);
    assert_eq!(two, [1, 2]);

    let mut rng = lcg(0xCAFE_D00D);
    for &n in &[3usize, 7, 16, 17, 100, 1000, 5000] {
        let original: Vec<i64> = (0..n).map(|_| (rng() >> 40) as i64).collect();
        let mut v = original.clone();
        heapsort(&mut v);
        assert!(is_sorted(&v), "unsorted n={n}");
        assert!(same_multiset(&v, &original), "not a permutation n={n}");
        // duplicates
        let mut dup: Vec<i64> = (0..n).map(|i| (i % 5) as i64).collect();
        heapsort(&mut dup);
        assert!(is_sorted(&dup), "dups n={n}");
        // reverse
        let mut desc: Vec<i64> = (0..n as i64).rev().collect();
        heapsort(&mut desc);
        assert!(is_sorted(&desc), "reverse n={n}");
    }
}

#[test]
fn comparison_count_obeys_the_worst_case_bound() {
    // Heapsort is O(n log n) in the WORST case; the loose bound (§5.2.3) is
    // 2 n lg n + O(n). We assert < 2 n lg n + 4 n on random data at n = 10000,
    // and check the same holds for reverse-sorted input (no easy case).
    let mut rng = lcg(0x9911_7733);
    let n = 10_000usize;
    let original: Vec<i64> = (0..n).map(|_| (rng() >> 33) as i64).collect();
    let mut v = original.clone();
    let comps = heapsort_counting(&mut v) as f64;
    assert!(is_sorted(&v));
    assert!(same_multiset(&v, &original));
    let nn = n as f64;
    let bound = 2.0 * nn * nn.log2() + 4.0 * nn;
    assert!(comps < bound, "heapsort made {comps} comps, expected < {bound:.0}");

    let mut desc: Vec<i64> = (0..n as i64).rev().collect();
    let c2 = heapsort_counting(&mut desc) as f64;
    assert!(is_sorted(&desc));
    assert!(c2 < bound, "reverse: {c2} comps, expected < {bound:.0}");
}
