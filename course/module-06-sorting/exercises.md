# Exercises — Module 06 (Sorting)

Self-contained problems on this module's material — inversions, straight
insertion, Shellsort's $h$-ordering, quicksort's average cost, heap
construction, natural merge sort, the comparison lower bound, and LSD radix
sort. You can work every one **without the books**: each states the problem in
full, gives a **hint** to peek at when stuck, and a worked **answer sketch** to
check against after you try. Computational answers here are reproduced by the
code you write in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Where a problem mirrors a TAOCP exercise its number is noted for
readers who own Volume 3.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Reversed permutation attains the maximum $n(n-1)/2$ inversions | 15 | ⬜ |
| 2 | ▶ Inversions equal straight-insertion moves | 22 | ⬜ |
| 3 | An $h$-ordered file stays $h$-ordered after a $k$-sort | 20 | ⬜ |
| 4 | ▶ Derive quicksort's average comparison count $C_N$ | 24 | ⬜ |
| 5 | `make_heap` uses fewer than $2n$ comparisons | 22 | ⬜ |
| 6 | Natural merge takes $\le \lceil \lg r \rceil$ passes on $r$ runs | 20 | ⬜ |
| 7 | ▶ Stirling gives $\lg(n!) = n \lg n - n \lg e + O(\log n)$ | 25 | ⬜ |
| 8 | LSD radix sort correct by induction on digit position | 20 | ⬜ |

## Problems

### 1. Reversed permutation attains the maximum $n(n-1)/2$ inversions (rating 15 · cf. 5.1.1-2)

**Problem.** An *inversion* of a permutation $a[1]\,a[2]\,\ldots\,a[n]$ is a pair
of positions $i < j$ with $a[i] > a[j]$. Show that the fully reversed
permutation $n, n-1, \ldots, 2, 1$ has $\binom{n}{2} = n(n-1)/2$ inversions, and
that no permutation can have more.

**Hint.** How many pairs $(i, j)$ with $i < j$ are there in total? For the
reversed sequence, which of them are inverted?

**Answer sketch.** There are exactly $\binom{n}{2} = n(n-1)/2$ position pairs
$i < j$, and each is *either* an inversion or not — so $n(n-1)/2$ is a hard upper
bound for any permutation. In the reversed sequence $a[i] = n - i + 1$ is
strictly decreasing, so *every* pair $i < j$ satisfies $a[i] > a[j]$: all
$n(n-1)/2$ pairs are inversions, meeting the bound. (Verified: $n = 16$ gives
$16 \cdot 15 / 2 = 120$, the value the lab asserts for a reversed length-100
array scaled down — the reference pins $n(n-1)/2$ at $n = 100$.) This maximum is
exactly why straight insertion is worst on reverse-sorted input.

### 2. ▶ Inversions equal straight-insertion moves (rating 22 · cf. 5.1.1-8)

**Problem.** Straight insertion sort (Algorithm S) inserts $a[j]$ into the sorted
prefix $a[1..j-1]$ by shifting larger keys one slot to the right (step S4). Prove
that the *total* number of S4 shifts over the whole sort equals the inversion
count $I$ of the input.

**Hint.** Fix the step that inserts $a[j]$. Which earlier elements get shifted,
and how do they relate to inversions ending at position $j$? Nothing outside step
$j$ moves during step $j$.

**Answer sketch.** When inserting $a[j]$, step S4 shifts right exactly those
records $a[i]$ ($i < j$) whose key exceeds $a[j]$ — i.e. exactly the pairs
$(i, j)$ that form an inversion with second coordinate $j$. Each is moved once and
nothing else moves during step $j$. Summing over $j = 2, \ldots, n$, the total
moves equal the total number of inverted pairs, namely $I$. (Verified on Knuth's
sixteen numbers: the per-step moves $1{+}0{+}3{+}0{+}3{+}1{+}4{+}2{+}5{+}8{+}4{+}3{+}2{+}2{+}3 = 41$,
and a direct inversion count also gives $41$.) This is the model example of
"analyze an algorithm by finding the combinatorial quantity it is really
counting," and it immediately explains insertion sort's $\sim n^2/4$ average
(since the average inversion count is $n(n-1)/4$) and its linearity on
nearly-sorted input.

### 3. An $h$-ordered file stays $h$-ordered after a $k$-sort (rating 20 · cf. 5.2.1-14)

**Problem.** A file is *$h$-ordered* if $a[i] \le a[i+h]$ for every valid $i$.
Shellsort works by $h$-sorting for a decreasing sequence of increments. Prove the
lemma that makes the decreasing sequence sound: if a file is $h$-ordered, then
after $k$-sorting it (for any increment $k$) it is *still* $h$-ordered — so later
passes never undo earlier ones.

**Hint.** A $k$-sort sorts each of the $k$ subfiles taken at stride $k$
independently. Compare the file before and after against a known combinatorial
fact: sorting corresponding subsequences preserves an existing "$\le$ at distance
$h$" relation. Think of the elements at positions $i$ and $i+h$ and where they
land after the $k$-sort.

