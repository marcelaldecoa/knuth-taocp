//! Stage 3 — Counting structures in graphs with ZDDs (§7.1.4 applications).
//!
//! Implement `matchings_zdd` and `independent_sets_zdd` in src/lab.rs.
//! Paths and cycles reproduce Fibonacci and Lucas numbers; complete
//! graphs reproduce the telephone numbers; and the 4×4 grid's independent
//! sets are counted two ways — by your ZDD and by brute force over all
//! 2^16 subsets — before being pinned.
//! The lesson: course/module-17-zdd-xcc/README.md.

use lab_17_zdd_xcc::{independent_sets_zdd, matchings_zdd};

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

/// Edges of the complete graph K_n.
fn complete_edges(n: usize) -> Vec<(usize, usize)> {
    let mut e = Vec::new();
    for u in 0..n {
        for v in (u + 1)..n {
            e.push((u, v));
        }
    }
    e
}

/// F_1 = F_2 = 1, F_{k} = F_{k−1} + F_{k−2}.
fn fibonacci(upto: usize) -> Vec<u128> {
    let mut f = vec![0u128; upto + 1];
    if upto >= 1 {
        f[1] = 1;
    }
    if upto >= 2 {
        f[2] = 1;
    }
    for i in 3..=upto {
        f[i] = f[i - 1] + f[i - 2];
    }
    f
}

/// L_1 = 1, L_2 = 3, L_k = L_{k−1} + L_{k−2}.
fn lucas(upto: usize) -> Vec<u128> {
    let mut l = vec![0u128; upto + 1];
    if upto >= 1 {
        l[1] = 1;
    }
    if upto >= 2 {
        l[2] = 3;
    }
    for i in 3..=upto {
        l[i] = l[i - 1] + l[i - 2];
    }
    l
}

/// Brute-force matching count: try all 2^m edge subsets.
fn matchings_brute(n: usize, edges: &[(usize, usize)]) -> u128 {
    let m = edges.len();
    assert!(m <= 20);
    let mut count = 0u128;
    'mask: for mask in 0u32..(1 << m) {
        let mut used = vec![false; n];
        for (i, &(u, v)) in edges.iter().enumerate() {
            if mask >> i & 1 == 1 {
                if used[u] || used[v] {
                    continue 'mask;
                }
                used[u] = true;
                used[v] = true;
            }
        }
        count += 1;
    }
    count
}

/// Brute-force independent-set count: try all 2^n vertex subsets.
fn independent_brute(n: usize, edges: &[(usize, usize)]) -> u128 {
    assert!(n <= 20);
    (0u32..(1 << n))
        .filter(|mask| {
            edges
                .iter()
                .all(|&(u, v)| mask >> u & 1 == 0 || mask >> v & 1 == 0)
        })
        .count() as u128
}

#[test]
fn independent_sets_match_module_13() {
    // Independent sets of P_n number F_{n+2}; of C_n, the Lucas number
    // L_n — exactly the counts Module 13 got from BDD model counting.
    // Same family, different diagram, same answer.
    let fib = fibonacci(20);
    let luc = lucas(18);
    for n in 1..=18 {
        assert_eq!(
            independent_sets_zdd(n, &path_edges(n)),
            fib[n + 2],
            "independent sets of P_{n}"
        );
    }
    for n in 3..=18 {
        assert_eq!(
            independent_sets_zdd(n, &cycle_edges(n)),
            luc[n],
            "independent sets of C_{n}"
        );
    }
    // No edges: the full power set.
    assert_eq!(independent_sets_zdd(10, &[]), 1 << 10);
    assert_eq!(independent_sets_zdd(0, &[]), 1);
}

#[test]
fn matchings_of_paths_are_fibonacci() {
    // m(P_1) = 1, m(P_2) = 2, m(P_3) = 3, m(P_4) = 5: m(P_n) = F_{n+1}.
    let fib = fibonacci(20);
    assert_eq!(matchings_zdd(1, &path_edges(1)), 1);
    assert_eq!(matchings_zdd(2, &path_edges(2)), 2);
    assert_eq!(matchings_zdd(3, &path_edges(3)), 3);
    assert_eq!(matchings_zdd(4, &path_edges(4)), 5);
    for n in 1..=18 {
        assert_eq!(matchings_zdd(n, &path_edges(n)), fib[n + 1], "matchings of P_{n}");
    }
}

#[test]
fn matchings_of_cycles_are_lucas() {
    // m(C_3) = 4 (∅ and three single edges), m(C_4) = 7: m(C_n) = L_n.
    let luc = lucas(18);
    assert_eq!(matchings_zdd(3, &cycle_edges(3)), 4);
    assert_eq!(matchings_zdd(4, &cycle_edges(4)), 7);
    for n in 3..=18 {
        assert_eq!(matchings_zdd(n, &cycle_edges(n)), luc[n], "matchings of C_{n}");
    }
}

#[test]
fn matchings_of_complete_graphs_are_telephone_numbers() {
    // T(n) counts the ways n telephone subscribers can be pairwise
    // connected (equivalently, involutions of n elements): 1, 1, 2, 4,
    // 10, 26, 76, 232 for n = 0..7.
    let telephone: [u128; 8] = [1, 1, 2, 4, 10, 26, 76, 232];
    for (n, &want) in telephone.iter().enumerate() {
        assert_eq!(matchings_zdd(n, &complete_edges(n)), want, "matchings of K_{n}");
    }
}

#[test]
fn brute_force_agreement_on_small_graphs() {
    // A gallery of tiny graphs, both counters, both methods.
    let gallery: Vec<(usize, Vec<(usize, usize)>)> = vec![
        (3, complete_edges(3)),                      // triangle
        (4, complete_edges(4)),                      // K_4
        (4, vec![(0, 1), (0, 2), (0, 3)]),           // star K_{1,3}
        (5, vec![(0, 1), (1, 2), (0, 2), (2, 3), (3, 4), (2, 4)]), // bowtie
        (6, cycle_edges(6)),
        (5, path_edges(5)),
        (4, vec![]),                                 // no edges at all
        (6, vec![(0, 1), (2, 3), (4, 5)]),           // perfect matching itself
    ];
    for (n, edges) in gallery {
        assert_eq!(
            matchings_zdd(n, &edges),
            matchings_brute(n, &edges),
            "matchings, n={n}, edges={edges:?}"
        );
        assert_eq!(
            independent_sets_zdd(n, &edges),
            independent_brute(n, &edges),
            "independent sets, n={n}, edges={edges:?}"
        );
    }
}

#[test]
fn grid_4x4_independent_sets() {
    // The 4×4 grid graph: 16 vertices, 24 edges. Brute force over all
    // 65536 subsets confirms the ZDD, and the value is pinned: 1234
    // (the hard-square sequence for n×n grids runs 2, 7, 63, 1234, …).
    let mut edges = Vec::new();
    for r in 0..4usize {
        for c in 0..4usize {
            let v = 4 * r + c;
            if c + 1 < 4 {
                edges.push((v, v + 1));
            }
            if r + 1 < 4 {
                edges.push((v, v + 4));
            }
        }
    }
    assert_eq!(edges.len(), 24);
    let zdd = independent_sets_zdd(16, &edges);
    assert_eq!(zdd, independent_brute(16, &edges));
    assert_eq!(zdd, 1234);
}
