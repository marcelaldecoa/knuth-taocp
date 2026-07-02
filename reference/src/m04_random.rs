//! Module 04 — Random Numbers.
//! Source: TAOCP Vol. 2, 3rd ed., Ch. 3 (§3.2.1 linear congruential
//! generators, §3.3.1 the chi-square test, §3.4.2 random sampling and
//! shuffling).

/// A linear congruential generator (§3.2.1):
///
/// ```text
///     X_{n+1} = (a * X_n + c) mod m .
/// ```
///
/// Knuth's four magic quantities: the modulus `m`, the multiplier `a`, the
/// increment `c`, and the starting value (seed) `X_0`.
///
/// **Encoding of the modulus.** `m == 0` encodes m = 2^64 (the natural
/// "word size" modulus, computed with wrapping arithmetic). For `m > 0`
/// all state and parameters are reduced mod `m` and intermediate products
/// are carried out in `u128`, so any `u64` parameters are safe.
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
    /// If `m > 0`, the seed and both parameters are reduced mod `m`.
    /// If `m == 0` (meaning 2^64) they are used as given.
    pub fn new(seed: u64, a: u64, c: u64, m: u64) -> Lcg {
        if m == 0 {
            Lcg { x: seed, a, c, m }
        } else {
            Lcg {
                x: seed % m,
                a: a % m,
                c: c % m,
                m,
            }
        }
    }

    /// Advance one step and return the new state X_{n+1} = (a X_n + c) mod m.
    ///
    /// The first call returns X_1; the state after `new` is X_0 (never
    /// returned — Knuth indexes outputs from X_1 when tracing sequences).
    pub fn next(&mut self) -> u64 {
        self.x = if self.m == 0 {
            // m = 2^64: the "mod" is the wrap of 64-bit arithmetic itself.
            self.a.wrapping_mul(self.x).wrapping_add(self.c)
        } else {
            ((self.a as u128 * self.x as u128 + self.c as u128) % self.m as u128) as u64
        };
        self.x
    }
}

/// The period of the cycle that `seed` eventually enters, for a *small*
/// modulus `m > 0`, by direct cycle detection.
///
/// The sequence X, f(X), f(f(X)), ... over a finite state set is eventually
/// periodic: a (possibly empty) tail leads into a cycle. This returns the
/// cycle length — the tail is *not* counted. (When gcd(a, m) > 1 the map
/// is not a permutation of Z_m and tails really occur.)
///
/// Theorem 3.2.1.2A characterizes when the period is the maximum possible,
/// m, for *every* seed: c ⟂ m; a ≡ 1 (mod p) for every prime p | m; and
/// a ≡ 1 (mod 4) if 4 | m.
///
/// Panics if `m == 0` or `m` is too large for direct detection (> 2^26).
pub fn period(a: u64, c: u64, m: u64, seed: u64) -> u64 {
    assert!(m > 0, "period() needs a finite modulus m > 0");
    assert!(
        m <= 1 << 26,
        "period() is only for small m (direct cycle detection)"
    );
    // first_seen[x] = step number at which state x first appeared.
    let mut first_seen = vec![u64::MAX; m as usize];
    let mut x = seed % m;
    let mut step = 0u64;
    loop {
        if first_seen[x as usize] != u64::MAX {
            return step - first_seen[x as usize];
        }
        first_seen[x as usize] = step;
        x = ((a as u128 * x as u128 + c as u128) % m as u128) as u64;
        step += 1;
    }
}

/// The chi-square statistic V of §3.3.1:
///
/// ```text
///     V = sum over categories s of  (Y_s - n*p_s)^2 / (n*p_s)
/// ```
///
/// `observed[s]` is the count Y_s actually seen in category s and
/// `expected[s]` is the count n·p_s the probability model predicts.
/// Compare V against the chi-square percentile table with k − 1 degrees
/// of freedom (k = number of categories).
///
/// Panics if the slices differ in length, are empty, or any expected
/// count is not positive (Knuth requires every n·p_s to be reasonably
/// large — merge sparse categories before calling).
pub fn chi_square(observed: &[u64], expected: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        expected.len(),
        "observed and expected must have the same number of categories"
    );
    assert!(!observed.is_empty(), "chi_square needs at least one category");
    observed
        .iter()
        .zip(expected.iter())
        .map(|(&obs, &exp)| {
            assert!(exp > 0.0, "every expected count must be positive");
            let d = obs as f64 - exp;
            d * d / exp
        })
        .sum()
}

