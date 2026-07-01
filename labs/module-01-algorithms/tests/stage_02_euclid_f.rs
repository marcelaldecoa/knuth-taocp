//! Stage 2 — Avoiding trivial replacements (Algorithm 1.1F, exercise 1.1-3).

use lab_01_algorithms::{euclid_e, euclid_f};

#[test]
fn agrees_with_algorithm_e_everywhere() {
    for m in 1..=120u64 {
        for n in 1..=120u64 {
            assert_eq!(euclid_f(m, n), euclid_e(m, n), "F and E disagree on ({m},{n})");
        }
    }
}

#[test]
fn worked_examples() {
    assert_eq!(euclid_f(544, 119), 17);
    assert_eq!(euclid_f(119, 544), 17);
    assert_eq!(euclid_f(2166, 6099), 57);
    assert_eq!(euclid_f(1, 1), 1);
}

#[test]
#[should_panic(expected = "positive")]
fn zero_is_rejected() {
    euclid_f(0, 1);
}
