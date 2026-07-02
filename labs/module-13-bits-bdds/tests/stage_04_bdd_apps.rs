//! Stage 4 — BDDs at work: independent sets and queens (§7.1.4).
//!
//! Implement `independent_set_count` and `queens_bdd_count` in src/lab.rs.
//! Counting independent sets of paths and cycles reproduces Fibonacci and
//! Lucas numbers exactly — a beautiful sanity anchor — and the n-queens
//! BDD counts solutions without ever enumerating a single board.
//! The lesson: course/module-13-bits-bdds/README.md.

use lab_13_bits_bdds::{independent_set_count, queens_bdd_count};

/// Edges of the path P_n: 0—1—2—…—(n−1).
fn path_edges(n: usize) -> Vec<(usize, usize)> {
    (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect()
}

/// Edges of the cycle C_n (n ≥ 3).
fn cycle_edges(n: usize) -> Vec<(usize, usize)> {
    let mut e = path_edges(n);
    e.push((n - 1, 0));
    e
}

#[test]
fn paths_count_fibonacci() {
    // Independent sets of P_n number F_{n+2} (F_1 = F_2 = 1): a classic
    // §7.1.4 example. n = 1: 2 sets; n = 2: 3; n = 3: 5; ...
    let mut fib = vec![0u128; 23]; // fib[i] = F_i, F_1 = F_2 = 1
    fib[1] = 1;
    fib[2] = 1;
    for i in 3..fib.len() {
        fib[i] = fib[i - 1] + fib[i - 2];
    }
    for n in 1..=20 {
        assert_eq!(
            independent_set_count(n, &path_edges(n)),
            fib[n + 2],
            "independent sets of P_{n}"
        );
    }
}

#[test]
fn cycles_count_lucas() {
    // Independent sets of C_n number the Lucas numbers L_n
    // (L_1 = 1, L_2 = 3, L_n = L_{n-1} + L_{n-2}).
    let mut lucas = vec![0u128; 21];
    lucas[1] = 1;
    lucas[2] = 3;
    for i in 3..lucas.len() {
        lucas[i] = lucas[i - 1] + lucas[i - 2];
    }
    for n in 3..=20 {
        assert_eq!(
            independent_set_count(n, &cycle_edges(n)),
            lucas[n],
            "independent sets of C_{n}"
        );
    }
}

#[test]
fn empty_graph_counts_all_subsets() {
    // No edges: every one of the 2^n subsets is independent.
    for n in 0..=30 {
        assert_eq!(independent_set_count(n, &[]), 1u128 << n, "empty graph, n={n}");
    }
}

#[test]
fn complete_graph_counts_singletons_plus_empty() {
    // K_n: only ∅ and the n singletons are independent — n + 1 sets.
    for n in 1..=15usize {
        let mut edges = Vec::new();
        for u in 0..n {
            for v in (u + 1)..n {
                edges.push((u, v));
            }
        }
        assert_eq!(
            independent_set_count(n, &edges),
            (n as u128) + 1,
            "independent sets of K_{n}"
        );
    }
}

#[test]
fn small_hand_checked_graphs() {
    // Triangle: ∅ and three singletons.
    assert_eq!(independent_set_count(3, &[(0, 1), (1, 2), (2, 0)]), 4);
    // Star K_{1,3}: center in (1 way) or center out (2^3 leaf subsets).
    assert_eq!(independent_set_count(4, &[(0, 1), (0, 2), (0, 3)]), 9);
    // One isolated vertex doubles the count of the rest.
    assert_eq!(independent_set_count(4, &[(0, 1), (1, 2), (2, 0)]), 8);
}

#[test]
fn queens_tiny_boards() {
    // 1×1 has one solution; 2×2 and 3×3 have none.
    assert_eq!(queens_bdd_count(1), 1);
    assert_eq!(queens_bdd_count(2), 0);
    assert_eq!(queens_bdd_count(3), 0);
}

#[test]
fn queens_4_and_5() {
    // The classical counts: 2 solutions on 4×4, 10 on 5×5.
    assert_eq!(queens_bdd_count(4), 2);
    assert_eq!(queens_bdd_count(5), 10);
}

#[test]
fn queens_6_is_the_cap() {
    // 4 solutions on 6×6. The BDD over n² variables grows exponentially
    // with n under this variable order (the lesson explains why), so the
    // tests stop here — Module 09's backtracking and Module 10's SAT
    // solving carry the queens further.
    assert_eq!(queens_bdd_count(6), 4);
}
