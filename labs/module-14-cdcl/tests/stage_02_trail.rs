//! Stage 2 — The trail: decisions, levels, reasons (§7.2.2.2).
//!
//! Implement `Trail` in src/lab.rs. The lesson:
//! course/module-14-cdcl/README.md, §3.

use lab_14_cdcl::{ClauseDb, Trail};

#[test]
fn levels_are_tracked_through_decides_and_enqueues() {
    let mut trail = Trail::new(6);
    assert_eq!(trail.decision_level(), 0);
    assert_eq!(trail.len(), 0);

    trail.enqueue(-6, None); // a root-level fact (e.g. from a unit clause)
    assert_eq!(trail.decision_level(), 0);
    assert_eq!(trail.level_of(6), Some(0));

    trail.decide(1);
    trail.enqueue(2, Some(0));
    trail.enqueue(3, Some(1));
    trail.decide(-4);
    trail.enqueue(5, Some(2));

    assert_eq!(trail.decision_level(), 2);
    assert_eq!(trail.len(), 6);
    assert_eq!(trail.literals(), &[-6, 1, 2, 3, -4, 5]);
    for (var, level) in [(6, 0), (1, 1), (2, 1), (3, 1), (4, 2), (5, 2)] {
        assert_eq!(trail.level_of(var), Some(level), "level of x{var}");
    }
    assert_eq!(trail.assignments_at_level(0), &[-6]);
    assert_eq!(trail.assignments_at_level(1), &[1, 2, 3]);
    assert_eq!(trail.assignments_at_level(2), &[-4, 5]);
}

#[test]
fn reasons_only_for_propagated_literals() {
    let mut trail = Trail::new(5);
    trail.decide(3);
    trail.enqueue(-1, Some(7));
    trail.enqueue(2, Some(4));

    assert_eq!(trail.reason_of(3), None, "decisions have no reason");
    assert_eq!(trail.reason_of(1), Some(7));
    assert_eq!(trail.reason_of(2), Some(4));
    assert_eq!(trail.reason_of(5), None, "unassigned variables have no reason");
    assert_eq!(trail.level_of(5), None, "unassigned variables have no level");
}

#[test]
fn backjump_pops_exactly_the_right_suffix() {
    let mut trail = Trail::new(9);
    trail.enqueue(9, None); // level 0
    trail.decide(1); // level 1
    trail.enqueue(2, Some(0));
    trail.decide(3); // level 2
    trail.enqueue(4, Some(1));
    trail.enqueue(5, Some(2));
    trail.decide(-6); // level 3
    trail.enqueue(7, Some(3));

    // Backjumping to the current level is a no-op.
    assert_eq!(trail.backjump(3), vec![]);
    assert_eq!(trail.len(), 8);

    // Jump over level 3 AND level 2 in one go, straight to level 1: exactly
    // the literals of levels 2 and 3 come back, newest first.
    assert_eq!(trail.backjump(1), vec![7, -6, 5, 4, 3]);
    assert_eq!(trail.decision_level(), 1);
    assert_eq!(trail.literals(), &[9, 1, 2]);
    for var in [3, 4, 5, 6, 7] {
        assert_eq!(trail.level_of(var), None, "x{var} must be forgotten");
        assert_eq!(trail.reason_of(var), None);
    }
    // The surviving prefix is untouched.
    assert_eq!(trail.level_of(9), Some(0));
    assert_eq!(trail.level_of(1), Some(1));
    assert_eq!(trail.reason_of(2), Some(0));
    assert_eq!(trail.assignments_at_level(1), &[1, 2]);

    // The freed levels can be rebuilt with different content.
    trail.decide(5);
    assert_eq!(trail.decision_level(), 2);
    assert_eq!(trail.assignments_at_level(2), &[5]);

    // And a jump to the root clears everything but level 0.
    assert_eq!(trail.backjump(0), vec![5, 2, 1]);
    assert_eq!(trail.literals(), &[9]);
    assert_eq!(trail.decision_level(), 0);
}

/// The stage-1 and stage-2 structures in lockstep, checked step by step —
/// a miniature of what the stage-4 driver will do.
#[test]
fn integrated_scenario_with_watched_clauses() {
    let mut db = ClauseDb::new(4);
    db.add_clause(vec![-1, 2]); // c0
    db.add_clause(vec![-2, 3]); // c1
    db.add_clause(vec![-3, -4]); // c2
    let mut trail = Trail::new(4);

    // Decision level 1: x1 = true. Propagation forces x2 then x3.
    trail.decide(1);
    db.assign(1);
    assert_eq!(db.propagate(), None);
    for (lit, reason) in db.take_implications() {
        trail.enqueue(lit, Some(reason));
    }
    assert_eq!(trail.literals(), &[1, 2, 3]);
    assert_eq!(trail.decision_level(), 1);
    assert_eq!(trail.level_of(2), Some(1));
    assert_eq!(trail.level_of(3), Some(1));
    assert_eq!(trail.reason_of(2), Some(0));
    assert_eq!(trail.reason_of(3), Some(1));

    // Decision level 2: x4 = true falsifies c2 = (¬x3 ∨ ¬x4).
    trail.decide(4);
    db.assign(4);
    assert_eq!(db.propagate(), Some(2), "the conflict must name c2");

    // Undo both levels; ClauseDb and Trail stay in sync, no watch fixup.
    for lit in trail.backjump(0) {
        db.unassign(lit.unsigned_abs() as usize);
    }
    assert_eq!(trail.len(), 0);
    assert_eq!(trail.decision_level(), 0);
    for v in 1..=4 {
        assert_eq!(db.value(v), None);
        assert_eq!(trail.level_of(v as usize), None);
    }
    assert!(db.check_watch_invariant());

    // A different level 1 succeeds: ¬x4 first, then x1 propagates cleanly.
    trail.decide(-4);
    db.assign(-4);
    assert_eq!(db.propagate(), None);
    trail.decide(1);
    db.assign(1);
    assert_eq!(db.propagate(), None);
    for (lit, reason) in db.take_implications() {
        trail.enqueue(lit, Some(reason));
    }
    assert_eq!(db.value(2), Some(true));
    assert_eq!(db.value(3), Some(true));
    assert_eq!(trail.assignments_at_level(2), &[1, 2, 3]);
    assert!(db.check_watch_invariant());
}