**Answer sketch.** This is a special case of the fact that if two sequences
$x_1 \le \cdots \le x_m$ and $y_1 \le \cdots \le y_m$ satisfy $x_t \le y_t$ for all
$t$, and each is then sorted, the pointwise inequality survives. In the file,
partition into stride-$k$ subfiles; the $k$-sort sorts each subfile. Consider any
$i$: the $h$-ordering $a[i] \le a[i+h]$, applied across all positions, means the
sorted subfiles remain pointwise dominated at offset $h$, so $a[i] \le a[i+h]$
still holds afterward. Hence the file stays $h$-ordered. (The lab checks a
concrete instance from the $\{13, 4, 1\}$ sequence: after 13-sorting then
4-sorting, the file is *both* 4-ordered and still 13-ordered.) Because finer
orderings accumulate instead of fighting, a decreasing increment sequence makes
monotone progress toward sorted, which is the whole reason Shellsort works.

### 4. ▶ Derive quicksort's average comparison count $C_N$ (rating 24 · cf. 5.2.2-13)

**Problem.** Let $C_N$ be the expected number of key comparisons quicksort makes
on a random permutation of $N$ distinct keys (take cutoff $M = 1$, so no
insertion pass). Partitioning a size-$N$ segment costs $N - 1$ comparisons and
leaves the pivot equally likely to be the $k$th smallest. Set up and solve the
recurrence for $C_N$ in closed form, and give its leading asymptotic term.

**Hint.** Start from

$$
C_N = (N-1) + \frac{2}{N}\sum_{k=0}^{N-1} C_k, \qquad C_0 = C_1 = 0.
$$

Multiply by $N$, write the same relation for $N-1$, subtract to eliminate the
sum, then divide by $N(N+1)$ and telescope. The partial fraction
$\frac{2(N-1)}{N(N+1)} = \frac{4}{N+1} - \frac{2}{N}$ is the key algebra.

**Answer sketch.** Multiplying by $N$ and subtracting the $N-1$ version kills the
sum:

$$
N C_N = (N+1) C_{N-1} + 2(N-1).
$$

Dividing by $N(N+1)$ and using the partial fraction gives

$$
\frac{C_N}{N+1} = \frac{C_{N-1}}{N} + \frac{4}{N+1} - \frac{2}{N}.
$$

Telescoping from $C_1 = 0$ yields $\frac{C_N}{N+1} = 2 H_N - 4 + \frac{4}{N+1}$,
so the exact closed form is

$$
C_N = 2(N+1) H_N - 4N,
$$

where $H_N = 1 + \tfrac12 + \cdots + \tfrac1N$. (Verified against the raw
recurrence: $N = 5 \to 7.4$, $N = 10 \to 24.4373$, $N = 100 \to 647.8503$, both
formulas agreeing exactly.) Since $H_N \approx \ln N + \gamma$,

$$
C_N \sim 2N \ln N = (2\ln 2)\,N \lg N \approx 1.386\, N \lg N,
$$

about 39% above the information-theoretic floor $N \lg N$ (Problem 7).
Median-of-three pivoting lowers the constant to about $1.19\, N \lg N$.

### 5. `make_heap` uses fewer than $2n$ comparisons (rating 22 · cf. 5.2.3-20)

**Problem.** `make_heap` turns an arbitrary array into a max-heap by sifting each
internal node, from index $n/2 - 1$ down to $0$. Sifting a node of height $h$
above the leaves costs at most $2h$ comparisons (two per level descended). Prove
that the whole build costs fewer than $2n$ comparisons — it is *linear*, not
$n \log n$.

**Hint.** Count how many nodes sit at each height $h$ in a complete binary tree
of $n$ nodes: at most $\lceil n / 2^{h+1} \rceil$. Then sum $2h$ times that over
all heights, and use the closed form of $\sum_{h \ge 0} h x^h$ at $x = \tfrac12$.

**Answer sketch.** At most $\lceil n/2^{h+1}\rceil$ nodes have height $h$, each
costing $\le 2h$ comparisons, so the total is at most

$$
\sum_{h \ge 0} \frac{n}{2^{h+1}} \cdot 2h = n \sum_{h \ge 0} \frac{h}{2^h}.
$$

Using $\sum_{h \ge 0} h x^h = \dfrac{x}{(1-x)^2}$ at $x = \tfrac12$, the inner sum
is $\dfrac{1/2}{(1/2)^2} = 2$. (Verified numerically: $\sum_{h\ge0} h/2^h = 2$.)
Hence the build costs at most $2n$ comparisons — linear. The intuition: sifting
is cheap for the *many* short subtrees near the leaves and expensive only for the
*one* tall root, so the cost is dominated by the abundant cheap cases. (The lab
pins the max at the root after `make_heap` — 908 for the sixteen numbers.) This
is why heapsort's $\Theta(n \log n)$ comes entirely from the $n-1$ extract-max
siftups, not from building the heap.

