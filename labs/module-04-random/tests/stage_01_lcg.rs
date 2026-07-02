//! Stage 1 — The linear congruential generator and its period.
//!
//! Implement `Lcg` (with `new`/`next`) and `period` in src/lab.rs.
//! Lesson: course/module-04-random/README.md (§3.2.1, Theorem 3.2.1.2A).

use lab_04_random::{period, Lcg};

// Knuth's 64-bit "MMIX" constants; m = 2^64 is encoded as m = 0.
const MMIX_A: u64 = 6364136223846793005;
const MMIX_C: u64 = 1442695040888963407;

#[test]
fn opening_example_of_chapter_3() {
    // §3.1 opens with X_0 = a = c = 7, m = 10: the sequence cycles
    // 7, 6, 9, 0, 7, 6, 9, 0, ... — period 4, dismal even for m = 10.
    let mut g = Lcg::new(7, 7, 7, 10);
    assert_eq!(
        (0..8).map(|_| g.next()).collect::<Vec<_>>(),
        vec![6, 9, 0, 7, 6, 9, 0, 7]
    );
    assert_eq!(period(7, 7, 10, 7), 4);
}

#[test]
fn mmix_generator_from_seed_zero() {
    // m = 2^64 is encoded m = 0; the update is the wrap of 64-bit arithmetic.
    // From X_0 = 0 the first output is X_1 = c.
    let mut g = Lcg::new(0, MMIX_A, MMIX_C, 0);
    assert_eq!(g.next(), 1442695040888963407); // X_1 = c
    assert_eq!(g.next(), 1876011003808476466); // X_2
    assert_eq!(g.next(), 11166244414315200793); // X_3
}

#[test]
fn new_reduces_parameters_when_m_is_finite() {
    // With m > 0 the seed and both parameters are reduced mod m, so these two
    // constructions must generate identical sequences.
    let mut a = Lcg::new(37, 21, 103, 10);
    let mut b = Lcg::new(7, 1, 3, 10);
    for _ in 0..20 {
        assert_eq!(a.next(), b.next());
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

#[test]
fn theorem_3_2_1_2a_is_an_iff_mod_16() {
    // m = 16 = 2^4. The only prime dividing m is 2 and 4 | m, so Theorem
    // 3.2.1.2A predicts maximum period 16 (for EVERY seed) iff c is odd and
    // a ≡ 1 (mod 4). We verify the full "if and only if": the theorem's
    // predicate matches "period is 16 from every starting value" exactly.
    for a in 0..16u64 {
        for c in 0..16u64 {
            let predicted_full = (c % 2 == 1) && (a % 4 == 1);
            let all_seeds_full = (0..16u64).all(|s| period(a, c, 16, s) == 16);
            assert_eq!(
                predicted_full, all_seeds_full,
                "a={a} c={c}: theorem predicts full={predicted_full}"
            );
        }
    }
}

#[test]
fn theorem_3_2_1_2a_is_an_iff_mod_100() {
    // m = 100 = 2^2 · 5^2. Primes dividing m are 2 and 5, and 4 | m, so the
    // conditions collapse to: gcd(c, 100) = 1, a ≡ 1 (mod 4) and a ≡ 1 (mod 5)
    // — i.e. a ≡ 1 (mod 20). If the period is m it visits all residues, so
    // checking one seed decides fullness for all of them.
    for a in 0..100u64 {
        for c in 0..100u64 {
            let predicted_full = gcd(c, 100) == 1 && a % 4 == 1 && a % 5 == 1;
            let full = period(a, c, 100, 0) == 100;
            assert_eq!(predicted_full, full, "a={a} c={c}");
        }
    }
    // A couple of concrete witnesses, checked from every seed.
    for s in 0..100u64 {
        assert_eq!(period(21, 3, 100, s), 100);
    }
    assert!(period(11, 3, 100, 0) < 100); // 11 ≡ 3 (mod 4): fails
    assert!(period(21, 5, 100, 0) < 100); // gcd(5, 100) = 5: fails
}

#[test]
fn tail_before_the_cycle_is_not_counted() {
    // When gcd(a, m) > 1 the map is not a permutation of Z_m: some states are
    // never revisited (a tail), yet period() reports only the cycle length.
    // §3.1's opener again: 7,7,7 mod 10 has a pure cycle 7->6->9->0->7 of 4.
    assert_eq!(period(7, 7, 10, 7), 4);
    // a = 2, c = 0, m = 16: 1 -> 2 -> 4 -> 8 -> 0 -> 0 ... a tail of length 4
    // leading into the fixed point 0. period is 1, not 5.
    assert_eq!(period(2, 0, 16, 1), 1);
}

#[test]
fn randu_satisfies_the_planes_identity() {
    // The infamous RANDU: a = 65539 = 2^16 + 3, c = 0, m = 2^31. Then
    // a^2 = 2^32 + 6·2^16 + 9 = 2·2^31 + 6·(a − 3) + 9 ≡ 6a − 9 (mod 2^31),
    // so X_{n+2} ≡ 6 X_{n+1} − 9 X_n, i.e. 9 X_n − 6 X_{n+1} + X_{n+2} ≡ 0.
    // Every triple lies on one of just 15 planes in the unit cube.
    let m: i128 = 1 << 31;
    let mut g = Lcg::new(1, 65539, 0, 1 << 31);
    let xs: Vec<i128> = (0..200).map(|_| g.next() as i128).collect();
    assert_eq!(xs[0], 65539); // X_1
    assert_eq!(xs[1], 393225); // X_2
    for w in xs.windows(3) {
        assert_eq!((9 * w[0] - 6 * w[1] + w[2]).rem_euclid(m), 0);
    }
}

#[test]
#[should_panic(expected = "finite modulus")]
fn period_rejects_the_word_size_modulus() {
    // period() does direct cycle detection and needs a small finite m; m = 0
    // (meaning 2^64) is out of bounds.
    period(MMIX_A, MMIX_C, 0, 0);
}
