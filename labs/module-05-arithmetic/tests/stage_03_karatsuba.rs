//! Stage 3 — Faster multiplication by divide and conquer (§4.3.3).
//!
//! Implement `big_mul_karatsuba` in src/lab.rs.
//! Lesson: course/module-05-arithmetic/README.md.

use lab_05_arithmetic::{big_from_u128, big_mul, big_mul_karatsuba, big_to_u128};

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn rand_big(state: &mut u64, len: usize) -> Vec<u32> {
    let mut v: Vec<u32> = (0..len).map(|_| lcg(state) as u32).collect();
    while v.last() == Some(&0) {
        v.pop();
    }
    v
}

#[test]
fn tiny_cases() {
    assert_eq!(big_mul_karatsuba(&[], &[]), Vec::<u32>::new());
    assert_eq!(big_mul_karatsuba(&[], &[5]), Vec::<u32>::new());
    assert_eq!(big_mul_karatsuba(&[5], &[]), Vec::<u32>::new());
    assert_eq!(big_mul_karatsuba(&[1], &[1]), vec![1]);
    assert_eq!(big_mul_karatsuba(&[u32::MAX], &[u32::MAX]), vec![1, 0xFFFF_FFFE]);
    let mut s = 11u64;
    for _ in 0..200 {
        let a = lcg(&mut s);
        let b = lcg(&mut s);
        assert_eq!(
            big_to_u128(&big_mul_karatsuba(
                &big_from_u128(a as u128),
                &big_from_u128(b as u128)
            )),
            Some(a as u128 * b as u128)
        );
    }
}

#[test]
fn agrees_with_classical_across_the_cutoff() {
    // Every length pair from 0x0 up to 68x68 straddles any sensible
    // classical-fallback threshold (the spec allows any cutoff below ~64).
    let mut s = 271828u64;
    for la in (0..=68).step_by(4) {
        for lb in (0..=68).step_by(4) {
            let a = rand_big(&mut s, la);
            let b = rand_big(&mut s, lb);
            assert_eq!(
                big_mul_karatsuba(&a, &b),
                big_mul(&a, &b),
                "sizes {la} x {lb}"
            );
        }
    }
}

#[test]
fn agrees_with_classical_on_unbalanced_operands() {
    // Karatsuba's split leaves one high part empty when operands are very
    // unbalanced — the identity must still hold.
    let mut s = 161803u64;
    for (la, lb) in [
        (1, 300),
        (300, 1),
        (5, 257),
        (33, 400),
        (400, 33),
        (63, 64),
        (65, 129),
        (128, 128),
        (127, 255),
    ] {
        let a = rand_big(&mut s, la);
        let b = rand_big(&mut s, lb);
        assert_eq!(big_mul_karatsuba(&a, &b), big_mul(&a, &b), "sizes {la} x {lb}");
    }
}

#[test]
fn powers_of_two_edge_cases() {
    // 2^(32j + r) as limbs: j zero limbs then a single set bit. Products of
    // powers of two stress the "shift by b^p" and canonical-form logic:
    // exactly one nonzero limb may appear, in exactly the right place.
    for (j1, r1, j2, r2) in [(0, 0, 0, 0), (3, 0, 4, 0), (2, 31, 2, 31), (7, 5, 0, 27), (10, 16, 9, 17)] {
        let mut a = vec![0u32; j1];
        a.push(1u32 << r1);
        let mut b = vec![0u32; j2];
        b.push(1u32 << r2);
        let expect_bit = 32 * (j1 + j2) + r1 + r2;
        let mut expected = vec![0u32; expect_bit / 32];
        expected.push(1u32 << (expect_bit % 32));
        assert_eq!(
            big_mul_karatsuba(&a, &b),
            expected,
            "2^{} * 2^{}",
            32 * j1 + r1,
            32 * j2 + r2
        );
    }
}

#[test]
fn all_ones_carry_storm() {
    // (2^(32k) - 1)^2 = 2^(64k) - 2^(32k+1) + 1 forces maximal carries in
    // the recombination additions.
    for k in [40usize, 100] {
        let a = vec![u32::MAX; k];
        let got = big_mul_karatsuba(&a, &a);
        assert_eq!(got, big_mul(&a, &a), "k = {k}");
        assert_eq!(got.len(), 2 * k);
        assert_eq!(got[0], 1);
        assert_eq!(got[2 * k - 1], u32::MAX);
    }
}

#[test]
fn two_thousand_limb_multiply_completes_and_matches() {
    // Soft performance sanity: ~64,000-bit operands. Karatsuba does about
    // 3^11 ≈ 177k small products where classical does 4M limb products;
    // both finish comfortably, and they must agree limb for limb.
    let mut s = 141421u64;
    let a = rand_big(&mut s, 2000);
    let b = rand_big(&mut s, 2000);
    let fast = big_mul_karatsuba(&a, &b);
    let classical = big_mul(&a, &b);
    assert_eq!(fast.len(), classical.len());
    assert_eq!(fast, classical);
    // Squaring consistency: (a*b) computed once must equal b*a.
    assert_eq!(big_mul_karatsuba(&b, &a), fast);
}
