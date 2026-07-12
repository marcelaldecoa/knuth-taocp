//! Module 01 — The Notion of an Algorithm.
//! Source: TAOCP Vol. 1, 3rd ed., §1.1 (and §1.2.1 for the extended algorithm).

/// Algorithm 1.1E (Euclid's algorithm), step-faithful.
///
/// Given two positive integers `m` and `n`, find their greatest common
/// divisor: the largest positive integer that evenly divides both.
///
/// Panics if either input is zero — Algorithm E is stated for positive
/// integers only (definiteness!).
pub fn euclid_e(mut m: u64, mut n: u64) -> u64 {
    assert!(m > 0 && n > 0, "Algorithm E requires positive integers");
    loop {
        // E1. [Find remainder.] Divide m by n and let r be the remainder.
        //     (We will have 0 <= r < n.)
        let r = m % n;

        // E2. [Is it zero?] If r = 0, the algorithm terminates; n is the answer.
        if r == 0 {
            return n;
        }

        // E3. [Reduce.] Set m <- n, n <- r, and go back to step E1.
        m = n;
        n = r;
    }
}

/// Algorithm 1.1F (exercise 1.1-3): Euclid's algorithm rewritten so that the
/// trivial replacement operations `m <- n, n <- r` of step E3 are avoided —
/// the roles of `m` and `n` alternate instead of being swapped.
pub fn euclid_f(mut m: u64, mut n: u64) -> u64 {
    assert!(m > 0 && n > 0, "Algorithm F requires positive integers");
    loop {
        // F1. [Remainder m/n.] Divide m by n; set m to the remainder.
        m %= n;
        // F2. [Is it zero?] If m = 0, terminate with answer n.
        if m == 0 {
            return n;
        }
        // F3. [Remainder n/m.] Divide n by m; set n to the remainder.
        n %= m;
        // F4. [Is it zero?] If n = 0, terminate with answer m.
        if n == 0 {
            return m;
        }
    }
}

/// Algorithm 1.2.1E (Extended Euclid). Returns `(d, a, b)` such that
/// `a*m + b*n = d = gcd(m, n)`.
///
/// The coefficients certify the answer: a claimed gcd can be *checked* by
/// verifying the identity plus divisibility — the same idea that later gives
/// modular inverses (§4.5.2).
pub fn extended_euclid(m: u64, n: u64) -> (u64, i128, i128) {
    assert!(m > 0 && n > 0, "extended Euclid requires positive integers");
    // E1. [Initialize.] Maintain the invariant
    //         a1*m + b1*n = c,   a*m + b*n = d.
    let (mut a1, mut b1): (i128, i128) = (1, 0); // coefficients for c
    let (mut a, mut b): (i128, i128) = (0, 1); //   coefficients for d
    let (mut c, mut d) = (m, n);
    loop {
        // E2. [Divide.] Let q and r be the quotient and remainder of c / d.
        let q = c / d;
        let r = c % d;
        // E3. [Remainder zero?] If r = 0, terminate: a*m + b*n = d = gcd(m, n).
        if r == 0 {
            return (d, a, b);
        }
        // E4. [Recycle.] c <- d, d <- r; update coefficients the same way:
        //     (a1, a) <- (a, a1 - q*a),  (b1, b) <- (b, b1 - q*b).
        c = d;
        d = r;
        let (na, nb) = (a1 - q as i128 * a, b1 - q as i128 * b);
        a1 = a;
        b1 = b;
        a = na;
        b = nb;
    }
}

/// The number of times step E1 (a division) executes when Algorithm E runs
/// on `(m, n)`. Knuth calls this T(m, n) in the analysis of Algorithm E;
/// Lamé's theorem (Vol. 2, §4.5.3) bounds it via the Fibonacci sequence.
pub fn division_steps(mut m: u64, mut n: u64) -> u32 {
    assert!(m > 0 && n > 0, "Algorithm E requires positive integers");
    let mut t = 0;
    loop {
        let r = m % n;
        t += 1;
        if r == 0 {
            return t;
        }
        m = n;
        n = r;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knuth_worked_examples() {
        // §1.1 traces gcd(544, 119) = 17.
        assert_eq!(euclid_e(544, 119), 17);
        // Exercise 1.1-4's numbers: gcd(2166, 6099) = 57.
        assert_eq!(euclid_e(2166, 6099), 57);
        assert_eq!(euclid_e(6099, 2166), 57);
        assert_eq!(euclid_e(1, 1), 1);
        assert_eq!(euclid_e(1, 999), 1);
    }

    #[test]
    fn f_agrees_with_e() {
        for m in 1..=60 {
            for n in 1..=60 {
                assert_eq!(euclid_e(m, n), euclid_f(m, n), "gcd({m},{n})");
            }
        }
    }

    #[test]
    fn extended_certifies() {
        // §1.2.1 illustrates m = 1769, n = 551: 5*1769 - 16*551 = 29.
        let (d, a, b) = extended_euclid(1769, 551);
        assert_eq!(d, 29);
        assert_eq!(a * 1769 + b * 551, 29);
        for m in 1..=40u64 {
            for n in 1..=40u64 {
                let (d, a, b) = extended_euclid(m, n);
                assert_eq!(d, euclid_e(m, n));
                assert_eq!(a * m as i128 + b * n as i128, d as i128);
            }
        }
    }

    #[test]
    fn fibonacci_pairs_are_worst_case() {
        // T(F_{k+1}, F_k) = k - 1: consecutive Fibonacci numbers force the
        // quotient to be 1 at every step, the slowest possible descent.
        let (mut fk, mut fk1) = (1u64, 2u64); // F_2, F_3
        let mut k = 2;
        while fk1 < 100_000 {
            assert_eq!(division_steps(fk1, fk), k - 1, "T(F_{},F_{})", k + 1, k);
            let next = fk + fk1;
            fk = fk1;
            fk1 = next;
            k += 1;
        }
        assert_eq!(division_steps(544, 119), 4);
    }
}
