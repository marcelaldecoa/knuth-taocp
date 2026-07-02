//! Module 15 — External Sorting.
//! Source: TAOCP Vol. 3, 2nd ed., §5.4 (Algorithm 5.4.1R replacement
//! selection, §5.4.1 multiway merging with a tree of losers, §5.4.2
//! polyphase merge / Algorithm 5.4.2D, and the §5.4 pass/I-O cost model).
//!
//! "External" storage is modelled in memory: a *run* is a sorted `Vec<i64>`,
//! a *tape* is a queue of runs, and an [`IoStats`] accountant counts every
//! record that crosses the (simulated) memory/storage boundary so tests can
//! assert I/O bounds — the cost model of §5.4, where comparisons are nearly
//! free and record transfers are everything.

use std::collections::VecDeque;

/// A run: a non-decreasing sequence of records. The unit of currency of
/// external sorting (§5.4): internal sorting makes runs, merging consumes
/// them, and every algorithm in this module is measured by how many times
/// each record travels to and from "tape".
pub type Run = Vec<i64>;

/// The I/O accountant. In the §5.4 cost model the running time of an
/// external sort is proportional to the number of records read from and
/// written to external storage; comparisons are a lower-order term. One
/// *pass* over the file costs `n` reads plus `n` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IoStats {
    /// Records read from simulated external storage into memory.
    pub records_read: u64,
    /// Records written from memory out to simulated external storage.
    pub records_written: u64,
}

// ---------------------------------------------------------------------------
// Stage 1 — Algorithm 5.4.1R: replacement selection
// ---------------------------------------------------------------------------

/// Algorithm 5.4.1R (replacement selection), on the simplified memory model:
/// a P-record selection structure ordered by the pair `(RN, KEY)` — run
/// number first, then key.
///
/// Given the input stream and a memory capacity of `p` records, produce the
/// initial runs, in order. Records that arrive *smaller than the last key
/// output* cannot extend the current run; they are "frozen" (tagged with the
/// next run number) and sleep in memory until the current run ends.
///
/// ```text
/// R1. [Initialize.]       Fill the P slots with the first P records, all
///                         tagged RN = 0. (Fewer than P if input is short.)
/// R2. [End of run?]       If the smallest tag in memory exceeds the current
///                         run number, close the current run, start the next.
/// R3. [Output top.]       Output the record with the smallest (RN, KEY);
///                         remember its key as LASTKEY.
/// R4. [Input new record.] Read the next record X (if input is exhausted the
///                         slot simply empties; when memory empties, stop).
/// R5. [Freeze or not.]    If X < LASTKEY it cannot join this run: tag it
///                         RN + 1. Otherwise tag it RN. Insert; go to R2.
/// ```
///
/// Knuth's 5.4.1R keeps the P records in a tree of losers and fuses steps
/// R2–R5 with the tournament replay; ordering any priority structure by
/// `(RN, KEY)` is behaviourally identical. Expected run length on random
/// input: **2P**, by the snow-plow argument (E. F. Moore, 1961) — see the
/// module lesson.
pub fn replacement_selection(input: &[i64], p: usize) -> Vec<Run> {
    assert!(
        p > 0,
        "replacement selection needs at least one record of memory (P >= 1)"
    );
    if input.is_empty() {
        return Vec::new();
    }
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    // R1. [Initialize.] Fill memory with the first P records, run number 0.
    let mut next = 0usize; // index of the next input record to read
    let mut mem: BinaryHeap<Reverse<(usize, i64)>> = BinaryHeap::new();
    while next < input.len() && mem.len() < p {
        mem.push(Reverse((0, input[next])));
        next += 1;
    }

    let mut runs: Vec<Run> = Vec::new();
    let mut current_rn = 0usize;
    let mut current: Run = Vec::new();

    while let Some(Reverse((rn, key))) = mem.pop() {
        // R2. [End of run?] The smallest live tag exceeds the current run
        //     number exactly when every record in memory is frozen.
        if rn != current_rn {
            runs.push(std::mem::take(&mut current));
            current_rn = rn;
        }
        // R3. [Output top.] `key` is the smallest key that may still extend
        //     run `rn`; it becomes LASTKEY.
        current.push(key);
        // R4/R5. [Input; freeze or not.] X < LASTKEY would break the run:
        //     it belongs to run rn + 1. Otherwise it can ride this run.
        if next < input.len() {
            let x = input[next];
            next += 1;
            let tag = if x < key { rn + 1 } else { rn };
            mem.push(Reverse((tag, x)));
        }
    }
    runs.push(current);
    runs
}

