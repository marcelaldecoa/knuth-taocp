# Syllabus

Twenty-three modules, 98 stages. Each stage is one graded test suite
(`labs/<module>/tests/stage_NN_*.rs`); `./grade <n>` runs them in order.
Theory a stage needs is in its module's lesson, `course/<module>/README.md` —
self-contained, book optional.

> **New to Knuth?** Read [docs/for-newcomers.md](docs/for-newcomers.md) first
> (a book-optional primer + how to study), then
> [docs/getting-started.md](docs/getting-started.md) for setup and commands.
> [docs/why-knuth-matters.md](docs/why-knuth-matters.md) tours where these
> algorithms live in modern technology, and
> [docs/concrete-mathematics.md](docs/concrete-mathematics.md) is the companion
> book for the mathematics (Module 02's §1.2, grown into a full course).

The mathematics deepens gradually; nothing beyond comfortable undergraduate
discrete math is assumed, and the lessons build the rest (induction → sums →
asymptotics → probabilistic analysis → combinatorics).

## Pacing and difficulty

Rough effort alongside a job — a planning aid, not a stopwatch. Difficulty
climbs gently inside the core arc and steps up in the advanced tier. The amount
of hand-holding in the lab *tapers* as you go (see
[docs/for-newcomers.md §5](docs/for-newcomers.md)): Module 01 is a fully guided
tour, Modules 02–04 give the algorithm and let you reach for the Rust yourself,
and from Module 05 on you get the algorithm and the contract.

| Tier | Modules | Character | Per module |
|---|---|---|---|
| Foundations | 01–03 | Definitions, the math toolkit, Knuth's memory model. A gentle ramp. | ~1 week |
| Core methods | 04–10 | Randomness, arithmetic, sorting, searching, generation, backtracking, SAT. The heart of the course. | 1–2 weeks |
| Advanced tier | 11–23 | Each revisits a volume and pushes its flagship idea further; take any one after its prerequisite core module. Steeper, and a few (12, 14, 16) are genuinely hard. | 1–2 weeks, some more |

Within a module the stages are ordered easy → hard, so partial progress is
always real progress. `./grade` remembers exactly where you are.

## Module 01 — The Notion of an Algorithm (Vol. 1, §1.1)

What an algorithm *is* (finiteness, definiteness, effectiveness), Knuth's
step-labeled style, correctness by invariant + decreasing quantity.

| Stage | You implement | Theory |
|---|---|---|
| 1 | `euclid_e` — Algorithm 1.1E | gcd lemma, termination proof |
| 2 | `euclid_f` — Algorithm 1.1F | same function ≠ same algorithm |
| 3 | `extended_euclid` — Algorithm 1.2.1E | Bézout certificates, modular inverses |
| 4 | `division_steps` + experiments | Lamé 1845: Fibonacci worst case |

## Module 02 — Mathematical Preliminaries (Vol. 1, §1.2)

The analyst's toolkit: induction, sums, binomial coefficients, Fibonacci,
harmonic numbers, O-notation — capped by Knuth's model analysis of finding
the maximum (E[A] = H_n − 1).

Stages: closed-form sums · binomials · fast Fibonacci · harmonic numbers
(exact rationals + asymptotics) · Algorithm 1.2.10M with counting.

## Module 03 — Information Structures (Vol. 1, Ch. 2)

Knuth's memory model in Rust: arenas, links, AVAIL lists. Sequential vs
linked trade-offs; Catalan numbers.

Stages: array stack/queue with overflow discipline · linked arena ·
topological sort (Algorithm 2.2.3T) · binary tree traversal (2.3.1T) ·
threaded trees (2.3.1S).

## Module 04 — Random Numbers (Vol. 2, Ch. 3)

What "random" can mean, and how to be honestly pseudo-random.

Stages: LCGs + full-period theorem 3.2.1.2A (+ RANDU's fall) · chi-square ·
Fisher–Yates (Algorithm 3.4.2P, plus *why the naive shuffle is biased*) ·
reservoir sampling (3.4.2R).

## Module 05 — Arithmetic (Vol. 2, Ch. 4)

Numbers bigger than the machine: classical algorithms A/S/M on 2³²-limbs,
Karatsuba's divide-and-conquer, binary gcd, and probabilistic primality —
Knuth's case that randomized algorithms are respectable mathematics.

Stages: bignum add/sub · classical multiply (50! exactly) · Karatsuba ·
binary gcd (4.5.2B) · Miller–Rabin (deterministic for u64).

## Module 06 — Sorting (Vol. 3, Ch. 5)

The most implementation-rich chapter in TAOCP. Inversions as the measure of
disorder; average-case analyses derived, then verified by your counters; the
information-theoretic lg(n!) lower bound — and how radix sort sidesteps it.

Stages: straight insertion + inversions (5.2.1S) · shellsort (5.2.1D) ·
quicksort with explicit stack + cutoff (5.2.2Q) · heapsort (5.2.3H) ·
natural merge (5.2.4N) · LSD radix (5.2.5R).

## Module 07 — Searching (Vol. 3, Ch. 6)

Stages: binary search with the ⌊lg n⌋+1 bound (6.2.1B) · BSTs incl. Hibbard
deletion (6.2.2T/D) · AVL trees with the 1.4405 lg n height proof (6.2.3A) ·
open addressing: linear probing vs double hashing, load-factor analysis
(6.4L/D) — the 1962 calculation that founded analysis of algorithms.

## Module 08 — Combinatorial Generation (Vol. 4A, §7.2.1)

Visiting every object of a combinatorial family, in the right order, at
amortized constant cost.

Stages: Gray codes (7.2.1.1G) · lexicographic permutations, multisets
included (7.2.1.2L) · plain changes (7.2.1.2P) · combinations (7.2.1.3T) ·
integer partitions + conjugation + p(n) (7.2.1.4P).

## Module 09 — Backtracking and Dancing Links (Vol. 4B, §7.2.2–7.2.2.1)

The backtrack paradigm made precise, then made fast, then made beautiful.

Stages: n queens by Algorithm 7.2.2B · Walker's bitwise queens (n=14) ·
Algorithm 7.2.2.1X — exact cover with dancing links, MRV heuristic · Sudoku
as a 324-item exact cover.

## Module 10 — Satisfiability (Vol. 4B, §7.2.2.2)

The queen of combinatorial problems; your course capstone is a working SAT
solver and the art of encoding problems into it.

Stages: CNF + DIMACS · unit propagation · DPLL (Algorithm D's lineage),
van der Waerden W(3,3)=9 as the running example · encodings: exactly-one,
queens, graph coloring (Petersen graph).

---

Modules 11–23 are the advanced tier: each returns to a volume you already
know and takes its flagship idea further. Do them in any order after their
prerequisites (11 after 07, 12 after 04, 13 after 08, 14 after 10, 15 after
06, 16 after 12 and 05, 17 after 13 and 09, 18 after 01 and 03, 19 after 05,
20 after 06, 21 after 13, 22 after 09 and 13, 23 after 09 and 10).

## Module 11 — Multiway Trees and Digital Searching (Vol. 3, §6.2.4 & §6.3)

The disk changes the rules: one node = one page, so fan out. Then stop
comparing keys and start reading their bits.

Stages: B-tree search/insert with node splitting · invariants + the
log_⌈m/2⌉ height bound · binary tries · Patricia (path compression).

## Module 12 — The Spectral Test (Vol. 2, §3.3.4)

The most demanding mathematics in the course, and Knuth's favorite test of a
random-number generator: every LCG's t-tuples form a lattice; measure the
gap between its hyperplanes, exactly.

Stages: the lattice theorem, verified on RANDU · exact 2-D test by
Gauss–Lagrange reduction · certified short-vector search in 3-D (RANDU's
ν₃² = 118) · figures of merit for real generators.

## Module 13 — Bitwise Tricks and Binary Decision Diagrams (Vol. 4A, §7.1.3–7.1.4)

Broadword computing — the word as a 64-lane vector — then the data structure
that made hardware verification possible.

Stages: ruler function, SWAR popcount, Gosper's hack · reduced ordered BDDs
with hash-consing (canonicity theorem) · model counting + the variable-
ordering experiment · applications: independent sets are Fibonacci, queens
by BDD.

## Module 14 — Conflict-Driven Clause Learning (Vol. 4B, §7.2.2.2, Algorithm C)

The capstone: what modern SAT solvers add to Module 10's DPLL, built piece
by piece.

Stages: two watched literals (backtracking becomes free) · the trail:
decisions, levels, reasons · first-UIP conflict analysis (learn a clause
from every failure) · the complete CDCL solver, cross-checked against brute
force and pitted against pigeonhole and van der Waerden instances.

## Module 15 — External Sorting (Vol. 3, §5.4)

When data doesn't fit: the memory hierarchy as the real cost model.

Stages: replacement selection and the snow-plow 2P law (5.4.1R) · k-way
merging with a loser tree · polyphase merge and its Fibonacci distributions ·
the full pipeline with I/O accounting.

## Module 16 — The Spectral Test in Higher Dimensions (Vol. 2, §3.3.4, Algorithm S)

Module 12 extended to t ≤ 6, following Algorithm S's architecture:
reduce, then enumerate with a certificate.

Stages: the U/V dual-basis pair and its m·I invariant · size reduction by
unimodular transformations · the certified exhaustive search · ν_t and μ_t
for real generators (RANDU condemned in every dimension; 48271 vindicated).

## Module 17 — ZDDs and Exact Covering with Colors (Vol. 4A §7.1.4, Vol. 4B §7.2.2.1)

Sparse families of sets as first-class data, then Dancing Links' final form.

Stages: zero-suppressed DDs and their canonical form · the family algebra
(union, intersection, join) · counting matchings and independent sets in
graphs (Fibonacci, Lucas, and telephone numbers fall out) · XCC: exact
cover with colors, with Latin squares as the star witness.

## Module 18 — MMIX: Knuth's Machine (Vol. 1, Fascicle 1)

The closer. Build an MMIX subset — registers, big-endian memory, real
opcodes, floor division — plus a two-pass assembler, then run the course's
first algorithm on it and count the mems.

Stages: machine state, loads and stores · arithmetic with MMIX's exact
semantics · branches and loops · Euclid and FindMax on the metal, with
Knuth's υ/μ cost model.

## Module 19 — Floating-Point Arithmetic (Vol. 2, §4.2)

Numbers that only approximate — and how to bound the lie.

Stages: representation and normalization · Algorithm 4.2.1A addition with
round-to-even · multiplication and division · error analysis: the unit
roundoff, why `+` isn't associative, and Kahan compensated summation.

## Module 20 — Optimum Sorting and Sorting Networks (Vol. 3, §5.3)

How few comparisons can possibly suffice — and how to sort without looking
at the data.

Stages: the ⌈lg n!⌉ decision-tree bound (and the S(12)=30 gap) · merge
insertion (Ford–Johnson), comparison-optimal for small n · Batcher's
odd-even merge network · the zero-one principle as a 2ⁿ-not-n! correctness
certificate.

## Module 21 — Boolean Functions and Optimal Evaluation (Vol. 4A, §7.1.1–7.1.2)

The truth table as an integer, and the search for the cheapest circuit.

Stages: truth tables and normal forms · Boolean chains and combinational
cost · median, threshold, and symmetric functions (with Dedekind's monotone
counting) · optimum chains for small functions.

## Module 22 — Hamiltonian Paths and Cycles (toward Vol. 4C, §7.2.2.4)

Where NP-hardness meets everything you've built.

Stages: Hamiltonian paths by backtracking (Petersen has none) · Warnsdorff's
knight's tour, and where the heuristic blinks · hypercube cycles *are* Gray
codes (Module 08 returns) · Held–Karp shortest path by bitmask DP (Module 13
returns) — still the best known for general TSP.

## Module 23 — Constraint Satisfaction (Vol. 4 Fascicle 7, §7.2.2.3)

The finale, on the newest material Knuth has published: the draft
pre-fascicle where backtracking (Module 09), SAT (Module 10), and ordering
heuristics reunite under one model. Prerequisites: modules 09 and 10.

Stages: the CSP model — queens and coloring as instances — with basic
backtracking (Algorithm 7.2.2B) · forward checking with save-and-restore
(the honest cousin of dancing cells) plus the MRV ordering heuristic ·
arc consistency to the unique AC-3 fixpoint · the direct CSP→SAT encoding,
its model-for-solution bijection certified by truth-table counting.

## Where to go next (not yet modules — contributions welcome)

Retrieval on secondary keys (§6.5) · BDD/ZDD variable reordering · optimum
sorting networks beyond Batcher (the AKS network) · Toom–Cook and FFT
multiplication (§4.3.3) · the spectral test in dimensions 7–8 · dancing
cells as a real sparse-set engine (Fascicle 7's implementation track) · and
Vol. 4C itself, as its pre-fascicles firm up.
