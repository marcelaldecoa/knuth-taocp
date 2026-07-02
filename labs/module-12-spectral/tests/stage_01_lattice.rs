//! Stage 1 — The lattice structure of linear congruential sequences.
//!
//! Overlapping t-tuples of x_{n+1} = a·x_n mod m are not "random dust":
//! every one of them satisfies u·x ≡ 0 (mod m) for each dual vector u,
//! i.e. u with u1 + a·u2 + ... + a^{t-1}·ut ≡ 0 (mod m). Implement
//! `tuples` and `is_dual_vector` in src/lab.rs; the lesson proves the
//! two-line theorem these tests exercise.

use lab_12_spectral::{is_dual_vector, tuples};

const RANDU_A: i64 = 65539; // 2^16 + 3
const RANDU_M: i64 = 1 << 31;

#[test]
fn tuples_follow_the_recurrence_and_overlap() {
    // x_{n+1} = 137·x_n mod 256 from x_0 = 1.
    let ts = tuples(137, 256, 3, 1, 40);
    assert_eq!(ts.len(), 40);
    for w in &ts {
        assert_eq!(w.len(), 3);
        for &x in w {
            assert!((0..256).contains(&x), "coordinates must lie in [0, m)");
        }
        assert_eq!(w[1], w[0] * 137 % 256, "second coordinate = a·first mod m");
        assert_eq!(w[2], w[1] * 137 % 256);
    }
    // Overlapping windows: tuple i+1 starts where tuple i's tail begins.
    for i in 0..ts.len() - 1 {
        assert_eq!(&ts[i][1..], &ts[i + 1][..2], "tuples must overlap by t-1");
    }
    // The first tuple starts at the seed itself.
    assert_eq!(ts[0][0], 1);
    assert_eq!(ts[0], vec![1, 137, 81]); // 137^2 = 18769 ≡ 81 (mod 256)
}

#[test]
fn randu_famous_dual_vector() {
    // §3.3.4: a = 65539 = 2^16 + 3 with m = 2^31 gives
    //   a^2 = 2^32 + 6·2^16 + 9 ≡ 6a - 9 (mod 2^31),
    // so 9 - 6a + a^2 ≡ 0 (mod 2^31): u = (9, -6, 1) is a dual vector.
    assert!(is_dual_vector(&[9, -6, 1], RANDU_A, RANDU_M));
}

#[test]
fn every_randu_triple_satisfies_the_planes_equation() {
    // The theorem in action: ALL consecutive triples of RANDU obey
    // 9·x_n - 6·x_{n+1} + x_{n+2} ≡ 0 (mod 2^31) — the entire output
    // lies on at most 9 + 6 + 1 = 16 parallel planes (15 hit the cube).
    for seed in [1i64, 7, 12345, 654321, 2_000_000_001] {
        let seed = seed % RANDU_M;
        for tup in tuples(RANDU_A, RANDU_M, 3, seed, 500) {
            let s = 9 * tup[0] as i128 - 6 * tup[1] as i128 + tup[2] as i128;
            assert_eq!(
                s.rem_euclid(RANDU_M as i128),
                0,
                "triple {tup:?} escaped RANDU's planes"
            );
        }
    }
}

#[test]
fn toy_lcg_tuples_satisfy_claimed_dual_relations() {
    // a = 137, m = 256. Claimed dual vectors (verify the congruences by
    // hand once: -7 + 137·15 = 2048 = 8·256; 2 + 137·5 + 81·1 = 768 = 3·256,
    // using 137^2 ≡ 81 mod 256).
    assert!(is_dual_vector(&[-7, 15], 137, 256));
    assert!(is_dual_vector(&[2, 5, 1], 137, 256));
    for tup in tuples(137, 256, 2, 1, 256) {
        let s = -7 * tup[0] as i128 + 15 * tup[1] as i128;
        assert_eq!(s.rem_euclid(256), 0);
    }
    for tup in tuples(137, 256, 3, 99, 256) {
        let s = 2 * tup[0] as i128 + 5 * tup[1] as i128 + tup[2] as i128;
        assert_eq!(s.rem_euclid(256), 0);
    }
}

#[test]
fn scaled_dual_vectors_stay_dual() {
    // The dual vectors form a lattice: integer multiples stay dual.
    assert!(is_dual_vector(&[18, -12, 2], RANDU_A, RANDU_M));
    assert!(is_dual_vector(&[-9, 6, -1], RANDU_A, RANDU_M));
    // ... and so does m·e_1 in any dimension.
    assert!(is_dual_vector(&[256, 0], 137, 256));
    assert!(is_dual_vector(&[0, 256, 0], 137, 256));
}

#[test]
fn non_dual_vectors_are_rejected() {
    assert!(!is_dual_vector(&[9, -6, 2], RANDU_A, RANDU_M));
    assert!(!is_dual_vector(&[9, 6, 1], RANDU_A, RANDU_M));
    assert!(!is_dual_vector(&[1, 0, 0], RANDU_A, RANDU_M));
    assert!(!is_dual_vector(&[-7, 14], 137, 256));
    assert!(!is_dual_vector(&[1], 137, 256));
    // The zero vector satisfies the congruence but is NOT a dual vector.
    assert!(!is_dual_vector(&[0, 0, 0], RANDU_A, RANDU_M));
    assert!(!is_dual_vector(&[0, 0], 137, 256));
}

#[test]
fn exhaustive_duality_check_on_a_tiny_generator() {
    // a = 5, m = 8 (a^2 = 25 ≡ 1): u = (u1, u2, u3) is dual iff
    // u1 + 5·u2 + u3 ≡ 0 (mod 8). Cross-check is_dual_vector against the
    // definition on the whole cube [-8, 8]^3, and check the theorem: every
    // dual u annihilates every generated triple mod m.
    let ts = tuples(5, 8, 3, 1, 16);
    for u1 in -8i64..=8 {
        for u2 in -8i64..=8 {
            for u3 in -8i64..=8 {
                let dual = is_dual_vector(&[u1, u2, u3], 5, 8);
                let nonzero = (u1, u2, u3) != (0, 0, 0);
                let congruent = (u1 + 5 * u2 + u3).rem_euclid(8) == 0;
                assert_eq!(dual, nonzero && congruent, "u = ({u1},{u2},{u3})");
                if dual {
                    for tup in &ts {
                        let s = u1 * tup[0] + u2 * tup[1] + u3 * tup[2];
                        assert_eq!(s.rem_euclid(8), 0);
                    }
                }
            }
        }
    }
}
