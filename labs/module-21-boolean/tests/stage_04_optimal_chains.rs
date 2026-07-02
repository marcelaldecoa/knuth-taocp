//! Stage 4 — Optimum chains for small functions (TAOCP Vol. 4A, §7.1.2).
//!
//! Implement `optimal_cost`, `full_basis`, and `standard_basis` in src/lab.rs.
//! Lesson: course/module-21-boolean/README.md.

use lab_21_boolean::{
    chain_computes, chain_cost, full_basis, optimal_cost, standard_basis, BoolFunc, Chain, AND,
    OR, XOR,
};

#[test]
fn basis_sizes() {
    assert_eq!(full_basis().len(), 16); // all 2-input operations
    assert_eq!(standard_basis().len(), 3); // {AND, OR, NOT}
}

#[test]
fn projections_and_constants_are_free() {
    let basis = full_basis();
    // C(f) = 0 for the constants and the projections.
    let zero = BoolFunc { n: 3, table: 0 };
    let one = BoolFunc { n: 3, table: 0xFF };
    let x1 = BoolFunc::from_closure(3, |x| x & 1 == 1);
    let x2 = BoolFunc::from_closure(3, |x| (x >> 1) & 1 == 1);
    let x3 = BoolFunc::from_closure(3, |x| (x >> 2) & 1 == 1);
    for f in [zero, one, x1, x2, x3] {
        assert_eq!(optimal_cost(&f, &basis), 0);
    }
}

#[test]
fn xor2_needs_exactly_one_gate() {
    let basis = full_basis();
    let xor = BoolFunc::from_closure(2, |x| (x & 1) ^ ((x >> 1) & 1) == 1);
    assert_eq!(optimal_cost(&xor, &basis), 1);
}

#[test]
fn every_two_variable_function_costs_at_most_one() {
    // With the full basis each of the 16 two-variable functions is itself a
    // single gate; C(f) = 0 only for constants and projections.
    let basis = full_basis();
    // Only constants and the inputs themselves are free; a negated input
    // (e.g. ¬x1 = 0b0101) still costs one NOT gate.
    let free = [
        0b0000, // constant 0
        0b1111, // constant 1
        0b1010, // x1
        0b1100, // x2
    ];
    for table in 0..16u64 {
        let f = BoolFunc { n: 2, table };
        let c = optimal_cost(&f, &basis);
        assert!(c <= 1, "C(table={table}) = {c} should be <= 1");
        if free.contains(&table) {
            assert_eq!(c, 0, "table={table} is a constant or projection");
        } else {
            assert_eq!(c, 1, "table={table} is a genuine 2-input gate");
        }
    }
}

#[test]
fn xor3_optimal_cost_matches_hand_chain() {
    // Parity of three: x1 XOR x2 XOR x3. A hand-built chain uses two XOR
    // gates, and that is optimal.
    let basis = full_basis();
    let xor3 = BoolFunc::from_closure(3, |x| x.count_ones() % 2 == 1);

    let mut chain = Chain::new(3);
    let t = chain.gate(XOR, 0, 1);
    chain.gate(XOR, t, 2);
    assert!(chain_computes(&chain, &xor3));
    assert_eq!(chain_cost(&chain), 2);

    assert_eq!(optimal_cost(&xor3, &basis), 2);
    assert_eq!(optimal_cost(&xor3, &basis), chain_cost(&chain));
}

#[test]
fn majority3_optimal_cost_is_four() {
    // Majority-of-three needs four gates over the full basis — the naive
    // "reachable functions" frontier wrongly reports three, because it treats
    // a gate's two operands as jointly free. A real four-gate chain attains
    // the optimum: (x1 AND x2) OR (x3 AND (x1 OR x2)).
    let basis = full_basis();
    let maj = BoolFunc::from_closure(3, |x| x.count_ones() >= 2);

    let mut chain = Chain::new(3);
    let ab = chain.gate(AND, 0, 1);
    let aorb = chain.gate(OR, 0, 1);
    let m = chain.gate(AND, 2, aorb);
    chain.gate(OR, ab, m);
    assert!(chain_computes(&chain, &maj));
    assert_eq!(chain_cost(&chain), 4);

    assert_eq!(optimal_cost(&maj, &basis), 4);
}

#[test]
fn optimal_cost_never_exceeds_a_known_chain() {
    // Sanity across a handful of 3-variable functions: the optimum is never
    // larger than an exhibited chain, and never below the true minimum.
    let basis = full_basis();

    // f = (x1 AND x2) OR ¬x3  — a two-gate chain exists (NOR then ... ), and
    // the search agrees the optimum is 2.
    let f = BoolFunc::from_closure(3, |x| {
        let x1 = x & 1 == 1;
        let x2 = (x >> 1) & 1 == 1;
        let x3 = (x >> 2) & 1 == 1;
        (x1 && x2) || !x3
    });
    let c = optimal_cost(&f, &basis);
    assert_eq!(c, 2);
}
