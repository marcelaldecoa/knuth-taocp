//! Stage 6 — Radix sort (Algorithm 5.2.5R, least significant digit first).
//!
//! Implement `radix_sort_u64` in src/lab.rs. Lesson:
//! course/module-06-sorting/README.md.
//!
//! Radix sort is not a comparison sort — it distributes keys into piles by one
//! byte at a time, least significant first, using a *stable* counting sort per
//! pass. Stability per pass is the whole ballgame: by induction, after pass k
//! the keys are ordered by their k low bytes. Because no keys are ever
//! compared, the lg(n!) lower bound of §5.3.1 does not apply.

use lab_06_sorting::radix_sort_u64;

fn lcg(seed: u64) -> impl FnMut() -> u64 {
    let mut x = seed;
    move || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        x
    }
}

fn is_sorted_u64(v: &[u64]) -> bool {
    v.windows(2).all(|w| w[0] <= w[1])
}

#[test]
fn degenerate_inputs() {
    let mut e: [u64; 0] = [];
    radix_sort_u64(&mut e);
    assert_eq!(e, []);
    let mut one = [7u64];
    radix_sort_u64(&mut one);
    assert_eq!(one, [7]);
    // All zero: every byte is constant, every pass is the identity.
    let mut z = [0u64; 100];
    radix_sort_u64(&mut z);
    assert_eq!(z, [0u64; 100]);
    // All equal nonzero.
    let mut eq = [12345u64; 50];
    radix_sort_u64(&mut eq);
    assert_eq!(eq, [12345u64; 50]);
}

#[test]
fn small_worked_example_with_extremes() {
    let mut v = [u64::MAX, 0, 256, 255, 1, 1 << 32, 1 << 63, 42];
    let mut expect = v;
    expect.sort_unstable();
    radix_sort_u64(&mut v);
    assert_eq!(v, expect);
}

#[test]
fn agrees_with_std_sort_on_lcg_data_including_dups() {
    let mut rng = lcg(0xF00D_BABE);
    for &n in &[2usize, 3, 10, 255, 256, 257, 1000, 5000] {
        // full-range keys
        let mut v: Vec<u64> = (0..n).map(|_| rng()).collect();
        let mut expect = v.clone();
        expect.sort_unstable();
        radix_sort_u64(&mut v);
        assert_eq!(v, expect, "full-range n={n}");
        // heavy duplicates (small key space)
        let mut d: Vec<u64> = (0..n).map(|_| rng() % 7).collect();
        let mut de = d.clone();
        de.sort_unstable();
        radix_sort_u64(&mut d);
        assert_eq!(d, de, "dups n={n}");
    }
}

#[test]
fn one_hundred_thousand_keys_with_extremes() {
    let mut rng = lcg(0x0102_0304_0506_0708);
    let mut v: Vec<u64> = (0..100_000).map(|_| rng()).collect();
    v.push(0);
    v.push(u64::MAX);
    v.push(u64::MAX);
    let mut expect = v.clone();
    expect.sort_unstable();
    radix_sort_u64(&mut v);
    assert_eq!(v, expect);
    assert!(is_sorted_u64(&v));
}

#[test]
fn stability_via_packed_keys() {
    // Encode (payload, original_index) as (payload << 32) | index. Radix-sort
    // the packed keys: the low 32 bits break ties by original position, so if
    // the sort is genuinely stable per byte, records with equal payload come
    // out in their original relative order — exactly what the low half proves.
    let mut rng = lcg(0x51AB_1E17);
    let n = 5000u64;
    // Random payloads in a small range guarantee many ties.
    let payloads: Vec<u64> = (0..n).map(|_| rng() % 50).collect();
    let mut packed: Vec<u64> = payloads
        .iter()
        .enumerate()
        .map(|(i, &p)| (p << 32) | (i as u64))
        .collect();
    radix_sort_u64(&mut packed);

    // Decode and verify: sorted by payload, and within equal payloads the
    // original indices are strictly increasing (stable order preserved).
    let decoded: Vec<(u64, u64)> = packed.iter().map(|&k| (k >> 32, k & 0xFFFF_FFFF)).collect();
    for w in decoded.windows(2) {
        let (p0, i0) = w[0];
        let (p1, i1) = w[1];
        assert!(p0 <= p1, "payload order broken");
        if p0 == p1 {
            assert!(i0 < i1, "tie order broken: index {i0} before {i1}");
        }
    }
    // Sanity: the reconstructed payload sequence equals payloads sorted.
    let mut sorted_payloads = payloads.clone();
    sorted_payloads.sort_unstable();
    let got: Vec<u64> = decoded.iter().map(|&(p, _)| p).collect();
    assert_eq!(got, sorted_payloads);
}
