//! Stage 4 — Encoding problems into SAT (§7.2.2.2 encodings).
//!
//! Implement `exactly_one`, `queens_cnf`, `decode_queens`, `coloring_cnf`,
//! and `decode_coloring` in src/lab.rs. The lesson:
//! course/module-10-sat/README.md.
//!
//! Every SAT result is validated *semantically* — a decoded queens placement
//! is checked to be legal, a decoded colouring is checked proper — never
//! against a fixed model.

use lab_10_sat::{
    coloring_cnf, decode_coloring, decode_queens, exactly_one, queens_cnf, solve, Cnf,
};

#[test]
fn exactly_one_semantics() {
    // Build a formula that is *just* exactly-one over 3 variables, then check
    // that its models are precisely the three singletons.
    let clauses = exactly_one(&[1, 2, 3]);
    let cnf = Cnf { num_vars: 3, clauses };
    // The three valid singletons.
    assert!(cnf.evaluate(&[true, false, false]));
    assert!(cnf.evaluate(&[false, true, false]));
    assert!(cnf.evaluate(&[false, false, true]));
    // None, and any pair or triple, are rejected.
    assert!(!cnf.evaluate(&[false, false, false]));
    assert!(!cnf.evaluate(&[true, true, false]));
    assert!(!cnf.evaluate(&[true, true, true]));
}

#[test]
fn exactly_one_of_empty_is_unsatisfiable() {
    // "Exactly one of nothing" cannot hold: the encoding contains the empty
    // clause.
    let clauses = exactly_one(&[]);
    assert!(clauses.contains(&vec![]));
    let cnf = Cnf {
        num_vars: 1,
        clauses,
    };
    assert!(!cnf.evaluate(&[true]));
    assert!(!cnf.evaluate(&[false]));
}

#[test]
fn queens_6_is_sat_and_decodes_to_a_valid_placement() {
    let n = 6;
    let cnf = queens_cnf(n);
    let model = solve(&cnf).expect("6-queens is solvable");
    assert!(cnf.evaluate(&model));
    let cols = decode_queens(&model, n);
    assert_eq!(cols.len(), n);
    for &c in &cols {
        assert!(c < n);
    }
    // No two queens share a column or a diagonal (rows differ by construction).
    for r1 in 0..n {
        for r2 in (r1 + 1)..n {
            assert_ne!(cols[r1], cols[r2], "column clash");
            assert_ne!(cols[r1].abs_diff(cols[r2]), r2 - r1, "diagonal clash");
        }
    }
}

#[test]
fn queens_3_is_unsat() {
    assert_eq!(solve(&queens_cnf(3)), None);
}

#[test]
fn queens_1_is_trivially_sat() {
    let cnf = queens_cnf(1);
    let model = solve(&cnf).expect("1-queens is solvable");
    assert!(cnf.evaluate(&model));
    assert_eq!(decode_queens(&model, 1), vec![0]);
}

fn petersen_edges() -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for i in 0..5 {
        edges.push((i, (i + 1) % 5)); // outer 5-cycle
        edges.push((5 + i, 5 + (i + 2) % 5)); // inner pentagram
        edges.push((i, 5 + i)); // spokes
    }
    edges
}

#[test]
fn petersen_is_3_colourable_with_a_proper_colouring() {
    let edges = petersen_edges();
    let cnf = coloring_cnf(10, &edges, 3);
    let model = solve(&cnf).expect("Petersen graph is 3-colourable");
    assert!(cnf.evaluate(&model));
    let colours = decode_coloring(&model, 10, 3);
    assert_eq!(colours.len(), 10);
    for &c in &colours {
        assert!(c < 3);
    }
    for &(u, w) in &edges {
        assert_ne!(colours[u], colours[w], "edge ({u},{w}) is monochromatic");
    }
}

#[test]
fn k4_is_not_2_colourable() {
    let k4: Vec<(usize, usize)> = vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    assert_eq!(solve(&coloring_cnf(4, &k4, 2)), None);
}

#[test]
fn triangle_is_3_colourable_but_not_2_colourable() {
    // K3: 2 colours impossible, 3 colours proper.
    let k3: Vec<(usize, usize)> = vec![(0, 1), (1, 2), (0, 2)];
    assert_eq!(solve(&coloring_cnf(3, &k3, 2)), None);
    let cnf = coloring_cnf(3, &k3, 3);
    let model = solve(&cnf).expect("triangle is 3-colourable");
    let colours = decode_coloring(&model, 3, 3);
    for &(u, w) in &k3 {
        assert_ne!(colours[u], colours[w]);
    }
}