/// Chi-square against the *uniform* hypothesis: every one of the k
/// categories is equally likely, so each expects n/k of the n observations.
///
/// Panics if `counts` is empty or all zero.
pub fn chi_square_uniform(counts: &[u64]) -> f64 {
    assert!(
        !counts.is_empty(),
        "chi_square_uniform needs at least one category"
    );
    let n: u64 = counts.iter().sum();
    assert!(n > 0, "chi_square_uniform needs at least one observation");
    let e = n as f64 / counts.len() as f64;
    let expected = vec![e; counts.len()];
    chi_square(counts, &expected)
}

/// Algorithm 3.4.2P (Shuffling; Fisher–Yates in Knuth's formulation).
///
/// `rng(bound)` must return a value uniform in `0..bound` (bound ≥ 1).
/// In 0-based terms, with t = items.len():
///
/// ```text
/// P1. [Initialize.]  Set j <- t - 1.
/// P2. [Generate U.]  Set k <- rng(j + 1), uniform in 0..=j.
/// P3. [Exchange.]    Swap items[k] <-> items[j]; decrease j by 1;
///                    if j >= 1, return to P2.
/// ```
///
/// Note the *shrinking* range: position j is swapped only with positions
/// 0..=j. That is exactly what makes each of the t! outcomes occur for
/// exactly one sequence of draws, hence uniformly. Slices of length 0 or 1
/// are left untouched without consuming any randomness.
pub fn shuffle<T>(items: &mut [T], rng: &mut impl FnMut(u64) -> u64) {
    let t = items.len();
    if t < 2 {
        return;
    }
    // P1. [Initialize.] j <- t - 1.
    let mut j = t - 1;
    loop {
        // P2. [Generate U.] k uniform in 0..=j.
        let k = rng(j as u64 + 1) as usize;
        assert!(k <= j, "rng(bound) must return a value in 0..bound");
        // P3. [Exchange.] Swap X_k and X_j, decrease j.
        items.swap(k, j);
        if j == 1 {
            return;
        }
        j -= 1;
    }
}

/// The classic BROKEN shuffle, kept for the counting argument of the
/// lesson: at every position i, swap with a random index in the *full*
/// range 0..n.
///
/// It looks plausible and is *not* uniform: n positions each draw from n
/// choices, giving n^n equally likely "tapes", but n^n is not divisible
/// by n! for n > 2, so the n! permutations cannot be equally likely.
/// For n = 3, the 27 tapes map onto the 6 permutations as 4, 5, 5, 5, 4, 4
/// (in lexicographic order of the resulting permutation of [0, 1, 2]).
pub fn naive_shuffle<T>(items: &mut [T], rng: &mut impl FnMut(u64) -> u64) {
    let n = items.len();
    if n < 2 {
        return;
    }
    for i in 0..n {
        let k = rng(n as u64) as usize;
        assert!(k < n, "rng(bound) must return a value in 0..bound");
        items.swap(i, k);
    }
}

