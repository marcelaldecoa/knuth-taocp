//! Module 04 — Random Numbers (TAOCP Vol. 2, Ch. 3).
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 4` from the repository root. Work the stages in order — each
//! test file `tests/stage_NN_*.rs` corresponds to one stage, and the lesson
//! in `course/module-04-random/README.md` teaches the theory each stage
//! needs (full-period theorem, chi-square, uniformity proofs).
//!
//! Convention used throughout the course: keep Knuth's step labels
//! (P1, P2, ...) as comments in your implementation.
//!
//! One convention specific to this module: every consumer of randomness
//! takes `rng: &mut impl FnMut(u64) -> u64`, where `rng(bound)` must
//! return a value uniform in `0..bound`. The tests build such closures
//! from your `Lcg` using the *high-order* bits — the lesson explains why
//! `x % bound` on a power-of-two-modulus LCG would be a disaster.

/// Stage 1 — The linear congruential generator (§3.2.1):
///
/// ```text
///     X_{n+1} = (a * X_n + c) mod m .
/// ```
///
/// Four magic quantities: modulus `m`, multiplier `a`, increment `c`,
/// starting value (seed) `X_0`.
///
/// **Encoding of the modulus:** `m == 0` means m = 2^64 — the "mod" is
/// then just the wrap of 64-bit arithmetic (`wrapping_mul`/`wrapping_add`).
/// For `m > 0`, reduce the seed and both parameters mod `m` in `new`, and
/// do the update in `u128` so the product `a * x + c` cannot overflow.
#[allow(dead_code)] // the fields are read once you implement the methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lcg {
    /// Current state X_n.
    x: u64,
    /// Multiplier a.
    a: u64,
    /// Increment c.
    c: u64,
    /// Modulus m; `0` encodes 2^64.
    m: u64,
}

impl Lcg {
    /// Create the generator with starting value `seed` (= X_0).
    ///
    /// If `m > 0`, reduce `seed`, `a` and `c` mod `m`; if `m == 0`
    /// (meaning 2^64) use them as given.
    pub fn new(seed: u64, a: u64, c: u64, m: u64) -> Lcg {
        let _ = (seed, a, c, m);
        todo!("store the four quantities (reduced mod m when m > 0)")
    }

    /// Advance one step and return the new state
    /// X_{n+1} = (a X_n + c) mod m.
    ///
    /// The first call returns X_1 (the state constructed by `new` is X_0
    /// and is never returned — Knuth traces sequences from X_1).
    pub fn next(&mut self) -> u64 {
        todo!("one LCG step; use wrapping arithmetic when m == 0, u128 otherwise")
    }
}

/// Stage 1 — the period of the cycle that `seed` eventually enters, for a
/// *small* modulus `m > 0`, by direct cycle detection.
///
/// Iterate x <- (a·x + c) mod m, remembering the step number at which each
/// state was first seen (a `vec![u64::MAX; m]` works). The first time you
/// revisit a state, the period is `current_step - first_seen[state]` —
/// note that this correctly *excludes* the tail leading into the cycle
/// (tails occur whenever gcd(a, m) > 1, because the map is then not a
/// permutation of Z_m).
///
/// You will use this to verify Theorem 3.2.1.2A empirically: the period
/// is m (for every seed) iff c ⟂ m, a ≡ 1 (mod p) for every prime p | m,
/// and a ≡ 1 (mod 4) if 4 | m.
///
/// Panic if `m == 0` (this function is for finite small moduli only).
pub fn period(a: u64, c: u64, m: u64, seed: u64) -> u64 {
    let _ = (a, c, m, seed);
    todo!("direct cycle detection: first repeated state closes the cycle")
}

/// Stage 2 — the chi-square statistic V of §3.3.1:
///
/// ```text
///     V = sum over categories s of  (Y_s - n*p_s)^2 / (n*p_s)
/// ```
///
/// `observed[s]` is the count Y_s actually seen in category s;
/// `expected[s]` is the count n·p_s predicted by the model. The caller
/// compares V with the chi-square percentile table for k − 1 degrees of
/// freedom (k = number of categories).
///
/// Panic if the slices differ in length, are empty, or some expected
/// count is not positive.
pub fn chi_square(observed: &[u64], expected: &[f64]) -> f64 {
    let _ = (observed, expected);
    todo!("V = sum of (observed - expected)^2 / expected")
}

/// Stage 2 — chi-square against the *uniform* hypothesis: with k
/// categories and n total observations, every category expects n/k.
///
/// Panic if `counts` is empty or sums to zero.
pub fn chi_square_uniform(counts: &[u64]) -> f64 {
    let _ = counts;
    todo!("equal expected counts n/k, then delegate to chi_square")
}

/// Stage 3 — Algorithm 3.4.2P (Shuffling; Fisher–Yates).
///
/// `rng(bound)` returns a value uniform in `0..bound` (bound ≥ 1).
/// In 0-based terms, with t = items.len():
///
/// ```text
/// P1. [Initialize.]  Set j <- t - 1.
/// P2. [Generate U.]  Set k <- rng(j + 1), uniform in 0..=j.
/// P3. [Exchange.]    Swap items[k] <-> items[j]; decrease j by 1;
///                    if j >= 1, return to P2.
/// ```
///
/// The candidate range *shrinks* with j — that is the whole trick, and the
/// lesson proves it makes all t! permutations equally likely. Leave slices
/// of length 0 or 1 untouched without consuming any randomness.
pub fn shuffle<T>(items: &mut [T], rng: &mut impl FnMut(u64) -> u64) {
    let _ = (items, rng);
    todo!("implement Algorithm 3.4.2P (j runs backwards from t - 1 to 1)")
}

/// Stage 3 — the classic BROKEN shuffle, implemented on purpose: at every
/// position i in 0..n, swap `items[i]` with `items[rng(n)]` — the swap
/// candidate is drawn from the *full* range each time.
///
/// It cannot be uniform: the n draws form n^n equally likely "tapes", and
/// n^n is not divisible by n! for n > 2 (for n = 3: 27 tapes onto 6
/// permutations). The stage test enumerates all 27 tapes and checks the
/// exact biased distribution.
pub fn naive_shuffle<T>(items: &mut [T], rng: &mut impl FnMut(u64) -> u64) {
    let _ = (items, rng);
    todo!("for i in 0..n: swap items[i] with items[rng(n)]")
}

/// Stage 4 — Algorithm 3.4.2R (Reservoir sampling): choose `k` items
/// uniformly at random from a stream whose length is *not known in
/// advance*, in one pass and O(k) memory.
///
/// With t counting how many records have been seen so far:
///
/// ```text
/// R1. [Initialize.]   Fill the reservoir with the first k records;
///                     set t <- k.
/// R2. [Next record.]  If the stream is exhausted, terminate: the
///                     reservoir is the sample. Otherwise read a record
///                     and set t <- t + 1.
/// R3. [Draw.]         Set M <- rng(t), uniform in 0..t.
/// R4. [Replace?]      If M < k, replace reservoir[M] by the new record.
///                     Return to R2.
/// ```
///
/// Contract details the tests rely on:
/// - if the stream holds fewer than `k` items, return all of them
///   (in stream order);
/// - if `k == 0`, return an empty vector *without consulting `rng`*.
pub fn reservoir_sample<T>(
    stream: impl IntoIterator<Item = T>,
    k: usize,
    rng: &mut impl FnMut(u64) -> u64,
) -> Vec<T> {
    let _ = (stream, k, rng);
    todo!("implement Algorithm 3.4.2R (replace slot M with probability k/t)")
}
