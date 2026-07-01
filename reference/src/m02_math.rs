//! Module 02 — Mathematical Preliminaries.
//! Source: TAOCP Vol. 1, 3rd ed., §1.2: sums (§1.2.3), binomial coefficients
//! (§1.2.6), harmonic numbers (§1.2.7), Fibonacci numbers (§1.2.8), and the
//! analysis of Algorithm 1.2.10M (finding the maximum).

// ---------------------------------------------------------------------------
// §1.2.3 — Sums in closed form
// ---------------------------------------------------------------------------

/// The triangular number 1 + 2 + ... + n = n(n+1)/2  (TAOCP Vol. 1, Eq.
/// 1.2.3-(14) territory; the schoolroom Gauss sum).
///
/// Exact for every `n: u64`: n(n+1)/2 < 2^127 always fits in a `u128`.
pub fn sum_first_n(n: u64) -> u128 {
    let n = n as u128;
    // n and n+1 are consecutive, so exactly one of them is even: the
    // division below is exact whichever way we pair it.
    n * (n + 1) / 2
}

/// 1^2 + 2^2 + ... + n^2 = n(n+1)(2n+1)/6  (TAOCP Vol. 1, §1.2.3, derived
/// there by the perturbation method).
///
/// Exact whenever the result fits in a `u128` (any n up to about 5.5e12,
/// far beyond what the course needs).
pub fn sum_squares(n: u64) -> u128 {
    let n = n as u128;
    // n(n+1)/2 is an integer, and multiplying it by (2n+1) gives three
    // times the answer: (2n+1) is coprime to 2, and among any three
    // consecutive residues one of n, n+1, 2n+1 supplies the factor 3.
    let t = n * (n + 1) / 2;
    t * (2 * n + 1) / 3
}

/// 1^3 + 2^3 + ... + n^3 = (n(n+1)/2)^2 — Nicomachus's theorem: the sum of
/// the first n cubes is the square of the n-th triangular number.
///
/// Exact whenever the result fits in a `u128` (any n below 2^32 or so).
pub fn sum_cubes(n: u64) -> u128 {
    let t = sum_first_n(n);
    t * t
}

/// The geometric sum 1 + x + x^2 + ... + x^n  (n+1 terms, k running 0..=n).
///
/// §1.2.3's perturbation method derives the closed form: writing
/// S = sum of x^k for 0 <= k <= n and "peeling" a term off each end,
///
/// ```text
///     S + x^(n+1)  =  1 + x*S      =>      S = (x^(n+1) - 1) / (x - 1)
/// ```
///
/// valid for x != 1; for x = 1 every term is 1 and S = n + 1. The division
/// is exact in the integers. The caller must keep x^(n+1) within `i128`
/// (overflow panics in debug builds).
pub fn geometric_sum(x: i128, n: u32) -> i128 {
    if x == 1 {
        return n as i128 + 1;
    }
    (x.pow(n + 1) - 1) / (x - 1)
}

// ---------------------------------------------------------------------------
// §1.2.6 — Binomial coefficients
// ---------------------------------------------------------------------------

/// The binomial coefficient C(n, k) = n! / (k! (n-k)!), computed exactly.
/// Returns 0 when k > n (there are no k-subsets of an n-set then).
///
/// Uses the multiplicative method of §1.2.6, Eq. (3):
///
/// ```text
///     C(n, k) = (n / 1) * ((n-1) / 2) * ... * ((n-k+1) / k)
/// ```
///
/// evaluated left to right as  c <- c * (n - k + i) / i  for i = 1..=k.
/// After the i-th step c = C(n-k+i, i), an integer, so every division is
/// exact — no rational arithmetic needed. Symmetry C(n,k) = C(n,n-k) keeps
/// k small. Exact with no overflow for all n <= 100 (and well beyond):
/// the largest intermediate is C(100,50) * 100 < 2^104.
pub fn binomial(n: u32, k: u32) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k); // symmetry condition, §1.2.6 Eq. (6)
    let n = n as u128;
    let k = k as u128;
    let mut c: u128 = 1;
    for i in 1..=k {
        // c was C(n-k+i-1, i-1); after this step c = C(n-k+i, i). The
        // multiplication happens before the (exact) division.
        c = c * (n - k + i) / i;
    }
    c
}

// ---------------------------------------------------------------------------
// §1.2.8 — Fibonacci numbers
// ---------------------------------------------------------------------------

