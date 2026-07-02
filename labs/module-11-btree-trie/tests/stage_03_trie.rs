//! Stage 3 — Digital searching: binary tries (§6.3, Algorithm T with binary
//! "characters": the bits of a u32, most significant bit first).
//!
//! Implement `BinaryTrie::{insert, contains, remove, count}` in src/lab.rs.
//! Lesson: course/module-11-btree-trie/README.md.

use lab_11_btree_trie::BinaryTrie;
use std::collections::HashSet;

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn empty_trie_behaves() {
    let mut t = BinaryTrie::new();
    assert_eq!(t.count(), 0);
    assert!(!t.contains(0));
    assert!(!t.contains(u32::MAX));
    assert!(!t.remove(12345), "removing from an empty trie returns false");
    assert_eq!(t.count(), 0);
}

#[test]
fn basics_including_extreme_keys() {
    let mut t = BinaryTrie::new();
    for &k in &[0u32, 1, u32::MAX, u32::MAX - 1, 0x8000_0000, 0x7FFF_FFFF] {
        assert!(t.insert(k), "fresh insert of {k:#010x}");
        assert!(t.contains(k));
    }
    assert_eq!(t.count(), 6);
    assert!(!t.insert(0), "duplicate insert returns false");
    assert_eq!(t.count(), 6, "duplicate insert must not change count");
    assert!(t.remove(0x8000_0000));
    assert!(!t.contains(0x8000_0000));
    // 0x8000_0000 and 0 differ only in the very first bit tested; 0 and 1
    // only in the last — both neighbours must survive the removal.
    assert!(t.contains(0) && t.contains(1) && t.contains(0x7FFF_FFFF));
    assert_eq!(t.count(), 5);
}

#[test]
fn model_check_against_hashset() {
    // Mixed insert/remove/contains over a small pool of u32 keys so the
    // same key gets hit many times, checked move-by-move against HashSet.
    let mut s = 0xFACE_FEEDu64;
    let pool: Vec<u32> = (0..512).map(|_| lcg(&mut s) as u32).collect();
    let mut t = BinaryTrie::new();
    let mut model: HashSet<u32> = HashSet::new();
    for step in 0..20_000 {
        let k = pool[(lcg(&mut s) % 512) as usize];
        match lcg(&mut s) % 3 {
            0 => assert_eq!(t.insert(k), model.insert(k), "step {step}: insert {k:#010x}"),
            1 => assert_eq!(t.remove(k), model.remove(&k), "step {step}: remove {k:#010x}"),
            _ => assert_eq!(t.contains(k), model.contains(&k), "step {step}: contains {k:#010x}"),
        }
        if step % 500 == 0 {
            assert_eq!(t.count(), model.len(), "count drifted at step {step}");
        }
    }
    assert_eq!(t.count(), model.len());
    for &k in &pool {
        assert_eq!(t.contains(k), model.contains(&k), "final sweep {k:#010x}");
    }
}

#[test]
fn long_shared_prefix_families() {
    let mut t = BinaryTrie::new();
    // 256 keys sharing a 24-bit prefix: one long common spine, then fan-out.
    for i in 0..256u32 {
        assert!(t.insert(0xDEAD_BE00 | i));
    }
    // A second family agreeing with the first on nothing but disagreeing
    // among themselves only in the LAST bit tested.
    assert!(t.insert(0x0000_0002));
    assert!(t.insert(0x0000_0003));
    assert_eq!(t.count(), 258);
    for i in 0..256u32 {
        assert!(t.contains(0xDEAD_BE00 | i), "prefix family member {i}");
    }
    assert!(!t.contains(0xDEAD_BD00), "prefix sibling must be absent");
    assert!(!t.contains(0x0000_0001));
    // Remove every even member of the big family; odds must survive.
    for i in (0..256u32).step_by(2) {
        assert!(t.remove(0xDEAD_BE00 | i));
    }
    for i in 0..256u32 {
        assert_eq!(t.contains(0xDEAD_BE00 | i), i % 2 == 1, "member {i} after removals");
    }
    assert_eq!(t.count(), 130);
}

#[test]
fn remove_then_reinsert_cycles() {
    let mut t = BinaryTrie::new();
    let keys = [0xCAFE_BABEu32, 0xCAFE_BABF, 0x0BAD_F00D];
    for &k in &keys {
        assert!(t.insert(k));
    }
    for round in 0..100 {
        assert!(t.remove(0xCAFE_BABE), "round {round}: remove");
        assert!(!t.contains(0xCAFE_BABE));
        assert!(!t.remove(0xCAFE_BABE), "round {round}: double remove");
        assert_eq!(t.count(), 2);
        assert!(t.insert(0xCAFE_BABE), "round {round}: reinsert");
        assert!(t.contains(0xCAFE_BABE));
        assert_eq!(t.count(), 3);
        // The near-twin (differs in the last bit) must never be disturbed.
        assert!(t.contains(0xCAFE_BABF), "round {round}: twin survived");
    }
}

#[test]
fn count_tracks_distinct_keys_exactly() {
    let mut t = BinaryTrie::new();
    let mut s = 7u64;
    let mut model: HashSet<u32> = HashSet::new();
    for _ in 0..4000 {
        let k = (lcg(&mut s) % 1000) as u32;
        t.insert(k);
        model.insert(k);
        assert_eq!(t.count(), model.len());
    }
    let snapshot: Vec<u32> = model.iter().copied().collect();
    for k in snapshot {
        assert!(t.remove(k));
        model.remove(&k);
        assert_eq!(t.count(), model.len());
    }
    assert_eq!(t.count(), 0, "emptied trie");
    assert!(t.insert(42), "an emptied trie must still accept inserts");
    assert!(t.contains(42));
    assert_eq!(t.count(), 1);
}
