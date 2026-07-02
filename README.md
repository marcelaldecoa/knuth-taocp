# The Art of Computer Programming — a hands-on course in Rust

A complete, self-contained course on the essence of Donald E. Knuth's *The Art
of Computer Programming*, from Euclid's algorithm (Vol. 1, §1.1) to a SAT
solver (Vol. 4B, §7.2.2.2). You **implement every algorithm yourself in
Rust**, guided by lessons that carry Knuth's mathematical rigor — definitions,
theorems, proofs, and analyses — so you can complete the whole course **even
if you don't own the books**. If you do own them, every lesson tells you
exactly which sections it teaches.

Grading works like CodeCrafters: each module is a track of small **stages**,
each stage is a test suite, and a single command validates your work, unlocks
the next stage, and remembers your progress.

```text
$ ./grade 1

── Module 01: The Notion of an Algorithm (Vol. 1, §1.1) ──────────
  Stage 1/4 · Euclid's algorithm, step by step
    Algorithm 1.1E
    ✓ passed
  Stage 2/4 · Avoiding trivial replacements
    Algorithm 1.1F (ex. 1.1-3)
    ✗ failed
    ...
    Read: course/module-01-algorithms/README.md — stage 2 walkthrough
    Edit: labs/module-01-algorithms/src/lab.rs — your code
```

## Quick start

You need a Rust toolchain (any recent stable; the course has **zero
dependencies** and never touches the network). Then:

```bash
./grade              # course map + your progress
./grade 1            # start Module 01: read the failure, then...
$EDITOR labs/module-01-algorithms/src/lab.rs    # ...replace todo!()s
./grade 1            # ...until the module is green
```

Read the lesson first: `course/module-01-algorithms/README.md`. Each lesson
teaches the theory the stages need, states every algorithm in Knuth's
step-labeled style (E1, E2, …), traces it by hand, and proves the theorems
that the test suites then make you verify empirically — Lamé's worst case for
Euclid, H_n − 1 expected maxima, the lg(n!) sorting lower bound, AVL's
1.4405 lg n height bound, and so on.

## The curriculum

Twenty-two modules, 94 stages, spanning the published volumes. See
[SYLLABUS.md](SYLLABUS.md) for the full map of stages, algorithms, and the
mathematics each module teaches — and [docs/toolkit.md](docs/toolkit.md) for
what the journey builds in you: the proof techniques and engineering
judgments, module by module.

| Module | Covers | Flagship material |
|---|---|---|
| 01 The Notion of an Algorithm | Vol. 1 §1.1 | Algorithm E; correctness proofs; Lamé's theorem |
| 02 Mathematical Preliminaries | Vol. 1 §1.2 | Sums, binomials, Fibonacci, harmonic numbers; analysis of Algorithm M |
| 03 Information Structures | Vol. 1 Ch. 2 | Stacks/queues, linked arenas, topological sort, tree traversal, threaded trees |
| 04 Random Numbers | Vol. 2 Ch. 3 | LCGs and the full-period theorem; chi-square; shuffling done right |
| 05 Arithmetic | Vol. 2 Ch. 4 | Multiprecision arithmetic, Karatsuba, binary gcd, Miller–Rabin |
| 06 Sorting | Vol. 3 Ch. 5 | Insertion/Shell/quick/heap/merge/radix; inversions; the lg(n!) lower bound |
| 07 Searching | Vol. 3 Ch. 6 | Binary search, BSTs, AVL trees, open-address hashing and its analysis |
| 08 Combinatorial Generation | Vol. 4A §7.2.1 | Gray codes, permutations, plain changes, combinations, partitions |
| 09 Backtracking & Dancing Links | Vol. 4B §7.2.2–.1 | n queens, bitwise backtrack, Algorithm X/DLX, Sudoku |
| 10 Satisfiability | Vol. 4B §7.2.2.2 | DIMACS, unit propagation, a DPLL solver, SAT encodings |
| 11 Multiway Trees & Digital Searching | Vol. 3 §6.2.4, §6.3 | B-trees with the height-bound proof; tries; Patricia |
| 12 The Spectral Test | Vol. 2 §3.3.4 | The lattice hiding inside every LCG; RANDU's 16 planes, computed exactly |
| 13 Bitwise Tricks & BDDs | Vol. 4A §7.1.3–7.1.4 | Broadword computing; reduced ordered BDDs, canonicity, model counting |
| 14 Conflict-Driven Clause Learning | Vol. 4B §7.2.2.2 | Watched literals, implication graphs, first-UIP learning — a real SAT solver |
| 15 External Sorting | Vol. 3 §5.4 | Replacement selection's snow-plow, loser trees, polyphase merge on Fibonacci |
| 16 The Spectral Test in Higher Dimensions | Vol. 2 §3.3.4 | Algorithm S: dual bases, reduction, certified search, ν_t for t ≤ 6 |
| 17 ZDDs & Exact Covering with Colors | Vol. 4A §7.1.4, Vol. 4B §7.2.2.1 | Family algebra, structure counting in graphs, XCC with purify/unpurify |
| 18 MMIX: Knuth's Machine | Vol. 1 Fascicle 1 | Build the machine, then run Euclid and FindMax on it — full circle |
| 19 Floating-Point Arithmetic | Vol. 2 §4.2 | Round-to-even, why addition isn't associative, Kahan summation |
| 20 Optimum Sorting & Networks | Vol. 3 §5.3 | The lg n! floor, Ford–Johnson, Batcher's network, the zero-one principle |
| 21 Boolean Functions & Optimal Evaluation | Vol. 4A §7.1.1–7.1.2 | Truth-tables-as-integers, Boolean chains, Dedekind's monotone counting |
| 22 Hamiltonian Paths & Constraint Satisfaction | toward Vol. 4C §7.2.2.4 | Backtracking, Warnsdorff, hypercube↔Gray codes, Held–Karp — the finale |

