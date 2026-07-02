# Module 15 — External Sorting

> **Source:** *The Art of Computer Programming*, Vol. 3, 2nd ed., §5.4
> (Algorithm 5.4.1R replacement selection; §5.4.1 multiway merging and trees
> of losers; §5.4.2 the polyphase merge, Algorithm 5.4.2D).
> **Lab:** `labs/module-15-external` · **Grade it:** `./grade 15`
>
> This lesson is self-contained: you can complete the module without the
> book. If you own Vol. 3, read §5.4.1–§5.4.2 first.

Everything you sorted in Module 06 fit in memory. Now it doesn't. External
sorting is what happens when the file is bigger than the workspace — and it
is the *reason* Knuth spends half of Volume 3 on merging: when data lives on
a device that loves sequential streams and punishes random access, the whole
game changes. You will build the classic pipeline: **replacement selection**
to create initial runs (magically, runs *twice* as long as your memory),
a **tree of losers** to merge k runs at the information-theoretic rate, and
the **polyphase merge**, where the Fibonacci numbers walk in uninvited and
solve a scheduling problem about tape drives. An I/O accountant keeps
everyone honest.

---

## 1. Why external ≠ internal: the memory hierarchy is the algorithm

Internal sorting counts comparisons because comparisons are what you pay
for. External sorting counts **record transfers**, because moving a record
between memory and external storage costs thousands of times more than
comparing two keys already in memory.

Knuth's device is a 1960s magnetic tape; yours is an NVMe SSD or a cloud
object store. The constants have shrunk by nine orders of magnitude, but the
*asymmetry* is identical:

- **Sequential access is cheap.** A tape streams records past the head; an
  SSD serves large sequential reads at full bus bandwidth; S3 serves ranged
  GETs best in big contiguous chunks.
- **Random access is expensive.** A tape must physically rewind (seconds!);
  an SSD random 4K read costs ~100× the per-byte price of a sequential
  scan; a disk seek costs ~10 ms while the CPU executes tens of millions of
  instructions.

So every algorithm in this module is built from one legal move: *read a
stream from the front, write a stream at the back.* That is exactly what the
lab models:

- a **record** is an `i64` key;
- a **run** is a non-decreasing `Vec<i64>` — a sorted stream;
- a **tape** is a queue of runs;
- the **I/O accountant** (`IoStats`) charges one unit per record read into
  memory and one per record written out.

**Definition (pass).** One *pass* is the I/O of reading every record of the
file once and writing it once: `n` reads + `n` writes. The quality measure
of an external sort is *how many passes it makes*, and the entire module is
a campaign to shrink that number. The plan of every merge sort since 1945:

```text
Pass 0 (run formation): read the file through memory, write initial runs.
Merge passes:           combine runs into fewer, longer runs, until one.
```

Fewer initial runs ⇒ fewer merge passes. Higher merge order ⇒ fewer merge
passes. Cleverer scheduling ⇒ cheaper passes. Those are stages 1, 2 and 3.

---

## 2. Run formation: replacement selection (Algorithm 5.4.1R)

The obvious pass 0: fill memory (P records), sort, write a run; repeat.
That produces runs of length exactly P. Replacement selection produces runs
of expected length **2P** — from the same memory — by refusing to draw a
hard boundary between one memoryful and the next.

Keep the P records in a selection structure ordered by the two-part key
`(RN, KEY)`: run number first, then key. Output the smallest; refill the
slot from the input immediately. A new arrival that is ≥ the last key
output can still ride the current run; one that is smaller has missed its
chance and is **frozen** — tagged with the next run's number, asleep in
memory until the current run finishes.

### Algorithm R (replacement selection, simplified memory model)

```text
R1. [Initialize.]       Fill the P slots with the first P records of the
                        input, all tagged RN = 0.
R2. [End of run?]       If the smallest tag in memory exceeds the current
                        run number, close the current run and start the next.
R3. [Output top.]       Output the record with the smallest (RN, KEY);
                        call its key LASTKEY.
R4. [Input new record.] Read the next record X. (If the input is exhausted
                        the slot empties; when memory empties, stop.)
R5. [Freeze or not.]    If X < LASTKEY, tag X with RN + 1 — frozen.
                        Otherwise tag it RN. Insert; return to R2.
```

