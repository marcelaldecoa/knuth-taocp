//! Stage 1 — Hamiltonian paths and cycles by backtracking (§7.2.2.4).
//!
//! Implement `adjacency_matrix`, `hamiltonian_path`, `hamiltonian_cycle`, and
//! `count_hamiltonian_cycles` in src/lab.rs.
//! Lesson: course/module-22-hamilton/README.md.

use lab_22_hamilton::{
    adjacency_matrix, count_hamiltonian_cycles, hamiltonian_cycle, hamiltonian_path,
};

// ---- graph builders --------------------------------------------------------

fn complete_graph(n: usize) -> Vec<Vec<usize>> {
    (0..n)
        .map(|u| (0..n).filter(|&v| v != u).collect())
        .collect()
}

fn cycle_graph(n: usize) -> Vec<Vec<usize>> {
    (0..n).map(|u| vec![(u + n - 1) % n, (u + 1) % n]).collect()
}

fn path_graph(n: usize) -> Vec<Vec<usize>> {
    (0..n)
        .map(|u| {
            let mut v = Vec::new();
            if u > 0 {
                v.push(u - 1);
            }
            if u + 1 < n {
                v.push(u + 1);
            }
            v
        })
        .collect()
}

/// The Petersen graph: outer 5-cycle, inner pentagram, five spokes.
fn petersen() -> Vec<Vec<usize>> {
    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 4), (4, 0),
        (0, 5), (1, 6), (2, 7), (3, 8), (4, 9),
        (5, 7), (7, 9), (9, 6), (6, 8), (8, 5),
    ];
    let mut adj = vec![Vec::new(); 10];
    for (u, v) in edges {
        adj[u].push(v);
        adj[v].push(u);
    }
    adj
}

fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

/// Validate: `seq` is a permutation of 0..n with consecutive vertices adjacent.
fn is_ham_path(adj: &[Vec<usize>], seq: &[usize]) -> bool {
    let n = adj.len();
    if seq.len() != n {
        return false;
    }
    let mut seen = vec![false; n];
    for &v in seq {
        if v >= n || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    seq.windows(2).all(|w| adj[w[0]].contains(&w[1]))
}

// ---- adjacency_matrix ------------------------------------------------------

#[test]
fn adjacency_matrix_is_the_transpose_of_the_lists() {
    let g = petersen();
    let m = adjacency_matrix(&g);
    assert_eq!(m.len(), 10);
    for (u, nbrs) in g.iter().enumerate() {
        for v in 0..10 {
            assert_eq!(m[u][v], nbrs.contains(&v), "edge {u}-{v}");
        }
    }
}

// ---- counting Hamiltonian cycles ------------------------------------------

#[test]
fn complete_graph_has_half_factorial_cycles() {
    // K_n has exactly (n-1)!/2 distinct undirected Hamiltonian cycles.
    for n in 3..=7 {
        let expected = factorial(n as u64 - 1) / 2;
        assert_eq!(count_hamiltonian_cycles(&complete_graph(n)), expected, "K_{n}");
    }
}

#[test]
fn cycle_graph_has_exactly_one() {
    for n in 3..=9 {
        assert_eq!(count_hamiltonian_cycles(&cycle_graph(n)), 1, "C_{n}");
    }
}

#[test]
fn path_graph_has_none() {
    for n in 3..=9 {
        assert_eq!(count_hamiltonian_cycles(&path_graph(n)), 0, "P_{n}");
    }
}

// ---- path graph: path yes, cycle no ---------------------------------------

#[test]
fn path_graph_traceable_but_acyclic() {
    for n in 3..=9 {
        let g = path_graph(n);
        let p = hamiltonian_path(&g).expect("P_n has a Hamiltonian path");
        assert!(is_ham_path(&g, &p), "P_{n} path validity");
        assert!(hamiltonian_cycle(&g).is_none(), "P_{n} has no Hamiltonian cycle");
    }
}

// ---- the Petersen graph: the famous non-Hamiltonian graph -----------------

#[test]
fn petersen_has_no_hamiltonian_cycle() {
    let g = petersen();
    assert!(hamiltonian_cycle(&g).is_none());
    assert_eq!(count_hamiltonian_cycles(&g), 0);
}

#[test]
fn petersen_has_a_hamiltonian_path() {
    let g = petersen();
    let p = hamiltonian_path(&g).expect("the Petersen graph is traceable");
    assert!(is_ham_path(&g, &p), "returned Petersen path must be valid");
}

// ---- returned solutions are validated, not compared to a fixed answer -----

#[test]
fn returned_cycle_is_a_valid_cycle() {
    for n in 3..=7 {
        let g = complete_graph(n);
        let c = hamiltonian_cycle(&g).unwrap();
        assert!(is_ham_path(&g, &c), "K_{n}: must be a permutation with adjacencies");
        // ...and it closes: last vertex adjacent to the first.
        assert!(g[*c.last().unwrap()].contains(&c[0]), "K_{n}: cycle must close");
    }
}

#[test]
fn returned_cycle_on_cycle_graph_is_the_cycle() {
    let g = cycle_graph(8);
    let c = hamiltonian_cycle(&g).unwrap();
    assert!(is_ham_path(&g, &c));
    assert!(g[*c.last().unwrap()].contains(&c[0]));
}

// ---- disconnected graphs --------------------------------------------------

#[test]
fn disconnected_graph_has_no_path_or_cycle() {
    // Two disjoint edges 0-1 and 2-3.
    let g = vec![vec![1], vec![0], vec![3], vec![2]];
    assert!(hamiltonian_path(&g).is_none());
    assert!(hamiltonian_cycle(&g).is_none());
    assert_eq!(count_hamiltonian_cycles(&g), 0);
}

#[test]
fn small_graphs_have_no_cycle() {
    // A single edge and a single vertex: no Hamiltonian cycle.
    assert_eq!(count_hamiltonian_cycles(&vec![vec![1], vec![0]]), 0);
    assert_eq!(count_hamiltonian_cycles(&vec![vec![]]), 0);
}
