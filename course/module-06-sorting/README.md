# Module 06 — Sorting

> **Source:** *The Art of Computer Programming*, Vol. 3, 2nd ed., Ch. 5:
> §5.1.1 (inversions), §5.2.1 (Algorithms S and D), §5.2.2 (Algorithm Q),
> §5.2.3 (Algorithm H), §5.2.4 (Algorithm N), §5.2.5 (Algorithm R), and
> §5.3.1 (the information-theoretic lower bound).
> **Lab:** `labs/module-06-sorting` · **Grade it:** `./grade 6`
> **Concrete Mathematics companion:** Chapters 2 (Sums) and 9 (Asymptotics) —
> the machinery behind every average-case bound here — see [../../docs/concrete-mathematics.md](../../docs/concrete-mathematics.md).
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 3, read §5.1.1 and §5.2 alongside — this is the heart of the
> whole series and the single richest chapter Knuth ever wrote.

Sorting is where *analysis of algorithms* comes fully into its own. We will
implement six sorts spanning the whole design space — quadratic and
log-linear, comparison-based and not — and, more importantly, *count* what
each one does and match the count to a theorem. The unifying object is the
**inversion**: a single number that measures how far a permutation is from
sorted, explains exactly why insertion sort is slow, and reappears disguised
as "runs" when we get to merging.