(Knuth's Algorithm 5.4.1R fuses these steps with the loser tree you will
build in stage 2 — the freezing is done by treating `(RN, KEY)` as the
tournament key. Any priority structure ordered by `(RN, KEY)` behaves
identically; a `BinaryHeap<Reverse<(usize, i64)>>` is fine for stage 1.)

### Hand trace: P = 3 on Knuth's sixteen keys

Input: `503 087 512 061 908 170 897 275 653 426 154 509 612 677 765 703`.

| memory (RN:KEY)           | output      | read next | note              |
|---------------------------|-------------|-----------|-------------------|
| 0:503 0:087 0:512         | 087         | 061       | 061 < 087: freeze |
| 0:503 0:512 **1:061**     | 503         | 908       | rides run 0       |
| 0:512 0:908 1:061         | 512         | 170       | freeze            |
| 0:908 1:061 1:170         | 908         | 897       | freeze            |
| 1:061 1:170 1:897         | — run 0 ends: every record is frozen —   |
| 1:061 1:170 1:897         | 061         | 275       | rides run 1       |
| …                         | 170 275 426 653 897 | … | (freezes: 154 509 612) |
| 2:154 2:509 2:612         | — run 1 ends —                           |
| …                         | 154 509 612 677 703 765 |  | input dry |

Three runs — `087 503 512 908 | 061 170 275 426 653 897 | 154 509 612 677
703 765` — of lengths 4, 6, 6 from a memory that holds only 3 records.
Stage 1 pins this exact example.

Two boundary behaviours worth internalizing (both tested):

- **Already-sorted input ⇒ one run, for any P.** Nothing ever arrives
  smaller than the last output; nothing freezes; the run never ends.
