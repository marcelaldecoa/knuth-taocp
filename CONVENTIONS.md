# Authoring conventions

How every module of this course is built. Module 01 is the exemplar — when in
doubt, copy its shape.

## The contract

Each module `NN` with directory `module-NN-<slug>` ships four things:

1. **Lesson** — `course/module-NN-<slug>/README.md`. Self-contained theory:
   a student *without* the books must be able to complete the module from the
   lesson alone. Structure: source pointer header → theory with real
   mathematics (definitions, theorems, proofs or honest proof sketches,
   worked traces of each algorithm) → stage-by-stage lab guide → check-your-
   understanding questions → curated exercises with Knuth's ratings → where
   this leads. Paraphrase; never reproduce the book's text at length.
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
