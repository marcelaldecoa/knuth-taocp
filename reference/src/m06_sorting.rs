//! Module 06 — Sorting.
//! Source: TAOCP Vol. 3, 2nd ed., Ch. 5: §5.1.1 (inversions), §5.2.1
//! (Algorithms S and D), §5.2.2 (Algorithm Q), §5.2.3 (Algorithm H),
//! §5.2.4 (Algorithm N), §5.2.5 (Algorithm R).
//!
//! All internal sorts operate on `&mut [i64]`; the radix sort operates on
//! `&mut [u64]` (it inspects the binary representation of the keys, so an
//! unsigned type keeps the byte-order argument honest).

// ---------------------------------------------------------------------------
// Stage 1 — Straight insertion (Algorithm 5.2.1S) and inversions (§5.1.1)
// ---------------------------------------------------------------------------

/// Algorithm 5.2.1S (Straight insertion sort), step-faithful.
///
/// Records `R_1, ..., R_N` are rearranged in place so their keys are in
/// non-decreasing order. Knuth's steps (1-based, j running 2..=N):
///
/// ```text
/// S1. [Loop on j.]      Perform S2..S5 for j = 2, 3, ..., N.
/// S2. [Set up i, K, R.] Set i <- j - 1, K <- K_j, R <- R_j.
/// S3. [Compare K : K_i.] If K >= K_i, go to S5.
///     (R_{i+1}, ..., R_{j-1} have been moved up one; K < each of their keys.)
/// S4. [Move R_i, decrease i.] Set R_{i+1} <- R_i, i <- i - 1.
///     If i > 0, go back to S3.
/// S5. [R into R_{i+1}.]  Set R_{i+1} <- R.
/// ```
pub fn insertion_sort(v: &mut [i64]) {
    insertion_moves(v);
}

/// `insertion_sort`, instrumented: returns the number of *record moves*
/// performed in step S4 (each execution of `R_{i+1} <- R_i` counts one).
///
/// Theorem (§5.1.1 / §5.2.1): this equals the number of inversions of the
/// input permutation — each move cancels exactly one inversion.
pub fn insertion_sort_counting(v: &mut [i64]) -> u64 {
    insertion_moves(v)
}

fn insertion_moves(v: &mut [i64]) -> u64 {
    let n = v.len();
    let mut moves = 0u64;
    // S1. [Loop on j.] (0-based: j runs 1..n, i.e. Knuth's 2..=N.)
    for j in 1..n {
        // S2. [Set up i, K, R.]
        let k = v[j];
        let mut i = j; // `i` here indexes the hole Knuth calls R_{i+1}.
        // S3. [Compare K : K_i.] / S4. [Move R_i, decrease i.]
        while i > 0 && k < v[i - 1] {
            v[i] = v[i - 1];
            moves += 1;
            i -= 1;
        }
        // S5. [R into R_{i+1}.]
        v[i] = k;
    }
    moves
}

/// The number of inversions of `v` (§5.1.1): the number of index pairs
/// `i < j` with `v[i] > v[j]`. Zero iff the slice is sorted; maximal,
/// `n(n-1)/2`, iff it is strictly decreasing.
///
/// Computed by a merge-count in O(n log n) — the merge step counts, for each
/// element taken from the right half, how many left-half elements it jumped.
pub fn count_inversions(v: &[i64]) -> u64 {
    let mut work: Vec<i64> = v.to_vec();
    let mut buf = vec![0i64; work.len()];
    merge_count(&mut work, &mut buf)
}

