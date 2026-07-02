//! Stage 4 — Hashing with open addressing (Algorithm 6.4L linear probing,
//! Algorithm 6.4D double hashing).
//!
//! Implement `LinearProbe` and `DoubleHash` in src/lab.rs. Lesson:
//! course/module-07-searching/README.md.

use lab_07_searching::{DoubleHash, LinearProbe};
use std::collections::HashSet;

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// `count` distinct u64 keys from an LCG seeded at `seed`.
fn distinct_keys(count: usize, seed: u64) -> Vec<u64> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(count);
    let mut s = seed;
    while out.len() < count {
        let k = lcg(&mut s) >> 3;
        if seen.insert(k) {
            out.push(k);
        }
    }
    out
}

#[test]
fn worked_example_linear_probing_m7() {
    // §6.4, M = 7, h(K) = K mod 7. The probe sequence *decreases*.
    let mut lp = LinearProbe::new(7);
    assert!(lp.insert(12)); //          12 mod 7 = 5 -> slot 5
    assert!(lp.insert(19)); // 19 mod 7 = 5 taken   -> slot 4
    assert!(lp.insert(5)); //   5 mod 7 = 5,4 taken -> slot 3
    assert_eq!(lp.probes_for(12), 1);
    assert_eq!(lp.probes_for(19), 2);
    assert_eq!(lp.probes_for(5), 3);
    assert!(lp.contains(19) && !lp.contains(26));
    assert!(!lp.insert(19), "duplicate rejected");
    // Fill to N = M - 1 = 6, then overflow on the seventh.
    assert!(lp.insert(0) && lp.insert(1) && lp.insert(2));
    assert!(!lp.insert(100), "table full at N = M - 1");
    assert!(lp.contains(2));
    // Probing for an absent key stops at the first empty slot found.
    assert!(!lp.contains(26));
}

#[test]
fn worked_example_double_hashing_m7() {
    // c = h2(K) = 1 + (K mod 5); probe i <- i - c (mod 7).
    let mut dh = DoubleHash::new(7);
    assert!(dh.insert(12)); //                       slot 5
    assert!(dh.insert(19)); // 5 taken, c = 1+4 = 5, 5-5   -> slot 0
    assert!(dh.insert(5)); //  5 taken, c = 1+0 = 1, 5-1   -> slot 4
    assert_eq!(dh.probes_for(12), 1);
    assert_eq!(dh.probes_for(19), 2);
    assert_eq!(dh.probes_for(5), 2);
    assert!(dh.contains(5) && !dh.contains(26));
    assert!(!dh.insert(12), "duplicate rejected");
}

#[test]
#[should_panic(expected = "prime")]
fn linear_probe_rejects_composite_size() {
    LinearProbe::new(100);
}

#[test]
#[should_panic(expected = "prime")]
fn double_hash_rejects_composite_size() {
    DoubleHash::new(9);
}

const M: usize = 1009; // prime; M - 2 = 1007 keeps double hashing legal

#[test]
fn linear_probe_no_false_positive_or_negative() {
    for &(n, seed) in &[(M / 2, 111u64), (9 * M / 10, 222u64)] {
        let keys = distinct_keys(n, seed);
        let mut lp = LinearProbe::new(M);
        let mut model = HashSet::new();
        for &k in &keys {
            assert!(lp.insert(k), "fresh key {k} inserts");
            model.insert(k);
        }
        // Every inserted key is found (no false negatives).
        for &k in &keys {
            assert!(lp.contains(k), "false negative for {k}");
        }
        // A fresh stream of non-member keys is never found (no false pos).
        let mut s = seed ^ 0xFFFF;
        let mut checked = 0;
        while checked < 2000 {
            let q = lcg(&mut s) >> 3;
            if !model.contains(&q) {
                assert!(!lp.contains(q), "false positive for {q}");
                checked += 1;
            }
        }
    }
}

