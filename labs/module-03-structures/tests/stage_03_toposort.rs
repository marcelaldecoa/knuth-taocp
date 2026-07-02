//! Stage 3 — Topological sorting, Algorithm 2.2.3T (§2.2.3).
//!
//! Implement `topological_sort` in src/lab.rs. Lesson:
//! course/module-03-structures/README.md, §4.
//!
//! A relation (j, k) means "j must precede k" (j ≺ k). The output is a linear
//! arrangement of 1..=n consistent with every relation — a *linear extension*
//! of the partial order — or `None` when a cycle makes that impossible. The
//! FIFO queue discipline (initial zeros in increasing order, successors in
//! input order) fixes the output uniquely.

use lab_03_structures::*;

// ------------------------------------------- Knuth's worked example ----

#[test]
fn knuth_worked_example_2_2_3() {
    // §2.2.3's example: nine objects and the input relations
    //   9≺2, 3≺7, 7≺5, 5≺8, 8≺6, 4≺6, 1≺3, 7≺4, 9≺5, 2≺8.
    // With the FIFO discipline the algorithm outputs 1 9 3 2 7 5 4 8 6.
    let rel = [
        (9, 2), (3, 7), (7, 5), (5, 8), (8, 6),
        (4, 6), (1, 3), (7, 4), (9, 5), (2, 8),
    ];
    assert_eq!(
        topological_sort(9, &rel),
        Some(vec![1, 9, 3, 2, 7, 5, 4, 8, 6])
    );
}

// ------------------------------------------- cycles → None ----

#[test]
fn a_three_cycle_has_no_topological_order() {
    assert_eq!(topological_sort(3, &[(1, 2), (2, 3), (3, 1)]), None);
}

#[test]
fn a_self_loop_is_a_cycle() {
    // j ≺ j is already impossible to satisfy.
    assert_eq!(topological_sort(2, &[(1, 1)]), None);
    assert_eq!(topological_sort(3, &[(1, 2), (2, 2)]), None);
}

#[test]
fn a_cycle_hidden_among_free_objects() {
    // Objects 1 and 4 are unconstrained; 2 and 3 form a cycle.
    assert_eq!(topological_sort(4, &[(2, 3), (3, 2)]), None);
}

// ------------------------------------------- degenerate inputs ----

#[test]
fn no_relations_gives_the_identity_order() {
    // With nothing to constrain them, the objects come out in index order
    // (initial zeros are enqueued 1, 2, …, n).
    assert_eq!(topological_sort(5, &[]), Some(vec![1, 2, 3, 4, 5]));
    assert_eq!(topological_sort(0, &[]), Some(vec![]));
    assert_eq!(topological_sort(1, &[]), Some(vec![1]));
}

#[test]
fn a_total_order_is_reproduced() {
    // 1 ≺ 2 ≺ 3 ≺ 4 ≺ 5 forces exactly one arrangement.
    let rel = [(1, 2), (2, 3), (3, 4), (4, 5)];
    assert_eq!(topological_sort(5, &rel), Some(vec![1, 2, 3, 4, 5]));
    // The same order given "backwards" as pairs is the same constraint set.
    let rel_rev = [(4, 5), (3, 4), (2, 3), (1, 2)];
    assert_eq!(topological_sort(5, &rel_rev), Some(vec![1, 2, 3, 4, 5]));
}

// ------------------------------------------- out-of-range panics ----

#[test]
#[should_panic(expected = "outside")]
fn object_zero_is_out_of_range() {
    // Objects are numbered 1..=n; 0 is not a valid object.
    let _ = topological_sort(3, &[(0, 1)]);
}

#[test]
#[should_panic(expected = "outside")]
fn object_beyond_n_is_out_of_range() {
    let _ = topological_sort(3, &[(1, 4)]);
}

// ------------------------------------------- the defining property ----

/// Check that a claimed order really is a linear extension: it is a
/// permutation of 1..=n and every relation j ≺ k puts j before k.
fn is_linear_extension(n: usize, rel: &[(usize, usize)], order: &[usize]) -> bool {
    if order.len() != n {
        return false;
    }
    let mut seen = vec![false; n + 1];
    for &x in order {
        if x < 1 || x > n || seen[x] {
            return false;
        }
        seen[x] = true;
    }
    // position[x] = where x sits in the output.
    let mut pos = vec![0usize; n + 1];
    for (i, &x) in order.iter().enumerate() {
        pos[x] = i;
    }
    rel.iter().all(|&(j, k)| pos[j] < pos[k])
}

#[test]
fn output_is_always_a_valid_linear_extension() {
    // Generate acyclic relation sets by only ever pointing a smaller *rank*
    // to a larger one under a random permutation — guaranteeing no cycle —
    // then confirm the algorithm's output honours every relation.
    let mut rng: u64 = 987654321;
    let mut next = || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        rng >> 33
    };

    for _ in 0..500 {
        let n = 2 + (next() as usize % 9); // 2..=10 objects
        // Random permutation p of 1..=n via Fisher–Yates: p[i] gives a rank.
        let mut perm: Vec<usize> = (1..=n).collect();
        for i in (1..n).rev() {
            let j = next() as usize % (i + 1);
            perm.swap(i, j);
        }
        // rank[x] = position of object x in perm (its topological rank).
        let mut rank = vec![0usize; n + 1];
        for (i, &x) in perm.iter().enumerate() {
            rank[x] = i;
        }
        // Build relations only from lower rank to higher rank → acyclic.
        let mut rel = Vec::new();
        let m = next() as usize % (2 * n + 1);
        for _ in 0..m {
            let a = 1 + (next() as usize % n);
            let b = 1 + (next() as usize % n);
            if rank[a] < rank[b] {
                rel.push((a, b));
            } else if rank[b] < rank[a] {
                rel.push((b, a));
            }
        }
        let out = topological_sort(n, &rel)
            .expect("acyclic relations must have a topological order");
        assert!(
            is_linear_extension(n, &rel, &out),
            "not a linear extension: n={n} rel={rel:?} out={out:?}"
        );
    }
}