fn merge_count(v: &mut [i64], buf: &mut [i64]) -> u64 {
    let n = v.len();
    if n <= 1 {
        return 0;
    }
    let mid = n / 2;
    let mut inv = {
        let (left, right) = v.split_at_mut(mid);
        merge_count(left, &mut buf[..mid]) + merge_count(right, &mut buf[mid..])
    };
    // Merge v[..mid] and v[mid..] into buf, counting crossings: taking the
    // right element while `mid - i` left elements remain means it forms an
    // inversion with each of them.
    let (mut i, mut j, mut k) = (0usize, mid, 0usize);
    while i < mid && j < n {
        if v[i] <= v[j] {
            buf[k] = v[i];
            i += 1;
        } else {
            buf[k] = v[j];
            j += 1;
            inv += (mid - i) as u64;
        }
        k += 1;
    }
    buf[k..k + (mid - i)].copy_from_slice(&v[i..mid]);
    let k2 = k + (mid - i);
    buf[k2..n].copy_from_slice(&v[j..n]);
    v.copy_from_slice(&buf[..n]);
    inv
}

// ---------------------------------------------------------------------------
// Stage 2 — Shellsort (Algorithm 5.2.1D, diminishing increments)
// ---------------------------------------------------------------------------

/// Knuth's recommended increment sequence for Algorithm D (§5.2.1):
/// `h_1 = 1, h_{s+1} = 3 h_s + 1` — i.e. 1, 4, 13, 40, 121, ... — returned
/// in *decreasing* order, keeping every increment `< n`.
///
/// `knuth_gaps(16) == [13, 4, 1]`; for `n <= 1` there is nothing to sort
/// and the sequence is empty.
pub fn knuth_gaps(n: usize) -> Vec<usize> {
    if n <= 1 {
        return Vec::new();
    }
    let mut gaps = vec![1usize];
    loop {
        let next = 3 * gaps[gaps.len() - 1] + 1;
        if next >= n {
            break;
        }
        gaps.push(next);
    }
    gaps.reverse();
    gaps
}

/// Algorithm 5.2.1D (Shellsort / diminishing-increment sort) with the
/// `3h + 1` increments of [`knuth_gaps`].
///
/// ```text
/// D1. [Loop on s.]  Perform D2 for s = t, t-1, ..., 1 (h = h_s).
/// D2. [Loop on j.]  h-sort the file: for j = h+1, ..., N perform D3..D6,
///                   which are steps S2..S5 of straight insertion with the
///                   stride 1 replaced by the stride h.
/// D3. [Set up i, K, R.]       i <- j - h, K <- K_j, R <- R_j.
/// D4. [Compare K : K_i.]      If K >= K_i, go to D6.
/// D5. [Move R_i, decrease i.] R_{i+h} <- R_i, i <- i - h; if i > 0 go to D4.
/// D6. [R into R_{i+h}.]       R_{i+h} <- R.
/// ```
///
/// After the pass with increment h the file is *h-ordered*
/// (`K_i <= K_{i+h}` for all valid i). The final pass (h = 1) is straight
/// insertion, so the result is sorted; the earlier passes exist purely to
/// make that last pass cheap (few inversions survive them).
pub fn shell_sort(v: &mut [i64]) {
    let gaps = knuth_gaps(v.len());
    shell_sort_with_gaps(v, &gaps);
}

/// Algorithm D with a caller-supplied increment sequence, applied in the
/// order given. To fully sort, the sequence must end with 1 (a lone `[1]`
/// makes this exactly straight insertion). Each pass with increment `h`
/// leaves the file h-ordered. Panics if any increment is 0.
pub fn shell_sort_with_gaps(v: &mut [i64], gaps: &[usize]) {
    let n = v.len();
    // D1. [Loop on s.]
    for &h in gaps {
        assert!(h > 0, "increments must be positive");
        // D2. [Loop on j.]  0-based j = h..n plays Knuth's j = h+1..=N.
        for j in h..n {
            // D3. [Set up i, K, R.]
            let k = v[j];
            let mut i = j;
            // D4. [Compare K : K_i.] / D5. [Move R_i, decrease i.]
            while i >= h && k < v[i - h] {
                v[i] = v[i - h];
                i -= h;
            }
            // D6. [R into R_{i+h}.]
            v[i] = k;
        }
    }
}

