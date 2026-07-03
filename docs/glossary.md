# Glossary & conventions

*The one place the course's recurring conventions are defined — Knuth's, and this
repository's. When a lesson uses a rating like `[20]`, a reference like §1.2.3, a
step label like E1, or a term like "arena," this is what it means.*

Jump to: [Exercise ratings](#exercise-ratings-knuths-scale) ·
[Section numbers](#section-numbers) ·
[Algorithms & step labels](#algorithms-and-step-labels) ·
[Mathematical notation](#mathematical-notation) ·
[Course & code terms](#course-and-code-terms) ·
[Grader terms](#grader-terms)

---

## Exercise ratings (Knuth's scale)

Every exercise in TAOCP — and in this course's `exercises.md` logs — carries a
difficulty rating, a number 00–50 on a logarithmic scale:

| Rating | Meaning |
|---|---|
| **00** | Immediate: you know the answer as you read it. |
| **10** | Simple: a minute or so, maybe with pencil. |
| **20** | Moderate: fifteen minutes to an hour. |
| **30** | Hard: several hours. Worth a serious sitting. |
| **40** | Term project: quite difficult, suitable for a class assignment. |
| **50** | Research: an open problem (at the time of writing). |

The tens digit is what matters; the units digit fine-tunes within a band. A
prefix **▶** marks an especially instructive exercise — do these first. A rating
can also carry letters: **M** = it relies on mathematics, **HM** = it needs
higher math (calculus and beyond). So `[M25]` is an hour-ish problem that leans
on math.

Start with the low numbers and the ▶ exercises; that's where the ideas stick.

## Section numbers

References like **§1.2.3** or **Algorithm 4.5.2B** point into TAOCP by *chapter*,
and chapters map to volumes:

| A section starting with… | Chapter | Volume |
|---|---|---|
| §1, §2 | 1–2 | **Vol. 1** |
| §3, §4 | 3–4 | **Vol. 2** |
| §5, §6 | 5–6 | **Vol. 3** |
| §7 | 7 | **Vol. 4** (4A, 4B, 4C…) |

So §4.5.3 lives in Vol. 2 even where a Vol. 1 lesson cites it. Full explanation
with examples: [for-newcomers.md §8](for-newcomers.md#8-reading-taocps-section-numbers).
Each lesson also names its exact home in a **Source** header at the top.

## Algorithms and step labels

Knuth names each algorithm by its section plus a letter — *Algorithm 1.1E* is the
fifth algorithm displayed in §1.1 (Euclid's), *Algorithm 5.2.2Q* is quicksort in
§5.2.2. The algorithm's **steps inherit that letter**: E1, E2, E3 for Algorithm
E; Q1…Q9 for quicksort. Each step has a bracketed summary phrase, an action, and
sometimes a parenthetical *assertion* (a fact true whenever control reaches that
point — the seed of a correctness proof).

**This course asks you to keep those step labels as comments in your code**
("step-faithful first"), so your implementation stays auditable against the book
and the reference solution. See [CONVENTIONS.md](../CONVENTIONS.md).

## Mathematical notation

The compact notations TAOCP (and *Concrete Mathematics*) use, with fuller
explanation in [for-newcomers.md §7](for-newcomers.md#7-reading-knuths-notation-a-small-cheat-sheet):

| Notation | Reads as |
|---|---|
| `[P]` | **Iverson bracket**: 1 if proposition `P` is true, else 0. |
| `⌊x⌋`, `⌈x⌉` | Floor and ceiling. |
| `x mod y` | Remainder; in this course, Rust's `%` on unsigned integers. |
| `H_n` | The **harmonic number** 1 + 1/2 + ⋯ + 1/n. |
| `F_n` | The **Fibonacci number** (F₀=0, F₁=1). |
| `x^{underline n}` | **Falling factorial** x(x−1)⋯(x−n+1) (permutations). |
| `x^{overline n}` | **Rising factorial** x(x+1)⋯(x+n−1). |
| `{n atop k}` | **Stirling subset** number (partitions into k non-empty groups). |
| `[n atop k]` | **Stirling cycle** number (permutations with k cycles). |
| `O(f)`, `Θ(f)`, `Ω(f)` | Asymptotic upper / tight / lower bounds. |
| `lg n`, `ln n` | Base-2 and natural logarithm. |

The companion book for all of this is *Concrete Mathematics* — see
[concrete-mathematics.md](concrete-mathematics.md).

## Course and code terms

| Term | Meaning |
|---|---|
| **Stage** | One graded test suite (`labs/<module>/tests/stage_NN_*.rs`); the smallest unit of progress. |
| **Module** | A track of 4–6 stages covering one part of TAOCP. |
| **Lab** | Your workspace: `labs/<module>/src/lab.rs`, full of `todo!()` stubs to replace. |
| **Reference** | The complete, step-faithful solutions in `reference/`; read *after* passing a stage. |
| **`todo!()`** | Rust's "unimplemented" macro; it panics. Replacing every one is the job. |
| **Arena** | Knuth's linked memory modeled as a `Vec` of cells addressed by `usize` indices, instead of pointers (`Rc`/`RefCell`). More faithful to the MIX memory model, and more idiomatic Rust. |
| **`LAMBDA` (Λ)** | The null link, `usize::MAX` — Knuth's Λ, the "points to nothing" address in an arena. |
| **Limb** | One machine-word "digit" of a big number (radix 2³² or 2⁶⁴) in Module 05. |
| **mem / υ / μ** | MMIX cost units (Module 18): a **mem** is one memory access; υ and μ are Knuth's oops/mems time model. |
| **Flagship module** | One whose analysis predicts a growth curve (sorting, searching, arithmetic, external sorting); it ships a `bench`. |
| **Scaffolding tier** | How much the lab stub hands you: Module 01 full guided tour → 02–04 structure → 05+ contract only. See [for-newcomers.md §5](for-newcomers.md#5-the-training-wheels-come-off--on-purpose). |

## Grader terms

| Command / term | Meaning |
|---|---|
| `./grade` | Progress overview across all modules. |
| `./grade N` | Grade module N stage by stage; stops at the first failure. |
| `./grade next` | Jump to the module with your next unsolved stage. |
| `./grade N -s K --hint` | A graduated hint for stage K (three levels, gentlest first). |
| `./grade bench N` | Run a flagship module's growth-curve benchmark. |
| `./grade verify` | Course self-check: structure, doc links, and every lab suite against the reference. |
| `./grade doctor` | Diagnose your toolchain and workspace. |

Full command reference and Windows notes: [getting-started.md](getting-started.md).
