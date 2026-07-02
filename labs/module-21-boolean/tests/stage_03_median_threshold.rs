//! Stage 3 — Median, threshold, and symmetric functions (TAOCP Vol. 4A,
//! §7.1.1). Implement `majority`, `threshold_at_least`, `symmetric_function`,
//! `is_monotone`, and `is_self_dual` in src/lab.rs.
//! Lesson: course/module-21-boolean/README.md.

use lab_21_boolean::{
    is_monotone, is_self_dual, majority, symmetric_function, threshold_at_least, BoolFunc,
};

#[test]
fn majority_of_three_and_five() {
    // majority-of-3 truth table over (x1, x2, x3): true iff >= 2 ones.
    for x in 0..8u32 {
        let bits = [x & 1 == 1, (x >> 1) & 1 == 1, (x >> 2) & 1 == 1];
        assert_eq!(majority(&bits), x.count_ones() >= 2, "maj3 at {x}");
    }
    // majority-of-5: true iff >= 3 ones.
    for x in 0..32u32 {
        let bits: Vec<bool> = (0..5).map(|j| (x >> j) & 1 == 1).collect();
        assert_eq!(majority(&bits), x.count_ones() >= 3, "maj5 at {x}");
    }
}

#[test]
fn median_equals_threshold_at_half() {
    // For an odd number of inputs, median = threshold at (n+1)/2.
    for n in [1usize, 3, 5, 7] {
        for x in 0..(1u32 << n) {
            let bits: Vec<bool> = (0..n).map(|j| (x >> j) & 1 == 1).collect();
            let k = (n + 1) / 2;
            assert_eq!(majority(&bits), threshold_at_least(&bits, k), "n={n} x={x}");
        }
    }
}

#[test]
fn threshold_counts_ones() {
    let bits = [true, false, true, true, false];
    assert!(threshold_at_least(&bits, 3));
    assert!(!threshold_at_least(&bits, 4));
    assert!(threshold_at_least(&bits, 0)); // vacuously true
}

#[test]
fn symmetric_function_reconstructs_majority() {
    // weights[j] = value when exactly j inputs are true.
    for &n in &[3u32, 5] {
        let weights: Vec<bool> = (0..=n).map(|j| j > n / 2).collect();
        let sym = symmetric_function(n, &weights);
        let maj = BoolFunc::from_closure(n, |x| x.count_ones() > n / 2);
        assert_eq!(sym, maj, "symmetric majority-of-{n}");
    }
}

#[test]
fn symmetric_function_is_symmetric() {
    // Value depends only on popcount: parity of 4 inputs.
    let weights = [false, true, false, true, false]; // odd popcount -> true
    let parity = symmetric_function(4, &weights);
    assert_eq!(parity, BoolFunc::from_closure(4, |x| x.count_ones() % 2 == 1));
}

#[test]
fn monotone_flags() {
    let and2 = BoolFunc::from_closure(2, |x| (x & 1 == 1) && ((x >> 1) & 1 == 1));
    let or2 = BoolFunc::from_closure(2, |x| (x & 1 == 1) || ((x >> 1) & 1 == 1));
    let maj3 = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
    let xor2 = BoolFunc::from_closure(2, |x| (x & 1) ^ ((x >> 1) & 1) == 1);

    assert!(is_monotone(&and2));
    assert!(is_monotone(&or2));
    assert!(is_monotone(&maj3));
    assert!(!is_monotone(&xor2)); // raising x1 can flip 1 -> 0
}

#[test]
fn self_dual_flags() {
    let maj3 = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
    let dictator = BoolFunc::from_closure(3, |x| x & 1 == 1); // f = x1
    let and2 = BoolFunc::from_closure(2, |x| (x & 1 == 1) && ((x >> 1) & 1 == 1));

    assert!(is_self_dual(&maj3)); // majority of an odd count is self-dual
    assert!(is_self_dual(&dictator)); // a projection is self-dual
    assert!(!is_self_dual(&and2));
}

#[test]
fn dedekind_numbers_by_enumeration() {
    // The number of monotone Boolean functions of n variables is the
    // Dedekind number M(n): 2, 3, 6, 20, 168 for n = 0..4. We enumerate all
    // 2^(2^n) functions and count the monotone ones.
    let expected = [2u64, 3, 6, 20, 168];
    for n in 0..=4u32 {
        let total: u64 = 1 << (1u32 << n); // 2^(2^n)
        let mut count = 0u64;
        for table in 0..total {
            if is_monotone(&BoolFunc { n, table }) {
                count += 1;
            }
        }
        assert_eq!(count, expected[n as usize], "Dedekind number M({n})");
    }
}

#[test]
#[should_panic(expected = "one weight per popcount")]
fn symmetric_rejects_wrong_weight_count() {
    // weights must have exactly n + 1 entries (one per possible popcount).
    symmetric_function(3, &[true, false]);
}
