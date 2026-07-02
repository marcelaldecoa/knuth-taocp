//! Stage 5 — Probabilistic primality testing (§4.5.4, Miller–Rabin lineage).
//!
//! Implement `mul_mod`, `pow_mod`, `is_strong_probable_prime`, `is_prime_u64`
//! in src/lab.rs. Lesson: course/module-05-arithmetic/README.md.

use lab_05_arithmetic::{is_prime_u64, is_strong_probable_prime, mul_mod, pow_mod};

/// Sieve of Eratosthenes: the ground truth below `limit`.
fn sieve(limit: usize) -> Vec<bool> {
    let mut is_p = vec![true; limit];
    is_p[0] = false;
    if limit > 1 {
        is_p[1] = false;
    }
    let mut i = 2;
    while i * i < limit {
        if is_p[i] {
            let mut j = i * i;
            while j < limit {
                is_p[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    is_p
}

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn mul_mod_is_exact_at_the_top_of_u64() {
    // Naive (a * b) % m overflows for these; the u128 route must not.
    let m = u64::MAX;
    // a = -1, b = -2 (mod 2^64 - 1) => product = 2.
    assert_eq!(mul_mod(m - 1, m - 2, m), 2);
    let mut s = 17u64;
    for _ in 0..500 {
        let a = lcg(&mut s);
        let b = lcg(&mut s);
        let md = lcg(&mut s) | 1; // nonzero
        let expected = ((a as u128 * b as u128) % md as u128) as u64;
        assert_eq!(mul_mod(a, b, md), expected, "{a} * {b} mod {md}");
    }
    assert_eq!(mul_mod(123, 456, 1), 0);
}

#[test]
fn pow_mod_matches_naive_and_fermat() {
    // Small cases against naive repeated multiplication.
    for a in 0..12u64 {
        for e in 0..10u64 {
            for m in 1..15u64 {
                let mut naive = 1 % m;
                for _ in 0..e {
                    naive = naive * a % m;
                }
                assert_eq!(pow_mod(a, e, m), naive, "{a}^{e} mod {m}");
            }
        }
    }
    assert_eq!(pow_mod(0, 0, 7), 1, "0^0 = 1 by convention");
    assert_eq!(pow_mod(5, 0, 1), 0, "anything mod 1 is 0");
    // Fermat's little theorem: a^(p-1) = 1 (mod p) for p prime, p ∤ a.
    let p = (1u64 << 61) - 1;
    for a in [2u64, 3, 12345, u64::MAX % p] {
        assert_eq!(pow_mod(a, p - 1, p), 1, "Fermat at a = {a}");
    }
    // Large exponent sanity: 2^(2^62) mod 1000000007.
    assert_eq!(pow_mod(2, 1 << 62, 1_000_000_007), pow_mod(4, 1 << 61, 1_000_000_007));
}

#[test]
fn strong_test_never_rejects_a_prime() {
    // Every odd prime passes the strong test to *every* base — that is the
    // theorem the test rests on. Check all odd primes below 2000 against
    // several bases, including bases >= n (which reduce mod n).
    let is_p = sieve(2000);
    for n in (3..2000u64).step_by(2) {
        if is_p[n as usize] {
            for a in [2u64, 3, 5, 7, 31, 1_000_003] {
                assert!(
                    is_strong_probable_prime(n, a),
                    "prime {n} rejected by base {a}"
                );
            }
        }
    }
}

#[test]
fn strong_pseudoprime_2047_fools_base_2_only() {
    // 2047 = 23 * 89 is the *smallest* strong pseudoprime to base 2: the
    // witness test to base 2 passes, yet the number is composite — one
    // witness is evidence, not proof.
    assert!(is_strong_probable_prime(2047, 2));
    assert!(!is_prime_u64(2047));
    // Base 3 exposes it.
    assert!(!is_strong_probable_prime(2047, 3));
    // Two more classics from the base-2 strong pseudoprime list.
    assert!(is_strong_probable_prime(3277, 2) && !is_prime_u64(3277)); // 29*113
    assert!(is_strong_probable_prime(4033, 2) && !is_prime_u64(4033)); // 37*109
}

#[test]
fn carmichael_561_is_caught() {
    // 561 = 3 * 11 * 17 is the smallest Carmichael number: it fools the
    // plain Fermat test for EVERY base coprime to it...
    assert_eq!(pow_mod(2, 560, 561), 1);
    assert_eq!(pow_mod(13, 560, 561), 1);
    // ...but the strong test finds a square root of 1 other than +-1 and
    // convicts it, already at base 2.
    assert!(!is_strong_probable_prime(561, 2));
    assert!(!is_prime_u64(561));
    // Larger Carmichael numbers fall the same way.
    for c in [1105u64, 1729, 2465, 6601, 8911] {
        assert_eq!(pow_mod(2, c - 1, c), 1, "{c} fools Fermat base 2");
        assert!(!is_prime_u64(c), "{c} is composite");
    }
}

#[test]
fn agrees_with_a_sieve_below_ten_thousand() {
    let is_p = sieve(10_000);
    for n in 0..10_000u64 {
        assert_eq!(
            is_prime_u64(n),
            is_p[n as usize],
            "is_prime_u64({n}) disagrees with the sieve"
        );
    }
}

#[test]
fn edge_cases() {
    assert!(!is_prime_u64(0));
    assert!(!is_prime_u64(1));
    assert!(is_prime_u64(2));
    assert!(is_prime_u64(3));
    assert!(!is_prime_u64(4));
    // The witnesses themselves are prime; their products are not.
    for w in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        assert!(is_prime_u64(w));
        assert!(!is_prime_u64(w * w));
    }
}

#[test]
fn big_primes_and_composites() {
    // 2^61 - 1 is the ninth Mersenne prime.
    assert!(is_prime_u64((1u64 << 61) - 1));
    // 2^61 + 1 = 3 * 768614336404564651 is composite.
    assert!(!is_prime_u64((1u64 << 61) + 1));
    assert!(is_prime_u64(768614336404564651), "the large cofactor is prime");
    // The largest u64 primes and their neighborhood.
    assert!(is_prime_u64(18_446_744_073_709_551_557)); // largest prime < 2^64
    assert!(!is_prime_u64(u64::MAX)); // 3 * 5 * 17 * 257 * 641 * 65537 * 6700417
    // A semiprime of two 31-bit primes: no witness may be fooled.
    assert!(!is_prime_u64(2_147_483_647 * 2_147_483_629));
    assert!(is_prime_u64(2_147_483_647)); // 2^31 - 1, Mersenne
}
