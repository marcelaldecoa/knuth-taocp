//! Module 15 — External Sorting (TAOCP Vol. 3, §5.4).
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 15` from the repository root. Work the stages in order — each
//! test file `tests/stage_NN_*.rs` corresponds to one stage, and the lesson
//! in `course/module-15-external/README.md` walks you through the theory.
//!
//! The model: "external" storage lives in memory. A *run* is a sorted
//! `Vec<i64>`, a *tape* is a queue of runs, and [`IoStats`] is the I/O
//! accountant — in the §5.4 cost model an external sort is billed by the
//! number of records read from and written to storage, not by comparisons.
//! Keep Knuth's step labels (R1, R2, ...) as comments in your code.

/// A run: a non-decreasing sequence of records — the unit of currency of
/// external sorting. Internal sorting makes runs; merging consumes them.
pub type Run = Vec<i64>;

/// The I/O accountant. One *pass* over a file of n records costs n reads
/// plus n writes; the whole module is about minimising passes. (This struct
/// is given to you complete — your stage-4 job is to keep it honest.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IoStats {
    /// Records read from simulated external storage into memory.
    pub records_read: u64,
    /// Records written from memory out to simulated external storage.
    pub records_written: u64,
}

/// Stage 1 — Algorithm 5.4.1R (replacement selection).
///
/// Stream `input` through a selection structure holding at most `p` records
/// and emit the initial runs, in order. Order the structure by the pair
/// `(RN, KEY)` — run number first, then key:
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
///                         RN + 1 ("frozen" until the next run). Otherwise
///                         tag it RN. Insert; go to R2.
/// ```
///
/// `std::collections::BinaryHeap<std::cmp::Reverse<(usize, i64)>>` is a
/// perfectly good selection structure here (Knuth fuses this algorithm with
/// the stage-2 loser tree; the behaviour is identical). Contract points the
/// tests check: every run non-decreasing; concatenation is a permutation of
/// the input; sorted input gives ONE run for any p (the snow-plow eats
/// everything); reverse-sorted input gives runs of exactly p; random input
/// gives runs of expected length 2p; p = 1 degenerates to the natural
/// ascending runs; empty input gives no runs. Panic (message containing
/// "at least one") if `p == 0`.
pub fn replacement_selection(input: &[i64], p: usize) -> Vec<Run> {
    let _ = (input, p);
    todo!("implement Algorithm 5.4.1R")
}

/// Stage 2 — k-way merge with a tree of losers (§5.4.1).
///
/// Merge `k` sorted runs into one sorted sequence. Use a **loser tree**,
/// not a `BinaryHeap` — that is the point of the stage: the k current run
/// fronts sit at the leaves; each internal node remembers the *loser* of
/// the match played there; slot 0 remembers the champion. After outputting
/// the champion and advancing its run, ONE leaf-to-root walk (one
/// comparison per level) repairs the whole tournament, because the loser
/// stored at each node is exactly the only opponent the climbing candidate
/// still has to meet. A winner tree would have to look at two children per
/// level; a heap pays up to two comparisons per level.
///
/// Layout hint: pad k to `kk = k.next_power_of_two()`; internal nodes at
/// array slots `1..kk`, leaf j at conceptual slot `kk + j`, parent of slot
/// i is `i / 2`; every leaf then sits at depth ⌈lg k⌉ exactly. Represent an
/// exhausted (or padding) run front as +∞ — e.g. `Option<i64>` with `None`
/// losing to everything — so end-of-run needs no special cases.
pub fn merge_runs(runs: &[Run]) -> Vec<i64> {
    let _ = runs;
    todo!("k-way merge with a tree of losers")
}

/// Stage 2 — [`merge_runs`], also returning the number of key–key
/// comparisons performed.
///
/// Count only comparisons between two live keys; matches against +∞ are
/// flag checks and cost nothing. The tests assert the §5.4.1 bound: the
/// initial tournament costs at most k − 1 comparisons and each of the n
/// output records at most ⌈lg k⌉ (one per level), so
///
/// ```text
///     comparisons <= n·⌈lg k⌉ + k    (small additive slack allowed).
/// ```
///
/// A binary heap misses this bound by nearly 2×; a loser tree meets it.
pub fn merge_runs_counting(runs: &[Run]) -> (Vec<i64>, u64) {
    let _ = runs;
    todo!("loser-tree merge with a comparison counter")
}

