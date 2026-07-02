//! Flagship benchmark — Module 05: classical vs Karatsuba multiplication.
//!
//! Times `big_mul` (Algorithm 4.3.1M, O(n^2)) against `big_mul_karatsuba`
//! (§4.3.3, O(n^{lg 3}) ~ O(n^{1.585})) on random big numbers of a growing
//! number of limbs. The lesson predicts:
//!   * classical time should roughly QUADRUPLE when n doubles (2^2 = 4);
//!   * Karatsuba time should grow by ~2^{1.585} ~ 3.0 when n doubles;
//! so the "ratio-to-previous" columns expose the two growth classes, and the
//! speedup column shows Karatsuba overtaking classical past the ~32-limb
//! cutoff. Run with:
//!   cargo run -q -p lab-05-arithmetic --example bench --features solutions --release
//!
//! std only; data comes from a hand-rolled LCG (no external crates).

use std::time::{Duration, Instant};

use lab_05_arithmetic::{big_mul, big_mul_karatsuba};

/// The LCG from the course conventions: x <- x*6364136223846793005 + 1442695040888963407.
struct Lcg(u64);
impl Lcg {
    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 32) as u32
    }
    fn limbs(&mut self, n: usize) -> Vec<u32> {
        let mut v: Vec<u32> = (0..n).map(|_| self.next_u32()).collect();
        // Force a nonzero top limb so both operands really have n limbs.
        if let Some(last) = v.last_mut() {
            *last |= 1;
        }
        v
    }
}

/// Time `op(u, v)` averaged over enough repetitions to exceed ~60 ms of wall
/// clock, so even the tiny inputs are measured above timer noise. Returns the
/// per-call duration. The checksum defeats dead-code elimination in --release.
fn time_op(u: &[u32], v: &[u32], op: fn(&[u32], &[u32]) -> Vec<u32>) -> Duration {
    let mut reps: u64 = 1;
    loop {
        let mut checksum: u64 = 0;
        let start = Instant::now();
        for _ in 0..reps {
            let w = op(u, v);
            checksum = checksum.wrapping_add(w.len() as u64).wrapping_add(w.first().copied().unwrap_or(0) as u64);
        }
        let elapsed = start.elapsed();
        std::hint::black_box(checksum);
        if elapsed >= Duration::from_millis(60) || reps >= (1 << 30) {
            return elapsed / reps as u32;
        }
        reps *= 2;
    }
}

fn secs(d: Duration) -> f64 {
    d.as_secs_f64()
}

fn main() {
    println!("Module 05 — multiplication: classical O(n^2) vs Karatsuba O(n^1.585)");
    println!("radix b = 2^32 limbs; random operands of n limbs each (cutoff = 32 limbs)\n");
    println!(
        "{:>7} | {:>12} {:>7} | {:>12} {:>7} | {:>9}",
        "n", "classical", "x/prev", "karatsuba", "x/prev", "speedup"
    );
    println!("{}", "-".repeat(66));

    let sizes = [8usize, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
    let mut lcg = Lcg(0x9E37_79B9_7F4A_7C15);
    let mut prev_classical: Option<f64> = None;
    let mut prev_karatsuba: Option<f64> = None;

    for &n in &sizes {
        let u = lcg.limbs(n);
        let v = lcg.limbs(n);
        // Sanity: the two products must agree (correctness before timing).
        debug_assert_eq!(big_mul(&u, &v), big_mul_karatsuba(&u, &v));

        let tc = secs(time_op(&u, &v, big_mul));
        let tk = secs(time_op(&u, &v, big_mul_karatsuba));

        let rc = prev_classical.map(|p| tc / p);
        let rk = prev_karatsuba.map(|p| tk / p);

        let fmt_ratio = |r: Option<f64>| match r {
            Some(x) => format!("{x:6.2}x"),
            None => "     -".to_string(),
        };
        println!(
            "{:>7} | {:>10.2}us {:>7} | {:>10.2}us {:>7} | {:>8.2}x",
            n,
            tc * 1e6,
            fmt_ratio(rc),
            tk * 1e6,
            fmt_ratio(rk),
            tc / tk,
        );
        prev_classical = Some(tc);
        prev_karatsuba = Some(tk);
    }

    println!(
        "\nExpected per-doubling growth: classical ~4.0x (n^2), Karatsuba ~3.0x (n^1.585)."
    );
    println!("Speedup > 1 means Karatsuba wins; it should climb steadily once n >> 32.");
}
