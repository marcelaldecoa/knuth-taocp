# Contributing

Thanks for wanting to help. This course is **data**: modules are defined in a
manifest and every module satisfies the same contract, so contributions slot in
cleanly. Whether you're fixing a typo or adding a whole module, this page is the
front door; [CONVENTIONS.md](CONVENTIONS.md) is the full specification.

## Ways to contribute

- **Report a bug.** A wrong proof, a test that doesn't bite, a broken link, a
  stage that can't be passed as specified, a lesson that leans on something it
  never taught. Open an issue — and note Knuth's tradition, which this course
  keeps in spirit: the reward for the first person to find a real bug is
  **$2.56**, one hexadecimal dollar.
- **Improve a lesson, hint, or walkthrough.** Prose fixes, clearer proofs,
  better traces, an extra worked example. These are the most valuable and
  lowest-risk contributions.
- **Strengthen a test suite.** If a stage's tests would pass a subtly wrong
  implementation, tighten them (see *Test design* in CONVENTIONS.md, and the
  mutation-testing note below).
- **Add a module.** The advanced tier is open-ended — see
  [SYLLABUS.md](SYLLABUS.md)'s "Where to go next" list. This is the biggest
  lift; read the whole contract first.

## Before you start

```bash
./grade doctor      # check your toolchain and workspace
./grade verify      # course self-check: every lab passes against the reference,
                    # and every documentation link resolves
```

Requirements are deliberately minimal: **Rust 2021, std only, no `unsafe`, no
external crates, never touches the network.** The whole workspace has zero
dependencies by design; keep it that way.

## The contract in one screen

Each module `NN` with directory `module-NN-<slug>` ships six things, all
enforced by `./grade verify`:

1. **Lesson** — `course/module-NN-<slug>/README.md`, self-contained (a student
   without the books can complete it). Includes the three mandatory sections:
   *why it's done this way*, *in the real world*, *proof techniques you
   practiced*.
2. **Exercises log** — `course/module-NN-<slug>/exercises.md`.
3. **Reference solution** — `reference/src/mNN_<slug>.rs`, step-faithful (Knuth's
   labels as comments), with unit tests reproducing worked examples from the
   text.
4. **Lab** — `labs/module-NN-<slug>/src/lab.rs`, one stub per public item with a
   rich doc comment and a `todo!()` body; it must compile as-is with no
   warnings. Match the module's **scaffolding tier** (see below).
5. **Hints** — `course/module-NN-<slug>/hints.md`, three graduated hints per
   stage, gentlest first, never the full solution.
6. **Walkthrough** — `course/module-NN-<slug>/WALKTHROUGH.md`, read *after* a
   stage is green.

Stages are registered in `grader/src/manifest.rs` — the single source of truth.
The course website reads `website/src/data/manifest.json`, regenerated from it
with `./grade manifest`; the `manifest-drift` CI check enforces they stay in step.

Full details, including test-design rules and the flagship-bench option, are in
[CONVENTIONS.md](CONVENTIONS.md). **Module 01 is the exemplar — when in doubt,
copy its shape.**

## Scaffolding tiers

The lab's hand-holding tapers by module number (full detail in CONVENTIONS.md,
student-facing version in [docs/for-newcomers.md](docs/for-newcomers.md) §5):

- **Module 01** — full guided tour: algorithm + Rust recipe + doc links per stub.
- **Modules 02–04** — structure and contract; the student reaches for the Rust.
- **Modules 05+** — algorithm and contract only.

Match the tier of the module you're touching; don't add Module-01-style
spoon-feeding to a Module 08 stub, or vice versa.

## Checklist before opening a PR

- [ ] `./grade verify` is green (labs vs. reference **and** no broken doc links).
- [ ] `cargo check --workspace` is clean — no warnings, stubs included.
- [ ] New/changed lessons keep the three mandatory sections and cite TAOCP
      precisely (`Vol. 3, §5.2.3, Algorithm H`).
- [ ] New relative links point at files that exist (the link check will catch
      you, but check first).
- [ ] If you added a stage, update `grader/src/manifest.rs`, then regenerate the
      website map: `./grade manifest > website/src/data/manifest.json`.
- [ ] Prose matches the surrounding voice: precise, book-optional, no reproduced
      book text at length.

## Style

Rust 2021, std only, no `unsafe`, no external crates. Step-faithful first: mirror
Knuth's control flow (a `loop` with the step comments) even where iterators would
be prettier; idiomatic variants may follow as extra functions. Prefer index-based
arenas (`Vec<Node>` + `usize` links) over `Rc<RefCell<…>>` — more faithful to
Knuth's memory model and more idiomatic.

Small PRs merge fastest. Thank you for making the course better.
