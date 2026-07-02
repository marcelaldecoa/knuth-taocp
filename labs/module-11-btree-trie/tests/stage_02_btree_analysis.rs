//! Stage 2 — B-tree invariants and the height bound (§6.2.4 analysis).
//!
//! Implement `BTree::{is_valid, height}` in src/lab.rs (insert/contains/
//! keys_inorder from stage 1 are assumed working).
//!
//! The theorem under test (lesson, Theorem B): a B-tree of order m holding
//! n >= 1 keys has height (in levels)
//!
//!     h  <=  1 + log_t((n + 1) / 2),      t = ceil(m / 2).
//!
//! Lesson: course/module-11-btree-trie/README.md.

use lab_11_btree_trie::BTree;

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// The Theorem B ceiling: 1 + log_{ceil(m/2)}((n+1)/2), as an f64.
fn height_bound(n: usize, m: usize) -> f64 {
    let t = ((m + 1) / 2) as f64; // ceil(m/2)
    1.0 + (((n + 1) as f64) / 2.0).ln() / t.ln()
}

#[test]
fn empty_and_singleton_trees() {
    let t = BTree::new(3);
    assert!(t.is_valid(), "the empty tree is a valid B-tree");
    assert_eq!(t.height(), 0, "empty tree has height 0 (levels)");

    let mut t = BTree::new(3);
    t.insert(7);
    assert!(t.is_valid());
    assert_eq!(t.height(), 1, "a lone root leaf is one level");
}

#[test]
fn validity_checkpointed_during_random_op_sequences() {
    for &m in &[3usize, 4, 8] {
        let mut t = BTree::new(m);
        let mut s = 0xC0FF_EE00u64 ^ (m as u64);
        let mut inserted = 0usize;
        for step in 0..4000 {
            let k = (lcg(&mut s) % 8000) as i64;
            if t.insert(k) {
                inserted += 1;
            }
            if step % 97 == 0 {
                assert!(t.is_valid(), "invariants broken at step {step}, m={m}");
                let ks = t.keys_inorder();
                assert_eq!(ks.len(), inserted, "key count drifted at step {step}, m={m}");
                assert!(ks.windows(2).all(|w| w[0] < w[1]), "inorder not sorted, m={m}");
            }
        }
        assert!(t.is_valid(), "final tree invalid, m={m}");
    }
}

#[test]
fn height_bound_ten_thousand_keys_order_8() {
    // n = 10_000, m = 8, t = 4: h <= 1 + log_4(5000.5) ~ 7.14, so h <= 7.
    let (n, m) = (10_000usize, 8usize);
    let mut t = BTree::new(m);
    for i in 0..n as u64 {
        // Multiplication by an odd constant is a bijection mod 2^64:
        // n distinct keys, in scrambled order.
        let k = i.wrapping_mul(0x9E37_79B9_7F4A_7C15) as i64;
        assert!(t.insert(k));
    }
    assert!(t.is_valid());
    assert_eq!(t.keys_inorder().len(), n);
    let h = t.height();
    assert!(
        (h as f64) <= height_bound(n, m) + 1e-9,
        "height {h} exceeds Theorem B bound {:.3} (n={n}, m={m})",
        height_bound(n, m)
    );
    assert!(h >= 4, "10_000 keys in order-8 nodes need at least 4 levels");
}

#[test]
fn degenerate_order_3_ascending_stress() {
    // m = 3 is the smallest legal order (a 2-3 tree) and ascending input
    // forces the maximum number of splits along the right spine.
    let n = 5000usize;
    let mut t = BTree::new(3);
    let mut last_h = 0usize;
    for i in 0..n as i64 {
        assert!(t.insert(i));
        let h = t.height();
        assert!(
            h == last_h || h == last_h + 1,
            "height must grow only via root splits, one level at a time"
        );
        last_h = h;
        if i % 250 == 0 {
            assert!(t.is_valid(), "invalid after {i} ascending inserts");
        }
    }
    assert!(t.is_valid());
    let h = t.height();
    assert!(
        (h as f64) <= height_bound(n, 3) + 1e-9,
        "height {h} exceeds bound {:.3} for m=3",
        height_bound(n, 3)
    );
    assert_eq!(t.keys_inorder(), (0..n as i64).collect::<Vec<_>>());
}

#[test]
fn height_and_validity_across_orders() {
    // The same 3000 keys under growing fan-out — always within Theorem B.
    let n = 3000usize;
    for &m in &[3usize, 4, 7, 16, 64] {
        let mut t = BTree::new(m);
        let mut s = 99u64;
        let mut inserted = 0usize;
        while inserted < n {
            if t.insert(lcg(&mut s) as i64) {
                inserted += 1;
            }
        }
        assert!(t.is_valid(), "m={m}");
        let h = t.height();
        assert!((h as f64) <= height_bound(n, m) + 1e-9, "m={m}, h={h}");
        assert!(h >= 2, "3000 keys never fit in a single node");
    }
}
