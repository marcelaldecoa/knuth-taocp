//! Stage 1 — Truth tables and normal forms (TAOCP Vol. 4A, §7.1.1).
//!
//! Implement `BoolFunc` and its methods in src/lab.rs. Lesson:
//! course/module-21-boolean/README.md.

use lab_21_boolean::BoolFunc;

// Reference closures (x1 = bit 0, x2 = bit 1, x3 = bit 2).
fn and2(x: u32) -> bool {
    (x & 1 == 1) && ((x >> 1) & 1 == 1)
}
fn or2(x: u32) -> bool {
    (x & 1 == 1) || ((x >> 1) & 1 == 1)
}
fn xor2(x: u32) -> bool {
    (x & 1) ^ ((x >> 1) & 1) == 1
}
fn maj3(x: u32) -> bool {
    x.count_ones() >= 2
}

#[test]
fn eval_matches_the_table() {
    // AND's truth table: true only at x = 0b11 = 3, so table = 0b1000.
    let f = BoolFunc::from_closure(2, and2);
    assert_eq!(f.table, 0b1000);
    for x in 0..4 {
        assert_eq!(f.eval(x), and2(x), "AND at x={x}");
    }
}

#[test]
fn number_of_minterms_is_popcount() {
    assert_eq!(BoolFunc::from_closure(2, and2).num_minterms(), 1);
    assert_eq!(BoolFunc::from_closure(2, or2).num_minterms(), 3);
    assert_eq!(BoolFunc::from_closure(2, xor2).num_minterms(), 2);
    assert_eq!(BoolFunc::from_closure(3, maj3).num_minterms(), 4);
}

#[test]
fn dnf_of_the_standard_gates() {
    // Every minterm is a full assignment of all n variables.
    let f = BoolFunc::from_closure(3, maj3);
    let dnf = f.to_dnf();
    assert_eq!(dnf.len(), 4); // four inputs with two or more ones
    for term in &dnf {
        assert_eq!(term.len(), 3, "each minterm names all three variables");
    }
    // The DNF must reconstruct the function exactly.
    assert_eq!(BoolFunc::from_dnf(3, &dnf), f);
}

#[test]
fn dnf_and_cnf_round_trip_over_all_small_functions() {
    // Exhaustive check: every function of n <= 4 variables survives a
    // to_dnf -> from_dnf and to_cnf -> from_cnf round trip.
    for n in 0..=4u32 {
        let rows: u64 = 1 << (1u32 << n);
        for table in 0..rows {
            let f = BoolFunc { n, table };
            assert_eq!(BoolFunc::from_dnf(n, &f.to_dnf()), f, "DNF n={n} table={table}");
            assert_eq!(BoolFunc::from_cnf(n, &f.to_cnf()), f, "CNF n={n} table={table}");
            assert_eq!(f.to_dnf().len() as u32, f.num_minterms());
        }
    }
}

#[test]
fn cnf_maxterm_count_is_the_zero_rows() {
    // OR of two variables is false on exactly one row, so CNF has one clause.
    let f = BoolFunc::from_closure(2, or2);
    assert_eq!(f.to_cnf().len(), 1);
    assert_eq!(BoolFunc::from_cnf(2, &f.to_cnf()), f);
}

#[test]
fn de_morgan_via_table_complement() {
    // ¬(x1 AND x2) == (¬x1) OR (¬x2). Build the right side directly and
    // compare truth tables.
    let and = BoolFunc::from_closure(2, and2);
    let nand = and.complement();
    let de_morgan = BoolFunc::from_closure(2, |x| !(x & 1 == 1) || !((x >> 1) & 1 == 1));
    assert_eq!(nand, de_morgan);
    // Complement is an involution.
    assert_eq!(and.complement().complement(), and);
}

#[test]
fn constant_and_tautology_edge_cases() {
    let zero = BoolFunc { n: 3, table: 0 };
    let one = BoolFunc { n: 3, table: 0xFF }; // 2^3 = 8 rows all true
    assert_eq!(zero.num_minterms(), 0);
    assert_eq!(one.num_minterms(), 8);
    // The constant 0 has no minterms (empty DNF); the constant 1 has no
    // maxterms (empty CNF).
    assert!(zero.to_dnf().is_empty());
    assert!(one.to_cnf().is_empty());
    assert_eq!(BoolFunc::from_dnf(3, &zero.to_dnf()), zero);
    assert_eq!(BoolFunc::from_cnf(3, &one.to_cnf()), one);
    assert_eq!(zero.complement(), one);
}

#[test]
fn shorter_product_terms_cover_subcubes() {
    // A DNF term need not be a full minterm: the single literal [+1] means
    // "x1 is true", covering both rows where x1 = 1 (for n = 2).
    let f = BoolFunc::from_dnf(2, &[vec![1]]);
    assert!(f.eval(1) && f.eval(3)); // x1 = 1
    assert!(!f.eval(0) && !f.eval(2)); // x1 = 0
    assert_eq!(f.num_minterms(), 2);
}

#[test]
#[should_panic(expected = "<= 6")]
fn rejects_too_many_variables() {
    // Only n <= 6 fits in a u64 truth table.
    BoolFunc::from_closure(7, |_| true);
}
