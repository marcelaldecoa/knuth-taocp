//! Stage 4 — The complete CDCL solver (Algorithm 7.2.2.2C, simplified).
//!
//! Implement `solve`, `solve_brute`, `pigeonhole_cnf` and `waerden_cnf` in
//! src/lab.rs. The lesson: course/module-14-cdcl/README.md, §6–§7.
//!
//! Every SAT answer is verified *semantically* (the model must evaluate the
//! formula to true); every UNSAT answer is cross-checked against brute force
//! where feasible, and against known theorems (W(3,3) = 9, pigeonhole)
//! where not.

use lab_14_cdcl::{pigeonhole_cnf, solve, solve_brute, waerden_cnf, Cnf};

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// A random 3-SAT instance with `n` variables and `m` clauses, three
/// *distinct* variables per clause, random polarities.
fn random_3sat(rng: &mut Lcg, n: usize, m: usize) -> Cnf {
    let mut clauses = Vec::with_capacity(m);
    for _ in 0..m {
        let mut vars: Vec<usize> = Vec::new();
        while vars.len() < 3 {
            let v = 1 + (rng.next() as usize) % n;
            if !vars.contains(&v) {
                vars.push(v);
            }
        }
        clauses.push(
            vars.iter()
                .map(|&v| if rng.next() % 2 == 0 { v as i32 } else { -(v as i32) })
                .collect(),
        );
    }
    Cnf { num_vars: n, clauses }
}

#[test]
fn models_are_verified_semantically() {
    // The empty formula is satisfiable (by the empty assignment).
    let empty = Cnf { num_vars: 0, clauses: vec![] };
    assert_eq!(solve(&empty), Some(vec![]));

    // A formula containing the empty clause is unsatisfiable.
    let dead = Cnf { num_vars: 2, clauses: vec![vec![1, 2], vec![]] };
    assert_eq!(solve(&dead), None);

    // Units, chains, and a forced total assignment.
    let cnf = Cnf {
        num_vars: 4,
        clauses: vec![vec![1], vec![-1, -2], vec![2, 3], vec![-3, 4]],
    };
    let model = solve(&cnf).expect("satisfiable");
    assert_eq!(model.len(), 4);
    assert!(cnf.evaluate(&model));
    assert_eq!(model, vec![true, false, true, true], "every value is forced");

    // (x1)(¬x1): the smallest UNSAT formula with a nonempty clause set.
    let tiny_unsat = Cnf { num_vars: 1, clauses: vec![vec![1], vec![-1]] };
    assert_eq!(solve(&tiny_unsat), None);
}

#[test]
fn exhaustive_agreement_on_all_two_variable_formulas() {
    // Every one of the 2^8 = 256 formulas whose clauses are drawn from the
    // eight non-tautological clauses over {x1, x2}. Exhaustive: verdicts
    // must match brute force exactly, and models must check out.
    let pool: [&[i32]; 8] = [
        &[1],
        &[-1],
        &[2],
        &[-2],
        &[1, 2],
        &[1, -2],
        &[-1, 2],
        &[-1, -2],
    ];
    for mask in 0u32..256 {
        let clauses: Vec<Vec<i32>> = (0..8)
            .filter(|i| mask >> i & 1 == 1)
            .map(|i| pool[i].to_vec())
            .collect();
        let cnf = Cnf { num_vars: 2, clauses };
        match (solve(&cnf), solve_brute(&cnf)) {
            (Some(model), Some(_)) => {
                assert!(cnf.evaluate(&model), "mask {mask}: bogus model {model:?}")
            }
            (None, None) => {}
            (got, want) => panic!(
                "mask {mask:#010b}: solve says {}, brute force says {}",
                if got.is_some() { "SAT" } else { "UNSAT" },
                if want.is_some() { "SAT" } else { "UNSAT" },
            ),
        }
    }
}

#[test]
fn random_3sat_agrees_with_brute_force() {
    // ~200 deterministic instances, n <= 16, clause/variable ratios from
    // easy-SAT (2) through the hard region (~4.27) to easy-UNSAT (6).
    let mut rng = Lcg(0x14CDC1);
    let mut sat = 0;
    let mut unsat = 0;
    for trial in 0..200usize {
        let n = 5 + trial % 12; // 5..=16
        let ratio = 2 + trial % 5; // 2..=6
        let cnf = random_3sat(&mut rng, n, n * ratio);
        match (solve(&cnf), solve_brute(&cnf)) {
            (Some(model), Some(_)) => {
                assert!(cnf.evaluate(&model), "trial {trial}: bogus model");
                sat += 1;
            }
            (None, None) => unsat += 1,
            (got, want) => panic!(
                "trial {trial} (n={n}, m={}): solve says {}, brute force says {}",
                n * ratio,
                if got.is_some() { "SAT" } else { "UNSAT" },
                if want.is_some() { "SAT" } else { "UNSAT" },
            ),
        }
    }
    // The mix really exercises both verdicts.
    assert!(sat >= 40, "only {sat} satisfiable instances — bad generator?");
    assert!(unsat >= 40, "only {unsat} unsatisfiable instances — bad generator?");
}

#[test]
fn waerden_running_example() {
    // §7.2.2.2's running example: W(3, 3) = 9. Eight integers can be
    // 2-coloured with no monochromatic 3-term arithmetic progression...
    let sat = waerden_cnf(3, 3, 8);
    let model = solve(&sat).expect("waerden(3,3;8) is satisfiable");
    assert!(sat.evaluate(&model));
    // ... and the colouring genuinely avoids monochromatic 3-term APs.
    for d in 1..=3usize {
        for a in 1..=8 - 2 * d {
            let (x, y, z) = (model[a - 1], model[a + d - 1], model[a + 2 * d - 1]);
            assert!(
                !(x == y && y == z),
                "monochromatic AP {a}, {}, {}",
                a + d,
                a + 2 * d
            );
        }
    }
    // Nine integers cannot.
    assert_eq!(solve(&waerden_cnf(3, 3, 9)), None);
}

#[test]
fn pigeonhole_both_ways() {
    // 4 pigeons fit into 4 holes — and the model is a real matching.
    let php44 = pigeonhole_cnf(4, 4);
    let model = solve(&php44).expect("PHP(4,4) is satisfiable");
    assert!(php44.evaluate(&model));
    // 5 pigeons do not fit into 4 holes.
    assert_eq!(solve(&pigeonhole_cnf(5, 4)), None);
}

#[test]
fn scale_pigeonhole_7_6_unsat() {
    // PHP(7, 6): 42 variables, 133 clauses, and by Haken's theorem every
    // resolution refutation — hence every CDCL run — is "large". At this
    // size a correct learner still finishes in far under the time limit;
    // plain DPLL without learning already feels it.
    assert_eq!(solve(&pigeonhole_cnf(7, 6)), None);
}

#[test]
fn scale_random_3sat_n60_sat_with_verified_model() {
    // 60 variables, 150 clauses (ratio 2.5, comfortably below the ~4.27
    // threshold): far beyond brute force (2^60), routine for CDCL. The
    // instance is fixed by the seed and known satisfiable; any correct
    // solver must find *some* model, which we verify semantically.
    let mut rng = Lcg(0xC0FFEE);
    let cnf = random_3sat(&mut rng, 60, 150);
    let model = solve(&cnf).expect("this seeded instance is satisfiable");
    assert_eq!(model.len(), 60);
    assert!(cnf.evaluate(&model));
}