// ---------------------------------------------------------------------------
// Stage 3 — Quicksort (Algorithm 5.2.2Q, partition-exchange)
// ---------------------------------------------------------------------------

/// Subfiles of at most `M` records are left for a single straight-insertion
/// pass at the end, exactly as Algorithm Q prescribes (steps Q2/Q9). Knuth
/// derives M = 9 as the optimum for MIX in §5.2.2; any small constant works.
pub const M: usize = 9;

/// Algorithm 5.2.2Q (Quicksort), iterative with an explicit stack.
///
/// Faithful to the structure of Knuth's Algorithm Q:
///
/// ```text
/// Q1. [Initialize.]      Stack empty; (l, r) <- (1, N).
/// Q2. [Begin new stage.] If r - l < M, go to Q8 (leave small subfiles
///                        unsorted for the final insertion pass).
/// Q3-Q6. [Partition.]    Choose the pivot K, scan i up while K_i < K and
///                        j down while K_j > K, exchanging records, until
///                        the pointers cross: the file splits into a left
///                        part with keys <= K and a right part >= K.
/// Q7. [Put on stack.]    Push the LARGER subfile on the stack and iterate
///                        on the smaller (this bounds the stack by lg N).
/// Q8. [Take off stack.]  If the stack is empty, go to Q9; else pop (l, r)
///                        and return to Q2.
/// Q9. [Straight insertion.] Sort the nearly-sorted file by Algorithm S.
/// ```
///
/// Pivot choice here: **median-of-three** (Knuth's refinement, §5.2.2),
/// which also plants sentinels at both ends of the segment so the inner
/// scans need no bounds checks — and makes sorted input a best case, not
/// the Omega(n^2) worst case of the take-the-first-key rule.
pub fn quicksort(v: &mut [i64]) {
    quicksort_inner(v);
}

/// `quicksort`, instrumented: returns the number of key comparisons made
/// (partition scans, median-of-three, and the final insertion pass).
/// On random data this tracks Knuth's average, which is Theta(n ln n) with
/// a small constant; the tests only assert a loose multiple of `n ln n`.
pub fn quicksort_counting(v: &mut [i64]) -> u64 {
    quicksort_inner(v)
}

fn quicksort_inner(v: &mut [i64]) -> u64 {
    let n = v.len();
    let mut comps = 0u64;
    if n <= 1 {
        return comps;
    }
    // Q1. [Initialize.]
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let (mut l, mut r) = (0usize, n - 1);
    loop {
        // Q2. [Begin new stage.] Small subfiles wait for step Q9.
        if r - l + 1 <= M {
            // Q8. [Take off stack.]
            match stack.pop() {
                Some((pl, pr)) => {
                    l = pl;
                    r = pr;
                    continue;
                }
                None => break,
            }
        }
        // Q3-Q6. [Partition v[l..=r] about a median-of-three pivot.]
        let mid = l + (r - l) / 2;
        // Order v[l] <= v[mid] <= v[r]; the pivot is v[mid], and v[l], v[r]
        // act as sentinels for the two scans.
        comps += 3;
        if v[mid] < v[l] {
            v.swap(mid, l);
        }
        if v[r] < v[l] {
            v.swap(r, l);
        }
        if v[r] < v[mid] {
            v.swap(r, mid);
        }
        let k = v[mid];
        let (mut i, mut j) = (l, r);
        let split = loop {
            // Q3. [Compare K_i : K.] Scan up while keys are < pivot.
            loop {
                comps += 1;
                if v[i] >= k {
                    break;
                }
                i += 1;
            }
            // Q5. [Compare K : K_j.] Scan down while keys are > pivot.
            loop {
                comps += 1;
                if v[j] <= k {
                    break;
                }
                j -= 1;
            }
            if i >= j {
                break j;
            }
            // Q4/Q6. [Exchange.]
            v.swap(i, j);
            i += 1;
            j -= 1;
        };
        // (The sentinels guarantee l <= split < r, so both parts are
        // nonempty and strictly smaller — termination.)
        // Q7. [Put on stack.] Larger subfile to the stack, iterate on the
        // smaller: the stack never holds more than lg N entries.
        let (left, right) = ((l, split), (split + 1, r));
        let (small, large) = if split - l < r - split {
            (left, right)
        } else {
            (right, left)
        };
        stack.push(large);
        l = small.0;
        r = small.1;
    }
    // Q9. [Straight insertion.] Every record is within an M-sized subfile
    // of its final position, so this pass costs O(M n).
    for j in 1..n {
        let key = v[j];
        let mut i = j;
        loop {
            if i == 0 {
                break;
            }
            comps += 1;
            if key >= v[i - 1] {
                break;
            }
            v[i] = v[i - 1];
            i -= 1;
        }
        v[i] = key;
    }
    comps
}

