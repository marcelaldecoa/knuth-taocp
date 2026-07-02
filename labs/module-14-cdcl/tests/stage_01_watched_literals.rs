//! Stage 1 — Lazy data structures: two watched literals (§7.2.2.2, the
//! engine room of Algorithm C).
//!
//! Implement `ClauseDb` in src/lab.rs. The lesson:
//! course/module-14-cdcl/README.md, §5.

use lab_14_cdcl::ClauseDb;

fn db_with(num_vars: usize, clauses: &[&[i32]]) -> ClauseDb {
    let mut db = ClauseDb::new(num_vars);
    for (i, c) in clauses.iter().enumerate() {
        let idx = db.add_clause(c.to_vec());
        assert_eq!(idx, i, "add_clause must return consecutive indices from 0");
    }
    db
}

#[test]
fn values_track_assignments() {
    let mut db = db_with(3, &[&[1, 2, 3]]);
    assert_eq!(db.value(1), None);
    assert_eq!(db.value(-1), None);
    db.assign(-2);
    assert_eq!(db.value(2), Some(false));
    assert_eq!(db.value(-2), Some(true));
    assert_eq!(db.value(3), None);
}

#[test]
fn unit_clauses_propagate() {
    // (x1) ∧ (¬x1 ∨ x2): both x1 and x2 end up forced true.
    let mut db = db_with(3, &[&[1], &[-1, 2]]);
    assert_eq!(db.propagate(), None);
    assert_eq!(db.value(1), Some(true));
    assert_eq!(db.value(2), Some(true));
    assert_eq!(db.value(3), None, "x3 is untouched");
    // Each forced literal carries the clause that forced it, in causal order.
    assert_eq!(db.take_implications(), vec![(1, 0), (2, 1)]);
    // Draining is draining: a second take yields nothing new.
    assert_eq!(db.take_implications(), vec![]);
    assert!(db.check_watch_invariant());
}

#[test]
fn chains_propagate_transitively() {
    // x1 = true dominoes through four implications; the middle clause's
    // watch has to *move* (its second literal ¬x1 is falsified first).
    let mut db = db_with(5, &[&[-1, 2], &[-2, 3], &[-3, -1, 4], &[-4, 5]]);
    db.assign(1);
    assert_eq!(db.propagate(), None);
    for v in 1..=5 {
        assert_eq!(db.value(v), Some(true), "x{v} should be forced true");
    }
    assert_eq!(db.take_implications(), vec![(2, 0), (3, 1), (4, 2), (5, 3)]);
    assert!(db.check_watch_invariant());
}

#[test]
fn conflict_reports_the_right_clause() {
    // Decisions falsify (¬x1 ∨ ¬x2) directly: only clause 0 can conflict.
    let mut db = db_with(2, &[&[-1, -2]]);
    db.assign(1);
    assert_eq!(db.propagate(), None);
    db.assign(2);
    assert_eq!(db.propagate(), Some(0));

    // Forced conflict, unique no matter in which order watchers are
    // examined: x1 forces x2 (c0) and ¬x3 (c2), which together falsify
    // c1 = (¬x2 ∨ x3); c0 and c2 end up satisfied, so the conflict is c1.
    let mut db = db_with(3, &[&[-1, 2], &[-2, 3], &[-1, -3]]);
    db.assign(1);
    assert_eq!(db.propagate(), Some(1));
    assert_eq!(db.value(2), Some(true), "forced before the conflict");
    assert_eq!(db.value(3), Some(false), "forced before the conflict");
}

