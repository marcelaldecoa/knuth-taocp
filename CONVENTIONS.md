# Authoring conventions

How every module of this course is built. Module 01 is the exemplar — when in
doubt, copy its shape.

## The contract

Each module `NN` with directory `module-NN-<slug>` ships six things:

1. **Lesson** — `course/module-NN-<slug>/README.md`. Self-contained theory:
   a student *without* the books must be able to complete the module from the
   lesson alone. Structure: source pointer header → theory with real
   mathematics (definitions, theorems, proofs or honest proof sketches,
   worked traces of each algorithm) → stage-by-stage lab guide → check-your-
   understanding questions → curated exercises with Knuth's ratings → where
   this leads. Paraphrase; never reproduce the book's text at length.

   Three sections are mandatory in every lesson, because the course promises
   practicality as well as rigor:
   - **Why it's done this way** — the design rationale (may be woven through
     the theory instead of a single section, but it must be *explicit*).
   - **In the real world** — where this material lives in production systems
     today; concrete and accurate, no hand-waving.
   - **Proof techniques you practiced** — a recap naming each technique and
     the place it carried weight; these feed the course-wide map in
     `docs/toolkit.md`.
2. **Exercises log** — `course/module-NN-<slug>/exercises.md` (template as in
   module 01).
3. **Reference implementation** — `reference/src/mNN_<slug>.rs`. Complete,
   documented, step-faithful (Knuth's step labels E1, E2, … as comments),
   with `#[cfg(test)]` unit tests reproducing worked examples from the text.
4. **Lab** — `labs/module-NN-<slug>/`:
   - `src/lab.rs`: the student workspace. One stub per public item, rich doc
     comments quoting the algorithm's steps in Knuth style, body
     `todo!("...")`. Must compile as-is.
   - `tests/stage_NN_<name>.rs`: one file per stage, names fixed by
     `grader/src/manifest.rs`. Tests import the lab crate by its package
     name (e.g. `use lab_06_sorting::*;`).
   - `src/lib.rs` and `Cargo.toml` are pre-generated plumbing — don't touch.
5. **Hints** — `course/module-NN-<slug>/hints.md`. Graduated hints the grader
   surfaces via `./grade N --stage K --hint J`. One `## Stage K: <title>`
   heading per stage, then a numbered list of 3 hints in increasing
   specificity: (1) a conceptual nudge / which theorem to reach for,
   (2) the approach or data structure, (3) concrete pseudocode or the key
   line — never the full solution. Parser: `## Stage <k>` headers, then
   lines matching `^<n>.` are the hints in order.
6. **Walkthrough** — `course/module-NN-<slug>/WALKTHROUGH.md`. Read *after* a
   stage is green: a design commentary on the reference implementation —
   why it's shaped that way, the invariant that makes it correct, the
   idioms worth stealing, and how it differs from a naive version. Prose,
   one short section per stage. This is the "compare with Knuth's answer"
   step made explicit; it must not be needed to pass, only to deepen.

Optionally, flagship modules (those whose analysis predicts a growth curve —
sorting, searching, arithmetic, external sorting) add
`labs/module-NN-<slug>/examples/bench.rs`: a std-only `fn main()` that times
the public API at growing n and prints an `n | time | ratio` table, so
`./grade bench N` can show the asymptotics the lesson derives. Use
`std::time::Instant` only; no external crates.

## The invariants (checked by `./grade verify`)

- `cargo check -p <lab-crate>` passes with the raw stubs (no warnings about
  unused args: bind them with `let _ = ...;` inside the stub).
- `cargo test -p <lab-crate> --features solutions` is green: the reference
  implementation must export **exactly** the same public names and signatures
  as `src/lab.rs`, because the lab crate re-exports the reference under that
  feature.
- `cargo test -p taocp-reference mNN` is green.
- Stage tests run in ≲ 30 seconds each on a laptop (profiles already set
  `opt-level = 1` for tests).

Mutation testing (do the suites actually *bite*?) is not part of the fast CI
gate — it's slow. Run it on demand from the Actions tab (the "mutation
testing" workflow), or locally against a single module while developing:

```bash
cargo install cargo-mutants
cargo mutants --package taocp-reference --file '*m06*' -- --workspace --features solutions
```

A surviving mutant means a seeded bug in that reference module slipped past
every stage test — strengthen the test until it dies.

## Test design (this is a grading system, make it bite)

- Anchor with **worked examples from the text** — Knuth's tables and traces
  become `assert_eq!`s with a comment citing the section.
- Add **property tests** with deterministic loops or a small hand-rolled LCG
  (`x = x*6364136223846793005 + 1442695040888963407` works fine) — no
  external crates. The workspace has **zero dependencies** by design.
- Test the *contract Knuth states*, not one implementation: e.g. for
  extended Euclid check the Bézout identity, not specific coefficients; for
  quicksort check sortedness + permutation + comparison counts, not pivot
  choices.
- `#[should_panic]` tests must always use `expected = "..."` with a substring
  that a bare `todo!()` panic does **not** contain (otherwise stubs pass).
- Stages are ordered easy → hard and each teaches one idea. 4–6 stages per
  module, fixed in `grader/src/manifest.rs`.

## Style

- Rust 2021, std only, no unsafe, no external crates.
- Step-faithful first: mirror Knuth's control flow (a `loop` with the step
  comments) even where iterators would be prettier; idiomatic variants may
  follow as additional functions.
- Cite precisely: `TAOCP Vol. 3, §5.2.3, Algorithm H` in doc comments.
- Knuth's memory model (MIX-era links and arrays) maps naturally onto
  index-based arenas (`Vec<Node>` + `usize` links); prefer that over
  `Rc<RefCell<...>>` — it is both more faithful and more idiomatic.
