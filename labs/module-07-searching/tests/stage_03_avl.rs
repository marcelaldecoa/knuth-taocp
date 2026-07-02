//! Stage 3 — Balanced (AVL) trees (Algorithm 6.2.3A).
//!
//! Implement the `AvlTree` methods in src/lab.rs. Lesson:
//! course/module-07-searching/README.md.

use lab_07_searching::AvlTree;
use std::collections::BTreeSet;

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// Insert `keys`, assert the tree stays balanced and sorted the whole way.
fn build_checked(keys: impl IntoIterator<Item = i64>) -> AvlTree {
    let mut t = AvlTree::new();
    let mut n = 0usize;
    for k in keys {
        if t.insert(k) {
            n += 1;
        }
        assert!(t.is_balanced(), "AVL invariant broken after inserting {k}");
        let ino = t.inorder();
        assert_eq!(ino.len(), n, "inorder length tracks distinct insertions");
        assert!(ino.windows(2).all(|w| w[0] < w[1]), "inorder must stay sorted");
    }
    t
}

#[test]
fn the_four_rotation_cases() {
    // Each three-key sequence forces exactly one rebalancing shape; after it,
    // the tree is the perfect 2-level tree on {1,2,3}: root 2, leaves 1 and 3.
    // RR (needs a single left rotation): 1,2,3 ascending.
    // LL (single right rotation): 3,2,1 descending.
    // LR (double, left then right): 3,1,2.
    // RL (double, right then left): 1,3,2.
    for order in [[1, 2, 3], [3, 2, 1], [3, 1, 2], [1, 3, 2]] {
        let t = build_checked(order);
        assert_eq!(t.inorder(), vec![1, 2, 3], "order {order:?}");
        assert_eq!(t.height(), 1, "three keys rebalance to height 1 for {order:?}");
    }
}

#[test]
fn deeper_rotation_cases() {
    // Larger sequences that trigger a rotation at an interior node, not the
    // root — verify balance + order survive a rotation with hanging subtrees.
    // Left-left at the root's child:
    let t = build_checked([20, 10, 30, 5, 15, 3]);
    assert!(t.is_balanced());
    assert_eq!(t.inorder(), vec![3, 5, 10, 15, 20, 30]);
    // Right-left double rotation reached through a taller tree:
    let t = build_checked([20, 10, 30, 25, 40, 27]);
    assert!(t.is_balanced());
    assert_eq!(t.inorder(), vec![10, 20, 25, 27, 30, 40]);
}

#[test]
fn ascending_1_to_1000() {
    // The classic single-rotation workout; a perfectly-fed AVL tree.
    let t = build_checked(1..=1000);
    assert_eq!(t.inorder(), (1..=1000).collect::<Vec<_>>());
    // Fibonacci bound: h <= 1.4405 lg(n+2) - 0.3277. For n = 1000 that is
    // < 14; ascending insertion actually gives a near-perfect ~10-deep tree.
    assert!(t.height() <= 14, "height {} exceeds 14", t.height());
}

#[test]
fn descending_1000_to_1() {
    let t = build_checked((1..=1000).rev());
    assert_eq!(t.inorder(), (1..=1000).collect::<Vec<_>>());
    assert!(t.height() <= 14, "height {} exceeds 14", t.height());
}

#[test]
fn duplicates_are_rejected() {
    let mut t = AvlTree::new();
    assert!(t.insert(42));
    assert!(!t.insert(42), "duplicate must return false");
    assert!(t.insert(17));
    assert!(!t.insert(17));
    assert!(t.is_balanced());
    assert_eq!(t.inorder(), vec![17, 42]);
    // Re-inserting after building a bigger tree still no-ops.
    for k in 1..=50 {
        t.insert(k * 2);
    }
    let before = t.inorder();
    for k in 1..=50 {
        assert!(!t.insert(k * 2));
    }
    assert_eq!(t.inorder(), before);
    assert!(t.is_balanced());
}

#[test]
fn random_10k_stays_balanced_and_short() {
    let mut t = AvlTree::new();
    let mut s = 2463534242u64;
    let mut n = 0u32;
    for _ in 0..10_000 {
        let k = (lcg(&mut s) >> 16) as i64;
        if t.insert(k) {
            n += 1;
        }
    }
    assert!(t.is_balanced(), "10k random insertions must stay balanced");
    let ino = t.inorder();
    assert_eq!(ino.len(), n as usize);
    assert!(ino.windows(2).all(|w| w[0] < w[1]));
    // 1.4405 lg(n+2) - 0.3277 < 19 for n up to ~10^4 (height in edges).
    assert!(t.height() <= 18, "height {} exceeds the Fibonacci bound", t.height());
}

#[test]
fn model_matches_btreeset() {
    // Membership must agree with a BTreeSet oracle over a long insert/query
    // stream, and the AVL invariant must hold at the end.
    let mut t = AvlTree::new();
    let mut model = BTreeSet::new();
    let mut s = 0xBADC0DEu64;
    for _ in 0..8000 {
        let k = (lcg(&mut s) >> 30) as i64 % 2000;
        assert_eq!(t.insert(k), model.insert(k), "insert {k}");
        assert_eq!(t.contains(k), model.contains(&k));
    }
    // Query some keys that were never inserted.
    for q in [-5i64, -1, 100_000, 999_999] {
        assert_eq!(t.contains(q), model.contains(&q), "query {q}");
    }
    assert!(t.is_balanced());
    let expect: Vec<i64> = model.iter().copied().collect();
    assert_eq!(t.inorder(), expect);
}