// ---------------------------------------------------------------------------
// Stage 4 — Heapsort (Algorithm 5.2.3H)
// ---------------------------------------------------------------------------

/// Is `v` a (max-)heap? With 0-based indices, the heap condition of §5.2.3,
/// `K_{floor(j/2)} >= K_j` for 1 < j <= N (1-based), reads
/// `v[(j - 1) / 2] >= v[j]` for `1 <= j < n`.
pub fn is_heap(v: &[i64]) -> bool {
    (1..v.len()).all(|j| v[(j - 1) / 2] >= v[j])
}

/// The *siftup* procedure of Algorithm 5.2.3H (steps H3-H8): given that the
/// subtrees below `root` are heaps within `v[..=end]`, sink `v[root]` to
/// its proper place so the subtree rooted at `root` becomes a heap.
///
/// (Knuth's name "siftup" pictures the key sifting up through a sieve;
/// the record moves toward the leaves — modern texts say "sift down".)
///
/// ```text
/// H3. [Prepare for siftup.] Set K <- K_root, j <- root.
/// H4. [Advance downward.]   Set i <- j, j <- 2j   (0-based: j <- 2j + 1).
///                           If j > end, go to H8.
/// H5. [Find larger child.]  If j < end and K_j < K_{j+1}, set j <- j + 1.
/// H6. [Larger than K?]      If K >= K_j, go to H8.
/// H7. [Move it up.]         Set R_i <- R_j and return to H4.
/// H8. [Store R.]            Set R_i <- R.
/// ```
///
/// Requires `root <= end < v.len()`.
pub fn sift_up(v: &mut [i64], root: usize, end: usize) {
    sift_up_counting(v, root, end, &mut 0);
}

fn sift_up_counting(v: &mut [i64], root: usize, end: usize, comps: &mut u64) {
    assert!(root <= end && end < v.len(), "siftup range out of bounds");
    // H3. [Prepare for siftup.]
    let k = v[root];
    let mut i = root;
    loop {
        // H4. [Advance downward.]
        let mut j = 2 * i + 1;
        if j > end {
            break; // to H8
        }
        // H5. [Find larger child.]
        if j < end {
            *comps += 1;
            if v[j] < v[j + 1] {
                j += 1;
            }
        }
        // H6. [Larger than K?]
        *comps += 1;
        if k >= v[j] {
            break; // to H8
        }
        // H7. [Move it up.]
        v[i] = v[j];
        i = j;
    }
    // H8. [Store R.]
    v[i] = k;
}

/// Phase 1 of Algorithm H (steps H1-H2): turn `v` into a heap by sifting
/// each internal node, right to left. Costs O(n) comparisons in total —
/// the sum of the subtree heights is less than n.
pub fn make_heap(v: &mut [i64]) {
    let mut c = 0u64;
    make_heap_counting(v, &mut c);
}

fn make_heap_counting(v: &mut [i64], comps: &mut u64) {
    let n = v.len();
    // H1. [Initialize.] l <- floor(N/2) + 1; H2 decreases l and sifts.
    // 0-based: the internal nodes are 0 .. n/2 - 1, processed in reverse.
    for root in (0..n / 2).rev() {
        // H2. [Decrease l.] Siftup with (l, r) = (root, n - 1).
        sift_up_counting(v, root, n - 1, comps);
    }
}

