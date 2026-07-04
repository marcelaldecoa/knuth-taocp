# Module 20 — Optimum Sorting and Sorting Networks

> **Source:** *The Art of Computer Programming*, Vol. 3, 2nd ed., §5.3 —
> §5.3.1 (minimum-comparison sorting) and §5.3.4 (networks for sorting).
> **Lab:** `labs/module-20-networks` · **Grade it:** `./grade 20`
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 3, read §5.3.1 and §5.3.4 alongside it.

Module 06 taught you *how* to sort. This module asks the sharper questions:
**how few comparisons can any sort possibly use**, and **what if the sequence
of comparisons must be fixed in advance, before we see the data?** The first
question leads to a beautiful lower bound and to Ford and Johnson's
merge-insertion algorithm, which is provably optimal for small inputs. The
second leads to *sorting networks* — the data-oblivious sorters that run inside
GPU kernels, SIMD sort routines, and hardware switches — and to one of the most
elegant lemmas in all of computer science, the **zero-one principle**.

---

## 1. The comparison-tree model and a lower bound

Fix $n$ distinct inputs. A **comparison sort** learns about them only by asking
questions of the form "is $x_i < x_j$?". Every such algorithm, run on all
possible inputs, traces out a binary tree: each internal node is one
comparison, its two children are the "yes" and "no" continuations, and each
**leaf** is a final answer — a claim about the sorted order, i.e. a permutation
of the inputs.

For the algorithm to be *correct*, every one of the $n!$ possible input
orderings must reach a leaf that reports the right permutation, and two
different orderings can never share a leaf (they need different outputs). So the
tree has **at least $n!$ leaves**.

**Theorem (information-theoretic lower bound, §5.3.1).** Any comparison sort of
$n$ elements makes at least $\lceil \lg n! \rceil$ comparisons in the worst case.

*Proof.* A binary tree of height $h$ (the length of its longest root-to-leaf
path) has at most $2^h$ leaves — level $k$ holds at most $2^k$ nodes. The
worst-case number of comparisons is exactly the height $h$. Correctness forces
$2^h \ge (\text{number of leaves}) \ge n!$, so $h \ge \lg n!$. Because $h$ is an integer,
$h \ge \lceil \lg n! \rceil$. ∎

This is the first appearance in the course of an **adversary / counting
argument**: we did not analyze *any particular* algorithm; we bounded *all of
them at once* by counting the outcomes they must distinguish.

Using $\lg n! = \sum_{k \le n} \lg k \approx n \lg n - n/\ln 2$ (Stirling), the bound is
$n \lg n - 1.44 n + O(\lg n)$: asymptotically $n \lg n$, matching mergesort and
heapsort — those are asymptotically comparison-optimal.

For small $n$ the exact integer values matter:

| n | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|----|----|----|
| $\lceil \lg n! \rceil$ | 0 | 1 | 3 | 5 | 7 | 10 | 13 | 16 | 19 | 22 | 26 | 29 |

### The gap between the bound and the truth

Let $S(n)$ be the *true* minimum — the fewest comparisons that suffice in the
worst case, taken over the best possible algorithm. Clearly $S(n) \ge \lceil \lg n! \rceil$.
Is it equal?

For $n \le 11$, **yes**: $S(n) = \lceil \lg n! \rceil$. But at $n = 12$ the counting bound
says 29 while the truth is $S(12) = 30$. There simply is no 29-comparison sort
of 12 elements, even though a binary tree of height 29 has room for $2^{29} \approx 5.4
\times 10^8 \gg 12! = 4.8 \times 10^8$ leaves. The leaves cannot be *packed* to make every
one reachable by a consistent set of comparisons; the geometry of "which
comparisons are still informative" forbids it.

$S(n)$ is known only for small $n$ and remains, in general, an **open research
problem** — Knuth tabulates the frontier and it has barely moved in decades
($S(n) = \lceil \lg n! \rceil$ also for $n \le 11$ and $n = 20, 21$; it differs at $12$–$19$
and elsewhere). This is analysis of algorithms as a living science: a bound you
can *prove* in two lines, a truth nobody can compute.

> **Why it's done this way.** The counting bound is cheap and universal, so it
> is the right *first* question. When it turns out to be loose (as at $n=12$),
> that looseness is itself the discovery — it tells you the problem has
> structure the counting argument cannot see, and sends you hunting for a
> better lower bound or a cleverer algorithm.

