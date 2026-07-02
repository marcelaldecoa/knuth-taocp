//! Stage 4 — Held–Karp: shortest Hamiltonian path/cycle by bitmask DP.
//!
//! The classic exponential DP (1962), and the finale's tie back to Module 13:
//! the DP state is a *subset* carried as a bitmask. Implement
//! `shortest_hamiltonian_path` and `shortest_hamiltonian_cycle` in src/lab.rs.

use lab_22_hamilton::{shortest_hamiltonian_cycle, shortest_hamiltonian_path};

// ---- hand-computed tiny instances -----------------------------------------

#[test]
fn triangle_by_hand() {
    // Three cities; the only tour is 0-1-2-0 = 1 + 2 + 3 = 6.
    let d = vec![vec![0, 1, 3], vec![1, 0, 2], vec![3, 2, 0]];
    assert_eq!(shortest_hamiltonian_cycle(&d), 6);
    // The shortest path may drop the priciest edge (0-2 = 3): 0-1-2 = 3.
    assert_eq!(shortest_hamiltonian_path(&d), 3);
}

#[test]
fn square_by_hand() {
    // Four points on a unit square (Manhattan-ish integer weights):
    //   0=(0,0) 1=(0,1) 2=(1,1) 3=(1,0). Side 1, diagonal 2.
    let d = vec![
        vec![0, 1, 2, 1],
        vec![1, 0, 1, 2],
        vec![2, 1, 0, 1],
        vec![1, 2, 1, 0],
    ];
    // Best tour follows the four sides: 0-1-2-3-0 = 4.
    assert_eq!(shortest_hamiltonian_cycle(&d), 4);
    // Best path is three sides: 3.
    assert_eq!(shortest_hamiltonian_path(&d), 3);
}

// ---- degenerate sizes ------------------------------------------------------

#[test]
fn single_vertex_costs_nothing() {
    assert_eq!(shortest_hamiltonian_path(&[vec![0]]), 0);
    assert_eq!(shortest_hamiltonian_cycle(&[vec![0]]), 0);
}

#[test]
fn two_vertices() {
    let d = vec![vec![0, 5], vec![5, 0]];
    assert_eq!(shortest_hamiltonian_path(&d), 5);
    // A 2-cycle traverses the single edge both ways.
    assert_eq!(shortest_hamiltonian_cycle(&d), 10);
}

// ---- cross-check against brute-force permutation enumeration --------------

fn brute_path(d: &[Vec<u64>]) -> u64 {
    let n = d.len();
    let mut perm: Vec<usize> = (0..n).collect();
    let mut best = u64::MAX;
    permute(&mut perm, 0, &mut |p: &[usize]| {
        let cost: u64 = p.windows(2).map(|w| d[w[0]][w[1]]).sum();
        best = best.min(cost);
    });
    best
}

fn brute_cycle(d: &[Vec<u64>]) -> u64 {
    let n = d.len();
    let mut perm: Vec<usize> = (0..n).collect();
    let mut best = u64::MAX;
    permute(&mut perm, 0, &mut |p: &[usize]| {
        let mut cost: u64 = p.windows(2).map(|w| d[w[0]][w[1]]).sum();
        cost += d[p[n - 1]][p[0]];
        best = best.min(cost);
    });
    best
}

fn permute(a: &mut [usize], k: usize, visit: &mut impl FnMut(&[usize])) {
    if k == a.len() {
        visit(a);
        return;
    }
    for i in k..a.len() {
        a.swap(k, i);
        permute(a, k + 1, visit);
        a.swap(k, i);
    }
}

#[test]
fn matches_brute_force_on_symmetric_random_distances() {
    // Deterministic LCG (no external randomness) → symmetric weights.
    let mut x: u64 = 0x0f1e_2d3c_4b5a_6978;
    let mut next = || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (x >> 33) % 100 + 1
    };
    for n in 2..=8 {
        let mut d = vec![vec![0u64; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let w = next();
                d[i][j] = w;
                d[j][i] = w;
            }
        }
        assert_eq!(shortest_hamiltonian_path(&d), brute_path(&d), "path n={n}");
        assert_eq!(shortest_hamiltonian_cycle(&d), brute_cycle(&d), "cycle n={n}");
    }
}

#[test]
fn handles_asymmetric_weights_without_triangle_inequality() {
    // Directed, triangle-inequality-violating weights: d[i][j] != d[j][i], and
    // the direct edge can be far costlier than a detour.
    let d = vec![
        vec![0, 10, 15, 20],
        vec![5, 0, 9, 10],
        vec![6, 13, 0, 12],
        vec![8, 8, 9, 0],
    ];
    // Cross-check the DP against brute force on this arbitrary matrix.
    assert_eq!(shortest_hamiltonian_path(&d), brute_path(&d));
    assert_eq!(shortest_hamiltonian_cycle(&d), brute_cycle(&d));
}

// ---- the DP subset ordering is correct at a nontrivial size ---------------

#[test]
fn larger_instance_completes_and_matches_brute_force() {
    // n = 9: 2^9 * 81 DP; brute force still tractable (9! = 362880) as a check.
    let mut x: u64 = 0xdead_beef_cafe_1234;
    let mut next = || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (x >> 40) % 50 + 1
    };
    let n = 9;
    let mut d = vec![vec![0u64; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let w = next();
            d[i][j] = w;
            d[j][i] = w;
        }
    }
    assert_eq!(shortest_hamiltonian_cycle(&d), brute_cycle(&d));
    assert_eq!(shortest_hamiltonian_path(&d), brute_path(&d));
}

#[test]
fn thirteen_vertices_is_fast() {
    // 2^13 * 13^2 ~ 1.4M operations: the DP must finish in well under a second.
    let mut x: u64 = 0x1357_9bdf_0246_8ace;
    let mut next = || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (x >> 40) % 1000 + 1
    };
    let n = 13;
    let mut d = vec![vec![0u64; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let w = next();
            d[i][j] = w;
            d[j][i] = w;
        }
    }
    let tour = shortest_hamiltonian_cycle(&d);
    let path = shortest_hamiltonian_path(&d);
    // A tour must cost at least as much as the best path (it has one more edge
    // and both are minimizations over the same vertex set).
    assert!(tour >= path, "tour {tour} should be >= path {path}");
    assert!(tour > 0 && path > 0);
}