/// Algorithm 5.2.3H (Heapsort): build a heap, then repeatedly exchange the
/// maximum `v[0]` with the last unsorted record and restore the heap
/// (steps H2/H3-H8 with r decreasing). In place, O(n log n) *worst case*.
pub fn heapsort(v: &mut [i64]) {
    heapsort_counting(v);
}

/// `heapsort`, instrumented: returns the number of key comparisons.
/// At most 2 n lg n + O(n) (§5.2.3); the tests assert a loose version.
pub fn heapsort_counting(v: &mut [i64]) -> u64 {
    let mut comps = 0u64;
    let n = v.len();
    if n <= 1 {
        return comps;
    }
    // H1-H2: heap-creation phase.
    make_heap_counting(v, &mut comps);
    // Selection phase: H2's r decreases from N-1 to 1 (Knuth folds both
    // phases into one loop; we keep them apart for clarity).
    for r in (1..n).rev() {
        v.swap(0, r);
        sift_up_counting(v, 0, r - 1, &mut comps);
    }
    comps
}

// ---------------------------------------------------------------------------
// Stage 5 — Natural merge sort (Algorithm 5.2.4N)
// ---------------------------------------------------------------------------

/// The number of ascending runs of `v` (§5.1.3, §5.2.4): maximal
/// non-decreasing stretches. A sorted array is 1 run; a strictly decreasing
/// array of length n is n runs; the empty array is 0 runs.
pub fn count_runs(v: &[i64]) -> usize {
    if v.is_empty() {
        return 0;
    }
    // A new run begins exactly at each "step-down" K_i > K_{i+1}.
    1 + (1..v.len()).filter(|&i| v[i - 1] > v[i]).count()
}

/// Algorithm 5.2.4N (Natural merge sort): repeatedly scan the file for its
/// existing ascending runs and merge them pairwise until one run remains.
///
/// ```text
/// N1. [Begin pass.]    Start a merge pass over the whole file.
/// N2. [Find two runs.] Locate the next two consecutive ascending runs
///                      (a run ends at a step-down K_i > K_{i+1}); a lone
///                      trailing run is copied through unchanged.
/// N3. [Merge.]         Merge the two runs into the output area, taking
///                      the smaller leading key first (ties from the left
///                      run — the merge is stable).
/// N4. [Pass complete?] More runs left in this pass: back to N2. A pass
///                      that saw <= 2 runs leaves the file sorted: done.
/// ```
///
/// Each pass at least halves the number of runs, so an input with r runs
/// costs at most ceil(lg r) passes — the sort *adapts* to existing order
/// (a sorted file costs one scan of n - 1 comparisons). Knuth's Algorithm N
/// merges between two tape areas; this in-memory version plays the same
/// game between `v` and an auxiliary buffer of n records.
pub fn natural_merge_sort(v: &mut [i64]) {
    natural_merge_inner(v);
}

/// `natural_merge_sort`, instrumented: returns the number of key
/// comparisons (run detection + merging). Already-sorted input costs
/// exactly n - 1; random input is ~ n lg n.
pub fn natural_merge_sort_counting(v: &mut [i64]) -> u64 {
    natural_merge_inner(v)
}

