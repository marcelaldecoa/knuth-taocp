//! Stage 3 — The certified exhaustive search (§3.3.4, Algorithm S, search
//! steps S8–S10).
//!
//! After reduction the rows are short but not provably shortest (for
//! t >= 3 no cheap reduction guarantees the minimum). The finisher is an
//! enumeration whose box is CERTIFIED by Cramer's rule + Cauchy–Schwarz:
//! every lattice vector at least as short as the best row has coefficients
//! |x_i| <= sqrt(best·|g_i|²)/|det V| (g_i = adjugate column), so scanning
//! that finite box provably sees the true minimum.

use lab_16_spectral_hd::{dual_basis, primal_basis, reduce_basis, shortest_vector_squared};

const RANDU_A: i64 = 65539;
const RANDU_M: i64 = 1 << 31;

/// The full pipeline this stage certifies: dual pair -> reduce -> search.
fn pipeline(a: i64, m: i64, t: usize) -> i128 {
    let mut v = dual_basis(a, m, t);
    let mut u = primal_basis(a, m, t);
    reduce_basis(&mut v, &mut u);
    shortest_vector_squared(&v)
}

/// Module 12's Hermite-bounded brute force for nu_2^2 (independent oracle).
fn nu2_squared_brute(a: i64, m: i64) -> i128 {
    let (a, m) = (a as i128, m as i128);
    let b = ((2.0 / 3.0f64.sqrt()) * m as f64).sqrt().ceil() as i128;
    let mut best = m * m;
    for u2 in 1..=b {
        let r = (a * u2).rem_euclid(m);
        let u1 = if 2 * r > m { m - r } else { -r };
        best = best.min(u1 * u1 + u2 * u2);
    }
    best
}

/// Full brute force for nu_4^2, tiny m only: scan every residue class
/// (u_2, u_3, u_4) in [-m, m]^3 with the centered residue for u_1 —
/// classes repeat mod m, nu_4^2 <= m^2 always (witness (m, 0, 0, 0)), and
/// the centered u_1 is optimal within its class. Slow, obviously correct.
fn nu4_squared_brute_tiny(a: i64, m: i64) -> i128 {
    let (a, m) = (a as i128, m as i128);
    let a2 = a * a % m;
    let a3 = a2 * a % m;
    let mut best = m * m;
    for u4 in -m..=m {
        for u3 in -m..=m {
            for u2 in -m..=m {
                if (u2, u3, u4) == (0, 0, 0) {
                    continue;
                }
                let r = (a * u2 + a2 * u3 + a3 * u4).rem_euclid(m);
                let u1 = if 2 * r > m { m - r } else { -r };
                best = best.min(u1 * u1 + u2 * u2 + u3 * u3 + u4 * u4);
            }
        }
    }
    best
}

#[test]
fn t2_certified_search_matches_module_12_on_a_grid() {
    // In two dimensions the certified search must agree with an
    // exhaustively verified nu_2^2 for every (a, m), m <= 64.
    for m in 2..=64i64 {
        for a in 1..m {
            assert_eq!(
                pipeline(a, m, 2),
                nu2_squared_brute(a, m),
                "nu_2^2 mismatch at a={a}, m={m}"
            );
        }
    }
}

#[test]
fn randu_t3_is_exactly_118() {
    // The most famous number of the module: RANDU's triples fall on ~15
    // planes because nu_3^2 = 9^2 + 6^2 + 1^2 = 118, witnessed by
    // (9, -6, 1). Frozen by module 12's certified 3-D search.
    assert_eq!(pipeline(RANDU_A, RANDU_M, 3), 118);
}

#[test]
fn t4_matches_full_brute_force_on_tiny_moduli() {
    for &m in &[8i64, 13, 16, 27, 32] {
        for &a in &[2, 3, 5, 7, 11, m - 3, m - 1] {
            if a <= 0 || a >= m {
                continue;
            }
            assert_eq!(
                pipeline(a, m, 4),
                nu4_squared_brute_tiny(a, m),
                "nu_4^2 mismatch at a={a}, m={m}"
            );
        }
    }
}

#[test]
fn zero_vector_is_never_returned() {
    // The minimum is over NONZERO integer combinations: the scan must skip
    // x = 0, and the seed must be a genuine row norm.
    for &(a, m) in &[(137i64, 256i64), (5, 64), (21, 100), (16807, (1 << 31) - 1)] {
        for t in 2..=5 {
            let got = pipeline(a, m, t);
            assert!(got > 0, "nonpositive shortest vector for a={a}, m={m}, t={t}");
        }
    }
    // Directly on hand-made bases (no pipeline): the answer for the scaled
    // identity diag(2, 3, 5) is 4, not 0.
    let diag = vec![
        vec![2i128, 0, 0],
        vec![0, 3, 0],
        vec![0, 0, 5],
    ];
    assert_eq!(shortest_vector_squared(&diag), 4);
}

#[test]
fn hand_checkable_bases() {
    // Identity: shortest vector is a unit vector.
    let id4: Vec<Vec<i128>> = (0..4)
        .map(|i| (0..4).map(|j| i128::from(i == j)).collect())
        .collect();
    assert_eq!(shortest_vector_squared(&id4), 1);
    // A sheared basis of Z^2: rows (1, 1), (0, 1) still generate Z^2,
    // so the minimum is 1 — the row norms (2 and 1) alone would say 1,
    // but the scan must not report the non-lattice value 0.
    assert_eq!(shortest_vector_squared(&[vec![1, 1], vec![0, 1]]), 1);
    // Rows (2, 1), (1, 2): the lattice contains (2,1)-(1,2) = (1,-1) of
    // norm 2 < 5 — the enumeration must beat both rows.
    assert_eq!(shortest_vector_squared(&[vec![2, 1], vec![1, 2]]), 2);
}

#[test]
fn permuted_and_negated_bases_give_the_same_answer() {
    // The shortest vector is a property of the LATTICE, not the basis:
    // reordering rows and flipping signs must change nothing.
    for &(a, m, t) in &[(137i64, 256i64, 4usize), (RANDU_A, RANDU_M, 3), (21, 100, 5)] {
        let mut v = dual_basis(a, m, t);
        let mut u = primal_basis(a, m, t);
        reduce_basis(&mut v, &mut u);
        let want = shortest_vector_squared(&v);
        // Reverse the row order.
        let mut rev = v.clone();
        rev.reverse();
        assert_eq!(shortest_vector_squared(&rev), want, "row permutation changed the answer");
        // Negate one row and swap two others.
        let mut neg = v.clone();
        for x in &mut neg[0] {
            *x = -*x;
        }
        neg.swap(1, t - 1);
        assert_eq!(shortest_vector_squared(&neg), want, "negation/swap changed the answer");
    }
}
