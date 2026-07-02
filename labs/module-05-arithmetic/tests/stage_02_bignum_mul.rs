//! Stage 2 — Classical multiplication (Algorithm 4.3.1M).
//!
//! Implement `big_mul` and `big_to_decimal` in src/lab.rs.
//! Lesson: course/module-05-arithmetic/README.md.

use lab_05_arithmetic::{big_add, big_from_u128, big_mul, big_to_decimal, big_to_u128};

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

fn is_canonical(u: &[u32]) -> bool {
    u.last() != Some(&0)
}

#[test]
fn zero_and_one() {
    // 0 * x = 0 in canonical form (empty vector), even for x = 0.
    assert_eq!(big_mul(&[], &[]), Vec::<u32>::new());
    assert_eq!(big_mul(&[], &[1, 2, 3]), Vec::<u32>::new());
    assert_eq!(big_mul(&[1, 2, 3], &[]), Vec::<u32>::new());
    // 1 * x = x.
    assert_eq!(big_mul(&[1], &[9, 8, 7]), vec![9, 8, 7]);
    assert_eq!(big_mul(&[9, 8, 7], &[1]), vec![9, 8, 7]);
}

#[test]
fn single_limb_carry_invariant() {
    // (b-1)^2 = b^2 - 2b + 1 = [1, b-2]: the biggest single-digit product.
    // Step M4's bound t <= (b-1)^2 + (b-1) + (b-1) = b^2 - 1 is tight here.
    assert_eq!(big_mul(&[u32::MAX], &[u32::MAX]), vec![1, 0xFFFF_FFFE]);
    // (2^64 - 1)^2 = 2^128 - 2^65 + 1.
    let m = vec![u32::MAX, u32::MAX];
    assert_eq!(
        big_to_u128(&big_mul(&m, &m)),
        Some(u64::MAX as u128 * u64::MAX as u128)
    );
}

#[test]
fn agrees_with_u128() {
    let mut s = 314159u64;
    for _ in 0..500 {
        let a = lcg(&mut s);
        let b = lcg(&mut s);
        let w = big_mul(&big_from_u128(a as u128), &big_from_u128(b as u128));
        assert!(is_canonical(&w));
        assert_eq!(big_to_u128(&w), Some(a as u128 * b as u128), "{a} * {b}");
    }
}

#[test]
fn distributivity_on_big_inputs() {
    // a*(b+c) == a*b + a*c on multi-hundred-limb operands: exercises long
    // carry chains that u128-sized tests cannot reach.
    let mut s = 60902u64;
    for round in 0..4 {
        let a = rand_big(&mut s, 300);
        let b = rand_big(&mut s, 220);
        let c = rand_big(&mut s, 180);
        let lhs = big_mul(&a, &big_add(&b, &c));
        let rhs = big_add(&big_mul(&a, &b), &big_mul(&a, &c));
        assert_eq!(lhs, rhs, "distributivity, round {round}");
        assert!(is_canonical(&lhs));
        // Commutativity too.
        assert_eq!(big_mul(&a, &b), big_mul(&b, &a), "commutativity, round {round}");
    }
}

#[test]
fn decimal_conversion_basics() {
    assert_eq!(big_to_decimal(&[]), "0");
    assert_eq!(big_to_decimal(&[1]), "1");
    assert_eq!(big_to_decimal(&[999_999_999]), "999999999");
    // 2^32 = 4294967296; a value whose middle decimal chunk needs padding.
    assert_eq!(big_to_decimal(&[0, 1]), "4294967296");
    // 10^18 = [0x89E80000, 0x0DE0B6B3] in base 2^32: inner zeros must be
    // preserved by the 9-digit chunk padding.
    assert_eq!(
        big_to_decimal(&big_from_u128(1_000_000_000_000_000_000)),
        "1000000000000000000"
    );
    assert_eq!(big_to_decimal(&big_from_u128(u128::MAX)), u128::MAX.to_string());
    let mut s = 8u64;
    for _ in 0..100 {
        let x = (lcg(&mut s) as u128) << 64 | lcg(&mut s) as u128;
        assert_eq!(big_to_decimal(&big_from_u128(x)), x.to_string());
    }
}

#[test]
fn factorial_50_exact() {
    // 50! by repeated single-limb multiplication — a value far beyond any
    // primitive integer type, checked digit for digit.
    let mut f = vec![1u32];
    for i in 2..=50u32 {
        f = big_mul(&f, &[i]);
    }
    assert_eq!(
        big_to_decimal(&f),
        "30414093201713378043612608166064768844377641568960512000000000000"
    );
    // Sanity: 50! ends in exactly 12 zeros (floor(50/5) + floor(50/25)).
    assert!(!big_to_decimal(&f).ends_with(&"0".repeat(13)));
}
