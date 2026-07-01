//! Stage 4 — Counting divisions; Lamé's worst case.
//!
//! T(m, n) counts executions of step E1. Lamé's theorem (1845, proved in
//! TAOCP §4.5.3) says consecutive Fibonacci numbers are Euclid's worst case:
//! if n < F_{k+1} then T(m, n) never exceeds what the Fibonacci pair forces.
//! Here you confirm it by exhaustive experiment — analysis of algorithms
//! the way Knuth practices it.

use lab_01_algorithms::division_steps;

fn fib(k: usize) -> u64 {
    // F_0 = 0, F_1 = 1, F_2 = 1, ...
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..k {
        let t = a + b;
        a = b;
        b = t;
    }
    a
}

#[test]
fn worked_example() {
    // 544 -> 119 -> 68 -> 51 -> 17 -> 0: four divisions.
    assert_eq!(division_steps(544, 119), 4);
    // One division suffices when n divides m.
    assert_eq!(division_steps(100, 20), 1);
    assert_eq!(division_steps(7, 7), 1);
}

#[test]
fn fibonacci_pairs_hit_the_bound_exactly() {
    // T(F_{k+1}, F_k) = k - 1 for k >= 2: every quotient along the way is 1,
    // the slowest possible descent.
    for k in 2..=40 {
        assert_eq!(
            division_steps(fib(k + 1), fib(k)),
            (k - 1) as u32,
            "T(F_{}, F_{})",
            k + 1,
            k
        );
    }
}

#[test]
fn nothing_below_f16_beats_the_fibonacci_pair() {
    // Exhaustive check of Lamé's theorem for all m, n < F_16 = 987:
    // the maximum of T is achieved at (F_14, F_15) = (377, 610) — a swap
    // costs one extra division — and no pair does worse.
    let bound = fib(16); // 987
    let mut max_t = 0;
    let mut argmax = (0, 0);
    for m in 1..bound {
        for n in 1..bound {
            let t = division_steps(m, n);
            if t > max_t {
                max_t = t;
                argmax = (m, n);
            }
        }
    }
    assert_eq!(max_t, 14, "worst case below F_16 should be 14 divisions");
    assert_eq!(argmax, (377, 610), "first worst-case pair is (F_14, F_15)");
    assert_eq!(division_steps(610, 377), 13);
}

#[test]
fn average_behavior_is_logarithmic() {
    // Knuth's deeper result: T(m, n) averages about (12 ln 2 / pi^2) ln n.
    // We don't reproduce the constant here — just confirm the *scale*:
    // averaging over m for fixed n = 10009 (prime), the mean number of
    // divisions is well under 30 even though n has 5 digits.
    let n = 10009u64;
    let total: u64 = (1..n).map(|m| division_steps(m, n) as u64).sum();
    let mean = total as f64 / (n - 1) as f64;
    assert!(
        (5.0..12.0).contains(&mean),
        "mean T(m, {n}) = {mean:.2}, expected ~ 0.843 ln {n} + 1.47 ≈ 9.2"
    );
}
