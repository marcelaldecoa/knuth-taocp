//! Stage 3 — Conflict analysis: learning from failure (Algorithm 7.2.2.2C,
//! first UIP).
//!
//! Implement `analyze` and `brute_force_implies` in src/lab.rs. The lesson:
//! course/module-14-cdcl/README.md, §4 — the eight-variable scenario below is
//! the lesson's worked example, machine-checked.

use lab_14_cdcl::{analyze, brute_force_implies, ClauseDb, Cnf, Trail};

/// Build a `ClauseDb` + `Trail` from a hand-written implication-graph
/// script: `(literal, None)` is a decision, `(literal, Some(c))` a literal
/// propagated by clause `c`. Values are mirrored into the db so `analyze`
/// sees a consistent state; propagation itself is *not* run — the graph is
/// exactly what the script says, independent of watcher order.
fn scenario(num_vars: usize, clauses: &[&[i32]], script: &[(i32, Option<usize>)]) -> (ClauseDb, Trail) {
    let mut db = ClauseDb::new(num_vars);
    for c in clauses {
        db.add_clause(c.to_vec());
    }
    let mut trail = Trail::new(num_vars);
    for &(lit, reason) in script {
        match reason {
            None => trail.decide(lit),
            Some(r) => trail.enqueue(lit, Some(r)),
        }
        db.assign(lit);
    }
    (db, trail)
}

fn as_set(mut lits: Vec<i32>) -> Vec<i32> {
    lits.sort_unstable();
    lits
}

/// Every learned clause must be *asserting*: exactly one literal from the
/// conflicting decision level.
fn count_current_level(learned: &[i32], trail: &Trail) -> usize {
    learned
        .iter()
        .filter(|&&lit| trail.level_of(lit.unsigned_abs() as usize) == Some(trail.decision_level()))
        .count()
}

#[test]
fn brute_force_implies_sanity() {
    let cnf = Cnf {
        num_vars: 2,
        clauses: vec![vec![1, 2], vec![-1, 2]],
    };
    assert!(brute_force_implies(&cnf, &[2]), "resolving the two clauses gives (x2)");
    assert!(brute_force_implies(&cnf, &[1, 2]), "a clause of the formula is implied");
    assert!(brute_force_implies(&cnf, &[2, -2]), "a tautology is implied by anything");
    assert!(!brute_force_implies(&cnf, &[1]));
    assert!(!brute_force_implies(&cnf, &[-2]));
    // The empty clause is implied only by an unsatisfiable formula.
    assert!(!brute_force_implies(&cnf, &[]));
    let unsat = Cnf {
        num_vars: 1,
        clauses: vec![vec![1], vec![-1]],
    };
    assert!(brute_force_implies(&unsat, &[]));
}

/// The lesson's worked example: decisions ¬x7@1, ¬x8@2, x1@3; the level-3
/// propagations x2, x3, x4, x5, x6 crash into C5 = (¬x5 ∨ ¬x6). The first
/// UIP is x4 (every path from x1 to the conflict runs through it), the
/// learned clause is (¬x4 ∨ x8), and the solver backjumps to level 2.
#[test]
fn eight_variable_worked_example() {
    let clauses: &[&[i32]] = &[
        &[-1, 2],     // C0
        &[-1, 3, 7],  // C1
        &[-2, -3, 4], // C2
        &[-4, 5, 8],  // C3
        &[-4, 6],     // C4
        &[-5, -6],    // C5 — becomes the conflict
    ];
    let script = [
        (-7, None),   // level 1
        (-8, None),   // level 2
        (1, None),    // level 3
        (2, Some(0)),
        (3, Some(1)),
        (4, Some(2)),
        (5, Some(3)),
        (6, Some(4)),
    ];
    let (db, trail) = scenario(8, clauses, &script);

    let (learned, back) = analyze(5, &trail, &db);
    assert_eq!(as_set(learned.clone()), vec![-4, 8]);
    assert_eq!(back, 2, "second-highest level in the clause (the level of x8)");
    assert_eq!(count_current_level(&learned, &trail), 1, "asserting");

    // Soundness: the learned clause is a logical consequence of the formula
    // (it is a resolvent chain: C5, C4, C3), not just plausible.
    let cnf = Cnf {
        num_vars: 8,
        clauses: clauses.iter().map(|c| c.to_vec()).collect(),
    };
    assert!(brute_force_implies(&cnf, &learned));
    // ... and it is genuinely new information w.r.t. each single input clause:
    assert!(!brute_force_implies(&Cnf { num_vars: 8, clauses: vec![] }, &learned));
}