### 6. Natural merge takes $\le \lceil \lg r \rceil$ passes on $r$ runs (rating 20 · cf. 5.2.4-12)

**Problem.** A *run* is a maximal non-decreasing stretch; natural merge sort
detects the existing runs and merges them pairwise each pass, repeating until one
run remains. Show that on an input with $r$ runs the sort finishes in at most
$\lceil \lg r \rceil$ passes, each costing $O(n)$ comparisons, for total
$O(n \lg r)$. What does this give for already-sorted input?

**Hint.** How does the run count change across one pass that merges runs in
pairs? Set up the recurrence for the number of runs remaining and count how many
halvings reach $1$.

**Answer sketch.** Each pass pairs up adjacent runs and merges each pair into
one, so $r$ runs become at most $\lceil r/2 \rceil$ runs. Starting from $r$, after
$p$ passes at most $\lceil r/2^p \rceil$ runs remain; this reaches $1$ once
$2^p \ge r$, i.e. after $p = \lceil \lg r \rceil$ passes. Each pass scans the
whole file once, $O(n)$ comparisons, so the total is $O(n \lg r)$. For sorted
input $r = 1$: a single detection scan of $n - 1$ comparisons and we are done —
the sort *adapts* to pre-existing order (neither quicksort nor heapsort does).
(Verified: the sixteen numbers split into $r = 8$ runs, so
$\lceil \lg 8 \rceil = 3$ passes; the lab confirms sorted 100k input costs exactly
$n - 1$ comparisons.)

### 7. ▶ Stirling gives $\lg(n!) = n \lg n - n \lg e + O(\log n)$ (rating 25 · cf. 5.3.1-3)

**Problem.** A comparison sort is a binary decision tree with a distinct leaf for
each of the $n!$ orderings, so its worst-case height is at least
$\lceil \lg(n!) \rceil$. Use Stirling's approximation
$\ln(n!) = n \ln n - n + O(\ln n)$ to show

$$
\lg(n!) = n \lg n - n \lg e + O(\log n) \approx n \lg n - 1.44\,n,
$$

and conclude that every comparison sort needs $\ge n \lg n - O(n)$ comparisons in
the worst case.

**Hint.** Convert natural logs to base 2 by multiplying through by
$\lg e = 1/\ln 2$. The $-n$ term in Stirling becomes $-n \lg e$; the $O(\ln n)$
error becomes $O(\log n)$ (bases differ by a constant).

**Answer sketch.** Multiply Stirling's $\ln(n!) = n \ln n - n + O(\ln n)$ by
$\lg e = 1/\ln 2$:

$$
\lg(n!) = (\lg e)\ln(n!) = n(\lg e)\ln n - n \lg e + O(\log n)
= n \lg n - n \lg e + O(\log n),
$$

since $(\lg e)\ln n = \lg n$. With $\lg e = 1.44269\ldots$ this is
$\approx n \lg n - 1.44\,n$. (Verified: $n = 16$ gives $\lg(16!) = 44.25$ vs.
$n \lg n - n \lg e = 40.92$; $n = 100$ gives $524.77$ vs. $520.12$ — the gap is
the $O(\log n)$ term.) Because the decision tree needs $\ge n!$ leaves and a
height-$h$ binary tree has $\le 2^h$ leaves, $h \ge \lg(n!) = n \lg n - O(n)$: no
comparison sort can beat $n \lg n$ comparisons asymptotically. Merge sort's
$n \lg n$ and heapsort's $2n \lg n$ are therefore optimal up to the constant.

### 8. LSD radix sort correct by induction on digit position (rating 20 · cf. 5.2.5-4)

**Problem.** LSD radix sort makes 8 passes over `u64` keys, each a *stable*
counting sort on one byte, least-significant byte first. Prove it correctly sorts
the keys, by induction on the byte position, and identify exactly where
per-pass stability is required.

**Hint.** State the invariant "after the pass on byte $k$, the array is sorted by
the low $k$ bytes treated as a $k$-byte number." For the inductive step, the pass
on byte $k$ keys on byte $k$ only — what does stability guarantee about keys with
equal byte $k$?

**Answer sketch.** *Base ($k = 1$):* the first stable counting sort orders keys by
byte 1. *Step:* assume the array is sorted by bytes $1..k-1$. The pass on byte $k$
is a stable counting sort keyed on byte $k$, so keys with equal byte $k$ keep
their incoming relative order — which by hypothesis is sorted on bytes $1..k-1$.
Afterward keys are ordered first by byte $k$, ties broken by bytes $1..k-1$: i.e.
sorted by the low $k$ bytes. After byte 8 the array is sorted by all 64 bits.
**Stability is load-bearing at every step:** an unstable per-pass sort would
scramble the ordering the earlier passes established, breaking the induction.
(The lab proves stability directly by packing `(payload << 32) | original_index`
and checking equal-payload records emerge in increasing index order.) Total work
is $O(8n) = O(n)$ — linear, evading the $\lg(n!)$ comparison bound because radix
sort performs *no* key comparisons.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