#[test]
fn invariant_holds_after_arbitrary_assign_propagate_sequences() {
    // Deterministic LCG-driven torture: random clauses, random decisions,
    // propagate after each. At every conflict-free fixed point the watch
    // invariant must hold, and — the semantic contract of lazy propagation —
    // every clause must be satisfied or keep >= 2 unassigned literals.
    let mut x: u64 = 0x5DEECE66D;
    let mut rng = move || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        x >> 33
    };
    for trial in 0..40 {
        let n = 4 + (rng() as usize) % 6; // 4..=9 variables
        let m = 3 + (rng() as usize) % 16; // 3..=18 clauses
        let mut db = ClauseDb::new(n);
        for _ in 0..m {
            let len = 2 + (rng() as usize) % 2; // binary and ternary clauses
            let mut vars: Vec<usize> = Vec::new();
            while vars.len() < len {
                let v = 1 + (rng() as usize) % n;
                if !vars.contains(&v) {
                    vars.push(v);
                }
            }
            let clause: Vec<i32> = vars
                .iter()
                .map(|&v| if rng() % 2 == 0 { v as i32 } else { -(v as i32) })
                .collect();
            db.add_clause(clause);
        }
        for _ in 0..n {
            let unassigned: Vec<i32> = (1..=n as i32).filter(|&v| db.value(v).is_none()).collect();
            let Some(&v) = unassigned.get((rng() as usize) % unassigned.len().max(1)) else {
                break;
            };
            db.assign(if rng() % 2 == 0 { v } else { -v });
            match db.propagate() {
                Some(c) => {
                    // A reported conflict really is a falsified clause.
                    for &lit in db.clause(c) {
                        assert_eq!(
                            db.value(lit),
                            Some(false),
                            "trial {trial}: conflict clause {c} has a non-false literal {lit}"
                        );
                    }
                    break;
                }
                None => {
                    assert!(
                        db.check_watch_invariant(),
                        "trial {trial}: watch invariant broken at a fixed point"
                    );
                    for c in 0..db.num_clauses() {
                        let lits = db.clause(c);
                        let satisfied = lits.iter().any(|&l| db.value(l) == Some(true));
                        let unassigned =
                            lits.iter().filter(|&&l| db.value(l).is_none()).count();
                        assert!(
                            satisfied || unassigned >= 2,
                            "trial {trial}: at a fixed point clause {c} = {lits:?} is \
                             neither satisfied nor >= 2-unassigned (a missed unit or conflict)"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn unassign_needs_no_watch_fixup() {
    let mut db = db_with(6, &[&[-1, 2], &[-2, -3, 4], &[-2, -5, 6]]);
    db.assign(1);
    assert_eq!(db.propagate(), None);
    assert_eq!(db.take_implications(), vec![(2, 0)]);
    db.assign(3);
    assert_eq!(db.propagate(), None);
    assert_eq!(db.take_implications(), vec![(4, 1)]);
    assert!(db.check_watch_invariant());

    // Undo the x3 "level" in LIFO order. No watch surgery happens — the
    // invariant must hold immediately, by the stability lemma of the lesson.
    db.unassign(4);
    db.unassign(3);
    assert!(db.check_watch_invariant());
    assert_eq!(db.value(3), None);
    assert_eq!(db.value(4), None);
    assert_eq!(db.value(2), Some(true), "the x1 level is untouched");

    // The untouched watch lists still propagate correctly afterwards, in a
    // different direction first (c2), then the redone one (c1).
    db.assign(5);
    assert_eq!(db.propagate(), None);
    assert_eq!(db.value(6), Some(true));
    db.assign(3);
    assert_eq!(db.propagate(), None);
    assert_eq!(db.value(4), Some(true));
    assert_eq!(db.take_implications(), vec![(6, 2), (4, 1)]);
    assert!(db.check_watch_invariant());
}

#[test]
fn repropagation_in_a_different_order_is_confluent() {
    // Same clauses, same set of decisions, opposite order: unit propagation
    // is confluent, so the final values must agree literal for literal.
    let clauses: &[&[i32]] = &[&[-1, 3], &[-2, 4], &[-3, 5], &[-4, 5], &[-5, 6]];
    let mut a = db_with(6, clauses);
    a.assign(1);
    assert_eq!(a.propagate(), None);
    a.assign(2);
    assert_eq!(a.propagate(), None);

    let mut b = db_with(6, clauses);
    b.assign(2);
    assert_eq!(b.propagate(), None);
    b.assign(1);
    assert_eq!(b.propagate(), None);

    for v in 1..=6 {
        assert_eq!(a.value(v), b.value(v), "value of x{v} differs by order");
        assert_eq!(a.value(v), Some(true));
    }
    assert!(a.check_watch_invariant());
    assert!(b.check_watch_invariant());
}