- **Reverse-sorted input ⇒ runs of length exactly P.** Every arrival
  freezes, so each run is precisely the P records present at its start.
  This is the worst case; P = 1 similarly degenerates to the input's
  natural ascending runs (module 06's `count_runs` counts them).

### The snow-plow argument: why 2P

**Theorem (E. F. Moore, 1961).** On random input (keys i.i.d., all relative
orders equally likely), the expected run length of replacement selection
with memory P tends to **2P** in the steady state.

*Proof sketch — Knuth's snow-plow.* Map keys into position on a **circular
track** of circumference 1 (via the keys' quantiles — uniform arrivals).
Watch the algorithm run in its steady state:

- The last-output key LASTKEY sweeps upward through key space, wraps at the
  run boundary, and sweeps again: a **snow-plow driving laps around the
  track**.
- Arriving records are **snowflakes falling uniformly** on the track. A
  flake ahead of the plow (key ≥ LASTKEY) will be swept up *this* lap: it
  joins the current run. A flake behind the plow is frozen: it waits on the
  ground for the *next* lap.
- One flake falls for each flake plowed (R4 refills what R3 outputs), so in
  equilibrium **the snow on the ground is constant: exactly P records.**

Now compute. Let snow fall at rate *h* per unit length per unit time, and
let a lap take time *L*. When the plow reaches a point, snow there has been
accumulating since the plow last visited — a full lap, time *L* — so the
plow removes depth *hL* everywhere. One lap therefore sweeps `hL · 1 = hL`
records: **the run length is hL.**

How much snow lies on the ground at any instant? A point the plow passed
time *t* ago carries depth *ht*, and *t* is spread uniformly over [0, L)
around the track, so the total is

```text
    ∫₀¹ h·(time since plow passed) dx  =  h · L/2   =   P.
```

Hence `hL = 2P`. ∎

The picture also explains the boundaries: sorted input is snow that always
falls *just ahead* of the plow (one infinite lap); reverse-sorted input is
snow that always falls just behind (the plow only ever eats the P flakes
present at lap start). Stage 1's property test does the experiment: P = 64,
n = 100 000 random keys, and the average run length lands within a few
percent of 128. (The *first* run is shorter — expected `(e−1)P ≈ 1.72P` —
because the track starts bare; the steady state is what the test measures.)

---

## 3. k-way merging with a tree of losers (§5.4.1)

Merging k runs of total length n naively compares the k front records at
every step: `(k−1)·n` comparisons. With a tournament tree it costs
`⌈lg k⌉` per record — and since merging n records resolves `n lg k` bits of
uncertainty about the interleaving, that is the information-theoretic rate:
you cannot do better.

The right tournament tree is the **tree of losers**. The k run fronts sit
at the leaves. Play the tournament (smaller key wins); each internal node
remembers the **loser** of the match played there, and slot 0 above the
root remembers the champion.

Hand trace, four runs:

```text
R0 = [12, 50]   R1 = [7, 90]   R2 = [30, 40]   R3 = [8, 9]

          champion: R1 (7)                    champion: R3 (8)
        ┌──────[ 1: loser R3 (8) ]─┐        ┌──────[ 1: loser R0 (12) ]─┐
        │                          │  =>    │                           │
  [2: loser R0 (12)]   [3: loser R2 (30)]  [2: loser R1 (90)]  [3: loser R2 (30)]
    │           │        │          │        │          │        │          │
  R0:12       R1:7     R2:30      R3:8     R0:12     R1:90     R2:30      R3:9
```

Left: the initial tournament. Node 2 played 12 vs 7 (R1 won, R0 recorded
as loser); node 3 played 30 vs 8 (R3 won); node 1 played the winners, 7 vs
8 (R1 won, R3 recorded). Champion 7 is output and R1 advances to 90. Now
**replay only the path from R1's leaf to the root** — that produces the
right-hand tree:

- at node 2 the climber 90 meets the stored loser 12: 12 wins, 90 stays
  behind as the new loser;
- at node 1 the climber 12 meets the stored loser 8: 8 wins, 12 stays.

New champion: 8. Two comparisons — exactly `lg 4`.

**Why losers beat winners.** After the champion's leaf changes, the only
matches whose outcome can change are the ones on that leaf-to-root path.
And at each such node, the new entrant's opponent is *the best player from
the other side of that match* — which is precisely the **loser** stored
there (it lost only to the departed champion's line). A *winner* tree
stores at each node the winner of the subtree below, so the replay would
have to look sideways at the sibling subtree's winner at every level: two
node reads per level and messier code. The loser tree keeps the exact
opponent in the exact place the replay visits. One node, one comparison,
per level.

**Comparison bound (stage 2's contract).** Building the initial tournament
plays one match per internal node: ≤ k − 1 key comparisons. Each of the n
outputs replays one root path: ≤ ⌈lg k⌉ comparisons (pad k to the next
power of two so every leaf has depth exactly ⌈lg k⌉). Total:

```text
    comparisons  ≤  n·⌈lg k⌉ + k.
```

Exhausted runs are handled with a +∞ sentinel (`None` in the lab): matches
against +∞ are flag checks, not key comparisons, so a skewed merge (one
long run, many short) still meets the bound. A `BinaryHeap` misses it by a
factor approaching 2 — pop-then-push costs up to two comparisons per level
— and stage 2's test can tell.

---

## 4. The polyphase merge: Fibonacci on tape (§5.4.2)

Suppose you have T tape drives and S initial runs. The **balanced merge**
splits the drives half and half: k = T/2 input tapes, T/2 output tapes,
merge k-ways, swap roles, repeat. Every phase is a full pass, and there are
`⌈log_k S⌉` of them: total merge I/O `n·⌈log_k S⌉` written (plus pass 0).
With T = 4 and S = 13 that is `⌈lg 13⌉ = 4` full passes.

Polyphase asks a sneaky question: *why must a phase be a full pass?* Keep
**T − 1 input tapes and a single output tape**, and merge (T−1)-ways until
exactly *one* input tape is exhausted. Stop there. The exhausted tape
becomes the new output; **every other tape keeps its unread runs exactly
where they are** — partially consumed, mid-reel. A phase now moves only
*part* of the file, so you can afford more phases than balanced merging,
of higher order, on fewer drives.

### The Fibonacci insight

When does this schedule work perfectly — every phase a genuine (T−1)-way
merge, one tape (not two) dying per phase, ending with exactly one run?
Run the machine **backwards** from the end. Take T = 3 (two input tapes),
writing run counts as pairs:

- After the last phase: `(1, 0)` — one run, somewhere.
- That phase must have merged 1 run from each input: before it, `(1, 1)`.
- Un-merging the phase before: the tape that provided the *survivors* had
  1 extra run, so `(2, 1)`. Before that, `(3, 2)`; then `(5, 3)`, `(8, 5)`…

The perfect run counts for three tapes are **consecutive Fibonacci
numbers**, totals 1, 2, 3, 5, 8, 13, 21, … The forward statement of the
recurrence, for T tapes and level-n distribution `(a₁ ≥ a₂ ≥ … ≥ a_{T−1})`:

```text
    level 0:  (1, 0, …, 0)
    level n+1:  a₁' = a₁ + a₂,  a₂' = a₁ + a₃,  …,
                a_{T−2}' = a₁ + a_{T−1},  a_{T−1}' = a₁.
```

**Perfect-distribution theorem.** A polyphase merge on T tapes can run
perfectly from initial run counts `(a₁, …, a_{T−1})` if and only if they
form a level of this recurrence; the totals `t_n` obey the generalized
Fibonacci law

```text
    t_n = t_{n−1} + t_{n−2} + … + t_{n−(T−1)}
```

(T = 3: 1, 2, 3, 5, 8, 13 — Fibonacci. T = 4: 1, 3, 5, 9, 17, 31 — each
the sum of the previous three.)

*Proof sketch.* Backward induction, exactly as in the T = 3 story: a
perfect final state is level 0. Reversing one phase of a level-n state
adds, to each surviving tape, the a₁ runs that the exhausted tape consumed
alongside it — that is precisely the level n+1 formula; and any perfect
schedule *must* reverse this way because each phase, run backward, restores
one run to every input tape per merged output run. The totals recurrence
follows by summing: `t_{n+1} = (T−1)·a₁(n) + (t_n − a₁(n))`, with
`a₁(n) = t_{n−(T−2)} … ` telescoping into the sum of the previous T − 1
totals. ∎

### Hand trace: 13 runs on 3 tapes (Knuth's tape table)

Write `1⁸` for "8 runs of relative length 1". Initial distribution: level
5 = (8, 5). n = 13 records (unit runs).

| phase | tape 1 | tape 2 | tape 3 | action                | records moved |
|-------|--------|--------|--------|-----------------------|---------------|
| start | 1⁸     | 1⁵     | —      | distribute            | (13)          |
| 1     | 1³     | —      | 2⁵     | merge 5 pairs → T3    | 10            |
| 2     | —      | 3³     | 2²     | merge 3 pairs → T2    | 9             |
| 3     | 5²     | 3¹     | —      | merge 2 pairs → T1    | 10            |
| 4     | 5¹     | —      | 8¹     | merge 1 pair → T3     | 8             |
| 5     | —      | 13¹    | —      | merge 1 pair → T2     | 13            |

Read phase 2 carefully — it is the whole trick. Tape 3 still holds 2 unread
runs of length 2 *and stays where it is*; only tape 1's three runt runs and
three of tape 3's runs are touched. Total merge I/O: 10 + 9 + 10 + 8 + 13 =
**50 records ≈ 3.85 passes**, versus balanced 2-way's 4 full passes (52) —
*and balanced needed a fourth tape drive*. Asymptotically 3-tape polyphase
sorts S runs in about `1.04 lg S` effective passes; more tapes push the
growth ratio from φ toward 2 and the passes down further.

### Dummy runs

Real files do not arrive with Fibonacci run counts. If replacement
selection produced S runs and the smallest perfect total ≥ S is t, invent
`t − S` **dummy runs** — imaginary runs of length zero. A dummy merges at
zero I/O cost (merging "nothing" with real runs just renames them), so the
schedule keeps its perfect shape and the phase count is unchanged. Knuth's
Algorithm 5.4.2D interleaves the distribution and the dummy bookkeeping
(his D(j) counters) and places dummies where they will be merged fewest
times; the lab uses the simplest correct convention — **compute the
perfect distribution, deal out real runs, top up each tape's quota with
empty runs** — and the tests hold you only to that documented convention.

### The bill, precisely

Let S runs come out of pass 0 and let the perfect level be *phases*. Run
formation moves every record once (n reads + n writes). Each merge phase
reads and writes at most n records — usually far fewer. Hence the ceiling
stage 4 asserts:

```text
    records_written  ≤  n · (1 + phases)          (polyphase, ceiling)
    records_written  =  n · (1 + ⌈log_k S⌉)       (balanced k-way, exact)
```

and one run (already-sorted input, or memory ≥ n) costs exactly one pass:
n reads, n writes, zero phases. The accountant `IoStats` makes these
theorems executable.

---

## 5. Stage-by-stage lab guide

Open `labs/module-15-external/src/lab.rs`. Run `./grade 15`; stages go in
order, stopping at the first failure.

### Stage 1 — `replacement_selection` (Algorithm 5.4.1R)

Implement Algorithm R with a `BinaryHeap<Reverse<(usize, i64)>>` (or your
own structure) ordered by `(RN, KEY)`. Keep the step labels as comments.
Details that bite: the run boundary is detected when the *popped* record's
tag exceeds the current run number (step R2 before R3); refill from the
input on every output while input remains (R4); `p == 0` panics with a
message containing `"at least one"`; empty input yields no runs. Tests: the
P = 3 worked example pinned exactly, sortedness + permutation on LCG data,
one run for sorted input, exact-P runs for reverse input, the 2P law at
P = 64 / n = 100 000, P = 1 ≡ natural runs, P ≥ n ≡ one sorted run.

### Stage 2 — `merge_runs`, `merge_runs_counting` (loser tree, §5.4.1)

Build the tree of losers. The compact array layout: pad to
`kk = k.next_power_of_two()`; internal nodes at slots `1..kk`; leaf j lives
(conceptually) at slot `kk + j`; parent of slot i is `i/2`; slot 0 holds
the champion. Represent exhausted/padding fronts as `None` = +∞ and don't
count comparisons against them. Initial build: one match per internal node,
bottom-up (recursion is easiest: play both children, store the loser,
return the winner). After each output: advance that run, replay the one
root path. Tests: correctness against flatten-and-sort on equal, skewed,
empty, duplicate-heavy, k = 1 and k = 100 shapes, plus the
`n·⌈lg k⌉ + O(k)` comparison bound — which a `BinaryHeap` implementation
fails.

### Stage 3 — `polyphase_distribution`, `polyphase_merge` (§5.4.2)

`polyphase_distribution`: iterate the generalized-Fibonacci recurrence from
`(1, 0, …, 0)` until the total covers `num_runs`; return that level
(length T − 1, non-increasing; all zeros for `num_runs == 0`; panic with
`"at least 3"` for `tapes < 3`). `polyphase_merge`: deal runs onto T − 1
tapes per the distribution, topping quotas up with empty (dummy) runs; then
loop — find the empty tape, merge `min(input tape run counts)` groups onto
it with your stage-2 merger, count one phase — until one run remains.
Hand-check your phase counts against the table above before trusting the
pins: 2 runs → 1 phase, 3 → 2, 5 → 3, 8 → 4, 13 → 5.

### Stage 4 — `external_sort` (the pipeline, with the accountant)

Wire it together: charge n reads + n writes for run formation, then for
every merge group charge reads = sum of input-run lengths and writes =
merged length. Reuse your stage-3 phase loop (a shared helper taking
`&mut IoStats` is the tidy factoring). Tests: 200 000 records through 256
slots and 3 tapes — sorted, permutation, and
`records_written ≤ n·(1 + phases)` with phases recomputed from stage-3
logic; sorted input pinned at exactly n reads + n writes; `memory ≥ n`
ditto; empty input costs zero; reverse-sorted worst case still under the
ceiling.

---

## 6. Check your understanding

Answer before moving on (hints in parentheses).

1. Replacement selection and the runs-of-P method use the same memory. Where
   do the extra P records of each run *come from*? (Hint: while a run is
   being written, how many records pass *through* memory rather than sit in
   it? The plow eats snow that fell after the lap began.)
2. Why must the selection structure order by `(RN, KEY)` and not by `KEY`
   alone with a separate "frozen" set? (Hint: what is the first record of
   the next run, and when must it be ready? Both designs work — what does
   the pair ordering buy in code and in Knuth's fused Algorithm R?)
3. In the loser tree, after the champion's run advances, why is the stored
   loser at each path node *exactly* the right opponent — why can no other
   leaf's standing change? (Hint: which matches did the old champion play?)
4. Why does polyphase break down with T = 2 tapes, and why does the
   backward reconstruction *force* Fibonacci for T = 3? (Hint: with one
   input tape, a "merge" copies. Reverse a perfect phase and count what
   returns to each tape.)
5. Your laptop's SSD does random reads a mere ~100× slower than sequential,
   not ~100 000× like a tape seek. Which parts of this module survive that
   change and which soften? (Hint: think per-record cost of a k-way merge
   with k = 1000 — memory for input buffers is `k × buffer size`; what
   does shrinking the buffer do to I/O granularity?)

---

## 7. Exercises from the text

Ratings use Knuth's scale (00 immediate · 10 a minute · 20 fifteen minutes
to an hour · 30 hours · M = mathematical · ▶ = especially instructive).
Log your work in `course/module-15-external/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 5.4.1-1 | 10 | Run replacement selection by hand with P = 4 on the sixteen example keys; compare run lengths with P = 3. |
| 5.4.1-3 | 15 | Prove: on reverse-ordered input every run has length exactly P (stage 1 tests it; you write the argument). |
| ▶5.4.1-10 | M22 | Show a loser-tree replay makes exactly ⌈lg k⌉ key comparisons when k is a power of two, and never more after padding. Where does a winner tree pay double? |
| ▶5.4.2-1 | 15 | Reproduce the 13-run/3-tape table and verify the 50-record total; then redo it with 21 runs and 6 phases. |
| 5.4.2-3 | M20 | Derive perfect distributions for T = 4 through total 57 and verify tₙ = tₙ₋₁ + tₙ₋₂ + tₙ₋₃. |
| ▶5.4.2-13 | M28 | Dummy placement: show that putting dummies where they are merged *fewest* times beats our append-at-the-back convention, and quantify the saving for S = 6, T = 3. |
| 5.4.1-21 | M30 | The first run is special: show its expected length is (e − 1)P ≈ 1.718P, not 2P, and explain via the bare snow-plow track. |

---

## Why it's done this way

- **Charge for I/O, not comparisons.** Every design choice in §5.4 falls
  out of one inversion: the expensive resource is the *channel*, not the
  comparator. That is why run formation happily spends a heap operation per
  record (to halve the number of runs), why merging goes k-way instead of
  2-way (comparisons per record grow like lg k, but *passes* shrink like
  1/lg k — a wonderful trade), and why the lab's tests assert `IoStats`
  bounds rather than time.
- **Replacement selection over sort-a-chunk** because doubling run length
  costs *nothing* at sort time and removes one whole merge level about half
  the time (`log` of half as many runs). The snow-plow tells you it's not
  a hack: 2P is a steady-state law, not a lucky constant.
- **Loser tree over heap** because the merge inner loop is the hottest loop
  in the whole sort: one comparison per level, one node touched per level,
  no sift-down branching. Knuth builds Algorithm R *inside* this structure
  precisely so run formation and merging share their machinery.
- **Polyphase over balanced merging** because tape drives were the scarcest
  resource in the machine room: with T drives, balanced merging musters
  only (T/2)-way merges, while polyphase gets (T−1)-way power and
  partial-pass phases from the same hardware. The Fibonacci distribution is
  not aesthetics; it is the unique fixed point of "merge onto the empty
  tape and leave every other tape mid-reel".
- **Dummy runs** because a scheduling theorem that only works for perfect
  inputs is useless in production; padding with free-to-merge fictions is
  the standard trick (you will meet it again as "virtual" leaves in
  B-trees and phantom elements in networks) for extending an exact pattern
  to arbitrary sizes.

## In the real world

- **Every serious database contains this module.** PostgreSQL's
  `tuplesort.c` switches from quicksort to external merge the moment your
  `ORDER BY`/`CREATE INDEX` overflows `work_mem`: it writes sorted runs to
  "tapes" (its own name for them, straight from Knuth), then k-way-merges
  them — and its `EXPLAIN (ANALYZE)` output will tell you `Sort Method:
  external merge  Disk: …kB`. It used polyphase merge for decades
  (replaced by a simpler k-way scheme in v13, when the "tapes" became
  seekable files and the drive-count constraint evaporated — a beautiful
  case study in *cost models changing, algorithms following*).
- **LSM-trees are replacement selection's descendants.** RocksDB, LevelDB,
  Cassandra and friends absorb writes into an in-memory sorted structure
  (memtable), flush it as a sorted run (SSTable) when full, and then run a
  background *compaction* — a k-way merge of overlapping runs into longer
  ones. Level after level of merges, dummy-run-style accounting for
  tombstones, sequential-only I/O: §5.4 wearing a hoodie.
- **MapReduce/Spark shuffles are distributed external sorts.** Each mapper
  writes sorted spill runs; reducers fetch and k-way merge them; Hadoop's
  configuration literally exposes `io.sort.factor` — the merge order k.
  The shuffle is routinely the dominant cost of a big job, for exactly the
  reason this module counts records moved.
- **Analytic engines spill too.** DuckDB and ClickHouse execute
  larger-than-memory sorts, joins and aggregations by writing runs to disk
  and merging them out-of-core; DuckDB's sort was rewritten around
  cache-friendly run formation plus merge precisely because SSDs kept the
  old asymmetry (sequential ≫ random) even after seeks stopped costing
  10 ms. The devices changed; the accountant's ledger did not.

## Proof techniques you practiced

- **Steady-state / equilibrium analysis** — the snow-plow: instead of
  tracking the algorithm step by step, find the invariant regime (constant
  snow = P, uniform lap) and read the answer off a conservation law. This
  is your first *fluid limit* argument; queueing theory and amortized
  analyses reuse it constantly.
- **Adversarial/extremal inputs** — sorted and reverse-sorted inputs pin
  the two ends of replacement selection's behaviour (one run; runs of
  exactly P), turning "expected 2P" into a bracketed, tested claim.
- **Counting via tree depth** — the `n·⌈lg k⌉ + k` merge bound is a
  path-length argument: charge each output to one root path, bound the
  path by padding to a power of two, and account the build separately.
- **Backward induction / running the machine in reverse** — the
  perfect-distribution theorem is proved by un-merging phases, the same
  reversal trick as Lamé's theorem in Module 01 (build the worst case
  backwards) and CDCL's conflict analysis in Module 14.
- **Executable cost theorems** — `IoStats` turns "polyphase writes at most
  (1 + phases)·n records" from prose into an assertion. Stating resource
  bounds as machine-checkable contracts is the course's habit; here the
  resource is I/O for the first time.

## 8. Where this leads

- **§5.4.2–§5.4.9** go deeper than we did: cascade merge, oscillating
  sorts, optimal dummy placement, read-backward tapes — a zoo of schedules
  for hardware constraints that have mostly (not entirely!) melted away.
- **Module 11 (B-trees)** is the same memory-hierarchy inversion applied to
  *searching*: nodes sized to blocks, height = number of I/Os.
- **Cache-oblivious algorithms** (Frigo–Leiserson et al., after Knuth)
  redo this module's arithmetic with block size B and memory M unknown to
  the algorithm — funnel sort is a loser tree that adapts to every level
  of the hierarchy at once.
- The **2P law** returns whenever a bounded buffer smooths a stream:
  timsort's run detection, LSM memtables, even video-encoder lookahead —
  a P-record window buys you 2P of order.