/// The Fibonacci number F_n, with F_0 = 0, F_1 = 1, F_{n+1} = F_n + F_{n-1}
/// (TAOCP Vol. 1, §1.2.8, Eq. (1) — Knuth's indexing).
///
/// Computed by the defining iteration in O(n) additions. Panics for
/// n > 186: F_186 ≈ 3.33e38 is the largest Fibonacci number that fits in a
/// `u128`.
pub fn fibonacci(n: u32) -> u128 {
    assert!(n <= 186, "fibonacci: F_n overflows u128 for n > 186");
    if n == 0 {
        return 0;
    }
    // Invariant entering iteration i (1-based): (a, b) = (F_{i-1}, F_i).
    let (mut a, mut b) = (0u128, 1u128); // (F_0, F_1)
    for _ in 1..n {
        let t = a + b;
        a = b;
        b = t;
    }
    b
}

// ---------------------------------------------------------------------------
// §1.2.7 — Harmonic numbers
// ---------------------------------------------------------------------------

/// The harmonic number H_n = 1 + 1/2 + ... + 1/n as an exact reduced
/// fraction `(numerator, denominator)` with gcd(numerator, denominator) = 1.
///
/// Panics for n = 0 (H_0 = 0 is a sensible convention but Knuth's tables
/// start at H_1, and the course keeps the domain crisp). Exact for all
/// n <= 60 and beyond; the required range in this module is n <= 30, where
/// the reduced denominator of H_30 is 2329089562800.
pub fn harmonic(n: u32) -> (u128, u128) {
    assert!(n >= 1, "harmonic: H_n requires n >= 1");
    let (mut num, mut den) = (0u128, 1u128);
    for k in 1..=n as u128 {
        // num/den + 1/k = (num*k + den) / (den*k), then reduce by the gcd
        // so the intermediate values stay small.
        num = num * k + den;
        den *= k;
        let g = gcd_u128(num, den);
        num /= g;
        den /= g;
    }
    (num, den)
}

/// H_n in floating point. Summed from the smallest term up so the rounding
/// error stays a few ulps even for n in the millions.
pub fn harmonic_f64(n: u64) -> f64 {
    let mut h = 0.0f64;
    for k in (1..=n).rev() {
        h += 1.0 / k as f64;
    }
    h
}

fn gcd_u128(mut m: u128, mut n: u128) -> u128 {
    // Algorithm 1.1E again, one flight down.
    while n != 0 {
        let r = m % n;
        m = n;
        n = r;
    }
    m
}

// ---------------------------------------------------------------------------
// §1.2.10 — Algorithm M: finding the maximum
// ---------------------------------------------------------------------------

/// Algorithm 1.2.10M (Find the maximum), step-faithful.
///
/// Given n elements X[1], ..., X[n] with n >= 1, find m and j such that
/// m = X[j] = max_{1<=i<=n} X[i], where j is the *largest* such index —
/// Knuth scans from the right end and only replaces the current maximum on
/// a strict increase, so ties keep the rightmost occurrence.
///
/// Returns `(j, m, a)` where `j` is the 0-based index of the maximum and
/// `a` is A, the number of times step M4 (change m) executed. A is the
/// quantity whose distribution §1.2.10 analyzes: E[A] = H_n − 1 on a random
/// permutation of distinct values.
///
/// Panics on an empty slice — Algorithm M requires n >= 1.
pub fn find_max_counting(xs: &[i64]) -> (usize, i64, u64) {
    assert!(!xs.is_empty(), "Algorithm M requires n >= 1");
    let n = xs.len();
    // M1. [Initialize.] Set j <- n, k <- n - 1, m <- X[n].
    //     (Knuth's indices are 1-based; X[k] here is xs[k - 1].)
    let mut j = n; // 1-based for the loop; converted on return
    let mut k = n - 1;
    let mut m = xs[n - 1];
    let mut a = 0u64; // A = number of executions of step M4
    loop {
        // M2. [All tested?] If k = 0, the algorithm terminates.
        if k == 0 {
            return (j - 1, m, a);
        }
        // M3. [Compare.] If X[k] <= m, go to step M5.
        if xs[k - 1] > m {
            // M4. [Change m.] Set j <- k, m <- X[k].
            //     (m is the new current maximum.)
            j = k;
            m = xs[k - 1];
            a += 1;
        }
        // M5. [Decrease k.] Decrease k by one, return to step M2.
        k -= 1;
    }
}