// ---------------------------------------------------------------------------
// Stage 2 — k-way merge with a tree of losers (§5.4.1)
// ---------------------------------------------------------------------------

/// A selection tree that remembers *losers* (§5.4.1, Fig. 63). The `k` run
/// fronts sit at the external nodes; each internal node stores the loser of
/// the match played there, and the overall winner is kept beside the root
/// (slot 0). After the winner is output and its run advances, one walk from
/// that leaf to the root — one comparison per level, ⌈lg k⌉ in all —
/// restores the tournament. (A *winner* tree would need to re-examine two
/// children at every level; the loser already waiting at each node is
/// exactly the only opponent the climbing record ever needs to meet.)
///
/// Implementation notes: `k` is padded to `kk = k.next_power_of_two()` so
/// every leaf sits at depth exactly lg kk = ⌈lg k⌉; internal nodes live at
/// array slots `1..kk`, leaf `j` is conceptual slot `kk + j`, the parent of
/// slot `i` is `i / 2`. Exhausted runs (and padding) compare as +∞ via
/// `None`; matches against +∞ are flag checks, not key comparisons, and are
/// not counted.
struct LoserTree<'a> {
    runs: &'a [Run],
    /// kk = k padded to a power of two (0 when there are no runs).
    kk: usize,
    /// Front pointer into each real run.
    pos: Vec<usize>,
    /// `loser[i]`, i in 1..kk: the leaf that *lost* the match at internal
    /// node i. `loser[0]`: the overall winner.
    loser: Vec<usize>,
    /// Key–key comparisons performed (comparisons against +∞ are free).
    comps: u64,
}

impl<'a> LoserTree<'a> {
    fn new(runs: &'a [Run]) -> Self {
        let k = runs.len();
        let kk = if k == 0 { 0 } else { k.next_power_of_two() };
        let mut t = LoserTree {
            runs,
            kk,
            pos: vec![0; k],
            loser: vec![0; kk.max(1)],
            comps: 0,
        };
        if kk > 0 {
            // Play the initial tournament bottom-up: kk - 1 matches, of
            // which at most k - 1 are between two live keys.
            let w = t.play(1);
            t.loser[0] = w;
        }
        t
    }

    /// Current key at leaf `j`; `None` = exhausted run or padding = +∞.
    fn key(&self, j: usize) -> Option<i64> {
        if j < self.runs.len() {
            self.runs[j].get(self.pos[j]).copied()
        } else {
            None
        }
    }

    /// Does leaf `a` beat leaf `b`? Smaller key wins; +∞ never beats a live
    /// key; ties go to `a` (any consistent tie-break preserves the multiset).
    fn beats(&mut self, a: usize, b: usize) -> bool {
        match (self.key(a), self.key(b)) {
            (None, _) => false,
            (Some(_), None) => true,
            (Some(x), Some(y)) => {
                self.comps += 1;
                x <= y
            }
        }
    }

    /// Build the subtree rooted at internal slot `node`; store losers,
    /// return the winning leaf.
    fn play(&mut self, node: usize) -> usize {
        if node >= self.kk {
            return node - self.kk; // external slot kk + j holds leaf j
        }
        let l = self.play(2 * node);
        let r = self.play(2 * node + 1);
        let (winner, loser) = if self.beats(l, r) { (l, r) } else { (r, l) };
        self.loser[node] = loser;
        winner
    }

    /// Pop the smallest live key, advance its run, and replay the single
    /// leaf-to-root path: at each of the lg kk = ⌈lg k⌉ levels the climbing
    /// candidate meets the stored loser — one comparison — and the loser of
    /// that match stays behind.
    fn pop(&mut self) -> Option<i64> {
        let mut w = self.loser[0];
        let out = self.key(w)?;
        self.pos[w] += 1;
        let mut node = (self.kk + w) / 2;
        while node >= 1 {
            let l = self.loser[node];
            if self.beats(l, w) {
                self.loser[node] = w;
                w = l;
            }
            node /= 2;
        }
        self.loser[0] = w;
        Some(out)
    }
}

/// §5.4.1: merge `k` sorted runs into one sorted sequence with a tree of
/// losers. See [`merge_runs_counting`] for the comparison-counting variant
/// and the ⌈lg k⌉-per-record bound.
pub fn merge_runs(runs: &[Run]) -> Vec<i64> {
    merge_runs_counting(runs).0
}

