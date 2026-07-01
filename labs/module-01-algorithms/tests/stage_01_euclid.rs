//! Stage 1 — Euclid's algorithm, step by step (Algorithm 1.1E).
//!
//! Implement `euclid_e` in src/lab.rs. The lesson: course/module-01-algorithms/README.md.

use lab_01_algorithms::euclid_e;

#[test]
fn worked_example_from_the_text() {
    // §1.1 traces Algorithm E on m = 544, n = 119: the remainders run
    // 68, 51, 17, 0 — so the answer is 17.
    assert_eq!(euclid_e(544, 119), 17);
}

#[test]
fn exercise_1_1_1_numbers() {
    assert_eq!(euclid_e(2166, 6099), 57);
    assert_eq!(euclid_e(6099, 2166), 57);
}

#[test]
fn m_smaller_than_n_works() {
    // When m < n, the first execution of E1 leaves r = m, and E3 swaps the
    // operands — Algorithm E handles this case with no special test.
    assert_eq!(euclid_e(119, 544), 17);
    assert_eq!(euclid_e(1, 999_999_937), 1);
}

#[test]
fn divides_both_and_is_greatest() {
    for m in 1..=80u64 {
        for n in 1..=80u64 {
            let d = euclid_e(m, n);
            assert!(d > 0 && m % d == 0 && n % d == 0, "gcd({m},{n})={d} must divide both");
            for c in (d + 1)..=m.min(n) {
                assert!(
                    !(m % c == 0 && n % c == 0),
                    "gcd({m},{n})={d} but {c} also divides both"
                );
            }
        }
    }
}

#[test]
fn gcd_of_equal_numbers() {
    assert_eq!(euclid_e(42, 42), 42);
    assert_eq!(euclid_e(1, 1), 1);
}

#[test]
fn large_inputs_terminate_quickly() {
    // Finiteness: Algorithm E on word-sized inputs takes at most a few
    // dozen divisions (you'll prove why in stage 4).
    assert_eq!(euclid_e(u64::MAX, u64::MAX - 1), 1);
    assert_eq!(euclid_e(2u64.pow(61), 2u64.pow(40)), 2u64.pow(40));
}

#[test]
#[should_panic(expected = "positive")]
fn zero_m_is_rejected() {
    // Definiteness: Algorithm E is stated for positive integers only.
    euclid_e(0, 5);
}

#[test]
#[should_panic(expected = "positive")]
fn zero_n_is_rejected() {
    euclid_e(5, 0);
}
