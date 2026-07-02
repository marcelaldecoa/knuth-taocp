//! Stage 3 — Fibonacci numbers (§1.2.8).
//!
//! Implement `fibonacci` in src/lab.rs — exact up to F_186, the largest
//! Fibonacci number that fits in a u128.
//! The lesson: course/module-02-math/README.md, part 5.

use lab_02_math::fibonacci;

fn gcd(mut m: u128, mut n: u128) -> u128 {
    while n != 0 {
        let r = m % n;
        m = n;
        n = r;
    }
    m
}

fn gcd32(mut m: u32, mut n: u32) -> u32 {
    while n != 0 {
        let r = m % n;
        m = n;
        n = r;
    }
    m
}

#[test]
fn table_from_the_text() {
    // §1.2.8's table of F_0 through F_15 (Knuth's indexing: F_0 = 0).
    let expect: [u128; 16] = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610];
    for (n, &f) in expect.iter().enumerate() {
        assert_eq!(fibonacci(n as u32), f, "F_{n}");
    }
}

#[test]
fn f_100_exact() {
    assert_eq!(fibonacci(100), 354224848179261915075u128);
}

#[test]
fn recurrence_holds_over_the_whole_domain() {
    // F_{n+1} = F_n + F_{n-1} for every representable n, checked against an
    // independent additive reconstruction.
    let (mut a, mut b) = (0u128, 1u128); // (F_0, F_1)
    assert_eq!(fibonacci(0), a);
    assert_eq!(fibonacci(1), b);
    for n in 2..=186u32 {
        let f = a + b;
        assert_eq!(fibonacci(n), f, "F_{n}");
        a = b;
        b = f;
    }
}

#[test]
fn addition_law() {
    // F_{m+n} = F_m F_{n+1} + F_{m-1} F_n   (§1.2.8, Eq. (6)).
    for m in 1..=90u32 {
        for n in 0..=90u32 {
            assert_eq!(
                fibonacci(m + n),
                fibonacci(m) * fibonacci(n + 1) + fibonacci(m - 1) * fibonacci(n),
                "m = {m}, n = {n}"
            );
        }
    }
}

#[test]
fn cassini_identity() {
    // F_{n-1} F_{n+1} - F_n^2 = (-1)^n   (§1.2.8, Eq. (8)).
    for n in 1..=90u32 {
        let lhs = fibonacci(n - 1) as i128 * fibonacci(n + 1) as i128
            - (fibonacci(n) as i128) * (fibonacci(n) as i128);
        let rhs = if n % 2 == 0 { 1 } else { -1 };
        assert_eq!(lhs, rhs, "Cassini at n = {n}");
    }
}

#[test]
fn gcd_law() {
    // gcd(F_m, F_n) = F_{gcd(m, n)}   (§1.2.8; exercise 1.2.8-... the
    // Fibonacci numbers form a "strong divisibility sequence").
    for m in 1..=60u32 {
        for n in 1..=60u32 {
            assert_eq!(
                gcd(fibonacci(m), fibonacci(n)),
                fibonacci(gcd32(m, n)),
                "m = {m}, n = {n}"
            );
        }
    }
}

#[test]
fn golden_ratio_growth() {
    // Binet: F_n = (phi^n - phihat^n)/sqrt(5), and since |phihat| < 1,
    // F_n is phi^n/sqrt(5) rounded to the nearest integer (§1.2.8, Eq. (15)).
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let sqrt5 = 5.0f64.sqrt();
    for n in 0..=40u32 {
        let approx = phi.powi(n as i32) / sqrt5;
        assert_eq!(fibonacci(n), approx.round() as u128, "round at n = {n}");
    }
    // Consecutive ratios converge to phi.
    let r = fibonacci(61) as f64 / fibonacci(60) as f64;
    assert!((r - phi).abs() < 1e-12, "F_61/F_60 = {r} should be ~ phi");
    // And the growth really is exponential with base phi: for large n,
    // ln F_n = n ln(phi) - ln sqrt(5) + o(1).
    let f150 = fibonacci(150) as f64;
    let predicted = 150.0 * phi.ln() - sqrt5.ln();
    assert!((f150.ln() - predicted).abs() < 1e-9);
}

#[test]
#[should_panic(expected = "overflows")]
fn refuses_to_overflow_silently() {
    // F_187 > u128::MAX. Definiteness: state your domain and defend it.
    fibonacci(187);
}
