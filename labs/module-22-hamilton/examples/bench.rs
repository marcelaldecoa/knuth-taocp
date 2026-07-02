//! Held–Karp growth curve — the honest face of NP-hardness.
//!
//! Times `shortest_hamiltonian_path` (the O(2^n · n^2) bitmask DP) at growing
//! n and prints an `n | time | ratio` table. The ratio column should hover
//! around the theoretical step factor
//!
//!     2^(n+1)(n+1)^2 / (2^n n^2) = 2 * ((n+1)/n)^2  -> ~2 as n grows,
//!
//! i.e. each extra city roughly *doubles* the work. That doubling is why no
//! one runs exact TSP on a thousand cities — and why the lesson spends so long
//! on heuristics and approximation.
//!
//! Run with:  cargo run -q -p lab-22-hamilton --example bench --features solutions
//! (or `--release` for cleaner numbers). Std only; no external crates.

use std::time::Instant;

use lab_22_hamilton::shortest_hamiltonian_path;

/// Deterministic symmetric distance matrix from a hand-rolled LCG.
fn random_distances(n: usize, seed: u64) -> Vec<Vec<u64>> {
    let mut x = seed;
    let mut next = || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (x >> 40) % 1000 + 1
    };
    let mut d = vec![vec![0u64; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let w = next();
            d[i][j] = w;
            d[j][i] = w;
        }
    }
    d
}

fn main() {
    println!("Held-Karp shortest Hamiltonian path — O(2^n * n^2)");
    println!("{:>4} | {:>12} | {:>7}", "n", "time", "ratio");
    println!("{:-<4}-+-{:-<12}-+-{:-<7}", "", "", "");

    let mut prev: Option<f64> = None;
    for n in 4..=18 {
        let d = random_distances(n, 0x9e37_79b9_7f4a_7c15 ^ n as u64);

        // Time the DP; repeat small sizes so the timer has something to bite.
        let reps = if n <= 10 { 200 } else { 1 };
        let start = Instant::now();
        let mut sink = 0u64;
        for _ in 0..reps {
            sink = sink.wrapping_add(shortest_hamiltonian_path(&d));
        }
        let elapsed = start.elapsed().as_secs_f64() / reps as f64;
        std::hint::black_box(sink);

        let ratio = match prev {
            Some(p) if p > 0.0 => format!("{:.2}x", elapsed / p),
            _ => "—".to_string(),
        };
        let shown = if elapsed >= 1.0 {
            format!("{:.3} s", elapsed)
        } else if elapsed >= 1e-3 {
            format!("{:.3} ms", elapsed * 1e3)
        } else {
            format!("{:.1} us", elapsed * 1e6)
        };
        println!("{:>4} | {:>12} | {:>7}", n, shown, ratio);
        prev = Some(elapsed);
    }

    println!();
    println!("Each +1 city ~doubles the time: 2^n growth, made visible.");
}
