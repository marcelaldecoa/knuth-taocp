//! Bench — Module 20: the oblivious-sorting constant-factor story.
//!
//! Times Batcher's odd-even merge network (branch-free compare-exchanges over a
//! fixed comparator list) against a data-dependent comparison sort at growing
//! powers of two. Prints `n | net time | sort time | ratio`.
//!
//! Run with the reference solutions wired in:
//!   cargo run --release --example bench -p lab-20-networks --features solutions
//!
//! Networks make the same comparisons no matter the data — no branch
//! mispredictions on the comparator schedule — which is exactly why they map
//! onto SIMD and GPU sort kernels. This bench shows the network's cost is a
//! smooth function of its (fixed) comparator count.

use std::time::Instant;

use lab_20_networks::{apply_network, odd_even_merge_network};

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn main() {
    println!("{:>6} | {:>7} | {:>12} | {:>12} | {:>7}", "n", "comps", "net (ns/arr)", "sort (ns/arr)", "ratio");
    println!("{}", "-".repeat(60));

    for &lg in &[2u32, 3, 4, 5, 6, 7, 8, 9, 10] {
        let n = 1usize << lg;
        let net = odd_even_merge_network(n);
        let trials = 200_000usize / n + 8;

        // Prebuild random arrays so allocation isn't timed.
        let mut state = 0x2545_f491_4f6c_dd1du64 ^ (n as u64);
        let inputs: Vec<Vec<i64>> = (0..trials)
            .map(|_| (0..n).map(|_| (lcg(&mut state) >> 33) as i64).collect())
            .collect();

        // Network: apply the fixed comparator list.
        let mut work = inputs.clone();
        let t0 = Instant::now();
        for a in work.iter_mut() {
            apply_network(&net, a);
        }
        let net_ns = t0.elapsed().as_nanos() as f64 / trials as f64;

        // Comparison sort: the standard library's data-dependent sort.
        let mut work2 = inputs.clone();
        let t1 = Instant::now();
        for a in work2.iter_mut() {
            a.sort_unstable();
        }
        let sort_ns = t1.elapsed().as_nanos() as f64 / trials as f64;

        println!(
            "{:>6} | {:>7} | {:>12.1} | {:>12.1} | {:>7.2}",
            n,
            net.len(),
            net_ns,
            sort_ns,
            sort_ns / net_ns
        );
    }

    println!("\nThe network's comparator count grows as ~n(lg n)^2/4; its time is");
    println!("a smooth, branch-free function of that count. For small n the fixed");
    println!("schedule beats a general comparison sort's per-call overhead.");
}
