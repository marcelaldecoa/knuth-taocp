//! Stage 2 — Building reduced ordered BDDs with hash-consing (§7.1.4).
//!
//! Implement `Bdd` (constant/variable/and/or/not/xor/eval/node_count and
//! the arena accessors) in src/lab.rs. The headline property is
//! CANONICITY: two `Ref`s from the same `Bdd` are `==` exactly when they
//! denote the same boolean function.
//! The lesson: course/module-13-bits-bdds/README.md.

use lab_13_bits_bdds::{Bdd, Ref};
use std::collections::HashSet;

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// A random boolean formula, evaluated two ways: directly, and via the BDD.
enum Expr {
    Var(u32),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
}

fn gen_expr(s: &mut u64, n_vars: u32, depth: u32) -> Expr {
    let r = lcg(s) >> 32;
    if depth == 0 || r % 5 == 0 {
        return Expr::Var(((lcg(s) >> 32) as u32) % n_vars);
    }
    let a = Box::new(gen_expr(s, n_vars, depth - 1));
    match r % 4 {
        0 => Expr::Not(a),
        1 => Expr::And(a, Box::new(gen_expr(s, n_vars, depth - 1))),
        2 => Expr::Or(a, Box::new(gen_expr(s, n_vars, depth - 1))),
        _ => Expr::Xor(a, Box::new(gen_expr(s, n_vars, depth - 1))),
    }
}

fn eval_expr(e: &Expr, a: &[bool]) -> bool {
    match e {
        Expr::Var(i) => a[*i as usize],
        Expr::Not(x) => !eval_expr(x, a),
        Expr::And(x, y) => eval_expr(x, a) && eval_expr(y, a),
        Expr::Or(x, y) => eval_expr(x, a) || eval_expr(y, a),
        Expr::Xor(x, y) => eval_expr(x, a) ^ eval_expr(y, a),
    }
}

fn build(bdd: &mut Bdd, e: &Expr) -> Ref {
    match e {
        Expr::Var(i) => bdd.variable(*i),
        Expr::Not(x) => {
            let f = build(bdd, x);
            bdd.not(f)
        }
        Expr::And(x, y) => {
            let (f, g) = (build(bdd, x), build(bdd, y));
            bdd.and(f, g)
        }
        Expr::Or(x, y) => {
            let (f, g) = (build(bdd, x), build(bdd, y));
            bdd.or(f, g)
        }
        Expr::Xor(x, y) => {
            let (f, g) = (build(bdd, x), build(bdd, y));
            bdd.xor(f, g)
        }
    }
}

/// Build ¬e by pushing the negation all the way down with De Morgan's laws
/// — a syntactically alien route to the same function, so ref-equality
/// with `bdd.not(build(e))` is canonicity earning its keep.
fn build_negation(bdd: &mut Bdd, e: &Expr) -> Ref {
    match e {
        Expr::Var(i) => {
            let v = bdd.variable(*i);
            bdd.not(v)
        }
        Expr::Not(x) => build(bdd, x),
        Expr::And(x, y) => {
            // ¬(x ∧ y) = ¬x ∨ ¬y
            let (f, g) = (build_negation(bdd, x), build_negation(bdd, y));
            bdd.or(f, g)
        }
        Expr::Or(x, y) => {
            // ¬(x ∨ y) = ¬x ∧ ¬y
            let (f, g) = (build_negation(bdd, x), build_negation(bdd, y));
            bdd.and(f, g)
        }
        Expr::Xor(x, y) => {
            // ¬(x ⊕ y) = x ⊕ ¬y
            let (f, g) = (build(bdd, x), build_negation(bdd, y));
            bdd.xor(f, g)
        }
    }
}

#[test]
fn sinks_are_the_two_constants() {
    let mut b = Bdd::new();
    let t = b.constant(true);
    let z = b.constant(false);
    assert_ne!(t, z);
    assert!(b.is_terminal(t) && b.is_terminal(z));
    assert_eq!(b.node_count(t), 1);
    assert_eq!(b.node_count(z), 1);
    assert!(b.eval(t, &[]));
    assert!(!b.eval(z, &[]));
    // A variable is a single decision node over the two sinks.
    let x = b.variable(0);
    assert!(!b.is_terminal(x));
    assert_eq!(b.node_count(x), 3);
    assert_eq!(b.low(x), z);
    assert_eq!(b.high(x), t);
    assert_eq!(b.var(x), 0);
}

#[test]
fn canonicity_distributive_law() {
    // (x ∧ y) ∨ (x ∧ z)  ==  x ∧ (y ∨ z): same function, two shapes,
    // one Ref. This is THE theorem of §7.1.4 made executable.
    let mut b = Bdd::new();
    let (x, y, z) = (b.variable(0), b.variable(1), b.variable(2));
    let xy = b.and(x, y);
    let xz = b.and(x, z);
    let lhs = b.or(xy, xz);
    let y_or_z = b.or(y, z);
    let rhs = b.and(x, y_or_z);
    assert_eq!(lhs, rhs, "canonicity: identical Refs for identical functions");
}

