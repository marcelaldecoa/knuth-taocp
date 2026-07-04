# Exercises — Module 15 (External Sorting)

Self-contained problems on this module's material — replacement selection and
the $2P$ law, the tree of losers, and the polyphase merge with its Fibonacci
distributions. You can work every one **without the books**: each states the
problem in full, gives a **hint** to peek at when stuck, and a worked **answer
sketch** to check against. Computational answers here are reproduced by the
code you write in the lab (or a few lines at a REPL).

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` =
especially instructive). Numbers like "cf. 5.4.1–1" point at the matching
exercise of Vol. 3 §5.4 for readers who own it.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Replacement selection by hand, $P = 4$, sixteen keys | 10 | ⬜ |
| 2 | Reverse input $\Rightarrow$ runs of exactly $P$ | 15 | ⬜ |
| 3 | ▶ Loser-tree replay $= \lceil \lg k \rceil$; winner tree pays double | M22 | ⬜ |
| 4 | ▶ The 13-run/3-tape table; redo with 21 runs | 15 | ⬜ |
| 5 | Perfect distributions for $T = 4$; the totals recurrence | M20 | ⬜ |
| 6 | ▶ Optimal dummy placement vs. append-at-back | M28 | ⬜ |
| 7 | First run has expected length $(e-1)P$ | M30 | ⬜ |

## Problems

### 1. Replacement selection by hand, $P = 4$, sixteen keys (rating 10 · cf. 5.4.1–1)

**Problem.** Run replacement selection (Algorithm R, §2) by hand with memory
$P = 4$ on Knuth's sixteen keys
$$503,\ 087,\ 512,\ 061,\ 908,\ 170,\ 897,\ 275,\ 653,\ 426,\ 154,\ 509,\ 612,\ 677,\ 765,\ 703.$$
Give the runs it produces and their lengths, and compare with the $P = 3$
result from §2 (three runs of lengths $4, 6, 6$). What is special about the
$P = 4$ answer?

**Hint.** Keep the four in-memory records tagged by (run number, key); output
the smallest, refill from the input, and freeze an arrival whose key is *less*
than the last key output (tag it with the next run number). A run ends when
every in-memory record is frozen.

**Answer sketch.** With $P = 4$ the algorithm produces **two runs, each of
length exactly $8$**:
$$[61,\ 87,\ 170,\ 503,\ 512,\ 653,\ 897,\ 908] \quad\big|\quad [154,\ 275,\ 426,\ 509,\ 612,\ 677,\ 703,\ 765].$$
So $16$ records become $2$ runs of length $8 = 2P$ — a clean illustration of
Moore's $2P$ law (§2): raising memory from $3$ to $4$ dropped the run count
from $3$ to $2$ and pushed the average run length from $\tfrac{16}{3} \approx 5.3$
to $8$. (The first run being a *full* $2P$ here, rather than the shorter
$(e-1)P$ of Problem 7, is luck of this particular short input; the law is a
steady-state average.) Cross-check: a direct simulation of Algorithm R
reproduces both runs and the $4,6,6$ lengths of the $P = 3$ case.

### 2. Reverse input $\Rightarrow$ runs of exactly $P$ (rating 15 · cf. 5.4.1–3)

**Problem.** Prove that if the input is in strictly **decreasing** order, then
replacement selection with memory $P$ produces runs of length exactly $P$
(until the tail). This is the worst case the §2 lesson names; stage 1 tests it,
and here you write the argument.

**Hint.** After R1 fills memory with the first $P$ records, watch step R5. When
the input is decreasing, how does each new arrival compare with the key just
output?

**Answer sketch.** After R1, memory holds the first $P$ records, all tagged with
the current run number $r$. The algorithm outputs them in increasing key
order. Consider any arrival $X$ read in R4 during run $r$: it is later in the
(decreasing) input than every record already read, so $X$ is **smaller than
every key seen so far**, in particular smaller than LASTKEY (the key just
output). By R5, $X < \text{LASTKEY}$, so $X$ is **frozen** with tag $r+1$. Thus
*every* arrival during run $r$ is frozen: run $r$ consists of exactly the $P$
records present in memory when it began, and no more. When those $P$ are
output, all remaining in-memory records carry tag $r+1$, so R2 closes run $r$
and the same argument repeats for run $r+1$ with the next $P$ records. Hence
each run has length exactly $P$ — the worst case, since replacement selection
can do no worse than sort-a-chunk. (Contrast §2's other extreme: already-sorted
input never freezes anything and yields a single run.) $\blacksquare$

### 3. ▶ Loser-tree replay $= \lceil \lg k \rceil$; where a winner tree pays double (rating M22 · cf. 5.4.1–10)

**Problem.** In the tree of losers (§3) merging $k$ runs, show that after the
champion is output and its run advances, restoring the tournament costs exactly
$\lceil \lg k \rceil$ key comparisons when $k$ is a power of two (and never more
after padding $k$ up to the next power of two). Then explain where a **winner
tree** — one storing the *winner* of each subtree at each node — pays up to
**two** comparisons per level instead of one.

**Hint.** Only the matches on the path from the changed leaf to the root can
change outcome. At each such node, who is the new entrant's opponent, and is it
already sitting there? For the winner tree, ask what a node stores versus what
the replay needs to compare against.

**Answer sketch.** Pad $k$ to $kk = 2^{\lceil \lg k \rceil}$ so every leaf has depth
exactly $\lceil \lg k \rceil$; padding leaves are $+\infty$ and matches against them
are flag checks, not key comparisons. When the champion's leaf gets a new
front record, the *only* matches whose outcome can change are those on that
leaf-to-root path — everything else involves the same two subtrees as before.
There are $\lceil \lg k \rceil$ nodes on that path (one per level). At each, the
climbing record meets the record already stored there, and — this is the whole
point of a *loser* tree — that stored record is precisely the **loser** of the
match previously played at that node, i.e. the best player from the *other*
side of the match, the exact opponent the replay needs. So each level costs
**one** comparison and touches **one** node: $\lceil \lg k \rceil$ comparisons
total (e.g. $\lceil \lg 4 \rceil = 2$; $\lceil \lg 100 \rceil = 7$).

A **winner** tree stores at each node the *winner* of the subtree below it. On
replay, the climber's true opponent is the best player of the *sibling*
subtree — but that is not what the node holds; the node holds the winner of the
climber's *own* subtree. To find the sibling's champion the replay must read
the sibling node and compare, then compare again to record the new subtree
winner — up to **two node reads and two comparisons per level**, roughly
$2\lceil \lg k \rceil$. The loser tree keeps the right opponent in the exact place
the replay visits; that is why §3 (and stage 2's comparison-bound test,
$\le n\lceil \lg k \rceil + k$) prefers it over both a winner tree and a
`BinaryHeap` (pop-then-push also costs up to two comparisons per level).

### 4. ▶ The 13-run/3-tape table; redo with 21 runs (rating 15 · cf. 5.4.2–1)

**Problem.** Reproduce §4's polyphase table for $S = 13$ unit runs on $T = 3$
tapes — perfect distribution $(8, 5)$, five phases — and verify the total merge
I/O is **$50$ records** ($\approx 3.85$ passes). Then redo the whole trace for
$S = 21$ runs on $3$ tapes: give the distribution, the per-phase records moved,
the number of phases, and the total.

**Hint.** Perfect 3-tape totals are the Fibonacci numbers $1, 2, 3, 5, 8, 13,
21, \dots$; $21$ is itself perfect, so no dummies are needed. Each phase merges
onto the currently empty tape until one input tape empties, and every other
tape keeps its unread runs where they are.

**Answer sketch.** For $S = 13$: distribution $(8, 5)$; phases move
$10, 9, 10, 8, 13$ records, total $10+9+10+8+13 = \mathbf{50}$, i.e.
$50/13 \approx 3.85$ passes — beating balanced 2-way's $4$ full passes ($52$),
and on one fewer drive.

For $S = 21$: distribution is the next perfect level, $(13, 8)$, giving
**$6$ phases**:

| phase | merge | onto | records moved |
|---|---|---|---|
| 1 | 8 groups | tape 3 | 16 |
| 2 | 5 groups | tape 2 | 15 |
| 3 | 3 groups | tape 1 | 15 |
| 4 | 2 groups | tape 3 | 16 |
| 5 | 1 group  | tape 2 | 13 |
| 6 | 1 group  | tape 1 | 21 |

Total $16+15+15+16+13+21 = \mathbf{96}$ records, i.e. $96/21 \approx 4.57$
passes. (Both traces reproduced by a direct polyphase simulation with unit
runs.) The pattern to savor is §4's phase 2 analogue: the tape not chosen as
output keeps its partially consumed reel exactly in place, which is why a phase
moves only *part* of the file.

### 5. Perfect distributions for $T = 4$; the totals recurrence (rating M20 · cf. 5.4.2–3)

**Problem.** Using the level recurrence of §4 for $T$ tapes,
$$a_1' = a_1 + a_2,\quad a_2' = a_1 + a_3,\quad \dots,\quad a_{T-1}' = a_1,$$
compute the perfect distributions for $T = 4$ from level $0 = (1, 0, 0)$ up to a
total of $57$, and verify that the totals obey
$t_n = t_{n-1} + t_{n-2} + t_{n-3}$ (the "tribonacci" law).

**Hint.** Apply the three update rules to each level (sorting the result
non-increasing), and add the components to get each total. The totals should be
$1, 3, 5, 9, 17, 31, 57$.

**Answer sketch.** The $T = 4$ levels and totals:

| level | distribution | total |
|---|---|---|
| 0 | $(1, 0, 0)$ | 1 |
| 1 | $(1, 1, 1)$ | 3 |
| 2 | $(2, 2, 1)$ | 5 |
| 3 | $(4, 3, 2)$ | 9 |
| 4 | $(7, 6, 4)$ | 17 |
| 5 | $(13, 11, 7)$ | 31 |
| 6 | $(24, 20, 13)$ | 57 |

The totals $1, 3, 5, 9, 17, 31, 57$ satisfy $t_n = t_{n-1} + t_{n-2} + t_{n-3}$:
$9 = 5+3+1$, $17 = 9+5+3$, $31 = 17+9+5$, $57 = 31+17+9$. This is §4's
generalized Fibonacci law $t_n = t_{n-1} + \cdots + t_{n-(T-1)}$ at $T = 4$
(three predecessors). The recurrence follows because reversing one perfect
phase restores, to each surviving tape, the $a_1$ runs the exhausted tape
consumed alongside it — exactly the level formula — and summing telescopes into
the tribonacci sum. (All levels and the recurrence confirmed by direct
computation.)

### 6. ▶ Optimal dummy placement vs. append-at-back (rating M28 · cf. 5.4.2–13)

**Problem.** Real files rarely have a perfect number of runs, so §4 pads with
**dummy runs** — imaginary runs of length zero that merge for free. The lab uses
the simplest convention: append the dummies at the back of each tape's quota.
Knuth instead places dummies where they are **merged fewest times**. Show that
placement matters, and quantify the saving in total record I/O for $S = 6$ real
runs on $T = 3$ tapes.

**Hint.** The smallest perfect 3-tape total $\ge 6$ is $8$ (distribution
$(5, 3)$), so you deal $6$ real runs and $2$ dummies. A dummy has length $0$, so
it never moves a record itself — but *where* it sits changes how many phases
each *real* run survives, hence how many times each real record is copied.
Simulate the polyphase merge for different dummy positions and total the records
moved.

**Answer sketch.** With $S = 6$, $T = 3$: perfect total $8$, distribution
$(5, 3)$, $8 - 6 = 2$ dummies. Simulating the merge with unit-length real runs
and length-$0$ dummies, the total record I/O ranges over placements from
**$17$ (best) to $20$ (worst)**. The lab's **append-at-back** convention
(tape 1 $= 5$ real; tape 2 $= 1$ real then $2$ dummies at the back) costs
**$19$**. The **optimal** placement puts each dummy at the *front* of an input
tape, so it is consumed in phase 1 — merged exactly once, the fewest possible —
costing **$17$**. The saving is $19 - 17 = \mathbf{2}$ records (about $11\%$ of
the merge I/O at this tiny size). The mechanism, matching §4: a dummy merged
early frees a slot so a real run enters fewer subsequent merges; a dummy that
lingers (append-at-back) drags real records through extra phases. The saving
grows with $S$ and $T$, which is why Knuth's Algorithm 5.4.2D tracks dummy
positions ($D(j)$ counters) rather than dumping them at the back. (Best/worst/
convention totals all reproduced by exhaustive simulation over the $\binom{8}{2}$
dummy placements.)

### 7. First run has expected length $(e-1)P$ (rating M30 · cf. 5.4.1–21)

**Problem.** §2's snow-plow argument gives expected steady-state run length
$2P$. But the *first* run is special: the lesson states its expected length is
$(e-1)P \approx 1.718P$, shorter than $2P$. Explain why, in terms of the
snow-plow picture, and confirm the constant.

**Hint.** The steady state assumes the track already carries its equilibrium
$P$ units of snow spread uniformly. When the very first run begins, how much
snow is on the ground? What does the plow eat on a lap over a track that starts
*bare* and is only now accumulating?

**Answer sketch.** The $2P$ law is a **steady-state** result: it assumes the
"track" already holds its equilibrium load of $P$ frozen records spread
uniformly, so a lap sweeps depth $hL$ everywhere and the run length is $hL = 2P$
(§2). The **first** run starts from a *bare track* — memory was just filled in
R1 with $P$ records but nothing has been frozen yet, so there is no
pre-accumulated snow ahead of the plow. The plow eats only the snow that falls
during its first lap over ground that is still filling up, so the first run is
shorter. Carrying out the same conservation calculation with the bare-track
initial condition (the accumulation integral now runs over a track building up
from empty rather than a full uniform load) yields expected first-run length
$(e - 1)P$. Numerically $e - 1 = 1.71828\ldots$, so the first run averages about
$1.72P$ against the steady-state $2P$ — which is exactly the caveat §2 attaches
to its stage-1 experiment ($P = 64$, $n = 100{,}000$: the overall average lands
near $128 = 2P$ because the short first run is diluted by the long steady-state
tail). The constant $e - 1$ is confirmed to the stated precision, and a
simulation of first-run lengths over random inputs tracks $\approx 1.72P$.

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
