//! Stage 3 — Model counting and the ordering problem (Algorithm 7.1.4C).
//!
//! Implement `Bdd::count_models` in src/lab.rs: the number of satisfying
//! assignments over `n_vars` variables, with the 2^k weighting for skipped
//! levels. Then run THE ordering experiment: the same function, under two
//! variable orders, with BDD sizes 18 versus > 256.
//! The lesson: course/module-13-bits-bdds/README.md.

use lab_13_bits_bdds::{Bdd, Ref};

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

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

/// Pascal's triangle row n (binomial coefficients), computed exactly.
fn binomials(n: usize) -> Vec<u128> {
    let mut row = vec![1u128];
    for _ in 0..n {
        let mut next = vec![1u128; row.len() + 1];
        for i in 1..row.len() {
            next[i] = row[i - 1] + row[i];
        }
        row = next;
    }
    row
}

#[test]
fn constants_count_all_or_nothing() {
    let b = Bdd::new();
    let t = b.constant(true);
    let z = b.constant(false);
    for n in 0..=20 {
        assert_eq!(b.count_models(t, n), 1u128 << n, "⊤ over {n} vars");
        assert_eq!(b.count_models(z, n), 0, "⊥ over {n} vars");
    }
}

#[test]
fn skipped_levels_contribute_powers_of_two() {
    // f = x3 over 10 variables: the root skips x0..x2 and the sink edges
    // skip x4..x9, so the count must be 2^9 — the 2^k weighting at work.
    let mut b = Bdd::new();
    let x3 = b.variable(3);
    assert_eq!(b.count_models(x3, 10), 1 << 9);
    assert_eq!(b.count_models(x3, 4), 1 << 3);
    // x3 ∧ x7 over 10 variables: 8 free variables remain.
    let x7 = b.variable(7);
    let f = b.and(x3, x7);
    assert_eq!(b.count_models(f, 10), 1 << 8);
    // Same BDD, wider universe: every extra variable doubles the count.
    assert_eq!(b.count_models(f, 20), 1 << 18);
}

#[test]
fn exactly_k_of_10_counts_binomially() {
    // Build "exactly k of x0..x9" for all k by dynamic programming over
    // BDDs, then check count_models against C(10, k).
    let n = 10usize;
    let mut b = Bdd::new();
    let f = b.constant(false);
    let t = b.constant(true);
    // s[j] = "exactly j of the variables seen so far are true"
    let mut s: Vec<Ref> = vec![f; n + 1];
    s[0] = t;
    for i in 0..n {
        let xi = b.variable(i as u32);
        let nxi = b.not(xi);
        for j in (0..=n).rev() {
            let keep = b.and(s[j], nxi);
            let take = if j > 0 { b.and(s[j - 1], xi) } else { f };
            s[j] = b.or(keep, take);
        }
    }
    let c = binomials(n);
    for (k, &sk) in s.iter().enumerate() {
        assert_eq!(b.count_models(sk, n), c[k], "exactly {k} of {n}");
    }
    // And the counts exhaust the space: sum C(n,k) = 2^n.
    assert_eq!(c.iter().sum::<u128>(), 1 << n);
}

#[test]
fn parity_has_exactly_half_the_assignments() {
    // x0 ⊕ x1 ⊕ … ⊕ x_{n-1} is satisfied by exactly 2^(n-1) assignments,
    // yet its BDD has only about 2n nodes — counting beats enumerating.
    for n in 1..=16u32 {
        let mut b = Bdd::new();
        let mut f = b.constant(false);
        for i in 0..n {
            let xi = b.variable(i);
            f = b.xor(f, xi);
        }
        assert_eq!(b.count_models(f, n as usize), 1u128 << (n - 1), "parity of {n}");
    }
}

#[test]
fn count_agrees_with_brute_force_enumeration() {
    // Random formulas on up to 12 variables: count_models must equal the
    // number of satisfying assignments found by trying all 2^n of them.
    let mut s = 0x5EED_C0DEu64;
    for n_vars in 2..=12u32 {
        for _ in 0..4 {
            let e = gen_expr(&mut s, n_vars, 6);
            let mut b = Bdd::new();
            let f = build(&mut b, &e);
            let mut brute = 0u128;
            let mut a = vec![false; n_vars as usize];
            for bits in 0..(1u32 << n_vars) {
                for (i, ai) in a.iter_mut().enumerate() {
                    *ai = bits >> i & 1 == 1;
                }
                if eval_expr(&e, &a) {
                    brute += 1;
                }
            }
            assert_eq!(b.count_models(f, n_vars as usize), brute, "n_vars={n_vars}");
        }
    }
}

#[test]
fn the_ordering_experiment() {
    // One function, two variable orders, k = 8 pairs over 16 variables:
    //   f = (x_{p(1)} ∧ x_{p(2)}) ∨ … ∨ (x_{p(15)} ∧ x_{p(16)}).
    //
    // Interleaved order — each pair adjacent (2i, 2i+1): the BDD stays
    // tiny, 2k + 2 nodes. Bad order — all first elements before all second
    // elements, pairs (i, 8+i): some level must distinguish all 2^k subsets
    // of pending pairs, so the BDD exceeds 2^k = 256 nodes. Same function
    // count both times: 2^16 − 3^8 = 58975.
    let k = 8u32;
    let expected_count = (1u128 << 16) - 3u128.pow(8);

    let mut good = Bdd::new();
    let mut f = good.constant(false);
    for i in 0..k {
        let a = good.variable(2 * i);
        let b = good.variable(2 * i + 1);
        let ab = good.and(a, b);
        f = good.or(f, ab);
    }
    assert_eq!(good.count_models(f, 16), expected_count);
    assert!(
        good.node_count(f) <= 20,
        "interleaved order must stay near 2k + 2 = 18 nodes, got {}",
        good.node_count(f)
    );

    let mut bad = Bdd::new();
    let mut g = bad.constant(false);
    for i in 0..k {
        let a = bad.variable(i);
        let b = bad.variable(k + i);
        let ab = bad.and(a, b);
        g = bad.or(g, ab);
    }
    assert_eq!(bad.count_models(g, 16), expected_count, "same function!");
    assert!(
        bad.node_count(g) > 256,
        "separated order must blow past 2^k = 256 nodes, got {}",
        bad.node_count(g)
    );
}