/// Algorithm 3.4.2R (Reservoir sampling), in its in-place form: choose
/// `k` items uniformly from a stream whose length is *not known in
/// advance*, in a single pass and O(k) memory.
///
/// `rng(bound)` must return a value uniform in `0..bound`. With t counting
/// how many records have been seen so far:
///
/// ```text
/// R1. [Initialize.]   Fill the reservoir with the first k records;
///                     set t <- k.  (If the stream is exhausted first,
///                     return everything read — see below.)
/// R2. [Next record.]  If the stream is exhausted, terminate: the
///                     reservoir is the sample. Otherwise read a record
///                     and set t <- t + 1.
/// R3. [Draw.]         Set M <- rng(t), uniform in 0..t.
/// R4. [Replace?]      If M < k, replace reservoir[M] by the new record.
///                     Return to R2.
/// ```
///
/// Invariant (proved by induction in the lesson): after t records have
/// been seen, every one of them is in the reservoir with probability
/// exactly k/t.
///
/// If the stream holds fewer than `k` items, all of them are returned (in
/// stream order). If `k == 0` the result is empty and `rng` is never
/// consulted.
pub fn reservoir_sample<T>(
    stream: impl IntoIterator<Item = T>,
    k: usize,
    rng: &mut impl FnMut(u64) -> u64,
) -> Vec<T> {
    if k == 0 {
        return Vec::new();
    }
    let mut it = stream.into_iter();
    // R1. [Initialize.] The first k records fill the reservoir.
    let mut reservoir: Vec<T> = Vec::with_capacity(k);
    while reservoir.len() < k {
        match it.next() {
            Some(item) => reservoir.push(item),
            None => return reservoir, // stream shorter than k: return all
        }
    }
    let mut t = k as u64;
    for item in it {
        // R2. [Next record.] t counts records seen so far.
        t += 1;
        // R3. [Draw.] M uniform in 0..t.
        let m = rng(t);
        assert!(m < t, "rng(bound) must return a value in 0..bound");
        // R4. [Replace?] The new record enters with probability k/t.
        if (m as usize) < k {
            reservoir[m as usize] = item;
        }
    }
    reservoir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_example_of_chapter_3() {
        // §3.1 opens with X_0 = a = c = 7, m = 10: the sequence is
        // 7, 6, 9, 0, 7, 6, 9, 0, ... — period 4. "Only ten distinct
        // values were possible anyway", but 4 is dismal.
        let mut g = Lcg::new(7, 7, 7, 10);
        assert_eq!(
            (0..8).map(|_| g.next()).collect::<Vec<_>>(),
            vec![6, 9, 0, 7, 6, 9, 0, 7]
        );
        assert_eq!(period(7, 7, 10, 7), 4);
    }

    #[test]
    fn mmix_generator_from_seed_zero() {
        // Knuth's 64-bit constants (MMIX): a = 6364136223846793005,
        // c = 1442695040888963407, m = 2^64 (encoded m = 0).
        let mut g = Lcg::new(0, 6364136223846793005, 1442695040888963407, 0);
        assert_eq!(g.next(), 1442695040888963407); // X_1 = c
        assert_eq!(g.next(), 1876011003808476466);
        assert_eq!(g.next(), 11166244414315200793);
    }

    #[test]
    fn full_period_examples_mod_16() {
        // Theorem 3.2.1.2A: full period iff c ⟂ m, a ≡ 1 mod every prime
        // dividing m, and a ≡ 1 mod 4 when 4 | m. For m = 16 that means
        // a ∈ {1, 5, 9, 13} and c odd.
        assert_eq!(period(5, 3, 16, 0), 16);
        assert_eq!(period(1, 1, 16, 9), 16);
        assert!(period(3, 1, 16, 0) < 16); // a ≡ 3 (mod 4) fails
        assert!(period(5, 2, 16, 0) < 16); // c not coprime to m fails
    }

    #[test]
    fn randu_satisfies_the_planes_identity() {
        // RANDU: a = 65539 = 2^16 + 3, c = 0, m = 2^31.
        // a^2 = 2^32 + 6·2^16 + 9 ≡ 6a − 9 (mod 2^31), hence
        // x_{n+2} − 6 x_{n+1} + 9 x_n ≡ 0 (mod 2^31) for all n.
        let m: i128 = 1 << 31;
        let mut g = Lcg::new(1, 65539, 0, 1 << 31);
        let xs: Vec<i128> = (0..100).map(|_| g.next() as i128).collect();
        assert_eq!(xs[0], 65539);
        assert_eq!(xs[1], 393225);
        for w in xs.windows(3) {
            assert_eq!((9 * w[0] - 6 * w[1] + w[2]).rem_euclid(m), 0);
        }
    }

    #[test]
    fn chi_square_dice_example() {
        // §3.3.1's style of worked example: 144 throws of two dice,
        // categories s = 2..=12 with expected counts 144·p_s. This table
        // gives V = 7 7/48.
        let observed = [2u64, 4, 10, 12, 22, 29, 21, 15, 14, 9, 6];
        let expected = [4.0, 8.0, 12.0, 16.0, 20.0, 24.0, 20.0, 16.0, 12.0, 8.0, 4.0];
        let v = chi_square(&observed, &expected);
        assert!((v - 343.0 / 48.0).abs() < 1e-9, "V = {v}");
    }

    #[test]
    fn chi_square_uniform_matches_general_form() {
        let counts = [3u64, 7];
        // n = 10, k = 2, expected 5 each: V = 4/5 + 4/5 = 1.6.
        assert!((chi_square_uniform(&counts) - 1.6).abs() < 1e-12);
        assert!((chi_square(&counts, &[5.0, 5.0]) - 1.6).abs() < 1e-12);
    }

    #[test]
    fn algorithm_p_is_exactly_uniform_for_n_3() {
        // Algorithm P on 3 items consumes rng(3) then rng(2): 6 possible
        // tapes, and each yields a distinct permutation.
        let mut seen = std::collections::BTreeSet::new();
        for k2 in 0..3u64 {
            for k1 in 0..2u64 {
                let tape = [k2, k1];
                let mut i = 0;
                let mut rng = |b: u64| {
                    let v = tape[i];
                    i += 1;
                    assert!(v < b);
                    v
                };
                let mut arr = [0u8, 1, 2];
                shuffle(&mut arr, &mut rng);
                assert!(seen.insert(arr), "tape {tape:?} duplicated {arr:?}");
            }
        }
        assert_eq!(seen.len(), 6);
    }

    #[test]
    fn naive_shuffle_bias_on_27_tapes() {
        // 27 equally likely tapes onto 6 permutations: 4/27 or 5/27 each,
        // never the uniform 4.5/27.
        let mut counts = std::collections::BTreeMap::new();
        for t in 0..27u64 {
            let tape = [t / 9, (t / 3) % 3, t % 3];
            let mut i = 0;
            let mut rng = |_b: u64| {
                let v = tape[i];
                i += 1;
                v
            };
            let mut arr = [0u8, 1, 2];
            naive_shuffle(&mut arr, &mut rng);
            *counts.entry(arr).or_insert(0u32) += 1;
        }
        let expect: Vec<([u8; 3], u32)> = vec![
            ([0, 1, 2], 4),
            ([0, 2, 1], 5),
            ([1, 0, 2], 5),
            ([1, 2, 0], 5),
            ([2, 0, 1], 4),
            ([2, 1, 0], 4),
        ];
        assert_eq!(counts.into_iter().collect::<Vec<_>>(), expect);
    }

    #[test]
    fn reservoir_exact_uniformity_n4_k2() {
        // n = 4, k = 2 consumes rng(3) then rng(4): 12 tapes; each of the
        // C(4, 2) = 6 two-element subsets arises from exactly 2 tapes.
        let mut counts = std::collections::BTreeMap::new();
        for m3 in 0..3u64 {
            for m4 in 0..4u64 {
                let tape = [m3, m4];
                let mut i = 0;
                let mut rng = |b: u64| {
                    let v = tape[i];
                    i += 1;
                    assert!(v < b);
                    v
                };
                let mut s = reservoir_sample(0..4u32, 2, &mut rng);
                s.sort_unstable();
                *counts.entry(s).or_insert(0u32) += 1;
            }
        }
        assert_eq!(counts.len(), 6);
        assert!(counts.values().all(|&c| c == 2));
    }

    #[test]
    fn reservoir_short_stream_returns_everything() {
        let mut rng = |_b: u64| panic!("rng must not be consulted");
        assert_eq!(reservoir_sample(0..3u32, 10, &mut rng), vec![0, 1, 2]);
        assert_eq!(reservoir_sample(0..5u32, 0, &mut rng), Vec::<u32>::new());
    }
}