**Stage 1** has you compute $\lceil \lg n! \rceil$, a counting sort that *reports* its
comparison count so you can watch a real algorithm sit above the bound, and
`is_sorted`.

---

## 2. Merge insertion (Ford–Johnson), Algorithm 5.3.1M

If $\lceil \lg n! \rceil$ is a target, which algorithm *hits* it for small $n$? Not straight
insertion (quadratic), not binary insertion sort (which wastes comparisons when
$n$ is not near a power of two). The champion is **merge insertion**, due to
Lester Ford Jr. and Selmer Johnson (1959). Knuth gives it as Algorithm 5.3.1M.

The idea braids merging and binary insertion so that *every* insertion happens
into a range whose size is just under a power of two — so binary search never
"wastes" a comparison.

### Algorithm M (merge insertion), in Knuth's step style

Given $n$ elements to sort:

```text
M1. [Pair up.]   Divide the elements into ⌊n/2⌋ pairs (one element is left
                 over if n is odd). Compare the two members of each pair; call
                 the larger a_i and the smaller b_i, so b_i < a_i.
M2. [Recurse.]   Sort the ⌊n/2⌋ larger elements a_i by merge insertion. Relabel
                 so a_1 < a_2 < ... , and let b_i be the partner of a_i. The
                 sequence  b_1 a_1 a_2 ... a_s  is the "main chain": note
                 b_1 < a_1, so it goes at the very front for free.
M3. [Insert.]    Binary-insert b_2, b_3, ... (and the leftover, if any) into the
                 main chain — but in a carefully chosen ORDER, not left to right.
                 Insert them in the groups
                     (b_3, b_2), (b_5, b_4), (b_11, b_10, ..., b_6), ...
                 The group boundaries are the Jacobsthal numbers
                     t_k = (2^(k+1) + (-1)^k)/3 = 1, 3, 5, 11, 21, 43, ...
                 Within a group, insert from the highest index down.
```

### Why the strange insertion order?

When we insert $b_i$, we know $b_i < a_i$, so it can only land *before* $a_i$ in
the chain. The number of elements before $a_i$ at that moment is what a binary
search must probe — and binary search into a range of size $m$ costs $\lceil \lg(m+1) \rceil$
comparisons.

Binary search is "efficient" only when $m+1$ is exactly a power of two: inserting
into 1 element or 3 elements both cost 2 comparisons, so inserting into 3 is a
bargain and inserting into 4 is a rip-off (costs 3). The Jacobsthal grouping
picks exactly the indices so that at insertion time the range before $a_i$ has
size $2^k - 1$ for the current group $k$: **every insertion in group $k$ costs
exactly $k$ comparisons, none wasted.** Inserting from the top of each group
downward is what keeps the ranges at those magic sizes.

The Jacobsthal numbers $t_k = t_{k-1} + 2 t_{k-2}$ (1, 3, 5, 11, 21, 43, …) are
precisely the sequence for which "insert group $k$ from $t_k$ down to
$t_{k-1}+1$" makes each binary search a perfect fit. (They are also the number
of ways to tile a strip with squares and dominoes avoiding two adjacent
dominoes — a lovely coincidence Knuth notes.)

### Worked trace: n = 5

Start with `[3, 1, 5, 2, 4]` (using the values as their own ranks).

- **M1. Pair.** Pairs `(3,1)` and `(5,2)`; `4` is left over. Compare each:
  $a_1',b = (3,1)$, $a_2',b = (5,2)$. Two comparisons so far.
- **M2. Recurse** on the larger elements `{3, 5}`: one comparison, sorted to
  $a_1=3 < a_2=5$. Their partners: $b_1 = 1$ (partner of 3), $b_2 = 2$ (partner
  of 5). Main chain: $b_1 a_1 a_2 = [1, 3, 5]$. (Comparisons: 3 total.)
- **M3. Insert.** To insert we have $b_2 = 2$ and the leftover `4`.
  Jacobsthal order for the two pending elements (indices 2 and 3, treating the
  leftover as "b_3"): group boundary $t_2 = 3$, so insert index 3 then index 2:
  - Insert the leftover `4` into the whole chain `[1,3,5]` by binary search:
    compare $4 : 3$ → greater, compare $4 : 5$ → less → `[1,3,4,5]`.
    (2 comparisons; range of size 3, budget $\lceil \lg 4 \rceil = 2$.)
  - Insert $b_2 = 2$, known $< a_2 = 5$, into the part before 5, i.e. `[1,3,4]`.
    Binary search: compare $2 : 3$ → less, compare $2 : 1$ → greater →
    position between 1 and 3 → `[1,2,3,4,5]`. (2 comparisons.)

