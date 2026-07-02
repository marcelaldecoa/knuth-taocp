//! Stage 2 — Unit propagation (§7.2.2.2).
//!
//! Implement `unit_propagate` in src/lab.rs (and the `Propagation` enum is
//! already declared for you). The lesson: course/module-10-sat/README.md.

use lab_10_sat::{unit_propagate, Cnf, Propagation};

#[test]
fn propagation_chains_forced_literals() {
    // (x1) ∧ (¬x1 ∨ x2) ∧ (¬x2 ∨ x3): x1 is a unit, forcing x1; then the
    // second clause becomes a unit forcing x2; then the third forces x3.
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![1], vec![-1, 2], vec![-2, 3]],
    };
    let mut a = vec![None; 3];
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Implied(vec![1, 2, 3]));
    assert_eq!(a, vec![Some(true), Some(true), Some(true)]);
}

#[test]
fn conflict_is_detected() {
    // (x1) ∧ (¬x1): forcing x1 true empties the second clause.
    let cnf = Cnf {
        num_vars: 1,
        clauses: vec![vec![1], vec![-1]],
    };
    let mut a = vec![None];
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Conflict);
}

#[test]
fn no_op_when_nothing_is_unit() {
    // Every clause has >= 2 unassigned literals: nothing is forced.
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![1, 2], vec![-1, 3], vec![2, -3]],
    };
    let mut a = vec![None; 3];
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Implied(vec![]));
    assert_eq!(a, vec![None, None, None]);
}

#[test]
fn idempotence_after_reaching_fixed_point() {
    // Running again at the fixed point forces nothing more.
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![1], vec![-1, 2], vec![-2, 3]],
    };
    let mut a = vec![None; 3];
    let _ = unit_propagate(&cnf, &mut a);
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Implied(vec![]));
    assert_eq!(a, vec![Some(true), Some(true), Some(true)]);
}

#[test]
fn respects_preassigned_variables() {
    // Pre-assign x1 = false. Then (x1 ∨ x2) is a unit forcing x2 true.
    let cnf = Cnf {
        num_vars: 2,
        clauses: vec![vec![1, 2]],
    };
    let mut a = vec![Some(false), None];
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Implied(vec![2]));
    assert_eq!(a, vec![Some(false), Some(true)]);
}

#[test]
fn satisfied_clauses_are_skipped() {
    // (x1 ∨ x2) is already satisfied by x1 = true, so x2 stays unassigned.
    let cnf = Cnf {
        num_vars: 2,
        clauses: vec![vec![1, 2]],
    };
    let mut a = vec![Some(true), None];
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Implied(vec![]));
    assert_eq!(a, vec![Some(true), None]);
}

#[test]
fn negative_units_force_false() {
    // (¬x1) forces x1 false; recorded literal is -1.
    let cnf = Cnf {
        num_vars: 1,
        clauses: vec![vec![-1]],
    };
    let mut a = vec![None];
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Implied(vec![-1]));
    assert_eq!(a, vec![Some(false)]);
}

#[test]
fn empty_clause_present_is_conflict() {
    // An empty clause is unsatisfiable: propagation reports a conflict at once.
    let cnf = Cnf {
        num_vars: 2,
        clauses: vec![vec![]],
    };
    let mut a = vec![None; 2];
    assert_eq!(unit_propagate(&cnf, &mut a), Propagation::Conflict);
}

#[test]
#[should_panic(expected = "per variable")]
fn wrong_length_assignment_panics() {
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![1]],
    };
    let mut a = vec![None; 2];
    unit_propagate(&cnf, &mut a);
}
