//! Stage 2 — Shellsort (Algorithm 5.2.1D, diminishing increments).
//!
//! Implement `knuth_gaps`, `shell_sort`, and `shell_sort_with_gaps` in
//! src/lab.rs. Lesson: course/module-06-sorting/README.md.
//!
//! Shellsort is straight insertion done with a stride h, for a decreasing
//! sequence of strides ending at 1. The invariant that makes it work: after
//! the pass with increment h the file is *h-ordered* (v[i] <= v[i+h]).

use lab_06_sorting::{knuth_gaps, shell_sort, shell_sort_with_gaps};

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

/// v is h-ordered iff v[i] <= v[i+h] for every valid i (§5.2.1).
fn is_h_ordered(v: &[i64], h: usize) -> bool {
    (h..v.len()).all(|i| v[i - h] <= v[i])
}

#[test]
fn gaps_are_knuths_3h_plus_1_sequence() {
    // h_1 = 1, h_{s+1} = 3 h_s + 1  ->  1, 4, 13, 40, 121, ... kept below n,
    // returned largest first.
    assert_eq!(knuth_gaps(16), vec![13, 4, 1]);
    assert_eq!(knuth_gaps(100), vec![40, 13, 4, 1]);
    assert_eq!(knuth_gaps(2), vec![1]);
    assert_eq!(knuth_gaps(1), Vec::<usize>::new());
    assert_eq!(knuth_gaps(0), Vec::<usize>::new());
    // 3*40+1 = 121 >= 121, so it is excluded at n = 121; included at n = 122.
    assert_eq!(knuth_gaps(121), vec![40, 13, 4, 1]);
    assert_eq!(knuth_gaps(122), vec![121, 40, 13, 4, 1]);
}

#[test]
fn gaps_obey_the_recurrence_and_stay_below_n() {
    for &n in &[3usize, 17, 50, 500, 5000, 100_000] {
        let gaps = knuth_gaps(n);
        assert_eq!(*gaps.last().unwrap(), 1, "must end at 1 for n={n}");
        // Ascending, they should be 1, 4, 13, 40, ...
        let mut asc = gaps.clone();
        asc.reverse();
        assert_eq!(asc[0], 1);
        for w in asc.windows(2) {
            assert_eq!(w[1], 3 * w[0] + 1, "recurrence broken in {gaps:?}");
        }
        assert!(*gaps.first().unwrap() < n, "largest gap must be < n={n}");
    }
}

#[test]
fn shell_sort_sorts_knuths_example() {
    let mut v = KNUTH16;
    shell_sort(&mut v);
    assert_eq!(v, KNUTH16_SORTED);
}

#[test]
fn shell_sort_correct_on_many_sizes_and_shapes() {
    let mut rng = lcg(0x5EED);
    for &n in &[0usize, 1, 2, 5, 13, 40, 41, 100, 1000, 5000] {
        // random
        let mut v: Vec<i64> = (0..n).map(|_| (rng() >> 40) as i64).collect();
        shell_sort(&mut v);
        assert!(is_sorted(&v), "random n={n}");
        // already sorted
        let mut asc: Vec<i64> = (0..n as i64).collect();
        shell_sort(&mut asc);
        assert!(is_sorted(&asc), "ascending n={n}");
        // reverse
        let mut desc: Vec<i64> = (0..n as i64).rev().collect();
        shell_sort(&mut desc);
        assert!(is_sorted(&desc), "descending n={n}");
        // many duplicates
        let mut dup: Vec<i64> = (0..n).map(|i| (i % 3) as i64).collect();
        shell_sort(&mut dup);
        assert!(is_sorted(&dup), "duplicates n={n}");
    }
}

#[test]
fn with_gaps_one_is_exactly_straight_insertion() {
    // A lone [1] makes Algorithm D identical to Algorithm S.
    let mut rng = lcg(0x1);
    for _ in 0..20 {
        let n = (rng() % 300) as usize;
        let mut v: Vec<i64> = (0..n).map(|_| (rng() % 100) as i64).collect();
        shell_sort_with_gaps(&mut v, &[1]);
        assert!(is_sorted(&v));
    }
}

#[test]
fn each_pass_leaves_the_file_h_ordered() {
    // Run the passes one at a time and confirm h-orderedness after each.
    let mut rng = lcg(0xABCD_EF01);
    let n = 500usize;
    let base: Vec<i64> = (0..n).map(|_| (rng() % 1000) as i64).collect();
    let gaps = knuth_gaps(n); // e.g. [364, 121, 40, 13, 4, 1]
    let mut v = base.clone();
    for &h in &gaps {
        shell_sort_with_gaps(&mut v, &[h]);
        assert!(is_h_ordered(&v, h), "not {h}-ordered after its pass");
    }
    // Applying every gap in one call yields a fully sorted file.
    let mut w = base;
    shell_sort_with_gaps(&mut w, &gaps);
    assert!(is_sorted(&w));
}

#[test]
fn later_passes_preserve_earlier_h_orderings() {
    // Knuth's key fact (§5.2.1, Thm/lemma): an h-ordered file stays h-ordered
    // after a k-sort. Check 4-ordering survives the subsequent 1-sort... which
    // trivially it does, but also that after 13- then 4-sorting the file is
    // both 13- and 4-ordered.
    let mut rng = lcg(0x2222);
    let n = 400usize;
    let mut v: Vec<i64> = (0..n).map(|_| (rng() % 500) as i64).collect();
    shell_sort_with_gaps(&mut v, &[13]);
    assert!(is_h_ordered(&v, 13));
    shell_sort_with_gaps(&mut v, &[4]);
    assert!(is_h_ordered(&v, 4), "4-ordered");
    assert!(is_h_ordered(&v, 13), "13-ordering must be preserved by the 4-sort");
}

#[test]
#[should_panic(expected = "positive")]
fn zero_increment_is_rejected() {
    // Definiteness: an increment of 0 is meaningless.
    let mut v = [3, 1, 2];
    shell_sort_with_gaps(&mut v, &[2, 0, 1]);
}