/// Stage 3 — the perfect polyphase distribution (§5.4.2, Table 1).
///
/// How many initial runs should each of the T − 1 *input* tapes hold so
/// that a polyphase merge on `tapes` = T tapes always finds exactly one
/// tape empty? Return one entry per input tape, non-increasing (the T-th
/// tape starts empty). Generalized Fibonacci: level 0 is (1, 0, …, 0), and
/// level n + 1 comes from level n = (a₁, …, a_{T−1}) by
///
/// ```text
///     a₁' = a₁ + a₂,  a₂' = a₁ + a₃,  …,  a_{T−2}' = a₁ + a_{T−1},
///     a_{T−1}' = a₁.
/// ```
///
/// For T = 3 that is the Fibonacci pairs (1,0), (1,1), (2,1), (3,2), (5,3),
/// (8,5), … Return the smallest level whose total is >= `num_runs`.
/// **Dummy-run convention:** if the perfect total exceeds `num_runs`, the
/// shortfall is covered by *dummy runs* — imaginary empty runs that merge
/// at zero cost; [`polyphase_merge`] materialises them as empty `Vec`s.
/// `num_runs == 0` returns all zeros; panic (message containing "at least
/// 3") if `tapes < 3`.
pub fn polyphase_distribution(num_runs: usize, tapes: usize) -> Vec<usize> {
    let _ = (num_runs, tapes);
    todo!("generalized-Fibonacci perfect distribution")
}

/// Stage 3 — the polyphase merge pattern (§5.4.2, the merge phases of
/// Algorithm 5.4.2D), simulated on `tapes` in-memory tapes (T >= 3).
/// Returns the single sorted run and the number of merge *phases*.
///
/// The pattern:
/// 1. Distribute the runs onto T − 1 tapes per [`polyphase_distribution`],
///    topping up with dummy (empty) runs; one tape stays empty.
/// 2. Phase: (T−1)-way-merge one run from each input tape onto the empty
///    tape, repeatedly, until the *shortest* input tape is exhausted. That
///    tape becomes the next output tape; every other tape keeps its unread
///    runs in place. Count one phase.
/// 3. Repeat until a single run remains; return it with the phase count.
///
/// A perfect level-n distribution finishes in exactly n phases (dummies
/// included): 1 run → 0 phases, 2 → 1, 3 → 2, 5 → 3, 8 → 4, 13 → 5 (T = 3).
pub fn polyphase_merge(runs: Vec<Run>, tapes: usize) -> (Vec<i64>, usize) {
    let _ = (runs, tapes);
    todo!("simulate the polyphase merge phases")
}

/// Stage 4 — the complete external sort (§5.4 synthesis).
///
/// Pipeline: replacement selection with `memory` records of workspace forms
/// the initial runs; a polyphase merge on `tapes` tapes combines them.
/// Return the sorted output plus the I/O bill, accounted as follows:
///
/// - run formation reads all n input records and writes all n out as runs
///   (one full pass: `records_read += n`, `records_written += n`);
/// - each merge phase reads every record it touches once and writes it once
///   (charge each (T−1)-way merge: reads = sum of input-run lengths,
///   writes = length of the merged output).
///
/// Contract points the tests check: output sorted and a permutation of the
/// input; `records_written <= n·(1 + phases)` where phases follows the
/// stage-3 level of the run count; an input that forms a single run
/// (already sorted, or `memory >= n`) costs exactly n reads + n writes —
/// one pass, no merging. Panics as in the earlier stages for `memory == 0`
/// or `tapes < 3`.
pub fn external_sort(input: &[i64], memory: usize, tapes: usize) -> (Vec<i64>, IoStats) {
    let _ = (input, memory, tapes);
    todo!("run formation + polyphase merge, with I/O accounting")
}