fn natural_merge_inner(v: &mut [i64]) -> u64 {
    let n = v.len();
    let mut comps = 0u64;
    if n <= 1 {
        return comps;
    }
    let mut buf = vec![0i64; n];
    loop {
        // N1. [Begin pass.]
        let mut runs = 0usize;
        let mut i = 0usize; // read position in v
        let mut out = 0usize; // write position in buf
        while i < n {
            // N2. [Find two runs.] First run: v[i..j].
            let mut j = i + 1;
            while j < n && {
                comps += 1;
                v[j - 1] <= v[j]
            } {
                j += 1;
            }
            runs += 1;
            if j == n {
                // Lone trailing run: copy through.
                buf[out..out + (n - i)].copy_from_slice(&v[i..n]);
                break;
            }
            // Second run: v[j..k].
            let mut k = j + 1;
            while k < n && {
                comps += 1;
                v[k - 1] <= v[k]
            } {
                k += 1;
            }
            runs += 1;
            // N3. [Merge v[i..j] with v[j..k] into buf[out..].]
            let (mut a, mut b) = (i, j);
            while a < j && b < k {
                comps += 1;
                if v[a] <= v[b] {
                    buf[out] = v[a];
                    a += 1;
                } else {
                    buf[out] = v[b];
                    b += 1;
                }
                out += 1;
            }
            while a < j {
                buf[out] = v[a];
                a += 1;
                out += 1;
            }
            while b < k {
                buf[out] = v[b];
                b += 1;
                out += 1;
            }
            i = k;
        }
        v.copy_from_slice(&buf);
        // N4. [Pass complete?] <= 2 runs seen: the pass finished the job.
        if runs <= 2 {
            return comps;
        }
    }
}

// ---------------------------------------------------------------------------
// Stage 6 — Radix sort (Algorithm 5.2.5R, least significant digit first)
// ---------------------------------------------------------------------------

/// Algorithm 5.2.5R (Radix list sort), adapted from linked lists to arrays:
/// sort `u64` keys with 8 passes of a *stable* counting sort, one byte per
/// pass, least significant byte first.
///
/// ```text
/// R1. [Loop on k.]   Perform one distribution pass for each byte position
///                    k = 1 (least significant), 2, ..., 8.
/// R2. [Count.]       Count how many keys have each value 0..255 of byte k.
/// R3. [Allocate.]    Prefix-sum the counts: pile d starts at
///                    count[0] + ... + count[d-1] of the output area.
/// R4. [Distribute.]  Move each key, in input order, to the next free slot
///                    of its pile — first come, first placed, so keys with
///                    equal bytes keep their relative order (stability).
/// ```
///
/// Stability of every pass is what makes the whole sort work: by induction
/// on k, after pass k the keys are ordered by their k least significant
/// bytes. This is not a comparison sort — the lg(n!) lower bound of §5.3.1
/// does not apply — and the total work is O(8 n) = O(n).
pub fn radix_sort_u64(v: &mut [u64]) {
    let n = v.len();
    if n <= 1 {
        return;
    }
    let mut buf = vec![0u64; n];
    // R1. [Loop on k.]
    for pass in 0..8 {
        let shift = 8 * pass;
        // R2. [Count.]
        let mut count = [0usize; 256];
        for &x in v.iter() {
            count[((x >> shift) & 0xFF) as usize] += 1;
        }
        // (If this byte is constant across the file the pass is the
        // identity permutation; skip the data movement.)
        if count.iter().any(|&c| c == n) {
            continue;
        }
        // R3. [Allocate.]
        let mut start = [0usize; 256];
        let mut sum = 0usize;
        for (d, &c) in count.iter().enumerate() {
            start[d] = sum;
            sum += c;
        }
        // R4. [Distribute.] In input order — this is the stability.
        for &x in v.iter() {
            let d = ((x >> shift) & 0xFF) as usize;
            buf[start[d]] = x;
            start[d] += 1;
        }
        v.copy_from_slice(&buf);
    }
}