#[test]
fn double_hash_no_false_positive_or_negative() {
    for &(n, seed) in &[(M / 2, 333u64), (9 * M / 10, 444u64)] {
        let keys = distinct_keys(n, seed);
        let mut dh = DoubleHash::new(M);
        let mut model = HashSet::new();
        for &k in &keys {
            assert!(dh.insert(k), "fresh key {k} inserts");
            model.insert(k);
        }
        for &k in &keys {
            assert!(dh.contains(k), "false negative for {k}");
        }
        let mut s = seed ^ 0xABCD;
        let mut checked = 0;
        while checked < 2000 {
            let q = lcg(&mut s) >> 3;
            if !model.contains(&q) {
                assert!(!dh.contains(q), "false positive for {q}");
                checked += 1;
            }
        }
    }
}

/// Mean probes for a *successful* search over exactly `keys`.
fn avg_successful<F: Fn(u64) -> u32>(keys: &[u64], probes: F) -> f64 {
    let total: u64 = keys.iter().map(|&k| probes(k) as u64).sum();
    total as f64 / keys.len() as f64
}

#[test]
fn linear_probing_average_probes_matches_1962_analysis() {
    // Knuth 1962: a successful linear-probing search averages about
    // (1 + 1/(1-alpha))/2 probes. At alpha = 0.5 that is 1.5; assert < 2.0.
    let n = M / 2; // alpha ~ 0.5
    let keys = distinct_keys(n, 24601);
    let mut lp = LinearProbe::new(M);
    for &k in &keys {
        lp.insert(k);
    }
    let avg = avg_successful(&keys, |k| lp.probes_for(k));
    assert!(avg < 2.0, "alpha=0.5 successful average {avg:.3} should be < 2.0 (theory 1.5)");
    assert!(avg > 1.0, "average must exceed 1 whenever collisions occur");
}

#[test]
fn linear_probing_degrades_at_high_load() {
    // The same analysis predicts a steep rise near alpha = 1:
    // (1 + 1/(1-0.9))/2 = 5.5 at alpha = 0.9, far above the alpha = 0.5 cost.
    let keys_half = distinct_keys(M / 2, 555);
    let keys_high = distinct_keys(9 * M / 10, 666);

    let mut lp_half = LinearProbe::new(M);
    for &k in &keys_half {
        lp_half.insert(k);
    }
    let mut lp_high = LinearProbe::new(M);
    for &k in &keys_high {
        lp_high.insert(k);
    }
    let avg_half = avg_successful(&keys_half, |k| lp_half.probes_for(k));
    let avg_high = avg_successful(&keys_high, |k| lp_high.probes_for(k));
    assert!(
        avg_high > avg_half,
        "alpha=0.9 average {avg_high:.3} must exceed alpha=0.5 average {avg_half:.3}"
    );
    assert!(avg_high > 2.5, "alpha=0.9 successful average {avg_high:.3} should be well above 2");
}

#[test]
fn double_hashing_beats_linear_probing_at_high_load() {
    // Same key set, alpha = 0.9. Double hashing avoids primary clustering, so
    // its successful-search average (theory ~2.56) is far below linear
    // probing's (~5.5).
    let n = 9 * M / 10;
    let keys = distinct_keys(n, 0xF00D);

    let mut lp = LinearProbe::new(M);
    let mut dh = DoubleHash::new(M);
    for &k in &keys {
        assert!(lp.insert(k));
        assert!(dh.insert(k));
    }
    let lp_avg = avg_successful(&keys, |k| lp.probes_for(k));
    let dh_avg = avg_successful(&keys, |k| dh.probes_for(k));
    assert!(
        dh_avg < lp_avg,
        "double hashing average {dh_avg:.3} must beat linear probing {lp_avg:.3} at alpha=0.9"
    );
    assert!(dh_avg < 3.5, "double-hashing average {dh_avg:.3} should be near the ~2.56 theory");
}
