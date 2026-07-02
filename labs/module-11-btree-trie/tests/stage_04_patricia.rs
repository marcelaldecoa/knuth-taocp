//! Stage 4 — Patricia: compressed binary tries (§6.3, Algorithm P essence).
//!
//! Implement `Patricia::{insert, contains, node_count}` in src/lab.rs.
//! A correct Patricia tests only *distinguishing* bits: n keys must fit in
//! at most 2n - 1 nodes no matter how long their shared prefixes are.
//! Lesson: course/module-11-btree-trie/README.md.

use lab_11_btree_trie::Patricia;
use std::collections::HashSet;

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// Deterministic Fisher–Yates shuffle driven by the LCG.
fn shuffle(v: &mut [u64], seed: u64) {
    let mut s = seed;
    for i in (1..v.len()).rev() {
        let j = (lcg(&mut s) % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
}

#[test]
fn empty_singleton_and_pair() {
    let mut p = Patricia::new();
    assert!(!p.contains(0));
    assert!(!p.contains(u64::MAX));
    assert_eq!(p.node_count(), 0, "empty Patricia has no nodes");

    assert!(p.insert(0xF00Du64));
    assert_eq!(p.node_count(), 1, "one key = one node (a single leaf)");
    assert!(p.contains(0xF00D));
    assert!(!p.contains(0xF00C));
    assert!(!p.insert(0xF00D), "duplicate insert returns false");
    assert_eq!(p.node_count(), 1, "duplicate insert must not add nodes");

    assert!(p.insert(0xF00C));
    assert_eq!(p.node_count(), 3, "two keys = two leaves + one branch");
    assert!(p.contains(0xF00C) && p.contains(0xF00D));
}

#[test]
fn model_check_against_hashset_lcg_u64() {
    let mut p = Patricia::new();
    let mut model: HashSet<u64> = HashSet::new();
    let mut s = 0xBEEF_CAFE_1234u64;
    // Keys masked to 20 bits so duplicates actually occur in the stream.
    for step in 0..6000 {
        let k = lcg(&mut s) >> 44;
        assert_eq!(p.insert(k), model.insert(k), "step {step}: insert {k:#x}");
        if step % 250 == 0 {
            assert!(
                p.node_count() <= 2 * model.len().max(1) - 1,
                "step {step}: {} nodes for {} keys",
                p.node_count(),
                model.len()
            );
        }
    }
    // Membership sweep: everything inserted, plus fresh probes.
    for &k in &model {
        assert!(p.contains(k), "present key {k:#x}");
    }
    for _ in 0..4000 {
        let k = lcg(&mut s) >> 43; // 21-bit probes: half lie outside the key space
        assert_eq!(p.contains(k), model.contains(&k), "probe {k:#x}");
    }
    assert_eq!(p.node_count(), 2 * model.len() - 1);
}

#[test]
fn adversarial_shared_prefix_families() {
    let mut p = Patricia::new();
    let mut keys: Vec<u64> = Vec::new();
    // Family 1: 512 keys agreeing on their top 32 bits — a plain trie
    // would spend 32 levels before the first real decision.
    keys.extend((0..512u64).map(|i| 0xFFFF_FFFF_0000_0000 | i));
    // Family 2: the 64 powers of two — every pair differs first at a
    // different bit — plus both extremes.
    keys.extend((0..64).map(|i| 1u64 << i));
    keys.push(u64::MAX);
    // Family 3: all-ones prefixes 1, 11, 111, ... (0b1, 0b11, 0b111, ...).
    keys.extend((2..64).map(|i| (1u64 << i) - 1));
    // (1<<1)-1 = 1 and (1<<0)-1 = 0... 1 already in family 2; skip dups below.

    let mut inserted = 0usize;
    let mut seen: HashSet<u64> = HashSet::new();
    for &k in &keys {
        assert_eq!(p.insert(k), seen.insert(k), "insert {k:#018x}");
        if seen.len() > 0 {
            inserted = seen.len();
            assert!(
                p.node_count() <= 2 * inserted - 1,
                "{} nodes for {} keys after {k:#018x}",
                p.node_count(),
                inserted
            );
        }
    }
    for &k in &seen {
        assert!(p.contains(k), "present {k:#018x}");
    }
    // Near misses around family 1's fan-out and prefix.
    for i in 512..1024u64 {
        assert!(!p.contains(0xFFFF_FFFF_0000_0000 | i), "absent member {i}");
    }
    assert!(!p.contains(0xFFFF_FFFE_0000_0000));
    assert!(!p.contains(0x7FFF_FFFF_0000_0001));
    assert_eq!(p.node_count(), 2 * inserted - 1);
}

#[test]
fn insert_order_independence() {
    // The same key set in three very different orders must produce a tree
    // with the same node count and identical answers to every query —
    // Patricia's shape is determined by the SET, not the insertion history.
    let mut s = 0x0123_4567_89ABu64;
    let mut base: Vec<u64> = (0..300u64)
        .map(|i| if i % 3 == 0 { 0xAAAA_0000_0000_0000 | i } else { lcg(&mut s) })
        .collect();
    base.sort_unstable();
    base.dedup();

    let forward = base.clone();
    let mut backward = base.clone();
    backward.reverse();
    let mut scrambled = base.clone();
    shuffle(&mut scrambled, 0xD1CE);

    let mut trees = Vec::new();
    for order in [forward, backward, scrambled] {
        let mut p = Patricia::new();
        for &k in &order {
            assert!(p.insert(k));
        }
        trees.push(p);
    }
    let n = base.len();
    assert!(trees[0].node_count() <= 2 * n - 1);
    assert_eq!(trees[0].node_count(), trees[1].node_count());
    assert_eq!(trees[0].node_count(), trees[2].node_count());
    // 2000 probes: present and absent alike must agree across all three.
    let mut s2 = 4242u64;
    for _ in 0..2000 {
        let k = if lcg(&mut s2) % 2 == 0 {
            base[(lcg(&mut s2) % n as u64) as usize]
        } else {
            lcg(&mut s2)
        };
        let want = base.binary_search(&k).is_ok();
        for (i, p) in trees.iter().enumerate() {
            assert_eq!(p.contains(k), want, "tree {i}, probe {k:#018x}");
        }
    }
}

#[test]
fn node_count_bound_at_every_step() {
    // node_count <= 2*keys - 1, checked after EVERY successful insert.
    let mut p = Patricia::new();
    let mut n = 0usize;
    let mut s = 31u64;
    for _ in 0..2000 {
        let k = lcg(&mut s) >> 40; // 24-bit keys, some duplicates
        if p.insert(k) {
            n += 1;
        }
        assert!(
            p.node_count() <= 2 * n - 1,
            "{} nodes for {n} keys — are you storing non-distinguishing bits?",
            p.node_count()
        );
    }
}