Total: $3 + 2 + 2 = 7 = \lceil \lg 5! \rceil$. Merge insertion is **optimal** for $n = 5$.

### The comparison count

The worst-case number of comparisons $F(n)$ satisfies

$$F(n) = \sum_{k=1}^{n} \left\lceil \lg \tfrac{3k}{4} \right\rceil.$$

| n | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|----|----|----|
| $F(n)$ | 0 | 1 | 3 | 5 | 7 | 10 | 13 | 16 | 19 | 22 | 26 | **30** |

Compare with $\lceil \lg n! \rceil$ above: they agree up to $n = 11$, and at $n = 12$
merge insertion uses 30 — which is exactly $S(12)$. So $F(n) = S(n) = \lceil \lg n! \rceil$
for $n \le 11$, and $F(12) = S(12) = 30 > 29$. Merge insertion is the
comparison-optimal sort for every $n \le 15$ except a couple of larger cases;
Knuth details where it eventually loses (around $n = 47$) to hand-tuned
constructions.

**Stage 2** implements Algorithm M as `ford_johnson_sort` and
`ford_johnson_comparisons`. The tests check correctness on all sizes `0..=50`
and pin the worst case exhaustively for small $n$ — you get to *watch* your code
meet $F(n)$ and never exceed it.

---

## 3. Sorting networks: comparators, obliviousness, depth

A **comparator** on wires `i` and `j` is a device that reads the values on the
two wires and writes back the smaller on wire `i`, the larger on wire `j`. A
**sorting network** is a fixed sequence of comparators that sorts every input.

The defining feature: the sequence of comparators is **oblivious** — it does not
depend on the data. There is no "if `x < y` then compare these next"; the
comparisons are wired in advance. This is what lets a network become silicon, or
a straight-line SIMD routine with no data-dependent branches.

We draw a network with $n$ horizontal wires (time flows left to right) and each
comparator as a vertical connector:

```text
         0 ──●───────●───────────
             │       │
         1 ──●──●────┼────●───────
                │    │    │
         2 ─────●────●────┼───●───
                     │    │   │
         3 ──────────●────●───●───
```

Each `●──●` is a comparator; smaller value moves up. Reading the drawing is a
skill: pick any input, walk left to right, and at each connector swap the two
wires if the top value exceeds the bottom.

### Batcher's odd–even merge (Algorithm 5.3.4M)

How do we *build* a correct network? K. E. Batcher's 1968 construction is a
network version of mergesort. To sort $n = 2^t$ items:

```text
   oddeven_sort(lo, n):
       if n > 1:
           m = n/2
           oddeven_sort(lo, m)          # sort the first half
           oddeven_sort(lo + m, m)      # sort the second half
           oddeven_merge(lo, n, 1)      # merge the two sorted halves

   oddeven_merge(lo, n, r):             # merge, r = distance between compared wires
       m = 2r
       if m < n:
           oddeven_merge(lo, n, m)      # even-indexed subsequence
           oddeven_merge(lo + r, n, m)  # odd-indexed subsequence
           for i = lo+r, lo+r+m, ...  while i + r < lo + n:
               compare(i, i + r)
       else:
           compare(lo, lo + r)
```

The merge is the clever part: to combine two sorted halves, Batcher recursively
merges the even-position elements, merges the odd-position elements, then does a
single sweep of "cleanup" comparators between neighbours. That this sorts is not
obvious at all — we will *prove* it in §4 with the zero-one principle rather than
by unwinding the recursion.

**Comparator count.** $C(n) = C(n/2) \cdot 2 + M(n)$ where the merge uses
$M(n) = M(n/2) \cdot 2 + (n/2 - 1)$ comparators. Solving: $C(n) = (t^2 - t + 4) \cdot 2^{t-1} - 1$
with $t = \lg n$, i.e. $\Theta(n (\lg n)^2)$. Concretely:

| n | 2 | 4 | 8 | 16 | 32 |
|---|---|---|---|----|----|
| comparators | 1 | 5 | 19 | 63 | 191 |

