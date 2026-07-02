//! Stage 2 — Binary search trees (Algorithms 6.2.2T insert/search, 6.2.2D
//! deletion by symmetric successor).
//!
//! Implement the `Bst` methods in src/lab.rs. Lesson:
//! course/module-07-searching/README.md.

use lab_07_searching::Bst;
use std::collections::BTreeSet;

/// The hand-rolled LCG the whole course uses — deterministic, no crates.
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// Knuth's sixteen keys, sorted.
const SIXTEEN: [i64; 16] = [
    61, 87, 154, 170, 275, 426, 503, 509, 512, 612, 653, 677, 703, 765, 897, 908,
];

#[test]
fn inorder_is_sorted_whatever_the_insertion_order() {
    let order = [503, 87, 512, 61, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703];
    let mut t = Bst::new();
    for &k in &order {
        assert!(t.insert(k), "{k} newly inserted");
        assert!(!t.insert(k), "duplicate {k} rejected");
    }
    let mut sorted = SIXTEEN.to_vec();
    sorted.sort_unstable();
    assert_eq!(t.inorder(), sorted);
    for &k in &SIXTEEN {
        assert!(t.contains(k));
    }
    assert!(!t.contains(500));
}

#[test]
fn inorder_sorted_after_lcg_inserts() {
    let mut t = Bst::new();
    let mut model = BTreeSet::new();
    let mut s = 0x1234_5678u64;
    for _ in 0..3000 {
        let k = (lcg(&mut s) >> 20) as i64 % 5000;
        assert_eq!(t.insert(k), model.insert(k), "insert({k}) matches the model");
    }
    let expect: Vec<i64> = model.iter().copied().collect();
    assert_eq!(t.inorder(), expect);
    assert!(t.inorder().windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn deletion_all_four_shapes() {
    // Build a fixed tree:
    //            50
    //          /    \
    //        30      70
    //       /  \    /  \
    //      20  40  60   80
    //       \        \
    //       25        65
    let keys = [50, 30, 70, 20, 40, 60, 80, 25, 65];
    let build = || {
        let mut t = Bst::new();
        for &k in &keys {
            t.insert(k);
        }
        t
    };
    let sorted = {
        let mut v = keys.to_vec();
        v.sort_unstable();
        v
    };
    let without = |x: i64| -> Vec<i64> { sorted.iter().copied().filter(|&k| k != x).collect() };

    // Leaf (40): just unlinks.
    let mut t = build();
    assert!(t.delete(40));
    assert!(!t.contains(40));
    assert_eq!(t.inorder(), without(40));

    // One child (20 has only the right child 25): child is spliced in.
    let mut t = build();
    assert!(t.delete(20));
    assert!(!t.contains(20) && t.contains(25));
    assert_eq!(t.inorder(), without(20));

    // One child on the other side (60 has only right child 65).
    let mut t = build();
    assert!(t.delete(60));
    assert_eq!(t.inorder(), without(60));

    // Two children (30): symmetric successor (40) replaces it.
    let mut t = build();
    assert!(t.delete(30));
    assert!(!t.contains(30) && t.contains(40) && t.contains(20) && t.contains(25));
    assert_eq!(t.inorder(), without(30));

    // Root with two children (50): successor 60 takes over.
    let mut t = build();
    assert!(t.delete(50));
    assert!(!t.contains(50));
    assert_eq!(t.inorder(), without(50));

    // Deleting an absent key is a no-op returning false.
    let mut t = build();
    assert!(!t.delete(999));
    assert_eq!(t.inorder(), sorted);
}

#[test]
fn delete_down_to_empty() {
    // Successor deletion must survive emptying the tree in any order.
    let mut t = Bst::new();
    for &k in &SIXTEEN {
        t.insert(k);
    }
    // Delete the root repeatedly until nothing remains.
    let mut remaining: BTreeSet<i64> = SIXTEEN.iter().copied().collect();
    while !remaining.is_empty() {
        let cur = t.inorder();
        let root_key = cur[cur.len() / 2]; // some interior key, always present
        assert!(t.delete(root_key));
        remaining.remove(&root_key);
        let expect: Vec<i64> = remaining.iter().copied().collect();
        assert_eq!(t.inorder(), expect, "after deleting {root_key}");
    }
    assert_eq!(t.height(), 0);
    assert!(t.inorder().is_empty());
}

#[test]
fn long_op_sequence_matches_btreeset() {
    // Interleave insert / contains / delete against a BTreeSet oracle.
    let mut t = Bst::new();
    let mut model = BTreeSet::new();
    let mut s = 0xC0FFEEu64;
    for i in 0..20_000 {
        let k = (lcg(&mut s) >> 24) as i64 % 800; // small range => lots of hits
        match lcg(&mut s) % 3 {
            0 => assert_eq!(t.insert(k), model.insert(k), "insert {k}"),
            1 => assert_eq!(t.contains(k), model.contains(&k), "contains {k}"),
            _ => assert_eq!(t.delete(k), model.remove(&k), "delete {k}"),
        }
        if i % 500 == 0 {
            let expect: Vec<i64> = model.iter().copied().collect();
            assert_eq!(t.inorder(), expect, "inorder snapshot at op {i}");
        }
    }
    let expect: Vec<i64> = model.iter().copied().collect();
    assert_eq!(t.inorder(), expect);
}

#[test]
fn sorted_insertion_is_a_vine() {
    // Ascending keys degenerate into a right-leaning path: height == n - 1.
    let mut t = Bst::new();
    for k in 0..500i64 {
        t.insert(k);
    }
    assert_eq!(t.height(), 499);
    // A single node has height 0; the empty tree, 0 by convention.
    let mut one = Bst::new();
    one.insert(7);
    assert_eq!(one.height(), 0);
    assert_eq!(Bst::new().height(), 0);
}

#[test]
fn random_tree_height_is_logarithmic() {
    // A tree built from random keys is Theta(lg n) tall on average (Knuth:
    // ~2.99 lg n). We use a generous 5 lg n ceiling that random inputs meet
    // with huge margin, catching only genuinely broken (linear) trees.
    let mut t = Bst::new();
    let mut model = BTreeSet::new();
    let mut s = 987_654_321u64;
    for _ in 0..10_000 {
        let k = (lcg(&mut s) >> 8) as i64;
        if t.insert(k) {
            model.insert(k);
        }
    }
    let n = model.len();
    let lg_n = (n as f64).log2();
    assert!(
        (t.height() as f64) < 5.0 * lg_n,
        "height {} not < 5 lg {} = {:.1}",
        t.height(),
        n,
        5.0 * lg_n
    );
    let expect: Vec<i64> = model.iter().copied().collect();
    assert_eq!(t.inorder(), expect);
}
