//! Stage 2 — Binomial coefficients (§1.2.6).
//!
//! Implement `binomial` in src/lab.rs — exact, no overflow for n <= 100.
//! The lesson: course/module-02-math/README.md, part 3.

use lab_02_math::binomial;

#[test]
fn pascals_triangle_small_rows() {
    // §1.2.6, Table 1 — the first rows of Pascal's triangle.
    assert_eq!(binomial(0, 0), 1);
    let row4: Vec<u128> = (0..=4).map(|k| binomial(4, k)).collect();
    assert_eq!(row4, [1, 4, 6, 4, 1]);
    let row6: Vec<u128> = (0..=6).map(|k| binomial(6, k)).collect();
    assert_eq!(row6, [1, 6, 15, 20, 15, 6, 1]);
}

#[test]
fn out_of_range_is_zero() {
    // No k-subsets of an n-set when k > n: Eq. 1.2.6-(3) gives 0.
    assert_eq!(binomial(5, 6), 0);
    assert_eq!(binomial(0, 1), 0);
    assert_eq!(binomial(100, 101), 0);
}

#[test]
fn pascals_rule() {
    // C(n, k) = C(n-1, k-1) + C(n-1, k)   (§1.2.6, Eq. (9)).
    for n in 1..=80u32 {
        for k in 1..=n {
            assert_eq!(
                binomial(n, k),
                binomial(n - 1, k - 1) + binomial(n - 1, k),
                "Pascal at ({n},{k})"
            );
        }
        assert_eq!(binomial(n, 0), 1);
        assert_eq!(binomial(n, n), 1);
    }
}

#[test]
fn symmetry() {
    // C(n, k) = C(n, n-k)   (§1.2.6, Eq. (6)).
    for n in 0..=100u32 {
        for k in 0..=n {
            assert_eq!(binomial(n, k), binomial(n, n - k), "symmetry at ({n},{k})");
        }
    }
}

#[test]
fn row_sums_are_powers_of_two() {
    // Σ_k C(n, k) = 2^n: set x = y = 1 in the binomial theorem.
    for n in 0..=100u32 {
        let s: u128 = (0..=n).map(|k| binomial(n, k)).sum();
        assert_eq!(s, 1u128 << n, "row {n}");
    }
}

#[test]
fn alternating_row_sums_vanish() {
    // Σ_k (-1)^k C(n, k) = 0 for n >= 1: set x = -1, y = 1.
    for n in 1..=100u32 {
        let mut s = 0i128;
        for k in 0..=n {
            let c = binomial(n, k) as i128;
            s += if k % 2 == 0 { c } else { -c };
        }
        assert_eq!(s, 0, "alternating row {n}");
    }
}

#[test]
fn vandermonde_convolution() {
    // Σ_j C(m, j) C(n, k-j) = C(m+n, k)   (§1.2.6, Eq. (21)):
    // choosing k people from m men and n women, split by how many are men.
    for m in 0..=20u32 {
        for n in 0..=20u32 {
            for k in 0..=(m + n) {
                let mut s = 0u128;
                for j in 0..=k {
                    s += binomial(m, j) * binomial(n, k - j);
                }
                assert_eq!(s, binomial(m + n, k), "Vandermonde at m={m}, n={n}, k={k}");
            }
        }
    }
}

#[test]
fn central_binomial_100_choose_50_exact() {
    // The stage anchor: any floating-point or overflowing method fails here.
    assert_eq!(binomial(100, 50), 100891344545564193334812497256u128);
}

#[test]
fn row_100_matches_addition_only_construction() {
    // Build row 100 purely by Pascal additions in u128 (no division at
    // all), and demand exact agreement entry by entry.
    let mut row = vec![1u128];
    for _ in 0..100 {
        let mut next = vec![1u128];
        for w in row.windows(2) {
            next.push(w[0] + w[1]);
        }
        next.push(1);
        row = next;
    }
    for (k, &want) in row.iter().enumerate() {
        assert_eq!(binomial(100, k as u32), want, "C(100,{k})");
    }
}