The order is Knuth's own recommended path for implementers (it fronts Vol. 1
and Vol. 3's implementation-rich chapters and defers the heaviest
mathematics), and difficulty is gradual within and across modules. Modules
01–10 are the core arc; 11–22 are advanced extensions that deepen each
volume's flagship theme — ending where NP-hardness meets everything you've
learned, and (in Module 18) where the course's first algorithm runs on the
machine Knuth built for it.

## How the repository is laid out

```text
course/module-NN-*/README.md   the lesson: theory + stage-by-stage lab guide
course/module-NN-*/exercises.md your log for Knuth's exercises (with ratings)
labs/module-NN-*/src/lab.rs    YOUR file — stubs with todo!() to replace
labs/module-NN-*/tests/        one test file per stage (read them! they teach)
reference/                     complete solutions — spoilers! see below
grader/                        the ./grade tool
```

### The grader

```bash
./grade                  # progress overview
./grade 3                # grade module 03 stage by stage; stops at first failure
./grade 3 --stage 2      # re-run one stage
./grade 3 -s 2 --hint    # stuck? a graduated hint (add a number for the next)
./grade 3 -v             # full test output
./grade bench 6          # run a module's growth-curve benchmark
./grade doctor           # diagnose your toolchain and workspace
./grade all              # everything
./grade reset            # forget progress
./grade verify           # course self-check (see below)
```

When a stage is green, the pass message points you to that module's
`WALKTHROUGH.md` — a design commentary on the reference implementation, for
the "compare with Knuth's answer" step. Each module also ships graduated
`hints.md` (three levels, gentlest first) that `--hint` surfaces.

Prefer a visual map? Open [`docs/dashboard.html`](docs/dashboard.html) in a
browser: every module and stage grouped by volume, with a personal
click-to-track progress meter (saved in your browser; `./grade` remains the
authoritative record).

Progress lives in `.taocp/progress` (gitignored). You can always bypass the
grader and run `cargo test -p lab-03-structures --test stage_02_linked_list`
directly — the grader is cargo underneath, nothing magic.

### The reference implementations

`reference/` contains complete, documented, step-faithful solutions with
Knuth's step labels, plus unit tests reproducing worked examples from the
text. Two honest uses:

1. **After** a stage is green, compare your solution against the reference —
   that comparison is where a lot of the learning happens (Knuth says the
   same about his exercise answers).
2. `./grade verify` runs every lab test suite *against the reference*
   (labs re-export the reference under `--features solutions`), proving every
   stage is passable exactly as specified. CI for the course itself.

## Method: how to actually study

Per module: **read the lesson** (derive the math by hand — a notebook is not
optional equipment for TAOCP), **implement stage by stage** keeping Knuth's
step labels as comments, **let the tests argue with you**, and then do a few
**exercises** from the lesson's curated table, logging them in the module's
`exercises.md`. Knuth's rating scale: 00 trivial · 20 an hour · 30 hours+ ·
40 term project · 50 open research problem; ▶ marks the instructive ones.

## Source texts (optional but wonderful)

- Vols. 1–3 (3rd/2nd ed.), Vol. 4A (2011), Vol. 4B (2022), Addison-Wesley.
- Knuth's TAOCP page (errata, pre-fascicles toward Vol. 4C, MMIX):
  https://cs.stanford.edu/~knuth/taocp.html
- *Concrete Mathematics* (Graham–Knuth–Patashnik) expands Module 02's §1.2.

MMIX, Knuth's machine, is deliberately out of scope here — this course maps
his MIX-era memory model onto Rust arenas instead (see CONVENTIONS.md).

## Contributing / authoring

The course is data: `grader/src/manifest.rs` defines modules and stages, and
[CONVENTIONS.md](CONVENTIONS.md) defines the contract every module satisfies
(`./grade verify` enforces it). New modules — external sorting, the higher-
dimensional spectral test, ZDDs, XCC, MMIX itself — are welcome along the
same lines.
