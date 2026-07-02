//! Flagship benchmark — Module 06: the two growth classes of sorting.
//!
//! Times `insertion_sort` (Algorithm 5.2.1S, O(n^2)) against the three
//! O(n log n) sorts — `quicksort` (5.2.2Q), `heapsort` (5.2.3H), and
//! `natural_merge_sort` (5.2.4N) — on the same LCG-generated i64 data at
//! growing n. The "x/prev" ratio columns reveal the growth class: when n
//! doubles, an O(n^2) sort should take ~4x as long, an O(n log n) sort a bit
//! over ~2x. Run with:
//!   cargo run -q -p lab-06-sorting --example bench --features solutions --release
//!
//! std only; data comes from a hand-rolled LCG (no external crates).

use std::time::{Duration, Instant};

use lab_06_sorting::{heapsort, insertion_sort, natural_merge_sort, quicksort};

/// Course-standard LCG: x <- x*6364136223846793005 + 1442695040888963407.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
    fn data(&mut self, n: usize) -> Vec<i64> {
        (0..n).map(|_| (self.next() >> 33) as i64).collect()
    }
}

/// Time `sort` on a fresh copy of `base`, averaged over enough repetitions to
/// exceed ~50 ms of wall clock. Each rep refills a reusable buffer from `base`
/// (an O(n) copy, asymptotically below both O(n^2) and O(n log n)) and sorts
/// it, so every measurement starts from the same unsorted permutation.
fn time_sort(base: &[i64], sort: fn(&mut [i64])) -> Duration {
    let mut work = base.to_vec();
    let mut reps: u64 = 1;
    loop {
        let start = Instant::now();
        for _ in 0..reps {
            work.copy_from_slice(base);
            sort(&mut work);
        }
        let elapsed = start.elapsed();
        std::hint::black_box(work[0]);
        if elapsed >= Duration::from_millis(50) || reps >= (1 << 24) {
            return elapsed / reps as u32;
        }
        reps *= 2;
    }
}

fn main() {
    println!("Module 06 — sorting: insertion O(n^2) vs quicksort / heapsort / merge O(n log n)");
    println!("same LCG-generated i64 data; time per sort, and ratio to the previous n\n");
    println!(
        "{:>7} | {:>11} {:>7} | {:>11} {:>7} | {:>11} {:>7} | {:>11} {:>7}",
        "n", "insertion", "x/prev", "quicksort", "x/prev", "heapsort", "x/prev", "n.merge", "x/prev"
    );
    println!("{}", "-".repeat(98));

    let sizes = [1000usize, 2000, 4000, 8000, 16000, 32000, 64000];
    let mut lcg = Lcg(0x1234_5678_9ABC_DEF0);
    let mut prev = [f64::NAN; 4];

    for &n in &sizes {
        let base = lcg.data(n);
        let sorts: [(fn(&mut [i64]), &str); 4] = [
            (insertion_sort, "insertion"),
            (quicksort, "quicksort"),
            (heapsort, "heapsort"),
            (natural_merge_sort, "n.merge"),
        ];
        let mut cells: Vec<(f64, f64)> = Vec::with_capacity(4);
        for (k, (sort, _)) in sorts.iter().enumerate() {
            let t = time_sort(&base, *sort).as_secs_f64() * 1e6; // microseconds
            let ratio = t / prev[k];
            prev[k] = t;
            cells.push((t, ratio));
        }
        let fmt_ratio = |r: f64| if r.is_finite() { format!("{r:6.2}x") } else { "     -".to_string() };
        println!(
            "{:>7} | {:>9.1}us {:>7} | {:>9.1}us {:>7} | {:>9.1}us {:>7} | {:>9.1}us {:>7}",
            n,
            cells[0].0, fmt_ratio(cells[0].1),
            cells[1].0, fmt_ratio(cells[1].1),
            cells[2].0, fmt_ratio(cells[2].1),
            cells[3].0, fmt_ratio(cells[3].1),
        );
    }

    println!(
        "\nExpected per-doubling growth: insertion ~4.0x (n^2); quicksort/heapsort/merge ~2.1x (n log n)."
    );
    println!("Watch insertion pull away — it is ~n/log n times slower at the largest n.");
}
