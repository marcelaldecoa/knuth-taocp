//! Stage 1 — Ruler function, sideways addition, Gosper's hack (§7.1.3).
//!
//! Implement `ruler`, `sideways_addition`, `extract_rightmost_one`,
//! `smear_right`, and `next_same_weight` in src/lab.rs — *without* the std
//! intrinsics (`trailing_zeros`, `count_ones`, …). These tests use the
//! intrinsics as oracles; your implementations must agree with them.
//! The lesson: course/module-13-bits-bdds/README.md.

use lab_13_bits_bdds::{
    extract_rightmost_one, next_same_weight, ruler, sideways_addition, smear_right,
};

/// Deterministic pseudo-random words (Knuth-style LCG, MMIX constants).
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn ruler_agrees_with_std_on_all_16_bit_words_everywhere() {
    // Every nonzero 16-bit pattern, planted at each quarter of the word.
    for w in 1..=0xFFFFu64 {
        for shift in [0, 16, 32, 48] {
            let x = w << shift;
            assert_eq!(ruler(x), x.trailing_zeros(), "rho({x:#x})");
        }
    }
}

#[test]
fn sideways_addition_agrees_with_std_on_all_16_bit_words_everywhere() {
    for w in 0..=0xFFFFu64 {
        for shift in [0, 16, 32, 48] {
            let x = w << shift;
            assert_eq!(sideways_addition(x), x.count_ones(), "nu({x:#x})");
        }
        // Two copies of the pattern at once (exercises inter-byte sums).
        let x = w | (w << 48);
        assert_eq!(sideways_addition(x), x.count_ones(), "nu({x:#x})");
    }
}

#[test]
fn agreement_with_std_on_lcg_samples() {
    let mut s = 20260702u64;
    for _ in 0..20_000 {
        let x = lcg(&mut s);
        assert_eq!(sideways_addition(x), x.count_ones(), "nu({x:#x})");
        if x != 0 {
            assert_eq!(ruler(x), x.trailing_zeros(), "rho({x:#x})");
            assert_eq!(extract_rightmost_one(x), 1u64 << x.trailing_zeros());
        }
        assert_eq!(smear_right(x), x | x.wrapping_sub(1), "smear({x:#x})");
    }
}

#[test]
fn powers_of_two_and_extremes() {
    for a in 0..64 {
        let p = 1u64 << a;
        assert_eq!(ruler(p), a, "rho(2^{a})");
        assert_eq!(sideways_addition(p), 1);
        assert_eq!(extract_rightmost_one(p), p);
        // 2^a - 1 has exactly a one-bits.
        assert_eq!(sideways_addition(p - 1), a);
        // A solid run of ones starting at bit a.
        assert_eq!(ruler(u64::MAX << a), a);
    }
    assert_eq!(ruler(u64::MAX), 0);
    assert_eq!(sideways_addition(u64::MAX), 64);
    assert_eq!(sideways_addition(0), 0);
    assert_eq!(ruler(1u64 << 63), 63);
}

#[test]
fn extract_rightmost_one_hand_examples() {
    // The lesson's running example: x = 01011000 -> 00001000.
    assert_eq!(extract_rightmost_one(0b0101_1000), 0b0000_1000);
    assert_eq!(extract_rightmost_one(0), 0);
    assert_eq!(extract_rightmost_one(1), 1);
    assert_eq!(extract_rightmost_one(u64::MAX), 1);
    // The result is always a power of two dividing x (for x != 0).
    let mut s = 7u64;
    for _ in 0..1000 {
        let x = lcg(&mut s) | 1 << 40; // guarantee nonzero
        let b = extract_rightmost_one(x);
        assert!(b.is_power_of_two() && x % b == 0 && (x & (b - 1)) == 0);
    }
}

#[test]
fn smear_right_hand_examples() {
    // x = y 1 0^a  ->  y 1 1^a: fills everything at or below the low 1-bit.
    assert_eq!(smear_right(0b0101_1000), 0b0101_1111);
    assert_eq!(smear_right(1), 1);
    assert_eq!(smear_right(0b1000_0000), 0b1111_1111);
    assert_eq!(smear_right(u64::MAX), u64::MAX);
    // Convention for 0 (wrapping x - 1): all ones.
    assert_eq!(smear_right(0), u64::MAX);
}

#[test]
fn gosper_hand_trace_and_weight_2_chain() {
    // The lesson traces 0110 -> 1001 (run collapses, ones redistribute low).
    assert_eq!(next_same_weight(0b0110), 0b1001);
    // All weight-2 words on 4 bits, in increasing order.
    let chain = [0b0011u64, 0b0101, 0b0110, 0b1001, 0b1010, 0b1100];
    for w in chain.windows(2) {
        assert_eq!(next_same_weight(w[0]), w[1], "snoob({:#b})", w[0]);
    }
    assert_eq!(next_same_weight(1), 2);
    assert_eq!(next_same_weight(0b111), 0b1011);
}

#[test]
fn gosper_enumerates_all_3_subsets_of_12_bits_in_order() {
    // Start at the smallest weight-3 word and iterate: Gosper's hack must
    // visit every C(12,3) = 220 three-element subset of {0..11}, in
    // strictly increasing order, and then leave the 12-bit universe.
    let mut subsets = Vec::new();
    let mut x = 0b111u64;
    while x < (1 << 12) {
        subsets.push(x);
        x = next_same_weight(x);
    }
    let expected: Vec<u64> = (0..(1u64 << 12)).filter(|v| v.count_ones() == 3).collect();
    assert_eq!(subsets.len(), 220, "C(12,3) = 220 subsets");
    assert_eq!(subsets, expected, "in increasing order, none skipped");
}

#[test]
fn gosper_gives_the_immediate_successor() {
    // Minimality, brute force: for every 10-bit x, no y strictly between x
    // and next_same_weight(x) shares x's weight.
    for x in 1u64..(1 << 10) {
        let y = next_same_weight(x);
        assert!(y > x, "snoob must increase: {x:#b} -> {y:#b}");
        assert_eq!(y.count_ones(), x.count_ones(), "weight preserved at {x:#b}");
        for z in (x + 1)..y {
            assert_ne!(
                z.count_ones(),
                x.count_ones(),
                "snoob({x:#b}) skipped {z:#b}"
            );
        }
    }
}

#[test]
#[should_panic(expected = "undefined")]
fn ruler_of_zero_is_rejected() {
    // rho(0) does not exist — definiteness demands a loud failure.
    ruler(0);
}

#[test]
#[should_panic(expected = "weight 0")]
fn gosper_of_zero_is_rejected() {
    next_same_weight(0);
}
