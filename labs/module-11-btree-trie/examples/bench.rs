//! Flagship benchmark for Module 11 — B-trees (TAOCP Vol. 3, §6.2.4).
//!
//! What is timed and the asymptotics the lesson derives:
//!
//!   * Bulk insertion of n distinct keys into a B-tree of order m. Each
//!     insert visits O(log_t n) nodes (t = ⌈m/2⌉ by Theorem B) and does an
//!     O(log m) binary search per node, so the whole build is O(n log n).
//!     The `ratio` column should hover near the base as n grows by ×8:
//!     just above the ratio for perfectly linear work, the excess being the
//!     slowly-rising log factor.
//!   * n membership queries (`contains`) after the build — same O(log_t n)
//!     per probe.
//!   * The measured tree `height` next to Theorem B's bound
//!     h ≤ 1 + log_t((n+1)/2). Watch height crawl up by ~1 per few decades
//!     while n explodes: that is the log_{⌈m/2⌉} growth made visible.
//!
//! Run:
//!   cargo run -q -p lab-11-btree-trie --example bench --features solutions --release

use lab_11_btree_trie::*;
use std::time::Instant;

/// Hand-rolled LCG (no external crates). Full-period over u64, so the first
/// 2^64 outputs are a permutation — consecutive draws are always distinct,
/// which keeps every generated key unique for any n we bench.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
}

/// Theorem B height bound: 1 + log_t((n+1)/2), t = ⌈m/2⌉.
fn height_bound(m: usize, n: usize) -> f64 {
    let t = ((m + 1) / 2) as f64; // ⌈m/2⌉ in integer arithmetic
    1.0 + (((n + 1) as f64) / 2.0).ln() / t.ln()
}

fn main() {
    const ORDER: usize = 64;

    println!("B-tree bulk insert / search vs n   (order m = {ORDER}, t = ⌈m/2⌉ = {})", (ORDER + 1) / 2);
    println!("expected: build O(n log n); per-op O(log_t n); height ~ log_t n\n");
    println!(
        "{:>10} | {:>12} | {:>7} | {:>12} | {:>6} | {:>10}",
        "n", "insert(ms)", "ratio", "contains(ms)", "height", "bound"
    );
    println!("{:->10}-+-{:->12}-+-{:->7}-+-{:->12}-+-{:->6}-+-{:->10}", "", "", "", "", "", "");

    let sizes = [10_000usize, 80_000, 640_000, 5_120_000];
    let mut prev_insert: Option<f64> = None;

    for &n in &sizes {
        // Materialize n distinct keys first so we time the tree, not the LCG.
        let mut rng = Lcg(0x1234_5678_9abc_def0);
        let keys: Vec<i64> = (0..n).map(|_| rng.next() as i64).collect();

        // --- time the bulk insert ---
        let mut tree = BTree::new(ORDER);
        let t0 = Instant::now();
        for &k in &keys {
            tree.insert(k);
        }
        let insert_secs = t0.elapsed().as_secs_f64();

        // --- time n membership queries (all present) ---
        let t1 = Instant::now();
        let mut hits = 0usize;
        for &k in &keys {
            if tree.contains(k) {
                hits += 1;
            }
        }
        let contains_secs = t1.elapsed().as_secs_f64();
        assert_eq!(hits, n, "every inserted key must be found");

        let ratio = match prev_insert {
            Some(p) if p > 0.0 => format!("{:.2}x", insert_secs / p),
            _ => "  -".to_string(),
        };
        prev_insert = Some(insert_secs);

        println!(
            "{:>10} | {:>12.3} | {:>7} | {:>12.3} | {:>6} | {:>10.2}",
            n,
            insert_secs * 1e3,
            ratio,
            contains_secs * 1e3,
            tree.height(),
            height_bound(ORDER, n),
        );
    }

    println!("\n(insert/contains columns are milliseconds; ratio compares insert to the previous row.)");
    println!("Each row grows n by ×8: a pure-O(n) cost would show ratio 8.00. The surplus is the");
    println!("O(log_t n) factor plus growing cache-miss cost — the height column is the clean signal,");
    println!("crawling up by ~1 while n multiplies, exactly the log_{{⌈m/2⌉}} bound of Theorem B.");
}
