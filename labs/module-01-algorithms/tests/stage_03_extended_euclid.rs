//! Stage 3 — Extended Euclid: certifying the gcd (Algorithm 1.2.1E).
//!
//! Your (a, b) need not match anyone else's — the tests verify the Bézout
//! identity a*m + b*n = d, which is what makes the output a *certificate*.

use lab_01_algorithms::{euclid_e, extended_euclid};

#[test]
fn knuth_illustration() {
    // §1.2.1 illustrates the method with m = 1769, n = 551, whose gcd is 29
    // (Knuth's run finds 5*1769 - 16*551 = 29).
    let (d, a, b) = extended_euclid(1769, 551);
    assert_eq!(d, 29);
    assert_eq!(a * 1769 + b * 551, 29);
}

#[test]
fn bezout_identity_holds_on_a_grid() {
    for m in 1..=90u64 {
        for n in 1..=90u64 {
            let (d, a, b) = extended_euclid(m, n);
            assert_eq!(d, euclid_e(m, n), "wrong gcd for ({m},{n})");
            assert_eq!(
                a * m as i128 + b * n as i128,
                d as i128,
                "Bézout identity fails for ({m},{n}): {a}*{m} + {b}*{n} != {d}"
            );
        }
    }
}

#[test]
fn coprime_inputs_give_modular_inverses() {
    // When gcd(m, n) = 1, a is m^{-1} modulo n — the door to §4.5.2.
    for &(m, n) in &[(3u64, 7u64), (10, 21), (17, 100), (271, 1000)] {
        let (d, a, _b) = extended_euclid(m, n);
        assert_eq!(d, 1);
        let a_mod_n = a.rem_euclid(n as i128) as u64;
        assert_eq!((a_mod_n as u128 * m as u128) % n as u128, 1, "{m}^-1 mod {n}");
    }
}

#[test]
fn large_inputs() {
    let (m, n) = (2_147_483_647u64, 4_294_967_291u64); // two primes
    let (d, a, b) = extended_euclid(m, n);
    assert_eq!(d, 1);
    assert_eq!(a * m as i128 + b * n as i128, 1);
}
