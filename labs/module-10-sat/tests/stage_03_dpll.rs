//! Stage 3 — A DPLL solver (Algorithm 7.2.2.2D).
//!
//! Implement `solve`, `solve_brute`, `pigeonhole_cnf`, and `waerden_cnf` in
//! src/lab.rs. The lesson: course/module-10-sat/README.md.
//!
//! We never compare `solve` against a *fixed* model — any two DPLL solvers
//! may return different satisfying assignments. Instead we validate every
//! SAT result semantically with `Cnf::evaluate`, and every UNSAT result by
//! agreement with the exhaustive `solve_brute`.

use lab_10_sat::{pigeonhole_cnf, solve, solve_brute, waerden_cnf, Cnf};

#[test]
fn tiny_sat_model_is_valid() {
    // (x1 ∨ x2) ∧ (¬x1 ∨ x3): satisfiable; whatever model comes back must
    // actually satisfy the formula.
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![1, 2], vec![-1, 3]],
    };
    let model = solve(&cnf).expect("formula is satisfiable");
    assert!(cnf.evaluate(&model));
}

#[test]
fn tiny_unsat_returns_none() {
    // (x1) ∧ (¬x1): no model.
    let cnf = Cnf {
        num_vars: 1,
        clauses: vec![vec![1], vec![-1]],
    };
    assert_eq!(solve(&cnf), None);
}

#[test]
fn empty_clause_is_unsat() {
    let cnf = Cnf {
        num_vars: 2,
        clauses: vec![vec![1], vec![]],
    };
    assert_eq!(solve(&cnf), None);
}

#[test]
fn empty_formula_is_sat() {
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![],
    };
    let model = solve(&cnf).expect("empty formula is satisfiable");
    assert!(cnf.evaluate(&model));
}

#[test]
fn pigeonhole_4_3_is_unsat() {
    // Four pigeons cannot occupy three holes.
    assert_eq!(solve(&pigeonhole_cnf(4, 3)), None);
}

#[test]
fn pigeonhole_5_4_is_unsat() {
    assert_eq!(solve(&pigeonhole_cnf(5, 4)), None);
}

#[test]
fn pigeonhole_3_3_is_sat() {
    // Three pigeons fit in three holes; check the model semantically.
    let cnf = pigeonhole_cnf(3, 3);
    let model = solve(&cnf).expect("3 pigeons fit in 3 holes");
    assert!(cnf.evaluate(&model));
}

#[test]
fn waerden_3_3_8_is_sat() {
    // §7.2.2.2 running example: W(3,3) = 9, so 8 integers can be 2-coloured
    // with no monochromatic 3-term arithmetic progression.
    let cnf = waerden_cnf(3, 3, 8);
    let model = solve(&cnf).expect("waerden(3,3;8) is satisfiable");
    assert!(cnf.evaluate(&model));
}

#[test]
fn waerden_3_3_9_is_unsat() {
    // Nine integers cannot: W(3,3) = 9.
    assert_eq!(solve(&waerden_cnf(3, 3, 9)), None);
    assert_eq!(solve_brute(&waerden_cnf(3, 3, 9)), None);
}

#[test]
fn brute_force_matches_hand_examples() {
    let sat = Cnf {
        num_vars: 3,
        clauses: vec![vec![1, 2], vec![-1, 3]],
    };
    let model = solve_brute(&sat).expect("satisfiable");
    assert!(sat.evaluate(&model));

    let unsat = Cnf {
        num_vars: 1,
        clauses: vec![vec![1], vec![-1]],
    };
    assert_eq!(solve_brute(&unsat), None);
}

/// A deterministic linear congruential generator (no external crates).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 33) % n
    }
    /// A single well-mixed bit (low LCG bits have short periods, so use a
    /// high one).
    fn bit(&mut self) -> bool {
        (self.next() >> 40) & 1 == 1
    }
}

/// Build a random 3-CNF over `num_vars` variables with `num_clauses` clauses.
fn random_3sat(rng: &mut Lcg, num_vars: usize, num_clauses: usize) -> Cnf {
    let mut clauses = Vec::with_capacity(num_clauses);
    for _ in 0..num_clauses {
        let mut clause = Vec::with_capacity(3);
        while clause.len() < 3 {
            let v = (rng.below(num_vars as u64) + 1) as i32;
            let lit = if rng.bit() { v } else { -v };
            // avoid a variable and its negation, and duplicates, in one clause
            if !clause.iter().any(|&l: &i32| l.abs() == v) {
                clause.push(lit);
            }
        }
        clauses.push(clause);
    }
    Cnf { num_vars, clauses }
}

#[test]
fn random_3sat_cross_checked_against_brute_force() {
    // solve and solve_brute must agree on satisfiability, and each SAT result
    // must be a genuine model. Over the phase transition (clauses ≈ 4.26 n)
    // we get a healthy mix of SAT and UNSAT instances.
    let mut rng = Lcg(0x1234_5678_9abc_def0);
    let mut sat_count = 0;
    let mut unsat_count = 0;
    for _ in 0..200 {
        let num_vars = 3 + (rng.below(10) as usize); // 3..=12 vars
        // Sweep the clause/variable ratio from sparse (mostly SAT) through the
        // phase transition to dense (mostly UNSAT), so the sample exercises
        // both outcomes.
        let ratio = 3 + rng.below(5); // 3..=7 clauses per variable
        let num_clauses = num_vars * (ratio as usize);
        let cnf = random_3sat(&mut rng, num_vars, num_clauses);

        let s = solve(&cnf);
        let b = solve_brute(&cnf);
        assert_eq!(
            s.is_some(),
            b.is_some(),
            "solve and solve_brute disagree on satisfiability of {cnf:?}"
        );
        if let Some(model) = &s {
            assert!(cnf.evaluate(model), "solve returned a non-model for {cnf:?}");
            sat_count += 1;
        } else {
            unsat_count += 1;
        }
    }
    // Sanity: the sample really did exercise both outcomes.
    assert!(sat_count > 0 && unsat_count > 0, "sat={sat_count} unsat={unsat_count}");
}