**Depth.** Comparators that touch *disjoint* wires can fire in the same parallel
step. The **depth** is the fewest such steps — the *parallel time*. For
Batcher's network the depth is exactly

$$\operatorname{depth}(2^t) = \frac{t(t+1)}{2} = 1 + 2 + \cdots + t = O((\lg n)^2).$$

| n | 2 | 4 | 8 | 16 |
|---|---|---|---|----|
| depth | 1 | 3 | 6 | 10 |

To compute the depth of any network, do an **as-soon-as-possible schedule**:
give each wire a "ready layer" (initially 0); a comparator `(i,j)` goes to layer
`max(ready[i], ready[j])`, after which both wires are ready at the next layer.
The largest layer used is the depth. Because we process comparators in a valid
execution order, this ASAP pass computes the critical-path length exactly.

> **Why networks, when a good comparison sort is only $n \lg n$?** Two reasons.
> **Parallel time:** a network sorts in $O((\lg n)^2)$ depth — with $n/2$
> comparators firing at once, wall-clock time is polylogarithmic, ideal for
> hardware and GPUs. **No data-dependent branches:** the comparator schedule is
> fixed, so a network compiles to straight-line code that a CPU pipeline or a
> SIMD lane never mispredicts. You pay in *total work* ($n (\lg n)^2$ vs $n \lg n$)
> to win *latency* and *predictability*. That trade is why small networks
> ("sorting networks for 4, 8, 16 elements") are hand-inlined into the hottest
> sort kernels on Earth.

**Stage 3** builds `odd_even_merge_network`, `apply_network` (the
compare-exchange), and `network_depth`. The tests pin the counts and depths and
check the network sorts *every* permutation for $n \le 8$.

---

## 4. The zero-one principle (Theorem Z)

Checking that a network sorts by trying all $n!$ permutations is hopeless for
$n = 16$ (over $2 \times 10^{13}$). The zero-one principle collapses the work to $2^n$.

**Theorem Z (§5.3.4).** A comparator network sorts all inputs (from any ordered
set) **if and only if** it sorts all $2^n$ inputs consisting only of 0s and 1s.

*Proof.* "Only if" is trivial. For "if": suppose the network sorts every 0-1
input but fails on some input $x = (x_1, \ldots, x_n)$, producing an output that is
not nondecreasing. The key lemma is that comparators **commute with monotone
functions**: if $f$ is nondecreasing, then running the network on
$f(x) = (f(x_1), \ldots, f(x_n))$ produces $f(y)$ where $y$ is the output on $x$.
This is because a single comparator computes $\min$ and $\max$, and any monotone
$f$ satisfies $f(\min(a,b)) = \min(f(a),f(b))$ and $f(\max(a,b)) = \max(f(a),f(b))$;
apply comparator by comparator.

Now, since $y$ is not sorted, there is a position $k$ with $y_k > y_{k+1}$.
Choose the **threshold function**

$$f(v) = \begin{cases} 0 & \text{if } v < y_k,\\ 1 & \text{if } v \ge y_k. \end{cases}$$

$f$ is nondecreasing, so the network on $f(x)$ outputs $f(y)$. But
$f(y_k) = 1$ (since $y_k \ge y_k$) and $f(y_{k+1}) = 0$ (since $y_{k+1} < y_k$), so
$f(y)$ has a 1 immediately before a 0 — it is **not sorted**. Thus $f(x)$ is a
0-1 input the network fails to sort, contradicting the assumption. Hence no such
$x$ exists: the network sorts everything. ∎

This is a **reduction by a well-chosen map**: turn a hypothetical bad general
input into a bad 0-1 input via the threshold trick. It is one of the most reused
proof patterns in the theory of parallel and comparison algorithms.

Two payoffs:

1. **A cheap certificate.** To *verify* a network, enumerate all $2^n$ binary
   strings and check each is sorted. For $n = 16$ that is $65\,536$ tests instead
   of $16!$. Stage 4's `sorts_all_zero_one` is exactly this certificate.
2. **A design tool.** To *prove* Batcher's construction sorts, argue only about
   0-1 sequences — where "sorted" means "all 0s then all 1s" and merging two
   sorted 0-1 halves is a counting argument about how many 1s land where.

### Bitonic sorting

Batcher's *other* network is the **bitonic sorter**. A sequence is *bitonic* if
it rises then falls (or is a rotation of such). The bitonic merger takes a
bitonic sequence and sorts it; building an ascending run next to a descending
run makes their concatenation bitonic, and recursion sorts everything:

