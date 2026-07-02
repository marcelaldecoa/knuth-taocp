# Walkthrough — Module 06: Sorting

Read this AFTER a stage is green — it explains how the reference solution is
built and why.

## Stage 1: Straight insertion; inversions

The reference funnels both `insertion_sort` and `insertion_sort_counting`
through one private `insertion_moves`, which returns the move count; the plain
sort just discards it. That single-source design guarantees the counting variant
sorts *identically* to the plain one — there is no chance of the instrumented
and uninstrumented paths drifting apart. The inner loop uses `k < v[i-1]`
(strictly less), so equal keys are never shifted: the sort is stable, and each
executed shift removes exactly one inversion, which is why the returned count
equals `count_inversions` by the §5.1.1 theorem. `count_inversions` itself does
*not* reuse insertion sort — it runs an O(n log n) merge-count (`merge_count`),
counting `mid - i` crossings each time a right-half element is emitted while `i`
left elements remain. That way the "moves = inversions" identity is checked by
two genuinely different algorithms rather than one comparing with itself, which
is the honest way to anchor the theorem.

## Stage 2: Shellsort: diminishing increments

`shell_sort` delegates to `shell_sort_with_gaps`, so the increment *policy* and
the sorting *engine* are decoupled — you can feed Algorithm D any increment
sequence, and a lone `[1]` collapses it to exactly straight insertion (a nice
sanity anchor). The engine is Stage 1's inner loop with every `1` replaced by
`h`: `while i >= h && k < v[i-h]`. The correctness invariant is that after the
pass with increment `h` the file is *h-ordered* (`v[i] <= v[i+h]`), and the
key theorem (§5.2.1) is that a later `k`-sort preserves h-ordering — so passes
accumulate order rather than fight. `knuth_gaps` generates `3h+1` values while
they stay below `n` and reverses them; it returns empty for `n <= 1`. The
`assert!(h > 0, "increments must be positive")` is definiteness again: a zero
increment would loop forever, so the reference refuses it with a pinned message
rather than hanging.

## Stage 3: Quicksort: partition exchange

This is the fully hardened Algorithm Q, and every "engineering" choice is really
a correctness choice. The explicit `stack` of `(l, r)` pairs replaces recursion,
and the rule "push the *larger* subfile, iterate on the smaller" caps the stack
at lg N entries — without it an adversary forces Θ(N) depth and a crash. The
median-of-three step orders `v[l] <= v[mid] <= v[r]` before partitioning, which
does double duty: it makes sorted and reverse-sorted inputs a *best* case instead
of the Θ(n²) disaster of take-the-first-key, and it plants `v[l]`/`v[r]` as
sentinels so the two inner scans need no bounds checks (they cannot run off the
segment). Small subfiles (`<= M = 9`) are deliberately left unsorted during
partitioning and swept up by a single straight-insertion pass at the very end
(Q9), which is cheap because every element is already within `M` of home. The
comparison counter threads through all of this — scans, the three median
comparisons, and Q9 — so `quicksort_counting` reports the true `~1.2 n lg n`
that the analysis predicts.

## Stage 4: Heapsort

`sift_up`, `make_heap`, and `heapsort` all route through `*_counting` inner
functions carrying a `&mut u64` comparison counter — the same single-source
discipline as Stage 1, so the plain and instrumented forms cannot diverge. The
siftup is written with a *hole*: instead of swapping the sinking key down level
by level (two writes per level), it lifts the larger child into the hole and only
writes the saved key `k` once at the bottom — half the memory traffic, and the
`k >= v[j]` early exit stops as soon as the heap property is restored. `make_heap`
processes internal nodes `n/2-1 .. 0` in reverse, which is what makes the build
O(n): short subtrees near the leaves dominate the count, and their heights sum to
less than `2n`. The `assert!(root <= end && end < v.len(), "...out of bounds")`
guards the range the selection phase shrinks each step. The one bug this design
avoids is the off-by-one in the child index under 0-based arrays: the reference
consistently uses `2*i+1` for the left child, and the tests confirm the max (908
on KNUTH16) reaches `v[0]` after the build.

## Stage 5: Natural merge sort

`natural_merge_inner` allocates one auxiliary buffer of `n` and plays a
ping-pong game between `v` and `buf`, exactly like Knuth's two-tape Algorithm N
but in memory. The adaptivity that distinguishes merge sort lives in the pass
structure: each pass detects the *existing* ascending runs (advancing while
`v[j-1] <= v[j]`) and merges them two at a time, so the run count at least halves
per pass and an already-sorted file (one run) finishes after a single detection
scan of `n-1` comparisons — something neither quicksort nor heapsort can do. A
lone trailing run is copied straight through, and the pass loop terminates the
moment a pass sees `<= 2` runs (the file is then sorted). The merge uses `v[a] <=
v[b]` with ties taken from the left run, so the sort is stable. Counting both the
run-detection comparisons and the merge comparisons is what lets the test verify
the `< 2n` cost on sorted input and `~ n lg n` on random input.

## Stage 6: Radix sorting

`radix_sort_u64` is eight rounds of a stable counting sort, LSD first, and its
correctness is a clean induction: after the pass on byte `k`, the array is sorted
by its low `k` bytes, *because* each pass is stable and preserves the ordering
the previous passes established. The reference makes stability explicit by
distributing keys in *input order* into piles whose starting offsets come from a
prefix sum of the byte counts (`start[d] += 1` after each placement) — reversing
the scan or the offsets would silently break stability and scramble prior passes.
The `if count.iter().any(|&c| c == n) { continue; }` shortcut skips any pass whose
byte is constant across the whole file (a no-op permutation), a small but real
speedup on keys that don't use all 64 bits. Because this sort never compares two
keys, the `lg(n!)` lower bound of §5.3.1 simply does not apply — the reason it
runs in O(8n) and owns GPU and columnar-database sorting.