/// A UIP in mid-level that is not the decision: x1@3 leads through x2, x3 to
/// a conflict whose analysis stops at x3. Lower-level literals from two
/// different levels (x5@1, x6@2) show up; the backjump goes to the
/// second-highest, level 2.
#[test]
fn mid_level_uip_across_three_levels() {
    let clauses: &[&[i32]] = &[
        &[-1, 2, 5],  // C0
        &[-2, 3, 6],  // C1
        &[-3, 4],     // C2
        &[-3, -4, 6], // C3 — becomes the conflict
    ];
    let script = [
        (-5, None),   // level 1
        (-6, None),   // level 2
        (1, None),    // level 3
        (2, Some(0)),
        (3, Some(1)),
        (4, Some(2)),
    ];
    let (db, trail) = scenario(6, clauses, &script);

    let (learned, back) = analyze(3, &trail, &db);
    assert_eq!(as_set(learned.clone()), vec![-3, 6]);
    assert_eq!(back, 2);
    assert_eq!(count_current_level(&learned, &trail), 1, "asserting");
    let cnf = Cnf {
        num_vars: 6,
        clauses: clauses.iter().map(|c| c.to_vec()).collect(),
    };
    assert!(brute_force_implies(&cnf, &learned));
}

/// When the analysis resolves all the way back to a single-literal clause,
/// the learned clause is unit and the backjump level is 0: the solver
/// returns to the root and fixes the variable forever.
#[test]
fn unit_learned_clause_backjumps_to_root() {
    let clauses: &[&[i32]] = &[
        &[-1, 2], // C0
        &[-1, 3], // C1
        &[-2, -3], // C2 — becomes the conflict
    ];
    let script = [(1, None), (2, Some(0)), (3, Some(1))];
    let (db, trail) = scenario(3, clauses, &script);

    let (learned, back) = analyze(2, &trail, &db);
    assert_eq!(learned, vec![-1], "learned unit clause (¬x1)");
    assert_eq!(back, 0, "unit learned clause: backjump to the root");
    let cnf = Cnf {
        num_vars: 3,
        clauses: clauses.iter().map(|c| c.to_vec()).collect(),
    };
    assert!(brute_force_implies(&cnf, &learned));
}

/// The decision itself is the first UIP when the conflict clause and the
/// reasons keep two current-level chains separate until the decision.
#[test]
fn decision_can_be_the_first_uip() {
    let clauses: &[&[i32]] = &[
        &[-1, 2],  // C0: x1 → x2
        &[-1, 3],  // C1: x1 → x3
        &[-2, -3, 7], // C2 — becomes the conflict
    ];
    let script = [
        (-7, None),  // level 1
        (1, None),   // level 2
        (2, Some(0)),
        (3, Some(1)),
    ];
    let (db, trail) = scenario(7, clauses, &script);

    let (learned, back) = analyze(2, &trail, &db);
    // Resolving C2 with C1 (on x3) and C0 (on x2) leaves (¬x1 ∨ x7):
    // the two chains only meet at the decision x1.
    assert_eq!(as_set(learned.clone()), vec![-1, 7]);
    assert_eq!(back, 1);
    assert_eq!(count_current_level(&learned, &trail), 1, "asserting");
    let cnf = Cnf {
        num_vars: 7,
        clauses: clauses.iter().map(|c| c.to_vec()).collect(),
    };
    assert!(brute_force_implies(&cnf, &learned));
}
