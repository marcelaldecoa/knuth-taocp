# Hints — Module 06: Sorting

## Stage 1: Straight insertion; inversions

1. Reach for the central theorem of §5.1.1: straight insertion performs exactly
   one record move per *inversion* — a pair `i < j` with `v[i] > v[j]`. So
   `insertion_sort_counting` and `count_inversions` must return the same number
   (41 on KNUTH16), and a reversed array of length `n` hits the maximum
   `n(n−1)/2`.
2. Algorithm 5.2.1S: for each `j` from 1, lift `v[j]` into a temporary and shift
   larger earlier keys one slot to the right until you find its home. Count only
   the S4 *moves*, not the final S5 placement. For `count_inversions` the naive
   double loop is fine, or piggyback a merge (each right-half element taken while
   `m` left elements remain closes `m` inversions).
3. Inner loop: `let k = v[j]; let mut i = j; while i > 0 && k < v[i-1] { v[i] =
   v[i-1]; moves += 1; i -= 1; } v[i] = k;`. Use `<` (not `<=`) in the comparison
   so equal keys are never jumped — that is what makes the sort stable.

## Stage 2: Shellsort: diminishing increments

1. Shellsort is insertion sort with a stride `h > 1` that shrinks to 1, so
   distant elements leap `h` positions at once. Use Knuth's increment sequence
   `h_{s+1} = 3h_s + 1` (1, 4, 13, 40, ...) kept below `n` and applied
   largest-first; the final `h = 1` pass is plain insertion sort, so the output
   is fully sorted.
2. `knuth_gaps(n)` builds `1, 4, 13, ...` while the next value stays `< n`, then
   reverses. `shell_sort_with_gaps` is Algorithm D: for each gap `h`, run the
   Stage-1 insertion loop but with stride `h` instead of 1. A lone `[1]` must
   reduce to exactly straight insertion. Be *definite*: `assert!` on a zero
   increment with a message containing "positive".
3. The h-strided inner loop: `for j in h..n { let k = v[j]; let mut i = j; while
   i >= h && k < v[i-h] { v[i] = v[i-h]; i -= h; } v[i] = k; }`. Note `knuth_gaps`
   returns the empty vector for `n <= 1` (nothing to sort).

## Stage 3: Quicksort: partition exchange

1. Algorithm 5.2.2Q partitions a segment about a pivot so keys `≤ K` land left
   and keys `≥ K` land right, then recurses. Two ideas make it shippable:
   median-of-three pivoting (turns sorted/reverse input from the Θ(n²) worst case
   into a best case, and plants sentinels), and pushing the *larger* subfile
   while iterating on the smaller (bounds the stack at lg N).
2. Use an *explicit stack* of `(l, r)` ranges, not recursion. Stop partitioning
   subfiles of size `≤ M = 9` and finish with one straight-insertion pass at the
   end (step Q9). Order `v[l] ≤ v[mid] ≤ v[r]`, pivot on `v[mid]`; scan `i` up
   while `v[i] < K` and `j` down while `v[j] > K`, swap, until the pointers cross.
   Count every key comparison (the scans, the median-of-three, and Q9).
3. The partition core: `let split = loop { while v[i] < k { comps+=1; i+=1; }
   ...; while v[j] > k { comps+=1; j-=1; } ...; if i >= j { break j; }
   v.swap(i,j); i+=1; j-=1; };` (count the comparison that breaks each scan too).
   Then push the larger of `(l,split)` / `(split+1,r)` and continue on the
   smaller; pop from the stack when the current subfile is small.

## Stage 4: Heapsort

1. A max-heap in an array satisfies `v[(j-1)/2] >= v[j]` for `1 <= j < n`, so the
   maximum sits at `v[0]`. Heapsort (§5.2.3) builds a heap in O(n), then
   repeatedly swaps the root to the end and sifts. The build is linear by the
   sum-of-heights bound (`Σ h/2^h = 2`); the teardown adds the log factor.
2. `sift_up(v, root, end)` (steps H3–H8) sinks `v[root]` down through its larger
   child until it dominates both children within `v[..=end]`. `make_heap` sifts
   internal nodes `n/2-1 .. 0` in reverse. Heapsort = `make_heap` then, for `r`
   from `n-1` down to 1, `swap(0, r)` and `sift_up(v, 0, r-1)`.
3. Siftup with a hole: `let k = v[root]; let mut i = root; loop { let mut j =
   2*i+1; if j > end { break; } if j < end && v[j] < v[j+1] { j += 1; } if k >=
   v[j] { break; } v[i] = v[j]; i = j; } v[i] = k;`. Count the child comparison
   and the `k >= v[j]` comparison. Watch `n = 0, 1`.

## Stage 5: Natural merge sort

1. A *run* is a maximal non-decreasing stretch; a new one begins at each
   step-down `v[i-1] > v[i]`. Natural merge sort (§5.2.4) merges existing runs
   pairwise, halving their count each pass, so it *adapts*: an already-sorted
   file (one run) costs a single scan of `n−1` comparisons.
2. `count_runs` is `1 + (number of step-downs)`, or 0 for the empty slice. For
   the sort, use an auxiliary buffer of `n`: each pass scans for two consecutive
   runs and merges them into the buffer (a lone trailing run is copied through),
   then copies back. Stop when a pass sees `≤ 2` runs. Count run-detection *and*
   merge comparisons.
3. Detect a run by advancing while `v[j-1] <= v[j]`. Merge two runs `v[i..j]` and
   `v[j..k]` taking the smaller leading key, ties from the left (stable). End the
   pass loop with `if runs <= 2 { return comps; }`. `natural_merge_sort_counting`
   is the same routine returning the count; the plain version discards it.

## Stage 6: Radix sorting

1. Radix sort (§5.2.5) makes *no comparisons*, so the `lg(n!)` lower bound does
   not apply — it is O(n) for keys from a bounded universe. Sort `u64` keys with
   8 passes of a stable counting sort, one byte per pass, least significant byte
   first. Stability of every pass is load-bearing (the induction over bytes
   depends on it).
2. Each pass: count how many keys have each byte value 0..255, prefix-sum the
   counts into starting offsets, then distribute every key *in input order* to
   the next free slot of its pile. In-input-order distribution is exactly what
   keeps equal-byte keys in their prior relative order.
3. Per pass `p`: `let shift = 8*p; let mut count = [0usize;256]; for &x in v {
   count[((x>>shift)&0xFF) as usize] += 1; }` then prefix-sum into `start[]`,
   then `for &x in v { let d = ((x>>shift)&0xFF) as usize; buf[start[d]] = x;
   start[d] += 1; }` and copy `buf` back. You may skip a pass whose byte is
   constant across the file (an identity permutation).
