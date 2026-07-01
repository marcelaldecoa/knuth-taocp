# Syllabus

Ten modules, 46 stages. Each stage is one graded test suite
(`labs/<module>/tests/stage_NN_*.rs`); `./grade <n>` runs them in order.
Theory a stage needs is in its module's lesson, `course/<module>/README.md` —
self-contained, book optional.

Estimated pacing: a module every 1–2 weeks alongside a job. The mathematics
deepens gradually; nothing beyond comfortable undergraduate discrete math is
assumed, and the lessons build the rest (induction → sums → asymptotics →
probabilistic analysis → combinatorics).

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

## Where to go next (not yet modules — contributions welcome)

B-trees and tries (§6.2.4, §6.3) · external sorting (§5.4) · the spectral
test (§3.3.4) · bitwise tricks and BDDs (§7.1.3–7.1.4) · XCC with colors ·
CDCL (Algorithm 7.2.2.2C) · MMIX itself (Vol. 1, Fascicle 1).
