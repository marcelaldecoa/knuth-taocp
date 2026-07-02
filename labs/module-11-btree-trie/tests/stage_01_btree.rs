//! Stage 1 — B-tree search and insertion with node splitting (§6.2.4).
//!
//! Implement `BTree::{insert, contains, keys_inorder}` in src/lab.rs.
//! Lesson: course/module-11-btree-trie/README.md.

use lab_11_btree_trie::BTree;
use std::collections::BTreeSet;

/// Deterministic pseudo-random stream (no external crates, no clocks).
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn worked_example_from_the_lesson() {
    // The lesson hand-traces order m = 3 on 50 30 70 10 40 60 20 80.
    let mut t = BTree::new(3);
    for &k in &[50, 30, 70, 10, 40, 60, 20, 80] {
        assert!(t.insert(k), "fresh key {k} must insert");
    }
    assert_eq!(t.keys_inorder(), vec![10, 20, 30, 40, 50, 60, 70, 80]);
    for &k in &[10, 20, 30, 40, 50, 60, 70, 80] {
        assert!(t.contains(k), "key {k} must be found");
    }
    for &k in &[0, 15, 55, 75, 90] {
        assert!(!t.contains(k), "key {k} must be absent");
    }
}

#[test]
fn inorder_sorted_after_lcg_insertions_several_orders() {
    for &m in &[3usize, 4, 7, 32] {
        let mut t = BTree::new(m);
        let mut model = BTreeSet::new();
        let mut s = 0x1234_5678_9ABC_DEF0u64 ^ (m as u64);
        for _ in 0..2500 {
            let k = lcg(&mut s) as i64; // full-range i64 keys
            assert_eq!(t.insert(k), model.insert(k), "insert({k}) order {m}");
        }
        let expect: Vec<i64> = model.iter().copied().collect();
        assert_eq!(t.keys_inorder(), expect, "inorder must be sorted, order {m}");
        // Spot-check membership on present and absent keys.
        for (i, &k) in expect.iter().enumerate().step_by(97) {
            assert!(t.contains(k), "present key #{i} order {m}");
        }
        for _ in 0..200 {
            let k = lcg(&mut s) as i64; // fresh probes, almost surely absent
            assert_eq!(t.contains(k), model.contains(&k), "probe {k} order {m}");
        }
    }
}

#[test]
fn duplicates_rejected_and_tree_unchanged() {
    let mut t = BTree::new(4);
    let mut s = 42u64;
    let mut keys = Vec::new();
    for _ in 0..300 {
        keys.push((lcg(&mut s) % 120) as i64); // small range: many duplicates
    }
    let mut model = BTreeSet::new();
    for &k in &keys {
        assert_eq!(t.insert(k), model.insert(k), "first pass insert({k})");
    }
    let snapshot = t.keys_inorder();
    for &k in &keys {
        assert!(!t.insert(k), "second insert of {k} must return false");
    }
    assert_eq!(t.keys_inorder(), snapshot, "duplicates must not change the tree");
}

#[test]
fn model_check_against_std_btreeset() {
    // A long mixed op sequence over a small key universe, checked move by
    // move against std::collections::BTreeSet.
    for &m in &[3usize, 8] {
        let mut t = BTree::new(m);
        let mut model = BTreeSet::new();
        let mut s = 0xDEAD_BEEFu64 ^ (m as u64);
        for step in 0..12_000 {
            let k = (lcg(&mut s) % 600) as i64 - 300;
            if lcg(&mut s) % 2 == 0 {
                assert_eq!(t.insert(k), model.insert(k), "step {step}: insert({k}), m={m}");
            } else {
                assert_eq!(t.contains(k), model.contains(&k), "step {step}: contains({k}), m={m}");
            }
        }
        let expect: Vec<i64> = model.iter().copied().collect();
        assert_eq!(t.keys_inorder(), expect, "final inorder, m={m}");
    }
}

#[test]
fn ascending_and_descending_patterns() {
    // Sorted input is the classic BST killer; a B-tree must shrug it off.
    for &m in &[3usize, 32] {
        let mut asc = BTree::new(m);
        for k in 0..3000i64 {
            assert!(asc.insert(k));
        }
        assert_eq!(asc.keys_inorder(), (0..3000).collect::<Vec<i64>>());
        assert!(asc.contains(0) && asc.contains(1499) && asc.contains(2999));
        assert!(!asc.contains(-1) && !asc.contains(3000));

        let mut desc = BTree::new(m);
        for k in (0..3000i64).rev() {
            assert!(desc.insert(k));
        }
        assert_eq!(desc.keys_inorder(), (0..3000).collect::<Vec<i64>>());
    }
}

#[test]
fn root_splits_stressed_order_3() {
    // Order 3 splits at every opportunity — the root splits over and over.
    // Verify full membership after every single insert for a while.
    let mut t = BTree::new(3);
    let keys: Vec<i64> = (0..160).map(|i| (i * 37) % 160).collect(); // a permutation
    for (n, &k) in keys.iter().enumerate() {
        assert!(t.insert(k), "insert {k}");
        for &prev in &keys[..=n] {
            assert!(t.contains(prev), "after {n} inserts, {prev} vanished");
        }
        assert!(!t.contains(1000));
    }
    let mut expect: Vec<i64> = keys.clone();
    expect.sort_unstable();
    assert_eq!(t.keys_inorder(), expect);
}

#[test]
fn extreme_keys_work() {
    let mut t = BTree::new(5);
    for &k in &[i64::MIN, i64::MAX, 0, -1, 1, i64::MIN + 1, i64::MAX - 1] {
        assert!(t.insert(k));
    }
    assert_eq!(
        t.keys_inorder(),
        vec![i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX]
    );
    assert!(t.contains(i64::MIN) && t.contains(i64::MAX));
    assert!(!t.contains(2));
}

#[test]
#[should_panic(expected = "at least 3")]
fn order_two_is_rejected() {
    // A "B-tree of order 2" cannot satisfy the minimum-fullness invariant.
    BTree::new(2);
}