#[test]
fn canonicity_de_morgan() {
    let mut b = Bdd::new();
    let (x, y) = (b.variable(0), b.variable(1));
    let xy = b.and(x, y);
    let lhs = b.not(xy);
    let (nx, ny) = (b.not(x), b.not(y));
    let rhs = b.or(nx, ny);
    assert_eq!(lhs, rhs, "¬(x∧y) == ¬x ∨ ¬y as Refs");

    let x_or_y = b.or(x, y);
    let lhs = b.not(x_or_y);
    let rhs = b.and(nx, ny);
    assert_eq!(lhs, rhs, "¬(x∨y) == ¬x ∧ ¬y as Refs");
}

#[test]
fn xor_built_from_and_or_not_is_xor() {
    let mut b = Bdd::new();
    let (x, y) = (b.variable(0), b.variable(1));
    let (nx, ny) = (b.not(x), b.not(y));
    let a1 = b.and(x, ny);
    let a2 = b.and(nx, y);
    let handmade = b.or(a1, a2);
    let builtin = b.xor(x, y);
    assert_eq!(handmade, builtin, "(x∧¬y)∨(¬x∧y) == x⊕y as Refs");
}

#[test]
fn complement_involution_and_constants() {
    let mut b = Bdd::new();
    let (x, y, z) = (b.variable(0), b.variable(1), b.variable(2));
    let xy = b.and(x, y);
    let f = b.xor(xy, z);
    let nf = b.not(f);
    let nnf = b.not(nf);
    assert_eq!(nnf, f, "¬¬f == f");
    assert_ne!(nf, f);
    let t = b.constant(true);
    let z0 = b.constant(false);
    assert_eq!(b.not(t), z0);
    assert_eq!(b.not(z0), t);
}

#[test]
fn tautology_and_contradiction_collapse_to_sinks() {
    let mut b = Bdd::new();
    let x = b.variable(4);
    let nx = b.not(x);
    let taut = b.or(x, nx);
    assert_eq!(taut, b.constant(true), "x ∨ ¬x is ⊤ itself, not merely equivalent");
    let contra = b.and(x, nx);
    assert_eq!(contra, b.constant(false));
    // Excluded middle for a composite f too.
    let y = b.variable(1);
    let f = b.xor(x, y);
    let nf = b.not(f);
    let taut = b.or(f, nf);
    assert_eq!(taut, b.constant(true));
}

#[test]
fn arena_wide_reduction_invariants() {
    // Build a pile of random formulas, then audit every node in the arena:
    // (1) no redundant test (lo == hi), (2) no duplicate (var, lo, hi),
    // (3) ordered: children test strictly larger variables (or are sinks).
    let mut b = Bdd::new();
    let mut s = 0xB1D_5EEDu64;
    for _ in 0..40 {
        let e = gen_expr(&mut s, 6, 5);
        let _ = build(&mut b, &e);
    }
    assert!(b.len() > 10, "sanity: the formulas made some nodes");
    let mut triples = HashSet::new();
    for i in 0..b.len() as u32 {
        let r = Ref(i);
        if b.is_terminal(r) {
            continue;
        }
        let (v, lo, hi) = (b.var(r), b.low(r), b.high(r));
        assert_ne!(lo, hi, "node {i}: redundant test survived reduction");
        assert!(
            triples.insert((v, lo, hi)),
            "node {i}: duplicate (var, lo, hi) — unique table broken"
        );
        for child in [lo, hi] {
            if !b.is_terminal(child) {
                assert!(
                    b.var(child) > v,
                    "node {i}: child tests x{} ≤ x{v} — ordering violated",
                    b.var(child)
                );
            }
        }
    }
}

#[test]
fn eval_agrees_with_direct_evaluation_exhaustively() {
    // Random formulas on up to 10 variables; check every assignment.
    let mut s = 0xC0FFEEu64;
    for n_vars in 1..=10u32 {
        for _ in 0..8 {
            let e = gen_expr(&mut s, n_vars, 6);
            let mut b = Bdd::new();
            let f = build(&mut b, &e);
            let mut a = vec![false; n_vars as usize];
            for bits in 0..(1u32 << n_vars) {
                for (i, ai) in a.iter_mut().enumerate() {
                    *ai = bits >> i & 1 == 1;
                }
                assert_eq!(
                    b.eval(f, &a),
                    eval_expr(&e, &a),
                    "n_vars={n_vars}, assignment={bits:#b}"
                );
            }
        }
    }
}

#[test]
fn canonicity_on_random_formulas_via_de_morgan_pushdown() {
    // Two syntactically alien constructions of ¬e must collide on one Ref.
    let mut s = 0xDECAFu64;
    for _ in 0..30 {
        let e = gen_expr(&mut s, 7, 6);
        let mut b = Bdd::new();
        let f = build(&mut b, &e);
        let nf = b.not(f);
        let pushed = build_negation(&mut b, &e);
        assert_eq!(nf, pushed, "canonicity under De Morgan push-down");
    }
}