/// [`merge_runs`], also returning the number of key–key comparisons.
///
/// Bound (§5.4.1): the initial tournament costs at most k − 1 comparisons
/// and each of the n output records costs at most ⌈lg k⌉ (one per level of
/// the padded tree), so comparisons ≤ n·⌈lg k⌉ + k. A binary heap would pay
/// up to 2 comparisons per level per extraction — the loser tree is how
/// merge sorting reaches the information-theoretic rate.
pub fn merge_runs_counting(runs: &[Run]) -> (Vec<i64>, u64) {
    let total: usize = runs.iter().map(|r| r.len()).sum();
    let mut out = Vec::with_capacity(total);
    let mut tree = LoserTree::new(runs);
    while let Some(x) = tree.pop() {
        out.push(x);
    }
    (out, tree.comps)
}

// ---------------------------------------------------------------------------
// Stage 3 — Polyphase merge (§5.4.2, Algorithm 5.4.2D's pattern)
// ---------------------------------------------------------------------------

/// The *perfect distribution* of initial runs for a polyphase merge on
/// `tapes` tapes (§5.4.2, Table 1): how many runs each of the T − 1 input
/// tapes should hold (the T-th tape starts empty and receives the first
/// merge phase). Returned in non-increasing order, one entry per input tape.
///
/// Generalized Fibonacci recurrence: with level-0 distribution
/// (1, 0, …, 0), level n + 1 is obtained from level n = (a₁, …, a_{T−1}) by
///
/// ```text
///     a₁' = a₁ + a₂,   a₂' = a₁ + a₃,   …,   a_{T−2}' = a₁ + a_{T−1},
///     a_{T−1}' = a₁.
/// ```
///
/// For T = 3 this is the Fibonacci sequence: (1,0), (1,1), (2,1), (3,2),
/// (5,3), (8,5), … The function returns the *smallest perfect level whose
/// total is ≥ `num_runs`*. **Dummy-run convention:** when that total exceeds
/// `num_runs`, the difference is made up of *dummy runs* — imaginary empty
/// runs (§5.4.2 writes D(j) for their count) that take part in merges at
/// zero I/O cost; [`polyphase_merge`] materialises them as empty `Vec`s.
/// `num_runs == 0` returns all zeros.
pub fn polyphase_distribution(num_runs: usize, tapes: usize) -> Vec<usize> {
    assert!(tapes >= 3, "polyphase merging needs at least 3 tapes");
    let t = tapes - 1; // number of input tapes
    if num_runs == 0 {
        return vec![0; t];
    }
    // Level 0: a single run on the first tape.
    let mut a = vec![0usize; t];
    a[0] = 1;
    // Climb levels until the perfect total covers num_runs.
    while a.iter().sum::<usize>() < num_runs {
        let a1 = a[0];
        let mut b = vec![0usize; t];
        for i in 0..t - 1 {
            b[i] = a1 + a[i + 1];
        }
        b[t - 1] = a1;
        a = b;
    }
    a
}

/// The polyphase merge pattern of §5.4.2 (the merge phases of Algorithm
/// 5.4.2D), simulated on `tapes` in-memory tapes. Returns the single final
/// run and the number of merge *phases* performed.
///
/// The pattern: distribute the initial runs per [`polyphase_distribution`]
/// (padding with dummy = empty runs), leaving one tape empty; then
/// repeatedly perform (T−1)-way merges of one run from each input tape onto
/// the empty tape until the *shortest* input tape is exhausted — that tape
/// becomes the next output tape, and *every other tape keeps its unread
/// runs where they are*. Each such rewind-and-reverse is one phase. A
/// perfect level-n distribution finishes in exactly n phases (dummies
/// included); a single run needs 0.
pub fn polyphase_merge(runs: Vec<Run>, tapes: usize) -> (Vec<i64>, usize) {
    let mut io = IoStats::default();
    polyphase_core(runs, tapes, &mut io)
}