> **Companion exhibit — _The Sound of Order_.** The six sorts of this module
> race and sing in the Museum's
> [Algorithmic Maestro](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-3.1-sound-of-order.html):
> each is sonified (value → pitch) so bubble sort drones through its $\sim n^2/2$
> comparisons while quicksort and merge finish in $\sim n \cdot \log n$ and resolve into a
> rising scale. A "Race all six" mode runs them on one shuffled array so the
> quadratic-vs-log-linear gulf (§5.3.1's lower bound) is visible and audible at
> once.

Throughout, one running example — **Knuth's sixteen numbers** —

```
503  087  512  061  908  170  897  275  653  426  154  509  612  677  765  703
```

is sorted by every algorithm, so you can watch the same data flow through six
different machines.

---

## 1. Inversions: measuring disorder (§5.1.1)

Let $a[1]\, a[2]\, \ldots\, a[n]$ be a permutation. An **inversion** is a pair of
positions $i < j$ with $a[i] > a[j]$ — a pair that is "out of order." The
sorted sequence has zero inversions; the fully reversed sequence has every
one of the $\binom{n}{2} = n(n-1)/2$ pairs inverted, the maximum possible.

The inversion count $I$ is the natural *distance to sorted*, and it controls
the cost of any sort that fixes disorder one adjacent swap at a time.

### Theorem (insertion moves = inversions)

> Straight insertion sort (Algorithm S below) performs exactly `I` record
> moves in its inner loop, where `I` is the number of inversions of the input.

*Proof.* Consider inserting $a[j]$ into the already-sorted prefix
$a[1..j-1]$. The inner loop shifts right, one at a time, precisely those
earlier records whose key exceeds $a[j]$ — i.e. exactly the elements that
form an inversion $(i, j)$ with $i < j$. Each such element is moved once and
no other element moves during step $j$. Summing over all $j$, the total number of
moves equals the total number of inverted pairs $(i, j)$, which is $I$. ∎

This is the cleanest example in the book of the philosophy "to analyze an
algorithm, find the combinatorial quantity it is really counting."

### Average number of inversions

> Over all $n!$ permutations of $\{1, \ldots, n\}$ (each equally likely), the
> expected number of inversions is $n(n-1)/4$.

*Proof.* Fix a pair of positions $i < j$. By symmetry the two values landing
there are in the "wrong" order exactly half the time, so that pair is an
inversion with probability $1/2$. There are $\binom{n}{2} = n(n-1)/2$ pairs; by
linearity of expectation the expected total is $(1/2)\cdot n(n-1)/2 = n(n-1)/4$. ∎

So straight insertion averages $\sim n^2/4$ moves — quadratic, but with a small
constant, and *linear* on nearly-sorted input ($I$ small). That last fact is
why insertion sort is the workhorse for finishing off the small subfiles left
by quicksort (Algorithm Q, step Q9) and the whole idea behind Shellsort.

### Computing inversions in $O(n \log n)$

You do not need $n^2$ work to *count* inversions. Piggyback on a merge: when
merging two sorted halves, each time you take an element from the right half
while $m$ elements remain unconsumed in the left half, that element is smaller
than all $m$ of them — it closes exactly $m$ inversions. Add them up. The
reference uses this; the lab lets you use the naive double loop too (the tests
keep $n$ small for the direct check, but also verify the reverse-array
maximum $n(n-1)/2$ at $n = 100$).

---

## 2. Algorithm S: straight insertion (§5.2.1)

> Rearrange records $R_1, \ldots, R_N$ in place so keys are non-decreasing.

```text
S1. [Loop on j.]       Perform S2..S5 for j = 2, 3, ..., N.
S2. [Set up i, K, R.]  Set i <- j - 1, K <- K_j, R <- R_j.
S3. [Compare K : K_i.] If K >= K_i, go to S5.
                       (Assertion: R_{i+1}..R_{j-1} already shifted up; K is
                        less than every key among them.)
S4. [Move R_i, i--.]   Set R_{i+1} <- R_i, i <- i - 1. If i > 0, go to S3.
S5. [R into R_{i+1}.]  Set R_{i+1} <- R.
```

Note the $\ge$ in S3: equal keys are never jumped, so the sort is **stable**
(invisible for bare `i64`, but part of the contract — and it matters the
moment records carry satellite data).

### Hand trace on the sixteen numbers

Each row inserts one new key into the sorted prefix. The **moves** column is
the number of S4 shifts — and, by the theorem above, the number of inversions
that key closes. They sum to **41**, the inversion count of the whole input.

| j | key | moves | file after inserting (prefix sorted) |
|---|-----|-------|--------------------------------------|
| 1 | 087 | 1 | `087 503 512 ...` |
| 2 | 512 | 0 | `087 503 512 ...` |
| 3 | 061 | 3 | `061 087 503 512 ...` |
| 4 | 908 | 0 | `... 512 908 ...` |
| 5 | 170 | 3 | `061 087 170 503 512 908 ...` |
| 6 | 897 | 1 | `... 512 897 908 ...` |
| 7 | 275 | 4 | `061 087 170 275 503 512 897 908 ...` |
| 8 | 653 | 2 | `... 512 653 897 908 ...` |
| 9 | 426 | 5 | `061 087 170 275 426 503 512 653 897 908 ...` |
| 10 | 154 | 8 | `061 087 154 170 275 426 ... 908 ...` |
| 11 | 509 | 4 | `... 503 509 512 653 ...` |
| 12 | 612 | 3 | `... 512 612 653 897 908 ...` |
| 13 | 677 | 2 | `... 653 677 897 908 ...` |
| 14 | 765 | 2 | `... 677 765 897 908 ...` |
| 15 | 703 | 3 | `... 677 703 765 897 908` |

Total moves = 1+0+3+0+3+1+4+2+5+8+4+3+2+2+3 = **41 = I**. Verify with
`count_inversions` and `insertion_sort_counting` — the lab asserts they agree.

---

## 3. Algorithm D: Shellsort (§5.2.1)

Insertion sort is slow because it moves records only one position at a time,
so an element far from home takes many steps. **Shellsort** (D. L. Shell,
1959) fixes this by *h-sorting*: run insertion sort with a stride `h > 1`,
which lets records leap `h` positions at once, then shrink `h` down to 1.

A file is **h-ordered** if $a[i] \le a[i+h]$ for all valid $i$ — the
subsequences taken every $h$th element are each sorted. The last pass uses
$h = 1$, which is plain insertion sort, so the output is fully sorted; the
earlier passes exist only to make that final pass cheap, because after them
few inversions survive.

```text
D1. [Loop on s.]  For each increment h, largest first, do D2.
D2. [Loop on j.]  For j = h+1, ..., N do D3..D6  (insertion, stride h).
D3. [Set up.]     i <- j - h, K <- K_j, R <- R_j.
D4. [Compare.]    If K >= K_i, go to D6.
D5. [Move, i-=h.] R_{i+h} <- R_i, i <- i - h; if i > 0, go to D4.
D6. [Store.]      R_{i+h} <- R.
```

### The increment sequence: $h_{s+1} = 3 h_s + 1$

The reference and lab use Knuth's recommended sequence

$$1, 4, 13, 40, 121, 364, \ldots \qquad (h_1 = 1,\ h_{s+1} = 3 h_s + 1)$$

kept below $n$ and applied largest-first. So `knuth_gaps(16) = [13, 4, 1]` and
`knuth_gaps(100) = [40, 13, 4, 1]`. The choice of increments is the deep
mystery of Shellsort: its running time depends on the sequence in ways that
are *still* not fully understood. Knuth's $3h+1$ gives an empirical
$O(n^{1.5})$; Pratt's $2^i 3^j$ increments provably give $O(n (\log n)^2)$; a
best-possible sequence is an open problem (rated 50 in the text).

### Theorem (h-ordering is preserved)

> If a file is $h$-ordered, then after $k$-sorting it (for any $k$) it remains
> $h$-ordered.

This lemma (§5.2.1, and Knuth's Exercise 5.2.1-Thm K) is why later passes
never undo the work of earlier ones — successively finer orderings accumulate
rather than fight. It is the reason a decreasing increment sequence makes
progress. The lab checks a concrete instance: after 13-sorting then 4-sorting,
the file is *both* 4-ordered and still 13-ordered.

Why $3h+1$ and not, say, powers of two? With $h_s = 2^s$ the odd- and
even-indexed elements never interact until the very last pass, which can then
still face $\Theta(n^2)$ work (the classic Shell/$n/2$ bad case). Increments that
are relatively prime to each other mix the subfiles and provably help.

---

## 4. Algorithm Q: quicksort (§5.2.2)

**Partition-exchange** sort, Hoare 1962. Pick a pivot key $K$; rearrange the
segment so everything $\le K$ is on the left and everything $\ge K$ on the
right; then sort the two parts. The reference implements Knuth's Algorithm Q
faithfully: iterative with an explicit stack, a small-subfile cutoff $M$, and
median-of-three pivoting.

```text
Q1. [Initialize.]      Stack empty; (l, r) <- (1, N).
Q2. [Begin stage.]     If r - l < M, go to Q8 (leave small subfiles for Q9).
Q3-Q6. [Partition.]    Choose pivot K; scan i up while K_i < K, j down while
                       K_j > K, exchange, until i and j cross.
Q7. [Put on stack.]    Push the LARGER subfile, iterate on the smaller.
Q8. [Take off stack.]  Empty? go to Q9. Else pop (l, r), back to Q2.
Q9. [Straight insertion.] One Algorithm-S pass finishes the nearly-sorted file.
```

Two engineering points that are also correctness points:

- **Push the larger subfile, recurse on the smaller (Q7).** This bounds the
  stack to $\lg N$ entries: each pushed segment is at least twice the size of
  the one we keep, so at most $\lg N$ can be nested. Without it, an adversary
  forces $\Theta(N)$ stack depth and a crash at $N = 100{,}000$.
- **Median-of-three pivot.** Ordering $a[l], a[\mathit{mid}], a[r]$ and pivoting on the
  median (i) makes already-sorted and reverse-sorted input a *best* case
  rather than the $\Theta(n^2)$ disaster of the take-the-first-key rule, and (ii)
  plants sentinels at both ends so the inner scans need no bounds checks. The
  lab throws sorted, reverse, organ-pipe, and heavy-duplicate inputs at
  $n = 100{,}000$; naive pivoting fails them.

### Average number of comparisons

Let $C_N$ be the expected number of key comparisons quicksort makes on a
random permutation of $N$ distinct keys (ignore the $M$-cutoff; set $M = 1$).
Partitioning a segment of size $N$ costs $N + 1$ comparisons and splits it,
with the pivot equally likely to be the $k$th smallest for each $k$. So

$$C_N = (N + 1) + \frac{1}{N} \sum_{k=1}^{N} (C_{k-1} + C_{N-k}), \qquad C_0 = C_1 = 0.$$

The two sums are equal by symmetry, giving

$$C_N = (N + 1) + \frac{2}{N} \sum_{k=0}^{N-1} C_k.$$

Multiply by $N$, write the same relation for $N-1$, and subtract to kill the
sum:

$$\begin{aligned}
N C_N - (N-1) C_{N-1} &= N(N+1) - (N-1)N + 2 C_{N-1} \\
N C_N &= (N+1) C_{N-1} + 2N.
\end{aligned}$$

Divide by $N(N+1)$:

$$\frac{C_N}{N+1} = \frac{C_{N-1}}{N} + \frac{2}{N+1}.$$

Telescoping from $C_1 = 0$,

$$\frac{C_N}{N+1} = 2 \sum_{k=2}^{N} \frac{1}{k+1} \approx 2(H_{N+1} - 1.5),$$

where $H_N = 1 + 1/2 + \cdots + 1/N$ is the harmonic number (Module 02). Since
$H_N \approx \ln N + \gamma$,

$$C_N \approx 2(N+1) H_N - 3N \approx 1.386\, N \lg N.$$

So quicksort averages $\sim 2 N \ln N \approx 1.39\, N \lg N$ comparisons — about 39% more
than the information-theoretic minimum $N \lg N$ (§7 below), but with tiny
constant work per comparison. Median-of-three shaves the constant further
(to about $1.19\, N \lg N$). The lab asserts a loose ceiling $< 3 N \ln N$ at
$N = 10{,}000$.

### Worst case and its mitigation

If every pivot is the extreme key, one side of the partition is empty and the
recurrence degenerates to $C_N = (N+1) + C_{N-1}$, i.e. $\Theta(N^2)$ comparisons —
and, with naive recursion, $\Theta(N)$ stack. Median-of-three makes contrived
worst cases rare and turns the common ordered inputs into best cases; the
"push larger, iterate smaller" rule caps the stack at $\lg N$ regardless. This
is the difference between a textbook quicksort and one you can ship.

---

## 5. Algorithm H: heapsort (§5.2.3)

Heapsort (Williams/Floyd, 1964) gets quicksort's $O(N \log N)$ in the **worst**
case, in place, with no stack — by maintaining a *heap*.

A **max-heap** in an array satisfies, with 0-based indices,

$$a[(j-1)/2] \ge a[j] \quad \text{for all } 1 \le j < n,$$

i.e. every parent dominates its two children $2i+1$, $2i+2$. The maximum is
therefore always at the root $a[0]$. Two operations:

**siftup** (Knuth's name; modern texts say *sift-down*) repairs a single
root whose subtrees are already heaps, by sinking it toward the leaves:

```text
H3. [Prepare.]      K <- K_root; hole i <- root.
H4. [Advance down.] j <- 2i + 1 (left child); if j > end, go to H8.
H5. [Larger child.] If j < end and K_j < K_{j+1}, set j <- j + 1.
H6. [Larger than K?] If K >= K_j, go to H8.
H7. [Move up.]      a[i] <- a[j]; i <- j; go to H4.
H8. [Store.]        a[i] <- K.
```

**make_heap** turns an arbitrary array into a heap by sifting every internal
node from $n/2 - 1$ down to $0$. Then heapsort repeatedly swaps the root (the
max) with the last unsorted slot and sifts the new root over the shrunken
heap:

```text
H1-H2. Build the heap (siftup each internal node, right to left).
Then for r = n-1 down to 1:  swap a[0] <-> a[r];  siftup(a, 0, r-1).
```

### Theorem (building a heap is $O(n)$)

> `make_heap` performs fewer than $2n$ key comparisons — it is *linear*, not
> $n \log n$.

*Proof.* A node at height $h$ above the leaves costs at most $2h$ comparisons
(two per level descended). In a complete binary tree of $n$ nodes there are at
most $\lceil n / 2^{h+1} \rceil$ nodes of height $h$. The total is bounded by

$$\sum_{h \ge 0} \frac{n}{2^{h+1}} \cdot 2h = n \sum_{h \ge 0} \frac{h}{2^h} = n \cdot 2 = 2n,$$

using the standard sum $\sum h x^h = x/(1-x)^2$ at $x = 1/2$, which equals 2. The
key point: sifting is cheap for the *many* short subtrees near the leaves and
expensive only for the *one* root. ∎

### Worst-case bound

Each of the $n-1$ selection-phase siftups descends at most $\lg n$ levels at 2
comparisons per level, so heapsort makes at most $2n \lg n + O(n)$ comparisons
in the **worst** case — no bad inputs, unlike quicksort. The constant is
larger than quicksort's average, and heapsort's memory access pattern is
cache-hostile, which is why quicksort usually wins in practice despite the
worse worst case. The lab asserts $< 2n \lg n + 4n$ on random *and* reverse
input. On the sixteen numbers, `make_heap` lifts **908** to $a[0]$.

---

## 6. Algorithm N: natural merge sort (§5.2.4)

Merging exploits existing order. A **run** is a maximal non-decreasing stretch;
`count_runs` returns the number of them. The sixteen numbers break into
**eight** runs:

```
503 | 087 512 | 061 908 | 170 897 | 275 653 | 426 | 154 509 612 677 765 | 703
```

(A new run begins exactly at each step-down $a[i-1] > a[i]$; a sorted file is
one run, a strictly decreasing file of length $n$ is $n$ runs.)

**Natural** merge sort scans for these existing runs and merges them pairwise,
repeating until one run remains:

```text
N1. [Begin pass.]    Scan left to right.
N2. [Find two runs.] Detect the next two ascending runs; a lone trailing run
                     is copied through unchanged.
N3. [Merge.]         Merge them into the output area, smaller leading key
                     first, ties from the left run (stable).
N4. [Pass complete?] More input: back to N2. A pass that sees <= 2 runs
                     leaves the file sorted: done.
```

### Adaptivity

Each pass merges runs in pairs, so the number of runs at least **halves** each
pass: an input with $r$ runs finishes in $\lceil \lg r \rceil$ passes, each costing
$O(n)$ comparisons — total $O(n \lg r)$. When the input is already sorted,
$r = 1$: a single detection scan of $n - 1$ comparisons and we are done. The
sort *adapts* to pre-existing order, which neither quicksort nor heapsort
does. The lab measures this precisely: sorted input of 100k keys costs
$< 2n$ comparisons (indeed exactly $n - 1$), while random input stays under
$2.5\, n \lg n$.

Merge sort is also **stable** and its worst case is a clean $n \lg n$ — the
reasons it, not quicksort, backs stable library sorts and all external
(tape/disk) sorting, which is the historical subject of the rest of §5.2.4.

---

## 7. The comparison lower bound (§5.3.1), and how radix evades it

Every sort in §§2–6 learns about the data *only by comparing keys*. How few
comparisons can possibly suffice?

Model a comparison sort as a **binary decision tree**: each internal node is a
comparison $K_i : K_j$ with two outcomes, each leaf a final permutation. To
sort correctly the tree must have a distinct leaf for each of the $n!$
possible input orderings, so it needs at least $n!$ leaves. A binary tree of
height $h$ has at most $2^h$ leaves, hence

$$2^h \ge n! \implies h \ge \lg(n!).$$

Height $h$ is the worst-case number of comparisons, so:

> **Theorem (§5.3.1).** Any comparison sort makes at least $\lceil \lg(n!) \rceil$
> comparisons in the worst case (and, by a similar averaging over leaves, at
> least $\lg(n!)$ on average).

By **Stirling's approximation**, $\ln(n!) = n \ln n - n + O(\ln n)$, so

$$\lg(n!) = n \lg n - n \cdot \lg e + O(\lg n) \approx n \lg n - 1.44\, n.$$

Thus $n \lg n$ comparisons are *necessary*, and merge sort's $n \lg n$ and
heapsort's $2n \lg n$ are optimal up to the constant. Quicksort's average
$1.39\, n \lg n$ is within 39% of this floor. No comparison sort can do
asymptotically better — that is a theorem about *all possible algorithms*, the
kind of result that makes analysis of algorithms a science.

### Radix sort (Algorithm R, §5.2.5) sidesteps the bound

The lower bound assumes the *only* operation is comparison. **Radix sort makes
no comparisons at all** — it distributes keys into buckets by their digits —
so the $\lg(n!)$ bound simply does not apply.

The reference sorts `u64` keys by **8 passes of a stable counting sort**, one
byte per pass, **least significant byte first** (LSD):

```text
R1. [Loop on k.]   For byte position k = 1 (LSB), 2, ..., 8:
R2. [Count.]       count[d] = number of keys whose byte k equals d, d in 0..255.
R3. [Allocate.]    Prefix-sum: pile d begins at count[0]+...+count[d-1].
R4. [Distribute.]  Move each key, IN INPUT ORDER, to the next free slot of its
                   pile. Input order within a pile = stability.
```

Total work $O(8n) = O(n)$ — linear! The catch is that this beats $n \lg n$
only when keys are drawn from a bounded universe (here $2^{64}$, so 8 fixed
passes); it trades comparisons for extra space (the count array and a buffer)
and assumes digit extraction is $O(1)$.

### Why LSD works: stability by induction

> **Claim.** After the pass on byte $k$, the array is sorted by the low $k$
> bytes of the keys, treated as a $k$-byte number.

*Proof by induction on $k$.* **Base ($k=1$):** the first counting-sort pass
orders keys by byte 1. **Step:** assume the array is sorted by bytes $1..k-1$.
The pass on byte $k$ is a *stable* counting sort keyed on byte $k$: keys with
equal byte $k$ retain their relative order — which, by hypothesis, is sorted
order on bytes $1..k-1$. So afterward keys are ordered first by byte $k$, and
ties broken by bytes $1..k-1$: exactly sorted by the low $k$ bytes. ∎

After byte 8 the array is sorted by all 64 bits. **Stability of each pass is
load-bearing** — an unstable per-pass sort would scramble the ordering the
previous passes established. The lab proves stability directly: it packs
`(payload << 32) | original_index`, radix-sorts, and checks that records with
equal payload emerge in increasing original-index order.

---

## 8. Stage-by-stage lab guide

Open `labs/module-06-sorting/src/lab.rs`. Each stage has a test file
`tests/stage_NN_*.rs`; `./grade 6` runs them in order and stops at the first
failure. All internal sorts take `&mut [i64]` and sort in place;
`radix_sort_u64` takes `&mut [u64]`. Implement each `_counting` variant as the
*same* algorithm instrumented to return a count, and have the plain version
discard it.

### Stage 1 — `insertion_sort`, `insertion_sort_counting`, `count_inversions`

Algorithm S with the step labels as comments. Count the **S4 moves** (not the
final S5 placement). Implement `count_inversions` however you like (the naive
double loop is fine — or the merge-count of §1). The tests verify the theorem
of §1: moves == inversions, that KNUTH16 has **41** inversions, and that a
reversed array of length $n$ has $n(n-1)/2$.

### Stage 2 — `knuth_gaps`, `shell_sort`, `shell_sort_with_gaps`

Generate `1, 4, 13, 40, ...` below `n`, reversed: `knuth_gaps(16) = [13,4,1]`.
`shell_sort_with_gaps` is Algorithm D; `[1]` alone must reduce to straight
insertion. **Definiteness:** `assert!` on a zero increment with a message
containing `"positive"` (the grader checks it). The tests confirm each pass
leaves the file $h$-ordered and that later passes preserve earlier orderings.

### Stage 3 — `quicksort`, `quicksort_counting`

Algorithm Q with an **explicit stack** (no recursion), the $M = 9$ cutoff, a
final insertion pass (Q9), and **median-of-three** pivoting. Push the larger
subfile, iterate on the smaller. Count every $K_i : K$ / $K : K_j$ probe plus the
final pass. Tests: correctness on all-equal / sorted / reverse / organ-pipe,
100k random completes, $< 3 n \ln n$ comparisons at $n = 10{,}000$.

### Stage 4 — `is_heap`, `sift_up`, `make_heap`, `heapsort`, `heapsort_counting`

`sift_up(v, root, end)` is steps H3–H8 on `v[..=end]`. `make_heap` sifts
internal nodes $n/2-1 \ldots 0$. Heapsort = build + repeated extract-max. Watch
$n = 0, 1, 2$. Tests: `make_heap` $\Rightarrow$ `is_heap` with the max on top (908 for
KNUTH16), and $< 2n \lg n + 4n$ comparisons.

### Stage 5 — `count_runs`, `natural_merge_sort`, `natural_merge_sort_counting`

`count_runs` = 1 + step-downs, or 0 when empty (KNUTH16 → **8**). Algorithm N
with an auxiliary buffer; count run-detection *and* merge comparisons. The
adaptivity test: sorted 100k costs $< 2n$ (exactly $n - 1$).

### Stage 6 — `radix_sort_u64`

LSD counting sort, one byte per pass, 8 passes, **stable** distribution in
input order. Handle all-zero, single, duplicates, and `u64::MAX`. The
stability test uses packed `(payload << 32 | index)` keys.

---

## 9. Check your understanding

1. Straight insertion made 41 moves on KNUTH16. Without re-running it, how
   many comparisons did it make in the *best* possible input of the same
   length? ($n - 1 = 15$: one comparison per already-in-place key.)
2. Why does pushing the *larger* subfile in Q7 bound the quicksort stack to
   $\lg N$? (Each kept segment is $\le$ half the pushed one, so nesting depth
   $\le \lg N$.)
3. `make_heap` is $O(n)$ but heapsort is $\Theta(n \log n)$. Where does the extra
   $\log n$ factor come from? (The $n-1$ extract-max siftups, each up to
   $\lg n$ deep — the build is cheap, the teardown is not.)
4. A file has 8 runs. At most how many passes does natural merge sort take?
   ($\lceil \lg 8 \rceil = 3$.)
5. Why doesn't radix sort contradict the $\lg(n!)$ lower bound? (It performs no
   key comparisons; the bound only constrains comparison sorts.)

## 10. Exercises from the text

Ratings are Knuth's: 00 immediate · 10 a minute · 20 up to an hour · 30 hours
· 40 term project · 50 open research problem. ▶ marks especially instructive
ones. Log attempts in `course/module-06-sorting/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 5.1.1-2 | 15 | Show a reversed permutation has the maximum number of inversions, $n(n-1)/2$. |
| ▶5.1.1-8 | 22 | Relate the inversion count to the number of moves made by straight insertion (you proved this — §1). |
| 5.2.1-14 | 20 | Prove that an $h$-ordered file stays $h$-ordered after a $k$-sort (the §3 lemma). |
| ▶5.2.2-13 | 24 | Derive the average number of comparisons $C_N$ of quicksort by solving the recurrence (§4). |
| 5.2.3-20 | 22 | Prove `make_heap` uses fewer than $2n$ comparisons via the sum-of-heights bound (§5). |
| 5.2.4-12 | 20 | Show natural merge sort takes at most $\lceil \lg r \rceil$ passes on an input of $r$ runs. |
| ▶5.3.1-3 | 25 | Use Stirling to show $\lg(n!) = n \lg n - n \lg e + O(\log n)$, hence the $n \lg n$ comparison floor (§7). |
| 5.2.5-4 | 20 | Prove LSD radix sort correct by induction on the digit position, using per-pass stability (§7). |

## In the real world

The standard library sorts you call every day are direct descendants of
this module. TimSort (Python, Java, and Rust's stable `sort`) is natural
merge sort industrialized: it finds the runs your `count_runs` counts,
extends short ones with insertion sort, and merges adaptively — Knuth's
§5.2.4 plus fifty years of constant-factor engineering. Rust's unstable
`sort_unstable` (pdqsort) is Algorithm Q hardened: median-of-three
pivoting, the same insertion-sort cutoff M you implemented, plus modern
pattern defenses against the adversarial inputs you built in stage 3's
tests. Radix sort owns GPUs and database columnar engines, exactly because
it escapes the $\lg n!$ bound you proved. And inversions are Kendall's tau:
your stage-1 counter is the kernel of rank-correlation statistics used in
recommender-system evaluation.

## Proof techniques you practiced

- **Loop invariants** — every sort carries one; quicksort's partition
  invariant is the exemplar.
- **Bijection and conservation** — moves = inversions (each insertion-sort
  move removes exactly one), and $n(n-1)/4$ average inversions by pairing
  each permutation with its reverse.
- **Recurrence solving** — quicksort's average unwound to $2(n+1)H_n$-style
  form by the telescoping/perturbation trick from Module 02.
- **Amortization by summation** — heap construction is O(n) because the sum
  of subtree heights telescopes; a bound nobody guesses and everybody can
  verify.
- **Information-theoretic lower bound** — $\lg n! \approx n \lg n - n \lg e$ via
  Stirling, plus the adversary that enforces it — and the two honest escape
  hatches (radix: more information per operation; hashing: Module 07).
- **Induction over passes** — LSD radix correctness rides entirely on
  per-pass stability.

## 11. Where this leads

- **Inversions** and the $\lg(n!)$ bound are the entry point to §5.3
  (optimum sorting) and §5.3.4 (sorting networks — Module 07's territory).
- **Quicksort's partition** returns as *selection* / order statistics
  (find the $k$th smallest in $O(n)$ average, §5.3.3).
- **Merging** scales up to external and multiway merge sorting (tapes, disks,
  §5.4) — the reason merge, not quicksort, dominates when data exceeds RAM.
- The **stability** you built into insertion, merge, and radix is exactly what
  lets you sort by composite keys in stages — sort by minor key, then stably
  by major key — the pattern behind every database `ORDER BY a, b`.
