//! Stage 1 — Multiple-precision addition and subtraction
//! (Algorithms 4.3.1A and 4.3.1S).
//!
//! Implement `big_cmp`, `big_add`, `big_sub`, `big_from_u128`, `big_to_u128`
//! in src/lab.rs. Lesson: course/module-05-arithmetic/README.md.
//!
//! Representation: little-endian base-2^32 limbs, canonical (no trailing
//! zero limbs; the empty vector is zero).

use lab_05_arithmetic::{big_add, big_cmp, big_from_u128, big_sub, big_to_u128};
use std::cmp::Ordering;

/// Deterministic pseudo-random stream (the course's standard LCG).
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn is_canonical(u: &[u32]) -> bool {
    u.last() != Some(&0)
}

#[test]
fn conversions_roundtrip() {
    assert_eq!(big_from_u128(0), Vec::<u32>::new(), "zero is the empty vec");
    assert_eq!(big_from_u128(1), vec![1]);
    assert_eq!(big_from_u128(u32::MAX as u128), vec![u32::MAX]);
    assert_eq!(big_from_u128(1 << 32), vec![0, 1]);
    assert_eq!(big_from_u128(u128::MAX), vec![u32::MAX; 4]);
    let mut s = 5u64;
    for _ in 0..200 {
        let x = (lcg(&mut s) as u128) << 64 | lcg(&mut s) as u128;
        let limbs = big_from_u128(x);
        assert!(is_canonical(&limbs), "big_from_u128({x}) not canonical");
        assert_eq!(big_to_u128(&limbs), Some(x));
    }
}

#[test]
fn to_u128_rejects_oversized_values() {
    // Five canonical limbs = at least 2^128: cannot fit.
    assert_eq!(big_to_u128(&[0, 0, 0, 0, 1]), None);
    assert_eq!(big_to_u128(&[7, 7, 7, 7, 7, 7]), None);
    // Four limbs always fit.
    assert_eq!(big_to_u128(&[u32::MAX; 4]), Some(u128::MAX));
}

#[test]
fn comparison_contract() {
    assert_eq!(big_cmp(&[], &[]), Ordering::Equal);
    assert_eq!(big_cmp(&[], &[1]), Ordering::Less);
    assert_eq!(big_cmp(&[1], &[]), Ordering::Greater);
    // Canonical ⇒ longer is larger, even when its low limbs are small.
    assert_eq!(big_cmp(&[0, 1], &[u32::MAX]), Ordering::Greater);
    // Equal length: most significant limb decides first.
    assert_eq!(big_cmp(&[9, 1], &[0, 2]), Ordering::Less);
    assert_eq!(big_cmp(&[5, 5], &[5, 5]), Ordering::Equal);
    let mut s = 99u64;
    for _ in 0..300 {
        let a = lcg(&mut s) as u128;
        let b = lcg(&mut s) as u128;
        assert_eq!(
            big_cmp(&big_from_u128(a), &big_from_u128(b)),
            a.cmp(&b),
            "cmp({a}, {b})"
        );
    }
}

#[test]
fn carry_chain_ripples_all_the_way() {
    // Adding 1 to [MAX; k] = 2^(32k) - 1 must produce [0, ..., 0, 1]:
    // the carry propagates through every limb (invariant: 0 <= k <= 1).
    for k in 1..=8 {
        let all_max = vec![u32::MAX; k];
        let mut expected = vec![0u32; k];
        expected.push(1);
        assert_eq!(big_add(&all_max, &[1]), expected, "k = {k}");
        assert_eq!(big_add(&[1], &all_max), expected, "k = {k} (swapped)");
    }
}

#[test]
fn addition_agrees_with_u128() {
    let mut s = 42u64;
    for _ in 0..500 {
        // Keep both below 2^127 so the u128 sum cannot overflow.
        let a = ((lcg(&mut s) as u128) << 64 | lcg(&mut s) as u128) >> 1;
        let b = ((lcg(&mut s) as u128) << 64 | lcg(&mut s) as u128) >> 1;
        let w = big_add(&big_from_u128(a), &big_from_u128(b));
        assert!(is_canonical(&w));
        assert_eq!(big_to_u128(&w), Some(a + b), "{a} + {b}");
    }
    // Unequal lengths and identities.
    assert_eq!(big_add(&[], &[]), Vec::<u32>::new());
    assert_eq!(big_add(&[], &[3, 4]), vec![3, 4]);
    assert_eq!(big_add(&[3, 4], &[]), vec![3, 4]);
}

#[test]
fn subtraction_agrees_with_u128() {
    let mut s = 7u64;
    for _ in 0..500 {
        let x = (lcg(&mut s) as u128) << 64 | lcg(&mut s) as u128;
        let y = (lcg(&mut s) as u128) << 64 | lcg(&mut s) as u128;
        let (hi, lo) = if x >= y { (x, y) } else { (y, x) };
        let w = big_sub(&big_from_u128(hi), &big_from_u128(lo));
        assert!(is_canonical(&w));
        assert_eq!(big_to_u128(&w), Some(hi - lo), "{hi} - {lo}");
    }
}

#[test]
fn borrow_propagation() {
    // 2^64 - 1: the borrow ripples through two zero limbs.
    assert_eq!(big_sub(&[0, 0, 1], &[1]), vec![u32::MAX, u32::MAX]);
    // 2^96 - 2^32 = [0, MAX, MAX].
    assert_eq!(big_sub(&[0, 0, 0, 1], &[0, 1]), vec![0, u32::MAX, u32::MAX]);
    // x - x = 0 must come back as the canonical empty vector.
    assert_eq!(big_sub(&[5, 6, 7], &[5, 6, 7]), Vec::<u32>::new());
    // A high-limb cancellation must be trimmed: (b+1) - b = 1.
    assert_eq!(big_sub(&[1, 1], &[0, 1]), vec![1]);
}

#[test]
fn add_then_sub_is_identity() {
    // Property: (a + b) - b == a, on random-length canonical inputs.
    let mut s = 2024u64;
    for _ in 0..200 {
        let la = (lcg(&mut s) % 20) as usize;
        let lb = (lcg(&mut s) % 20) as usize;
        let mut a: Vec<u32> = (0..la).map(|_| lcg(&mut s) as u32).collect();
        let mut b: Vec<u32> = (0..lb).map(|_| lcg(&mut s) as u32).collect();
        while a.last() == Some(&0) {
            a.pop();
        }
        while b.last() == Some(&0) {
            b.pop();
        }
        let sum = big_add(&a, &b);
        assert!(big_cmp(&sum, &a) != Ordering::Less);
        assert_eq!(big_sub(&sum, &b), a, "(a+b)-b, a={a:?} b={b:?}");
        assert_eq!(big_sub(&sum, &a), b, "(a+b)-a, a={a:?} b={b:?}");
        // Commutativity for free.
        assert_eq!(big_add(&b, &a), sum);
    }
}

#[test]
#[should_panic(expected = "nonnegative")]
fn negative_result_is_rejected() {
    // Algorithm S is defined for u >= v only: these functions compute on
    // nonnegative integers, and 1 - 2 is not one.
    big_sub(&[1], &[2]);
}

#[test]
#[should_panic(expected = "nonnegative")]
fn negative_result_is_rejected_multilimb() {
    // Same magnitude of limbs, but v is longer, hence larger.
    big_sub(&[u32::MAX, u32::MAX], &[0, 0, 1]);
}
