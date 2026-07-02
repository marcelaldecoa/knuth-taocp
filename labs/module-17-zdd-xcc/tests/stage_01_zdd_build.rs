//! Stage 1 — Zero-suppressed decision diagrams (§7.1.4, ZDD reduction rule).
//!
//! Implement the `Zdd` arena in src/lab.rs: the two sinks (⊥ = the empty
//! family ∅, ⊤ = the family {∅}), `single`, the accessors, the queries
//! `count_sets`/`contains_set`/`sets`/`node_count` — and `union`, which
//! this stage needs to assemble any family with more than one member
//! (stage 2 stresses the full algebra; implementing `union` first is the
//! natural on-ramp). The headline invariant is THE ZDD REDUCTION RULE:
//! no node in the arena ever has HI = ⊥. Note the contrast with Module
//! 13's BDD rule (`lo == hi` elided): that contrast is the whole point.
//! The lesson: course/module-17-zdd-xcc/README.md.

use lab_17_zdd_xcc::{Ref, Zdd};

#[test]
fn empty_family_and_family_of_empty_set_are_different_things() {
    // ∅ (no member sets) vs {∅} (one member: the empty set). Every ZDD
    // bug report starts with confusing these two.
    let z = Zdd::new();
    assert_ne!(z.empty(), z.unit());
    assert!(z.is_terminal(z.empty()) && z.is_terminal(z.unit()));
    assert_eq!(z.count_sets(z.empty()), 0);
    assert_eq!(z.count_sets(z.unit()), 1);
    assert!(!z.contains_set(z.empty(), &[]));
    assert!(z.contains_set(z.unit(), &[]));
    assert!(!z.contains_set(z.unit(), &[0]));
    assert_eq!(z.sets(z.empty()), Vec::<Vec<u32>>::new());
    assert_eq!(z.sets(z.unit()), vec![Vec::<u32>::new()]);
    // Both sinks are single "nodes" as far as reachability goes.
    assert_eq!(z.node_count(z.empty()), 1);
    assert_eq!(z.node_count(z.unit()), 1);
}

#[test]
fn single_is_the_family_of_one_singleton() {
    let mut z = Zdd::new();
    let s = z.single(3);
    assert!(!z.is_terminal(s));
    assert_eq!(z.var(s), 3);
    assert_eq!(z.lo(s), z.empty());
    assert_eq!(z.hi(s), z.unit());
    assert_eq!(z.count_sets(s), 1);
    assert!(z.contains_set(s, &[3]));
    assert!(!z.contains_set(s, &[]) && !z.contains_set(s, &[2]) && !z.contains_set(s, &[3, 4]));
    assert_eq!(z.sets(s), vec![vec![3]]);
    assert_eq!(z.node_count(s), 3); // the node plus both sinks
}

#[test]
fn canonicity_is_ref_equality() {
    // Hash-consing: the same family, built any way at all, is the same Ref.
    let mut z = Zdd::new();
    let a = z.single(5);
    let b = z.single(5);
    assert_eq!(a, b, "unique table: single(5) twice is one node");

    // {∅, {0}, {1}} assembled in three different orders/groupings.
    let (u, s0, s1) = (z.unit(), z.single(0), z.single(1));
    let f1 = {
        let t = z.union(u, s0);
        z.union(t, s1)
    };
    let f2 = {
        let t = z.union(s1, s0);
        z.union(t, u)
    };
    let f3 = {
        let t = z.union(s0, u);
        let t2 = z.union(t, t); // idempotent detour
        z.union(s1, t2)
    };
    assert_eq!(f1, f2);
    assert_eq!(f2, f3);
    assert_eq!(z.count_sets(f1), 3);
    assert_eq!(z.sets(f1), vec![vec![], vec![0], vec![1]]);
}

#[test]
fn union_of_singletons_behaves() {
    // {{0}, {2}, {4}}: count, membership, canonical enumeration agree.
    let mut z = Zdd::new();
    let (s0, s2, s4) = (z.single(0), z.single(2), z.single(4));
    let t = z.union(s0, s2);
    let f = z.union(t, s4);
    assert_eq!(z.count_sets(f), 3);
    for v in [0u32, 2, 4] {
        assert!(z.contains_set(f, &[v]), "{{{v}}} must be a member");
    }
    assert!(!z.contains_set(f, &[1]));
    assert!(!z.contains_set(f, &[0, 2]), "member sets, not their unions");
    assert!(!z.contains_set(f, &[]));
    assert_eq!(z.sets(f), vec![vec![0], vec![2], vec![4]]);
    // count, contains, enumerate tell one consistent story.
    let all = z.sets(f);
    assert_eq!(all.len() as u128, z.count_sets(f));
    for s in &all {
        assert!(z.contains_set(f, s));
    }
}

#[test]
fn the_zero_suppression_rule_holds_arena_wide() {
    // Build a pile of families, then audit EVERY node ever created:
    // no node may have HI = ⊥. (A BDD would instead forbid lo == hi —
    // and lo == hi nodes are perfectly legal here.)
    let mut z = Zdd::new();
    let mut f = z.empty();
    for i in 0..24u32 {
        let s = z.single(i % 12);
        f = z.union(f, s);
        let g = z.union(z.unit(), s);
        f = z.union(f, g);
    }
    assert!(z.len() > 12, "expected some real nodes to audit");
    let bottom = z.empty();
    for i in 2..z.len() {
        let r = Ref(i as u32);
        assert!(!z.is_terminal(r));
        assert_ne!(z.hi(r), bottom, "node {i} has HI = bottom: zero-suppression violated");
        // While we are sweeping: children must sit strictly below.
        for child in [z.lo(r), z.hi(r)] {
            if !z.is_terminal(child) {
                assert!(z.var(child) > z.var(r), "variable order violated at node {i}");
            }
        }
    }
}

#[test]
fn sparsity_showcase_singleton_family_is_linear() {
    // The family { {i} : 0 <= i < n } has exactly n + 2 nodes: one chain
    // node per variable plus the two sinks. This is the zero-suppression
    // dividend: sparse families cost what they mention, not what the
    // universe contains. (As a boolean function of 64 variables, the
    // "exactly one x_i is true" BDD needs about 2n nodes, and the
    // complement family has 2^64 - 64 members — families and their
    // complements are wildly asymmetric here, unlike f vs ¬f in a BDD.)
    for n in [8u32, 16, 32, 64] {
        let mut z = Zdd::new();
        let mut f = z.empty();
        for i in 0..n {
            let s = z.single(i);
            f = z.union(f, s);
        }
        assert_eq!(z.count_sets(f), n as u128);
        assert_eq!(z.node_count(f), n as usize + 2, "Z(f) must be linear in n");
    }
    // Sharper still: {{0}} costs 3 nodes no matter how big the universe
    // is, because variables absent from every member simply do not appear.
    let mut z = Zdd::new();
    let s = z.single(0);
    let far = z.single(63);
    assert_eq!(z.node_count(s), 3);
    assert_eq!(z.node_count(far), 3);
}

#[test]
fn contains_set_accepts_unsorted_and_duplicated_queries() {
    let mut z = Zdd::new();
    let s1 = z.single(1);
    let s9 = z.single(9);
    let f = z.union(s1, s9);
    assert!(z.contains_set(f, &[9]));
    assert!(z.contains_set(f, &[9, 9]));
    assert!(!z.contains_set(f, &[9, 1]), "{{1,9}} is not a member of {{{{1}},{{9}}}}");
}
