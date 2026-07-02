//! Flagship benchmark — Module 07: logarithmic search and tree height.
//!
//! Two tables, both showing O(log n) behavior growing with n:
//!   (A) `binary_search` (Algorithm 6.2.1B): average and worst-case
//!       comparisons per search, confirming Theorem B's floor(lg N)+1 bound,
//!       plus time per search (grows ~ +1 step per doubling, not multiplied).
//!   (B) BST vs AVL height as n grows: a random `Bst` stays ~2.99 lg n tall,
//!       an `AvlTree` obeys the ~1.4405 lg(n+2) − 0.3277 Fibonacci-tree bound,
//!       and a sorted-input `Bst` degenerates to an n−1 vine — the whole point
//!       of balancing.
//! Run with:
//!   cargo run -q -p lab-07-searching --example bench --features solutions --release
//!
//! std only; data comes from a hand-rolled LCG (no external crates).

use std::time::{Duration, Instant};

use lab_07_searching::{binary_search_comparisons, AvlTree, Bst};

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
}

fn lg(n: usize) -> f64 {
    (n as f64).log2()
}

fn binary_search_table() {
    println!("(A) Binary search — comparisons per search vs n, expected O(log n)");
    println!("sorted array 0..n; a 50/50 mix of present and absent LCG query keys\n");
    println!(
        "{:>9} | {:>6} | {:>8} | {:>8} | {:>9} | {:>11} {:>7}",
        "n", "lg n", "avg cmp", "max cmp", "bound", "time/search", "x/prev"
    );
    println!("{}", "-".repeat(70));

    let sizes = [1_000usize, 4_000, 16_000, 64_000, 256_000, 1_024_000];
    let mut lcg = Lcg(0xDEAD_BEEF_CAFE_F00D);
    let mut prev_t = f64::NAN;

    for &n in &sizes {
        let arr: Vec<i64> = (0..n as i64).collect();
        // A fixed batch of queries: keys in 0..2n, so about half are absent.
        let queries: Vec<i64> = (0..4096).map(|_| (lcg.next() % (2 * n as u64)) as i64).collect();

        let mut total_cmp: u64 = 0;
        let mut max_cmp: u32 = 0;
        for &q in &queries {
            let (_, c) = binary_search_comparisons(&arr, q);
            total_cmp += c as u64;
            max_cmp = max_cmp.max(c);
        }
        let avg_cmp = total_cmp as f64 / queries.len() as f64;

        // Time the whole query batch, adaptively.
        let mut reps: u64 = 1;
        let per = loop {
            let start = Instant::now();
            let mut sink: u64 = 0;
            for _ in 0..reps {
                for &q in &queries {
                    let (r, _) = binary_search_comparisons(&arr, q);
                    sink = sink.wrapping_add(r.is_ok() as u64);
                }
            }
            let elapsed = start.elapsed();
            std::hint::black_box(sink);
            if elapsed >= Duration::from_millis(50) || reps >= (1 << 20) {
                break elapsed.as_secs_f64() / (reps as f64 * queries.len() as f64);
            }
            reps *= 2;
        };
        let t_ns = per * 1e9;
        let ratio = if prev_t.is_finite() { format!("{:6.2}x", t_ns / prev_t) } else { "     -".to_string() };
        prev_t = t_ns;

        let bound = lg(n).floor() as u32 + 1; // Theorem B: floor(lg N)+1
        println!(
            "{:>9} | {:>6.2} | {:>8.2} | {:>8} | {:>9} | {:>9.1}ns {:>7}",
            n, lg(n), avg_cmp, max_cmp, bound, t_ns, ratio
        );
    }
    println!("\navg comparisons rises by ~1 per doubling (additive) — the signature of O(log n);");
    println!("max never exceeds Theorem B's floor(lg N)+1.\n");
}

fn tree_height_table() {
    println!("(B) Tree height vs n — random BST (~2.99 lg n) vs AVL (<= 1.4405 lg(n+2) - 0.3277)");
    println!("heights in edges; last column is a sorted-input BST, which degenerates to a vine\n");
    println!(
        "{:>7} | {:>6} | {:>10} | {:>9} | {:>10} | {:>12}",
        "n", "lg n", "BST(rand)", "AVL", "AVL bound", "BST(sorted)"
    );
    println!("{}", "-".repeat(66));

    let sizes = [1_000usize, 2_000, 4_000, 8_000, 16_000, 32_000];
    let mut lcg = Lcg(0x0BAD_F00D_1234_5678);

    for &n in &sizes {
        let mut bst = Bst::new();
        let mut avl = AvlTree::new();
        // Same random key stream into both structures.
        let mut inserted = 0usize;
        while inserted < n {
            let key = (lcg.next() >> 16) as i64;
            let a = bst.insert(key);
            let b = avl.insert(key);
            debug_assert_eq!(a, b); // both accept/reject duplicates identically
            if a {
                inserted += 1;
            }
        }
        let bst_h = bst.height();
        let avl_h = avl.height();

        // Sorted-input BST: the Theta(n) vine of height n-1.
        let mut vine = Bst::new();
        for k in 0..n as i64 {
            vine.insert(k);
        }
        let vine_h = vine.height();

        let bound = 1.4405 * lg(n + 2) - 0.3277;
        println!(
            "{:>7} | {:>6.2} | {:>10} | {:>9} | {:>10.2} | {:>12}",
            n, lg(n), bst_h, avl_h, bound, vine_h
        );
    }
    println!("\nAVL height stays under its Fibonacci-tree bound; the random BST is a small");
    println!("constant taller; the sorted-input BST is a straight n-1 vine (Theta(n)).");
}

fn main() {
    println!("Module 07 — searching: logarithmic cost with growing n\n");
    binary_search_table();
    tree_height_table();
}
