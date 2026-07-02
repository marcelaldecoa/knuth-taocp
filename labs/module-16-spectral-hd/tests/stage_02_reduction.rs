//! Stage 2 — Basis reduction by unimodular transformations (§3.3.4,
//! Algorithm S, transformation steps S5–S7).
//!
//! Pairwise size reduction: whenever 2|v_i·v_j| > |v_j|², replace
//! v_i <- v_i - q·v_j with q = round(v_i·v_j / v_j·v_j), and mirror the
//! step on the primal basis as u_j <- u_j + q·u_i so that U·Vᵀ = m·I
//! survives — THE invariant of this module. Termination: every applied
//! transformation strictly decreases Σ|v_i|², a positive integer.

use lab_16_spectral_hd::{check_duality, dual_basis, primal_basis, reduce_basis};

const RANDU_A: i64 = 65539;
const RANDU_M: i64 = 1 << 31;
const MM: i64 = (1 << 31) - 1;

fn norm_sq(row: &[i128]) -> i128 {
    row.iter().map(|&x| x * x).sum()
}

fn total_norm_sq(v: &[Vec<i128>]) -> i128 {
    v.iter().map(|r| norm_sq(r)).sum()
}

fn min_row_norm_sq(v: &[Vec<i128>]) -> i128 {
    v.iter().map(|r| norm_sq(r)).min().unwrap()
}

/// Module 12's Hermite-bounded brute force for nu_2^2, re-implemented as
/// this stage's independent oracle: nu_2^2 <= (2/√3)·m, so any dual vector
/// with |u_2| > sqrt((2/√3)·m) is already longer than the bound, and
/// scanning u_2 in [1, B] with the centered residue for u_1 is exhaustive.
fn nu2_squared_brute(a: i64, m: i64) -> i128 {
    let (a, m) = (a as i128, m as i128);
    let b = ((2.0 / 3.0f64.sqrt()) * m as f64).sqrt().ceil() as i128;
    let mut best = m * m; // class u_2 = 0: shortest nonzero is (m, 0)
    for u2 in 1..=b {
        let r = (a * u2).rem_euclid(m);
        let u1 = if 2 * r > m { m - r } else { -r };
        best = best.min(u1 * u1 + u2 * u2);
    }
    best
}

#[test]
fn duality_survives_reduction_the_invariant() {
    // The whole point of the mirrored update u_j += q·u_i: after any number
    // of transformations, U·Vᵀ = m·I must still hold exactly.
    for &(a, m, tmax) in &[
        (RANDU_A, RANDU_M, 6),
        (16807, MM, 6),
        (48271, MM, 6),
        (137, 256, 6),
        (21, 100, 4),
    ] {
        for t in 2..=tmax {
            let mut v = dual_basis(a, m, t);
            let mut u = primal_basis(a, m, t);
            reduce_basis(&mut v, &mut u);
            assert!(
                check_duality(&u, &v, m),
                "invariant U·Vᵀ = m·I lost for a={a}, m={m}, t={t}"
            );
        }
    }
}

#[test]
fn total_squared_norm_never_increases_and_usually_collapses() {
    for &(a, m) in &[(RANDU_A, RANDU_M), (16807, MM), (48271, MM), (137, 256)] {
        for t in 2..=6 {
            let mut v = dual_basis(a, m, t);
            let mut u = primal_basis(a, m, t);
            let before = total_norm_sq(&v);
            reduce_basis(&mut v, &mut u);
            let after = total_norm_sq(&v);
            // The termination argument in executable form: each applied
            // transformation strictly decreases the total, so the final
            // total is at most the initial one...
            assert!(after <= before, "total norm grew for a={a}, m={m}, t={t}");
            // ...and for these generators the unreduced V contains the row
            // (-(a^{t-1} mod m), 0, ..., 1), so reduction has real work to
            // do and the total must strictly drop.
            assert!(after < before, "reduction did nothing for a={a}, m={m}, t={t}");
        }
    }
}