/// Shared engine for [`polyphase_merge`] and [`external_sort`]: performs
/// the phases and charges every record moved (read once + written once per
/// merge it takes part in) to `io`.
fn polyphase_core(runs: Vec<Run>, tapes: usize, io: &mut IoStats) -> (Vec<i64>, usize) {
    assert!(tapes >= 3, "polyphase merging needs at least 3 tapes");
    let num_runs = runs.len();
    if num_runs == 0 {
        return (Vec::new(), 0);
    }

    // Distribution: deal the runs out according to the perfect
    // distribution, appending dummy (empty) runs to reach it exactly.
    let dist = polyphase_distribution(num_runs, tapes);
    let mut tape: Vec<VecDeque<Run>> = (0..tapes).map(|_| VecDeque::new()).collect();
    let mut supply = runs.into_iter();
    for (i, &want) in dist.iter().enumerate() {
        for _ in 0..want {
            // A missing real run is a dummy: an empty run, merged for free.
            tape[i].push_back(supply.next().unwrap_or_default());
        }
    }
    debug_assert!(supply.next().is_none(), "distribution too small");

    // Merge phases.
    let mut phases = 0usize;
    loop {
        let live: usize = tape.iter().map(|t| t.len()).sum();
        if live <= 1 {
            let result = tape
                .iter_mut()
                .find_map(|t| t.pop_front())
                .unwrap_or_default();
            return (result, phases);
        }
        // Exactly one tape is empty; it is this phase's output tape.
        let out = tape
            .iter()
            .position(|t| t.is_empty())
            .expect("polyphase invariant: one tape is always empty");
        // Merge until the shortest input tape runs dry.
        let m = tape
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != out)
            .map(|(_, t)| t.len())
            .min()
            .unwrap();
        for _ in 0..m {
            let group: Vec<Run> = (0..tapes)
                .filter(|&i| i != out)
                .map(|i| tape[i].pop_front().unwrap())
                .collect();
            let moved: u64 = group.iter().map(|r| r.len() as u64).sum();
            io.records_read += moved;
            let merged = merge_runs(&group);
            io.records_written += merged.len() as u64;
            tape[out].push_back(merged);
        }
        phases += 1;
    }
}

// ---------------------------------------------------------------------------
// Stage 4 — the full pipeline, I/O accounted (§5.4 synthesis)
// ---------------------------------------------------------------------------