```text
   bitonic_sort(lo, n, ascending):
       if n > 1:
           m = n/2
           bitonic_sort(lo, m, ascending)          # first half ascending...
           bitonic_sort(lo + m, m, descending)     # ...second half descending
           bitonic_merge(lo, n, ascending)         # now the whole is bitonic

   bitonic_merge(lo, n, dir):
       if n > 1:
           m = n/2
           for i in lo .. lo+m: compare_in_direction(i, i+m, dir)
           bitonic_merge(lo, m, dir)
           bitonic_merge(lo + m, m, dir)
```

A "descending" comparator just lists its wires the other way, $(j, i)$ with
$j > i$, so the same compare-exchange primitive still works. Bitonic sort uses
more comparators than odd-even merge ($n/2 \cdot t(t+1)/2$, e.g. 24 for $n=8$) but
has the same $O((\lg n)^2)$ depth and an extremely regular, index-arithmetic
structure — which is why **GPU implementations overwhelmingly use bitonic sort**:
every thread computes its partner with a couple of bit operations.

**Stage 4** implements `sorts_all_zero_one` and `bitonic_sort_network`. You will
certify both Batcher networks via the zero-one principle, then watch a
*deliberately broken* network (one comparator removed) get caught by a 0-1
counterexample.

---

## 5. Stage-by-stage lab guide

Open `labs/module-20-networks/src/lab.rs`. Run `./grade 20`; the grader stops at
the first failing stage.

### Stage 1 — `min_comparisons_lower_bound`, `sort_and_count`, `is_sorted`

Compute $\lceil \lg n! \rceil$ with a wide-integer factorial and a "smallest $c$ with
$2^c \ge n!$" loop. `sort_and_count` can be any correct sort that counts key
comparisons — binary insertion sort is a clean choice. The tests pin the table
of $\lceil \lg n! \rceil$ for $n \le 12$ and dramatize the $S(12) = 30 > 29$ gap.

### Stage 2 — `ford_johnson_sort`, `ford_johnson_comparisons`

Implement Algorithm M. The fiddly part is the **partner bookkeeping** across the
recursion and the **Jacobsthal insertion order** of step M3. A robust design
sorts *indices* by key (so partners survive the recursive sort) and folds the
smaller elements in group order. The tests verify correctness for all sizes up
to 50 and pin the exhaustive worst case $F(n)$ for small $n$.

### Stage 3 — `odd_even_merge_network`, `apply_network`, `network_depth`

Transcribe Batcher's recursion into a comparator list; `apply_network` is a
three-line compare-exchange; `network_depth` is the ASAP schedule. Counts and
depths are pinned to Batcher's formulas.

### Stage 4 — `sorts_all_zero_one`, `bitonic_sort_network`

`sorts_all_zero_one` enumerates $2^n$ binary inputs — the certificate from
Theorem Z. `bitonic_sort_network` is the bitonic recursion with direction-aware
comparators. The tests certify both networks, cross-check against random
integers, and confirm a broken network fails.

### Bench

`cargo run --release --example bench -p lab-20-networks --features solutions`
times the oblivious network against a data-dependent sort at growing $n$,
printing `n | comps | net | sort | ratio` — the constant-factor / branch-free
story made concrete.

---

## 6. Check your understanding

1. Why must a correct comparison-sort decision tree have at least $n!$ leaves,
   and why "at least" rather than "exactly"?
2. $\lceil \lg 12! \rceil = 29$ but $S(12) = 30$. Does this contradict the lower-bound
   theorem? (No — the theorem gives a lower bound; the truth can be higher.)
3. In merge insertion, why is $b_1$ placed at the front of the main chain with
   *no* comparison?
4. State the zero-one principle. In its proof, what property of a single
   comparator makes it commute with a monotone function $f$?
5. A network has depth 6 on 8 wires but 19 comparators. How can 19 comparators
   fit in 6 parallel steps? (Disjoint-wire comparators share a step; up to 4 can
   fire at once on 8 wires.)

## 7. Exercises from the text

