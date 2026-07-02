//! Stage 2 — Boolean chains and combinational cost (TAOCP Vol. 4A, §7.1.2).
//!
//! Implement `Chain`, `eval_chain`, `chain_computes`, `chain_cost`, and
//! `apply_gate` in src/lab.rs. Lesson: course/module-21-boolean/README.md.

use lab_21_boolean::{
    apply_gate, chain_computes, chain_cost, eval_chain, BoolFunc, Chain, AND, NOTL, OR, XOR,
};

#[test]
fn gate_truth_tables() {
    // The 4-bit encoding: output = bit (2*a + b) of the op code.
    assert_eq!(apply_gate(AND, false, false), false);
    assert_eq!(apply_gate(AND, true, true), true);
    assert_eq!(apply_gate(OR, false, false), false);
    assert_eq!(apply_gate(OR, true, false), true);
    assert_eq!(apply_gate(XOR, true, true), false);
    assert_eq!(apply_gate(XOR, true, false), true);
    assert_eq!(apply_gate(NOTL, false, true), true); // NOT of the left input
    assert_eq!(apply_gate(NOTL, true, true), false);
}

#[test]
fn xor_from_and_or_not_costs_four() {
    // XOR = (x1 OR x2) AND NOT(x1 AND x2), using only AND/OR/NOT.
    let mut c = Chain::new(2);
    let or = c.gate(OR, 0, 1);
    let and = c.gate(AND, 0, 1);
    let not_and = c.gate(NOTL, and, and); // NOT via a 2-input gate
    let out = c.gate(AND, or, not_and);
    c.set_output(out);

    let xor = BoolFunc::from_closure(2, |x| (x & 1) ^ ((x >> 1) & 1) == 1);
    assert!(chain_computes(&c, &xor));
    assert_eq!(chain_cost(&c), 4);
}

#[test]
fn single_xor_gate_also_works() {
    // The full basis has XOR directly: a one-gate chain.
    let mut c = Chain::new(2);
    c.gate(XOR, 0, 1);
    let xor = BoolFunc::from_closure(2, |x| (x & 1) ^ ((x >> 1) & 1) == 1);
    assert!(chain_computes(&c, &xor));
    assert_eq!(chain_cost(&c), 1);
}

#[test]
fn majority_of_three_by_hand() {
    // Carry of a full adder = majority(x1, x2, x3)
    //   = (x1 AND x2) OR (x3 AND (x1 OR x2)).
    let mut c = Chain::new(3);
    let ab = c.gate(AND, 0, 1);
    let aorb = c.gate(OR, 0, 1);
    let t = c.gate(AND, 2, aorb);
    c.gate(OR, ab, t);

    let maj = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
    assert!(chain_computes(&c, &maj));
    assert_eq!(chain_cost(&c), 4);
}

#[test]
fn full_adder_sum_and_carry() {
    // The two outputs of a full adder as separate chains.
    // Sum = x1 XOR x2 XOR x3.
    let mut sum_chain = Chain::new(3);
    let t = sum_chain.gate(XOR, 0, 1);
    sum_chain.gate(XOR, t, 2);
    let sum = BoolFunc::from_closure(3, |x| x.count_ones() % 2 == 1);
    assert!(chain_computes(&sum_chain, &sum));
    assert_eq!(chain_cost(&sum_chain), 2);

    // Carry = majority(x1, x2, x3).
    let mut carry_chain = Chain::new(3);
    let ab = carry_chain.gate(AND, 0, 1);
    let aorb = carry_chain.gate(OR, 0, 1);
    let m = carry_chain.gate(AND, 2, aorb);
    carry_chain.gate(OR, ab, m);
    let carry = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
    assert!(chain_computes(&carry_chain, &carry));
}

#[test]
fn eval_chain_matches_the_whole_table() {
    // A chain evaluated over all 2^n inputs must equal its target function's
    // table, bit for bit.
    let mut c = Chain::new(3);
    let t = c.gate(XOR, 0, 1);
    c.gate(XOR, t, 2);
    let sum = BoolFunc::from_closure(3, |x| x.count_ones() % 2 == 1);
    let mut table = 0u64;
    for x in 0..8u32 {
        if eval_chain(&c, x) {
            table |= 1 << x;
        }
    }
    assert_eq!(table, sum.table);
}

#[test]
fn output_selection_picks_an_internal_value() {
    // A chain can expose any value as its output, not just the last gate.
    let mut c = Chain::new(2);
    let and = c.gate(AND, 0, 1);
    let _or = c.gate(OR, 0, 1);
    c.set_output(and); // choose the AND value
    let f = BoolFunc::from_closure(2, |x| (x & 1 == 1) && ((x >> 1) & 1 == 1));
    assert!(chain_computes(&c, &f));
    assert_eq!(chain_cost(&c), 2); // both gates still count
}

#[test]
fn wrong_arity_chain_fails_gracefully() {
    // A 2-input chain cannot compute a 3-variable function.
    let mut c = Chain::new(2);
    c.gate(AND, 0, 1);
    let f = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);
    assert!(!chain_computes(&c, &f));
}