// ---------------------------------------------------------------------------
// Unit tests: Knuth's worked examples
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Knuth's standard 16-key example, used throughout Ch. 5 (§5.2).
    const KNUTH16: [i64; 16] = [
        503, 087, 512, 061, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703,
    ];
    const KNUTH16_SORTED: [i64; 16] = [
        061, 087, 154, 170, 275, 426, 503, 509, 512, 612, 653, 677, 703, 765, 897, 908,
    ];

    #[test]
    fn insertion_sorts_knuths_example() {
        let mut v = KNUTH16;
        insertion_sort(&mut v);
        assert_eq!(v, KNUTH16_SORTED);
    }

    #[test]
    fn knuth16_has_41_inversions_and_moves_match() {
        // Verified by brute force over all 120 pairs: 41 inversions.
        assert_eq!(count_inversions(&KNUTH16), 41);
        let mut v = KNUTH16;
        assert_eq!(insertion_sort_counting(&mut v), 41);
    }

    #[test]
    fn inversion_extremes() {
        assert_eq!(count_inversions(&[]), 0);
        assert_eq!(count_inversions(&[7]), 0);
        assert_eq!(count_inversions(&[1, 2, 3, 4]), 0);
        // Reverse order: n(n-1)/2.
        let rev: Vec<i64> = (0..100).rev().collect();
        assert_eq!(count_inversions(&rev), 100 * 99 / 2);
    }

    #[test]
    fn shellsort_matches_and_gaps_are_knuths() {
        assert_eq!(knuth_gaps(16), vec![13, 4, 1]);
        assert_eq!(knuth_gaps(100), vec![40, 13, 4, 1]);
        assert_eq!(knuth_gaps(1), Vec::<usize>::new());
        let mut v = KNUTH16;
        shell_sort(&mut v);
        assert_eq!(v, KNUTH16_SORTED);
    }

    #[test]
    fn quicksort_heapsort_merge_on_knuth16() {
        let mut a = KNUTH16;
        quicksort(&mut a);
        assert_eq!(a, KNUTH16_SORTED);
        let mut b = KNUTH16;
        heapsort(&mut b);
        assert_eq!(b, KNUTH16_SORTED);
        let mut c = KNUTH16;
        natural_merge_sort(&mut c);
        assert_eq!(c, KNUTH16_SORTED);
    }

    #[test]
    fn knuth16_has_8_runs() {
        // 503 | 087 512 | 061 908 | 170 897 | 275 653 | 426 |
        // 154 509 612 677 765 | 703  —  eight ascending runs.
        assert_eq!(count_runs(&KNUTH16), 8);
        assert_eq!(count_runs(&[]), 0);
        assert_eq!(count_runs(&[1, 2, 3]), 1);
        assert_eq!(count_runs(&[3, 2, 1]), 3);
    }

    #[test]
    fn heap_machinery() {
        let mut v = KNUTH16;
        make_heap(&mut v);
        assert!(is_heap(&v));
        // §5.2.3: after heap creation the largest key, 908, is at the root.
        assert_eq!(v[0], 908);
        assert!(!is_heap(&[1, 2, 3]));
        assert!(is_heap(&[3, 2, 1]));
    }

    #[test]
    fn radix_agrees_with_std() {
        let mut x = 42u64;
        let mut v: Vec<u64> = (0..3000)
            .map(|_| {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                x
            })
            .collect();
        v.push(0);
        v.push(u64::MAX);
        let mut expect = v.clone();
        expect.sort_unstable();
        radix_sort_u64(&mut v);
        assert_eq!(v, expect);
    }

    #[test]
    fn counting_variants_are_sane() {
        let mut x = 7u64;
        let mut v: Vec<i64> = (0..4096)
            .map(|_| {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                (x >> 33) as i64
            })
            .collect();
        let n = v.len() as f64;
        let mut q = v.clone();
        let qc = quicksort_counting(&mut q) as f64;
        assert!(qc < 3.0 * n * n.ln(), "quicksort comps {qc}");
        let mut h = v.clone();
        let hc = heapsort_counting(&mut h) as f64;
        assert!(hc < 2.0 * n * n.log2() + 4.0 * n, "heapsort comps {hc}");
        let mc = natural_merge_sort_counting(&mut v) as f64;
        assert!(mc < 2.5 * n * n.log2(), "merge comps {mc}");
        assert_eq!(q, h);
        assert_eq!(q, v);
    }
}
