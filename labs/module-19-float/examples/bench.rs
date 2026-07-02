//! Flagship benchmark for Module 19 — the *cost* of accuracy.
//!
//! Run with the reference solutions:
//!
//! ```text
//! cargo run -q -p lab-19-float --example bench --features solutions --release
//! ```
//!
//! Two tables:
//!   1. `naive_sum` vs `kahan_sum` at growing n — Kahan does ~4x the flops per
//!      element, so it should be a small constant factor slower while being far
//!      more accurate (correctness is proven in the tests, not here).
//!   2. `Float::add` (our software float) vs the hardware `f64 +` — the price of
//!      doing in software what silicon does in one cycle.
//!
//! Timing only; std::time::Instant, no external crates.

use lab_19_float::{kahan_sum, naive_sum, Float};
use std::time::Instant;

fn main() {
    println!("Module 19 — floating-point cost benchmark\n");
    bench_summation();
    println!();
    bench_add();
}

fn bench_summation() {
    println!("naive_sum vs kahan_sum (summing 0.1)");
    println!("{:>10} | {:>12} | {:>12} | {:>7}", "n", "naive (ns)", "kahan (ns)", "ratio");
    println!("{:->10}-+-{:->12}-+-{:->12}-+-{:->7}", "", "", "", "");
    let mut n = 1_000usize;
    while n <= 4_000_000 {
        let xs = vec![0.1f64; n];

        let reps = (8_000_000 / n).max(1);
        let mut sink = 0.0f64;

        let t = Instant::now();
        for _ in 0..reps {
            sink += naive_sum(&xs);
        }
        let naive_ns = t.elapsed().as_secs_f64() * 1e9 / (reps as f64 * n as f64);

        let t = Instant::now();
        for _ in 0..reps {
            sink += kahan_sum(&xs);
        }
        let kahan_ns = t.elapsed().as_secs_f64() * 1e9 / (reps as f64 * n as f64);

        println!(
            "{:>10} | {:>12.4} | {:>12.4} | {:>7.2}",
            n,
            naive_ns,
            kahan_ns,
            kahan_ns / naive_ns
        );
        std::hint::black_box(sink);
        n *= 4;
    }
    println!("(times are per element; ratio = kahan / naive)");
}

fn bench_add() {
    println!("Float::add (software) vs f64 + (hardware)");
    println!("{:>10} | {:>12} | {:>12} | {:>7}", "n", "soft (ns)", "hard (ns)", "ratio");
    println!("{:->10}-+-{:->12}-+-{:->12}-+-{:->7}", "", "", "", "");

    let mut n = 1_000usize;
    while n <= 4_000_000 {
        // Deterministic data in a safe normal band.
        let mut lcg = 0x9E3779B97F4A7C15u64;
        let data: Vec<f64> = (0..n)
            .map(|_| {
                lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
                let biased = 1000 + (lcg >> 12) % 48; // exponent in [-23, 24]
                let mantissa = lcg & 0x000f_ffff_ffff_ffff;
                f64::from_bits((biased << 52) | mantissa)
            })
            .collect();
        let fdata: Vec<Float> = data.iter().map(|&x| Float::from_f64(x)).collect();

        let reps = (8_000_000 / n).max(1);

        let t = Instant::now();
        let mut acc = Float::from_f64(0.0);
        for _ in 0..reps {
            for f in &fdata {
                acc = acc.add(f);
            }
        }
        let soft_ns = t.elapsed().as_secs_f64() * 1e9 / (reps as f64 * n as f64);
        std::hint::black_box(acc.to_f64());

        let t = Instant::now();
        let mut hacc = 0.0f64;
        for _ in 0..reps {
            for &x in &data {
                hacc += x;
            }
        }
        let hard_ns = t.elapsed().as_secs_f64() * 1e9 / (reps as f64 * n as f64);
        std::hint::black_box(hacc);

        println!(
            "{:>10} | {:>12.4} | {:>12.4} | {:>7.1}",
            n,
            soft_ns,
            hard_ns,
            soft_ns / hard_ns.max(1e-9)
        );
        n *= 4;
    }
    println!("(times are per addition; ratio = software / hardware)");
}