/// Algorithm 1.2.10M without the change-count: returns `(j, m)` where
/// `xs[j] = m` is the maximum and `j` is the largest index attaining it.
/// Panics on an empty slice.
pub fn find_max(xs: &[i64]) -> (usize, i64) {
    let (j, m, _a) = find_max_counting(xs);
    (j, m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauss_and_friends() {
        // The schoolroom example: 1 + 2 + ... + 100 = 5050.
        assert_eq!(sum_first_n(100), 5050);
        assert_eq!(sum_squares(4), 30); // 1 + 4 + 9 + 16
        assert_eq!(sum_cubes(3), 36); // 1 + 8 + 27 = 6^2
        for n in 0..=200u64 {
            let (mut s1, mut s2, mut s3) = (0u128, 0u128, 0u128);
            for k in 1..=n as u128 {
                s1 += k;
                s2 += k * k;
                s3 += k * k * k;
            }
            assert_eq!(sum_first_n(n), s1);
            assert_eq!(sum_squares(n), s2);
            assert_eq!(sum_cubes(n), s3);
            assert_eq!(sum_cubes(n), sum_first_n(n) * sum_first_n(n));
        }
    }

    #[test]
    fn geometric_closed_form() {
        // 1 + 2 + 4 + ... + 2^n = 2^(n+1) - 1.
        assert_eq!(geometric_sum(2, 5), 63);
        assert_eq!(geometric_sum(1, 9), 10);
        assert_eq!(geometric_sum(-1, 4), 1);
        assert_eq!(geometric_sum(-1, 5), 0);
        for x in -6i128..=6 {
            for n in 0..=15u32 {
                let mut s = 0i128;
                let mut p = 1i128;
                for _ in 0..=n {
                    s += p;
                    p *= x;
                }
                assert_eq!(geometric_sum(x, n), s, "x={x}, n={n}");
            }
        }
    }

    #[test]
    fn pascal_triangle_and_the_big_one() {
        // Row 4 of Pascal's triangle (§1.2.6, Table 1): 1 4 6 4 1.
        let row4: Vec<u128> = (0..=4).map(|k| binomial(4, k)).collect();
        assert_eq!(row4, [1, 4, 6, 4, 1]);
        assert_eq!(binomial(0, 0), 1);
        assert_eq!(binomial(5, 7), 0);
        // The stage-2 anchor value.
        assert_eq!(binomial(100, 50), 100891344545564193334812497256);
        // Pascal's rule on a grid.
        for n in 1..=60u32 {
            for k in 0..=n {
                let lhs = binomial(n, k);
                let rhs = if k == 0 {
                    1
                } else {
                    binomial(n - 1, k - 1) + binomial(n - 1, k)
                };
                assert_eq!(lhs, rhs, "C({n},{k})");
            }
        }
    }

    #[test]
    fn fibonacci_table_and_extremes() {
        // §1.2.8's table: F_0..F_15.
        let expect = [0u128, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610];
        for (n, &f) in expect.iter().enumerate() {
            assert_eq!(fibonacci(n as u32), f);
        }
        assert_eq!(fibonacci(100), 354224848179261915075);
        // The largest representable one exists and satisfies the recurrence.
        assert_eq!(fibonacci(186), fibonacci(185) + fibonacci(184));
    }

    #[test]
    #[should_panic(expected = "overflows")]
    fn fibonacci_beyond_u128_is_rejected() {
        fibonacci(187);
    }

    #[test]
    fn harmonic_exact_values() {
        // §1.2.7, Table: H_1 = 1, H_2 = 3/2, ..., H_4 = 25/12, H_5 = 137/60.
        assert_eq!(harmonic(1), (1, 1));
        assert_eq!(harmonic(2), (3, 2));
        assert_eq!(harmonic(3), (11, 6));
        assert_eq!(harmonic(4), (25, 12));
        assert_eq!(harmonic(5), (137, 60));
        assert_eq!(harmonic(6), (49, 20));
        for n in 1..=40u32 {
            let (p, q) = harmonic(n);
            assert_eq!(gcd_u128(p, q), 1, "H_{n} not reduced");
            let approx = p as f64 / q as f64;
            assert!((approx - harmonic_f64(n as u64)).abs() < 1e-12);
        }
    }

    #[test]
    fn algorithm_m_trace() {
        // Hand-traced in the lesson: X = (7, 2, 9, 4, 8, 3).
        // m runs 3 -> 8 -> 9, so A = 2, and the maximum 9 sits at index 2.
        assert_eq!(find_max_counting(&[7, 2, 9, 4, 8, 3]), (2, 9, 2));
        assert_eq!(find_max(&[7, 2, 9, 4, 8, 3]), (2, 9));
        // Ties keep the rightmost occurrence (j as large as possible).
        assert_eq!(find_max(&[5, 5, 5]), (2, 5));
        // Increasing input: the initial m is already the maximum, A = 0.
        assert_eq!(find_max_counting(&[1, 2, 3, 4]), (3, 4, 0));
        // Decreasing input: every comparison changes m, A = n - 1.
        assert_eq!(find_max_counting(&[4, 3, 2, 1]), (0, 4, 3));
        assert_eq!(find_max_counting(&[42]), (0, 42, 0));
    }
}