Ratings are Knuth's (00 immediate · 10 a minute · 20 up to an hour · 30 hours ·
40 term project · 50 open problem). ▶ marks especially instructive ones. Log
attempts in `course/module-20-networks/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 5.3.1-2 | 10 | Show $\lceil \lg n! \rceil$ equals $\sum_{k=1}^{n} \lceil \lg k \rceil$ corrected — compute both sides for $n \le 8$ and reconcile. |
| ▶5.3.1-3 | 22 | Prove the worst-case count of merge insertion is $F(n) = \sum \lceil \lg(3k/4) \rceil$. |
| 5.3.1-4 | 20 | Hand-trace merge insertion on $n = 11$; confirm it uses 26 comparisons. |
| 5.3.1-8 | 24 | For which $n \le 20$ is $F(n) = \lceil \lg n! \rceil$? Tabulate and explain the first failure. |
| ▶5.3.4-1 | 20 | Prove the zero-one principle (Theorem Z) in full. |
| 5.3.4-6 | 22 | Prove Batcher's odd-even merge is correct using the zero-one principle. |
| ▶5.3.4-10 | 28 | Show any sorting network on $n$ wires has depth $\ge \lceil \lg n \rceil$ and at least $\lceil \lg n! \rceil$ comparators. |
| 5.3.4-38 | 30 | Investigate the minimum-comparator networks for $n \le 10$ (the "known optimum" table). |

## In the real world

- **SIMD and vectorized sort.** Intel's `x86-simd-sort` (the library behind
  NumPy's and C++ `std::sort`'s fast paths) and Google Highway's `vqsort` sort
  small blocks with hard-coded sorting networks: for 16 or 32 elements a
  branch-free network of vector `min`/`max` instructions beats any comparison
  sort because it never mispredicts. The compare-exchange you write in
  `apply_network` *is* a `vmin`/`vmax` pair on real silicon.
- **GPU sorting.** Bitonic sort is the workhorse of GPU sort kernels (CUDA/OpenCL
  merge-path and bitonic builders): its comparator partners are pure index
  arithmetic, so every thread knows who to compare with no coordination. The
  $O((\lg n)^2)$ depth is $O((\lg n)^2)$ parallel steps — excellent when you have
  thousands of cores.
- **Hardware.** Switching fabrics and networking ASICs sort or route with fixed
  comparator arrays; obliviousness means fixed wiring and constant latency.
- **Median filters.** Image and signal pipelines find medians of small windows
  ($3\times3$, $5\times5$) with tiny optimal networks — a 9-element median network is a
  classic hand-optimized gadget.
- **The AKS network.** Ajtai, Komlós, and Szemerédi (1983) built a sorting
  network of *optimal* $O(n \lg n)$ comparators and $O(\lg n)$ depth — matching the
  lower bound asymptotically. But its constant factor is astronomically large
  (thousands), so it is never used in practice; Batcher's $O(n (\lg n)^2)$ network
  wins for every realistic $n$. It is the canonical example of an asymptotically
  optimal algorithm that is practically irrelevant.
- **Secure computation.** Oblivious sorting is essential in MPC and ORAM, where
  the *access pattern* must leak nothing about the data — a data-oblivious
  network is exactly what you need.

## Proof techniques you practiced

- **Counting / information-theoretic lower bound** — $n!$ leaves in a binary tree
  force height $\ge \lceil \lg n! \rceil$, bounding *every* comparison sort at once (§1).
- **Optimality by construction with a matching bound** — merge insertion is shown
  optimal for small $n$ by exhibiting an algorithm that *meets* $\lceil \lg n! \rceil$ (§2).
- **Amortized "budget" accounting** — the Jacobsthal insertion order keeps every
  binary search inside a $2^k - 1$ range so no comparison is wasted (§2).
- **Reduction via a well-chosen map** — the zero-one principle turns a bad
  general input into a bad 0-1 input through a threshold function, using that
  comparators commute with monotone maps (§4).
- **Critical-path (ASAP) analysis** — network depth as the longest dependency
  chain, computed by a one-pass schedule (§3).

## 8. Where this leads

- **Module 21 (Boolean functions)** builds *straight-line* combinational circuits
  — the comparator networks here are a special case, and the "median/threshold"
  networks connect directly to symmetric Boolean functions.
- The **lower-bound mindset** (count the outcomes, bound the tree) recurs for
  selection, merging, and searching; §5.3.2–5.3.3 in Knuth push it to the median
  and to merging.
- **Data-oblivious algorithms** are the entry point to parallel algorithm design
  and to privacy-preserving computation (ORAM, MPC) — the network is the seed
  crystal.
