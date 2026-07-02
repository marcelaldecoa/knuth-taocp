//! Stage 1 — Dual lattice bases in dimension t (§3.3.4, setup of
//! Algorithm S, steps S1–S3).
//!
//! The spectral problem lives on a PAIR of lattices: V generates the dual
//! lattice { u : u1 + a·u2 + ... + a^{t-1}·ut ≡ 0 (mod m) }, U generates
//! the m-scaled primal t-tuple lattice, and the two are tied by the exact
//! identity U·Vᵀ = m·I — the invariant that certifies every later
//! transformation. This stage builds the pair and the checker.

use lab_16_spectral_hd::{check_duality, dual_basis, primal_basis};

const RANDU_A: i64 = 65539;
const RANDU_M: i64 = 1 << 31;
const MM: i64 = (1 << 31) - 1; // 2^31 - 1, prime

/// Module 12's dual-vector predicate, re-implemented here so this stage
/// does not depend on later ones: u ≠ 0 and
/// u1 + a·u2 + ... + a^{t-1}·ut ≡ 0 (mod m), everything reduced as we go.
fn is_dual_vector(u: &[i128], a: i64, m: i64) -> bool {
    if u.iter().all(|&ui| ui == 0) {
        return false;
    }
    let (a, m) = (a as i128, m as i128);
    let mut pow = 1i128; // a^i mod m
    let mut s = 0i128;
    for &ui in u {
        s = (s + ui.rem_euclid(m) * pow).rem_euclid(m);
        pow = pow * a % m;
    }
    s == 0
}

/// Exact determinant by cofactor expansion, t <= 3 only — small enough to
/// be obviously correct, which is the point of a test-side oracle.
fn det_small(v: &[Vec<i128>]) -> i128 {
    match v.len() {
        1 => v[0][0],
        2 => v[0][0] * v[1][1] - v[0][1] * v[1][0],
        3 => {
            v[0][0] * (v[1][1] * v[2][2] - v[1][2] * v[2][1])
                - v[0][1] * (v[1][0] * v[2][2] - v[1][2] * v[2][0])
                + v[0][2] * (v[1][0] * v[2][1] - v[1][1] * v[2][0])
        }
        _ => panic!("det_small handles t <= 3 only"),
    }
}

#[test]
fn duality_identity_holds_for_classic_generators() {
    // U·Vᵀ = m·I for RANDU (t = 3, 4), the minimal standard and its
    // Park–Miller revision up to t = 6, and the toy generator of the
    // lesson's hand traces.
    for &(a, m, tmax) in &[
        (RANDU_A, RANDU_M, 4),
        (16807, MM, 6),
        (48271, MM, 6),
        (137, 256, 6),
    ] {
        for t in 2..=tmax {
            let u = primal_basis(a, m, t);
            let v = dual_basis(a, m, t);
            assert_eq!(u.len(), t, "U must be t x t");
            assert_eq!(v.len(), t, "V must be t x t");
            assert!(
                check_duality(&u, &v, m),
                "U·Vᵀ != m·I for a={a}, m={m}, t={t}"
            );
        }
    }
}

#[test]
fn every_dual_row_satisfies_the_congruence() {
    // The rows of V must actually BE dual vectors — checked against an
    // independent re-implementation of module 12's predicate.
    for &(a, m) in &[(RANDU_A, RANDU_M), (16807, MM), (48271, MM), (137, 256), (7, 11)] {
        for t in 2..=6 {
            for (i, row) in dual_basis(a, m, t).iter().enumerate() {
                assert!(
                    is_dual_vector(row, a, m),
                    "row {i} of V is not dual for a={a}, m={m}, t={t}"
                );
            }
        }
    }
}

#[test]
fn bases_match_the_documented_construction() {
    // Hand-checkable instance a = 137, m = 256, t = 3:
    // a^2 = 18769 = 73·256 + 81, so the reduced power is 81.
    assert_eq!(
        dual_basis(137, 256, 3),
        vec![vec![256, 0, 0], vec![-137, 1, 0], vec![-81, 0, 1]]
    );
    assert_eq!(
        primal_basis(137, 256, 3),
        vec![vec![1, 137, 81], vec![0, 256, 0], vec![0, 0, 256]]
    );
    // Powers must be REDUCED mod m: for RANDU a^2 = 4295360521 but the
    // stored entry is -(a^2 mod 2^31) = -393225 (= -(6a - 9), the algebra
    // behind the (9, -6, 1) catastrophe).
    let v = dual_basis(RANDU_A, RANDU_M, 3);
    assert_eq!(v[2][0], -393_225);
    assert_eq!(v[2][2], 1);
}

#[test]
fn determinant_of_the_dual_basis_is_m() {
    // det V = m: the dual lattice has index m in Z^t (one congruence
    // mod m). The documented V is lower triangular with diagonal
    // (m, 1, ..., 1), so an independent cofactor expansion must give m.
    for &(a, m) in &[(137i64, 256i64), (5, 8), (21, 100), (16807, MM), (RANDU_A, RANDU_M)] {
        for t in 2..=3 {
            let v = dual_basis(a, m, t);
            assert_eq!(det_small(&v), m as i128, "det V != m for a={a}, m={m}, t={t}");
        }
    }
}

#[test]
fn check_duality_rejects_broken_pairs() {
    let u = primal_basis(137, 256, 4);
    let mut v = dual_basis(137, 256, 4);
    assert!(check_duality(&u, &v, 256));
    // Wrong modulus: the diagonal of U·Vᵀ is 256, not 255 or 257.
    assert!(!check_duality(&u, &v, 255));
    assert!(!check_duality(&u, &v, 257));
    // A single perturbed entry must be caught.
    v[2][0] += 1;
    assert!(!check_duality(&u, &v, 256));
    v[2][0] -= 1;
    assert!(check_duality(&u, &v, 256));
    // Shape mismatch (4x4 against 3x3) is rejected, not a panic.
    assert!(!check_duality(&u, &dual_basis(137, 256, 3), 256));
}
