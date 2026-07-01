//! Module 01 — The Notion of an Algorithm (TAOCP Vol. 1, §1.1).
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 1` from the repository root. Work the stages in order — each
//! test file `tests/stage_NN_*.rs` corresponds to one stage, and the lesson
//! in `course/module-01-algorithms/README.md` walks you through the theory
//! each stage needs.
//!
//! Convention used throughout the course: keep Knuth's step labels
//! (E1, E2, ...) as comments in your implementation. Step-faithful first;
//! make it idiomatic later if you like, as long as the tests stay green.

/// Stage 1 — Algorithm 1.1E (Euclid's algorithm).
///
/// Given two positive integers m and n, return their greatest common divisor.
///
/// ```text
/// E1. [Find remainder.]  Divide m by n and let r be the remainder.
///                        (We will have 0 <= r < n.)
/// E2. [Is it zero?]      If r = 0, the algorithm terminates; n is the answer.
/// E3. [Reduce.]          Set m <- n, n <- r, and go back to E1.
/// ```
///
/// Algorithm E is defined for *positive* integers only. Make your function
/// panic (e.g. via `assert!`) with a message containing the word "positive"
/// if either argument is zero — definiteness is part of the definition of an
/// algorithm, and the grader checks it.
pub fn euclid_e(m: u64, n: u64) -> u64 {
    let _ = (m, n);
    todo!("implement Algorithm 1.1E")
}

/// Stage 2 — Algorithm 1.1F (exercise 1.1-3, rating 16).
///
/// Rewrite Euclid's algorithm so the trivial replacement `m <- n, n <- r`
/// is avoided: alternate the roles of m and n instead.
///
/// ```text
/// F1. [Remainder m/n.]  Divide m by n; set m to the remainder.
/// F2. [Is it zero?]     If m = 0, terminate with answer n.
/// F3. [Remainder n/m.]  Divide n by m; set n to the remainder.
/// F4. [Is it zero?]     If n = 0, terminate with answer m.  Go back to F1.
/// ```
pub fn euclid_f(m: u64, n: u64) -> u64 {
    let _ = (m, n);
    todo!("implement Algorithm 1.1F")
}

/// Stage 3 — Algorithm 1.2.1E (Extended Euclid).
///
/// Return `(d, a, b)` with `a*m + b*n = d = gcd(m, n)`.
///
/// Maintain the invariant  a1*m + b1*n = c  and  a*m + b*n = d  while the
/// pair (c, d) runs through the same values (m, n) take in Algorithm E.
/// When the remainder hits zero, (d, a, b) is your answer. Any (a, b)
/// satisfying the identity is accepted — the tests check the *invariant*,
/// not one particular coefficient pair.
pub fn extended_euclid(m: u64, n: u64) -> (u64, i128, i128) {
    let _ = (m, n);
    todo!("implement Algorithm 1.2.1E")
}

/// Stage 4 — The analysis of Algorithm E.
///
/// Return T(m, n): how many times step E1 (one division) executes when
/// Algorithm E runs on (m, n). You will use this to *verify empirically*
/// Lamé's 1845 theorem — the first practical algorithm analysis in history:
/// consecutive Fibonacci numbers are the worst case for Euclid's algorithm.
pub fn division_steps(m: u64, n: u64) -> u32 {
    let _ = (m, n);
    todo!("count executions of step E1")
}