/// The complete external sort of §5.4: replacement selection forms initial
/// runs with `memory` records of workspace, then a polyphase merge on
/// `tapes` tapes combines them. Returns the sorted output and the I/O bill.
///
/// Accounting (the §5.4 cost model):
/// - run formation reads all n records in and writes all n out as initial
///   runs — one full pass: n reads + n writes;
/// - every merge phase reads each record it touches once and writes it
///   once — polyphase's whole point is that a phase usually touches only
///   *part* of the file.
///
/// Consequently `records_written ≤ n · (1 + phases)`, and a file that
/// replacement selection swallows into a single run (already sorted, or
/// `memory ≥ n`) costs exactly n reads + n writes: one pass, no merging.
pub fn external_sort(input: &[i64], memory: usize, tapes: usize) -> (Vec<i64>, IoStats) {
    assert!(
        memory > 0,
        "external sorting needs at least one record of memory (P >= 1)"
    );
    assert!(tapes >= 3, "polyphase merging needs at least 3 tapes");
    let mut io = IoStats::default();

    // Pass 0 — run formation: stream the input through the P-record
    // selection structure, writing initial runs to tape.
    io.records_read += input.len() as u64;
    let runs = if input.is_empty() {
        Vec::new()
    } else {
        replacement_selection(input, memory)
    };
    io.records_written += input.len() as u64;

    // Merge phases, each billed to `io` by polyphase_core.
    let (sorted, _phases) = polyphase_core(runs, tapes, &mut io);
    (sorted, io)
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Knuth's sixteen keys (Vol. 3, the running example of Chapter 5,
    /// reused for replacement selection in §5.4.1).
    const KNUTH16: [i64; 16] = [
        503, 87, 512, 61, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703,
    ];

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

    #[test]
    fn replacement_selection_knuth_example() {
        // §5.4.1's worked example: P = 3 on the sixteen keys produces three
        // runs of lengths 4, 6, 6 although memory holds only 3 records.
        let runs = replacement_selection(&KNUTH16, 3);
        assert_eq!(
            runs,
            vec![
                vec![87, 503, 512, 908],
                vec![61, 170, 275, 426, 653, 897],
                vec![154, 509, 612, 677, 703, 765],
            ]
        );
    }

    #[test]
    fn replacement_selection_snowplow_edges() {
        // Sorted input: the plow never meets fresh snow behind it — one run.
        let sorted: Vec<i64> = (0..1000).collect();
        assert_eq!(replacement_selection(&sorted, 5).len(), 1);
        // Reverse-sorted: every arrival freezes — runs of exactly P.
        let rev: Vec<i64> = (0..1000).rev().collect();
        let runs = replacement_selection(&rev, 50);
        assert_eq!(runs.len(), 20);
        assert!(runs.iter().all(|r| r.len() == 50));
        // P = 1 degenerates to the natural ascending runs of the input.
        let runs = replacement_selection(&KNUTH16, 1);
        assert_eq!(runs.len(), 8); // KNUTH16 has 8 ascending runs (§5.1.3)
    }

    #[test]
    fn replacement_selection_two_p_law() {
        // The snow-plow argument: expected run length -> 2P on random input.
        let data = lcg_vec(100_000, 20260702);
        let p = 64;
        let runs = replacement_selection(&data, p);
        let avg = data.len() as f64 / runs.len() as f64;
        assert!(
            avg > 1.7 * p as f64 && avg < 2.3 * p as f64,
            "average run length {avg} should be near 2P = {}",
            2 * p
        );
    }

    #[test]
    fn loser_tree_merge_and_comparison_bound() {
        // The three runs of the §5.4.1 example merge back to sorted order.
        let runs = replacement_selection(&KNUTH16, 3);
        let (merged, _) = merge_runs_counting(&runs);
        let mut expect = KNUTH16.to_vec();
        expect.sort();
        assert_eq!(merged, expect);

        // n·⌈lg k⌉ + k bound on 16 equal runs (⌈lg 16⌉ = 4).
        let k = 16usize;
        let runs: Vec<Run> = (0..k)
            .map(|i| {
                let mut r = lcg_vec(500, 7 + i as u64);
                r.sort();
                r
            })
            .collect();
        let n = 500 * k;
        let (out, comps) = merge_runs_counting(&runs);
        assert_eq!(out.len(), n);
        assert!(out.windows(2).all(|w| w[0] <= w[1]));
        assert!(
            comps <= (n as u64) * 4 + k as u64,
            "comps = {comps} exceeds n*ceil(lg k) + k = {}",
            n * 4 + k
        );
    }

    #[test]
    fn polyphase_distribution_is_generalized_fibonacci() {
        // §5.4.2 Table 1, T = 3: Fibonacci pairs.
        assert_eq!(polyphase_distribution(1, 3), vec![1, 0]);
        assert_eq!(polyphase_distribution(2, 3), vec![1, 1]);
        assert_eq!(polyphase_distribution(3, 3), vec![2, 1]);
        assert_eq!(polyphase_distribution(5, 3), vec![3, 2]);
        assert_eq!(polyphase_distribution(8, 3), vec![5, 3]);
        assert_eq!(polyphase_distribution(13, 3), vec![8, 5]);
        // Non-Fibonacci counts round up to the next perfect level.
        assert_eq!(polyphase_distribution(6, 3), vec![5, 3]);
        // T = 4: totals 1, 3, 5, 9, 17, 31 (third-order Fibonacci).
        assert_eq!(polyphase_distribution(3, 4), vec![1, 1, 1]);
        assert_eq!(polyphase_distribution(9, 4), vec![4, 3, 2]);
        assert_eq!(polyphase_distribution(17, 4), vec![7, 6, 4]);
        assert_eq!(polyphase_distribution(31, 4), vec![13, 11, 7]);
    }

    #[test]
    fn polyphase_phase_counts_and_correctness() {
        // Hand-traced: 2 runs -> 1 phase, 3 -> 2, 5 -> 3, 8 -> 4, 13 -> 5.
        for (num_runs, want_phases) in [(1, 0), (2, 1), (3, 2), (5, 3), (8, 4), (13, 5)] {
            let data = lcg_vec(num_runs * 30, num_runs as u64);
            let runs: Vec<Run> = data
                .chunks(30)
                .map(|c| {
                    let mut r = c.to_vec();
                    r.sort();
                    r
                })
                .collect();
            let mut expect = data.clone();
            expect.sort();
            let (out, phases) = polyphase_merge(runs, 3);
            assert_eq!(out, expect, "{num_runs} runs");
            assert_eq!(phases, want_phases, "{num_runs} runs");
        }
    }

    #[test]
    fn external_sort_end_to_end() {
        let data = lcg_vec(20_000, 99);
        let mut expect = data.clone();
        expect.sort();
        let (out, io) = external_sort(&data, 100, 3);
        assert_eq!(out, expect);
        assert!(io.records_read >= 20_000 && io.records_written >= 20_000);

        // One-run inputs cost exactly one pass: n reads, n writes.
        let sorted: Vec<i64> = (0..5000).collect();
        let (out, io) = external_sort(&sorted, 10, 3);
        assert_eq!(out, sorted);
        assert_eq!(io.records_read, 5000);
        assert_eq!(io.records_written, 5000);
    }
}
