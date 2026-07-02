//! Bench — Module 15: External Sorting.
//!
//! Times the full external sort at growing n and prints the I/O bill the §5.4
//! cost model cares about: **records written**, and the number of *passes*
//! (records_written / n) that bill amounts to. Replacement selection turns n
//! records into about n/(2P) initial runs, and a polyphase merge on 3 tapes
//! then finishes in a number of phases that grows like the generalized
//! Fibonacci level of the run count — i.e. ~log_phi(runs). So as n grows by
//! 10x with memory P held fixed, the run count grows ~10x but the pass count
//! creeps up only *logarithmically*: that slow creep in the `passes` column,
//! against the ~10x jump in the `runs` column, is the whole lesson made visible.
//!
//! Run: cargo run -q -p lab-15-external --example bench --features solutions --release

use lab_15_external::*;
use std::time::Instant;

/// A hand-rolled LCG (Knuth's MMIX multiplier) — no external crates.
fn lcg_vec(n: usize, seed: u64) -> Vec<i64> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (x >> 16) as i64
        })
        .collect()
}

fn main() {
    // Fixed workspace P and a 3-tape polyphase merge, so the run count grows
    // linearly with n and the phase count grows only logarithmically.
    const P: usize = 64;
    const TAPES: usize = 3;

    println!("External sort (Algorithm §5.4): replacement selection (P = {P}) + polyphase merge on {TAPES} tapes");
    println!("Timing external_sort; records_written is the I/O bill, passes = records_written / n.");
    println!("Expected: runs ~ n/(2P) grows ~10x per row; passes ~ log_phi(runs) grows only additively.");
    println!();
    println!(
        "{:>10} | {:>10} | {:>6} | {:>8} | {:>14} | {:>7} | {:>7}",
        "n", "time", "ratio", "runs", "recs_written", "passes", "t/n(ns)"
    );
    println!("{}", "-".repeat(78));

    let mut prev: Option<f64> = None;
    for &n in &[1_000usize, 10_000, 100_000, 1_000_000, 10_000_000] {
        let data = lcg_vec(n, 0x5445_5354_0000_0001 ^ n as u64);

        // Report the initial run count (cheap relative to the sort itself).
        let runs = replacement_selection(&data, P).len();

        // Time the full pipeline.
        let t0 = Instant::now();
        let (out, io) = external_sort(&data, P, TAPES);
        let secs = t0.elapsed().as_secs_f64();

        // Cheap correctness guard so the optimizer can't elide the work.
        assert_eq!(out.len(), n);
        debug_assert!(out.windows(2).all(|w| w[0] <= w[1]));

        let passes = io.records_written as f64 / n as f64;
        let ratio = prev.map(|p| secs / p).unwrap_or(f64::NAN);
        let ratio_str = if ratio.is_nan() {
            "  -  ".to_string()
        } else {
            format!("{ratio:5.2}x")
        };

        println!(
            "{:>10} | {:>10} | {:>6} | {:>8} | {:>14} | {:>7.2} | {:>7.1}",
            n,
            format!("{:.4}s", secs),
            ratio_str,
            runs,
            io.records_written,
            passes,
            secs * 1e9 / n as f64,
        );
        prev = Some(secs);
    }

    println!();
    println!("Read the `passes` column: it rises roughly with log_phi of the `runs` count,");
    println!("not with n — that logarithmic pass growth is why external merging scales.");
}
