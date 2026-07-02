//! Stage 1 — Conjunctive normal form and DIMACS (§7.2.2.2 representations).
//!
//! Implement `Cnf::parse_dimacs`, `Cnf::to_dimacs`, and `Cnf::evaluate` in
//! src/lab.rs. The lesson: course/module-10-sat/README.md.

use lab_10_sat::Cnf;

#[test]
fn parse_with_comments_and_whitespace() {
    // §7.2.2.2 uses DIMACS everywhere; comments (c ...) and free whitespace
    // are legal and must be tolerated.
    let text = "c a comment\nc another\np cnf 3 2\n1 -2 0\n2 3 0\n";
    let cnf = Cnf::parse_dimacs(text).unwrap();
    assert_eq!(cnf.num_vars, 3);
    assert_eq!(cnf.clauses, vec![vec![1, -2], vec![2, 3]]);
}

#[test]
fn clauses_may_span_and_share_lines() {
    // Any whitespace separates tokens; a clause may wrap across lines and two
    // clauses may sit on one line.
    let text = "p cnf 4 3\n1 2\n3 0 -1 -2 0\n\t4 0\n";
    let cnf = Cnf::parse_dimacs(text).unwrap();
    assert_eq!(cnf.clauses, vec![vec![1, 2, 3], vec![-1, -2], vec![4]]);
}

#[test]
fn round_trip_parse_of_to_dimacs() {
    // Lossless: parse(to_dimacs(x)) == Ok(x) for well-formed x.
    let cnf = Cnf {
        num_vars: 5,
        clauses: vec![vec![1, -2, 3], vec![-4, 5], vec![2]],
    };
    let text = cnf.to_dimacs();
    assert_eq!(Cnf::parse_dimacs(&text), Ok(cnf));
}

#[test]
fn evaluate_hand_example() {
    // (x1 ∨ ¬x2) ∧ (x2 ∨ x3): check three assignments by hand.
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![1, -2], vec![2, 3]],
    };
    assert!(cnf.evaluate(&[true, true, false]));
    assert!(cnf.evaluate(&[false, false, true]));
    assert!(!cnf.evaluate(&[false, true, false]));
}

#[test]
fn empty_formula_is_true() {
    // A conjunction of nothing is vacuously satisfied by anything.
    let cnf = Cnf {
        num_vars: 2,
        clauses: vec![],
    };
    assert!(cnf.evaluate(&[false, false]));
    assert!(cnf.evaluate(&[true, true]));
}

#[test]
fn empty_clause_is_unsatisfiable() {
    // A disjunction of nothing is false, so a formula containing the empty
    // clause is false under every assignment. A bare `0` parses to one.
    let cnf = Cnf::parse_dimacs("p cnf 2 1\n0\n").unwrap();
    assert_eq!(cnf.clauses, vec![vec![]]);
    assert!(!cnf.evaluate(&[true, true]));
    assert!(!cnf.evaluate(&[false, false]));
}

#[test]
fn evaluate_ignores_extra_trailing_assignment() {
    // evaluate accepts an assignment covering at least num_vars variables.
    let cnf = Cnf {
        num_vars: 1,
        clauses: vec![vec![1]],
    };
    assert!(cnf.evaluate(&[true, false, true]));
}

#[test]
fn malformed_missing_header_is_err() {
    assert!(Cnf::parse_dimacs("1 2 0\n").is_err());
}

#[test]
fn malformed_duplicate_header_is_err() {
    assert!(Cnf::parse_dimacs("p cnf 2 1\np cnf 2 1\n1 0\n").is_err());
}

#[test]
fn malformed_bad_header_shape_is_err() {
    assert!(Cnf::parse_dimacs("p cnf 3\n1 0\n").is_err());
    assert!(Cnf::parse_dimacs("p sat 3 1\n1 0\n").is_err());
}

#[test]
fn non_integer_token_is_err() {
    assert!(Cnf::parse_dimacs("p cnf 3 1\n1 two 0\n").is_err());
}

#[test]
fn literal_out_of_range_is_err() {
    // Header declares 3 variables; literal 4 is illegal.
    assert!(Cnf::parse_dimacs("p cnf 3 1\n1 4 0\n").is_err());
}

#[test]
fn unterminated_final_clause_is_err() {
    // Missing trailing 0.
    assert!(Cnf::parse_dimacs("p cnf 3 1\n1 2 3\n").is_err());
}

#[test]
fn wrong_clause_count_is_err() {
    // Header promises 2 clauses but only 1 is present.
    assert!(Cnf::parse_dimacs("p cnf 3 2\n1 0\n").is_err());
}

#[test]
#[should_panic(expected = "covers")]
fn evaluate_panics_on_short_assignment() {
    let cnf = Cnf {
        num_vars: 3,
        clauses: vec![vec![1]],
    };
    cnf.evaluate(&[true, false]);
}