#[test]
fn t2_fixpoint_reproduces_module_12_nu2_squared() {
    // At a fixpoint of pairwise reduction with t = 2, both orderings give
    // 2|v_1·v_2| <= min(|v_1|², |v_2|²) — exactly the Gauss–Lagrange
    // reduced condition, so the shortest ROW is a shortest lattice VECTOR
    // (module 12's optimality proof). Values frozen by module 12:
    for &(a, m, want) in &[
        (137, 256, 274i128),                    // module 12's hand trace
        (16807, MM, 16807i128 * 16807 + 1),     // minimal standard: 282_475_250
        (48271, MM, 1_990_735_345),             // Park–Miller revision
        (RANDU_A, RANDU_M, 2_147_221_514),      // RANDU pairs are nearly perfect!
    ] {
        let mut v = dual_basis(a, m, 2);
        let mut u = primal_basis(a, m, 2);
        reduce_basis(&mut v, &mut u);
        assert_eq!(
            min_row_norm_sq(&v),
            want,
            "t=2 reduced minimum wrong for a={a}, m={m}"
        );
    }
}

#[test]
fn t2_fixpoint_matches_brute_force_on_a_grid() {
    // Same statement, adversarially: every (a, m) with m <= 64 against the
    // independent Hermite-bounded brute force.
    for m in 2..=64i64 {
        for a in 1..m {
            let mut v = dual_basis(a, m, 2);
            let mut u = primal_basis(a, m, 2);
            reduce_basis(&mut v, &mut u);
            assert_eq!(
                min_row_norm_sq(&v),
                nu2_squared_brute(a, m),
                "nu_2^2 mismatch at a={a}, m={m}"
            );
        }
    }
}

#[test]
fn randu_reduction_collapses_the_basis() {
    // The lesson's hand trace: for RANDU at t = 3 the unreduced V has rows
    // of norm ~2^31; a few transformations expose the catastrophic dual
    // vector (9, -6, 1). Any sweep order reaching a fixpoint must get the
    // basis down to this scale (we don't insist the minimum row equals 118
    // here — that certification is stage 3's job — only that reduction
    // collapsed the basis by orders of magnitude).
    let mut v = dual_basis(RANDU_A, RANDU_M, 3);
    let mut u = primal_basis(RANDU_A, RANDU_M, 3);
    reduce_basis(&mut v, &mut u);
    assert!(check_duality(&u, &v, RANDU_M));
    let min_row = min_row_norm_sq(&v);
    assert!(
        min_row < 118_000,
        "RANDU t=3 reduction left min row norm^2 = {min_row}; expected a collapse toward 118"
    );
    assert!(min_row >= 118, "shorter than nu_3^2 = 118 is impossible");
}

#[test]
fn reduction_is_idempotent() {
    // A fixpoint is a fixpoint: reducing an already-reduced pair must
    // change nothing at all (this also catches sweeps that "transform"
    // on the tie 2|d| = n and would never terminate).
    for &(a, m, t) in &[(137i64, 256i64, 4usize), (16807, MM, 5), (48271, MM, 6)] {
        let mut v = dual_basis(a, m, t);
        let mut u = primal_basis(a, m, t);
        reduce_basis(&mut v, &mut u);
        let (v1, u1) = (v.clone(), u.clone());
        reduce_basis(&mut v, &mut u);
        assert_eq!(v, v1, "second reduction changed V for a={a}, m={m}, t={t}");
        assert_eq!(u, u1, "second reduction changed U for a={a}, m={m}, t={t}");
    }
}

#[test]
fn reduced_rows_are_pairwise_size_reduced() {
    // The fixpoint condition itself: 2|v_i·v_j| <= |v_j|² for every pair
    // i != j. This is what "size-reduced" means and what makes stage 3's
    // search box small.
    for &(a, m, t) in &[(137i64, 256i64, 3usize), (16807, MM, 4), (48271, MM, 6)] {
        let mut v = dual_basis(a, m, t);
        let mut u = primal_basis(a, m, t);
        reduce_basis(&mut v, &mut u);
        for j in 0..t {
            let nj = norm_sq(&v[j]);
            for i in 0..t {
                if i == j {
                    continue;
                }
                let d: i128 = v[i].iter().zip(&v[j]).map(|(&x, &y)| x * y).sum();
                assert!(
                    2 * d.abs() <= nj,
                    "pair ({i},{j}) not size-reduced for a={a}, m={m}, t={t}"
                );
            }
        }
    }
}
