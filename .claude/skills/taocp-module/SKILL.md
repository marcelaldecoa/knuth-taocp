---
name: taocp-module
description: How to author or extend a course module in the knuth-taocp repo end-to-end — the ~10 coupled artifacts (lesson, exercises, reference, lab stubs, stage tests, hints, walkthrough) plus the manifest.rs↔manifest.json mirror and the verify/manifest-check gates. Encodes the solutions-feature wiring, the hints.md grammar, the scaffolding tiers, and the rigorous stage-test patterns (oracle cross-checks, property sweeps, exact-count pins, documented should_panic substrings). Use when adding a new module, adding/renaming a stage, or editing a module's lab/tests/reference/lesson/hints/walkthrough. Pairs with the taocp-brand skill for math notation and prose style.
---

# Authoring a TAOCP course module

A module is not one file — it is **~10 tightly-coupled artifacts that must stay
in lockstep**, guarded by `./grade verify` and the `manifest-check` CI drift
job. Miss one and something breaks *silently*: the drift check, the hints
parser, `verify`, or the website course map. This skill is the operational
checklist. The **authoritative spec is [`CONVENTIONS.md`](../../../CONVENTIONS.md)
and [`CONTRIBUTING.md`](../../../CONTRIBUTING.md)** — keep them in sync if you
change a rule here. **Module 01 is the exemplar: when in doubt, copy its shape.**

Use the **taocp-brand** skill for all math/formula notation (KaTeX `$…$`, never
raw Unicode or backtick pseudo-math) and prose voice.

---

## 1. The artifact checklist (all enforced by `./grade verify`)

For a module `NN` with slug, directory `module-NN-<slug>`:

| # | Artifact | Path |
|---|----------|------|
| 1 | **Lesson** (self-contained theory) | `course/module-NN-<slug>/README.md` |
| 2 | **Exercises log** | `course/module-NN-<slug>/exercises.md` |
| 3 | **Reference impl** (step-faithful, `#[cfg(test)]`) | `reference/src/mNN_<slug>.rs` |
| 4 | **Lab stubs** (one `todo!()` per public item) | `labs/module-NN-<slug>/src/lab.rs` |
| 5 | **Stage tests** (one file per stage) | `labs/module-NN-<slug>/tests/stage_KK_<name>.rs` |
| 6 | **Hints** (3 graduated per stage) | `course/module-NN-<slug>/hints.md` |
| 7 | **Walkthrough** (read after green) | `course/module-NN-<slug>/WALKTHROUGH.md` |
| — | **Lab plumbing** (pre-generated, don't hand-edit) | `labs/module-NN-<slug>/{Cargo.toml,src/lib.rs}` |
| — | **Reference export** | one `pub mod mNN_<slug>;` line in `reference/src/lib.rs` |
| — | **Manifest** (single source of truth) | `grader/src/manifest.rs` |
| — | **Website mirror** (regenerated, drift-checked) | `website/src/data/manifest.json` |

Optional for **flagship** modules (sorting, searching, arithmetic, external
sort — any whose analysis predicts a growth curve): `labs/module-NN-<slug>/examples/bench.rs`,
a std-only `fn main()` timing the API at growing `n` (uses `std::time::Instant`
only), so `./grade bench N` shows the asymptotics.

Lessons have **three mandatory sections**: *why it's done this way*, *in the
real world*, *proof techniques you practiced* (the last feeds `docs/toolkit.md`).

---

## 2. How the pieces wire together (the part that bites)

**The `solutions` feature is the spine.** The lab crate compiles the student's
`lab.rs` normally, but under `--features solutions` it re-exports the reference
instead — this is how `verify` proves every stage is passable. The reference
must therefore export **exactly** the same public names and signatures as
`lab.rs`. The plumbing (don't hand-edit) looks like:

```rust
// labs/module-NN-<slug>/src/lib.rs
#[cfg(not(feature = "solutions"))]
mod lab;
#[cfg(not(feature = "solutions"))]
pub use lab::*;

#[cfg(feature = "solutions")]
pub use taocp_reference::mNN_<slug>::*;
```

```toml
# labs/module-NN-<slug>/Cargo.toml
[dependencies]
taocp-reference = { path = "../../reference", optional = true }
[features]
solutions = ["dep:taocp-reference"]
```

**Stage tests** import the lab crate by package name (`use lab_NN_<slug>::*;`)
and their filenames are fixed by the manifest's `test_target`.

**The manifest is the single source of truth.** Add the module to `MODULES` in
`grader/src/manifest.rs`; each `Stage.test_target` must equal a real
`tests/<target>.rs`:

```rust
Module {
    id: "NN", dir: "module-NN-<slug>", lab_crate: "lab-NN-<slug>",
    title: "…", source: "Vol. X, §Y",   // e.g. "Vol. 3, §5.2.3"
    stages: &[
        Stage { test_target: "stage_01_<name>", title: "…", algorithm: "Algorithm 5.2.3H" },
        // 4–6 stages, ordered easy → hard, one idea each
    ],
},
```

Then **regenerate the website mirror** (or `manifest-check` CI fails):

```bash
./grade manifest > website/src/data/manifest.json
```

---

## 3. Scaffolding tier — fixed by module number, not taste

The lab stub's hand-holding tapers as the course advances. Match the tier of the
module you touch; never add Module-01 spoon-feeding to a Module 08 stub, or strip
a net. The safety nets (lesson, 3 hints, reference, walkthrough) are **always**
present at every tier.

- **Module 01** — full guided tour: algorithm in Knuth step form + plain-English
  Rust recipe + exact `std`/Rust-book doc links per stub, plus a header primer.
- **Modules 02–04** — structure + contract + suggested data layout; stop naming
  the specific Rust method. Header names the tier; each stage carries
  `Stuck? ./grade N -s K --hint`.
- **Modules 05+** — algorithm, invariant, and contract only; trust the student to
  translate. By 11–22 the lesson states the theorem and the reference is
  "compare notes," not a crutch.

Keep in sync with the student-facing description in `docs/for-newcomers.md` §5.

---

## 4. Stage-test design — make it BITE (this is a grading system)

The tests ARE the contract. A weak suite that passes a subtly-wrong solution is
the main failure mode. Rules (full list in CONVENTIONS.md "Test design"):

- **Anchor** with worked examples from the text — Knuth's tables/traces become
  `assert_eq!`s with a comment citing the section.
- **Property-test** the *contract Knuth states*, not one implementation: for
  extended Euclid check the Bézout identity (not specific coefficients); for a
  sort check sortedness + permutation + comparison-count bound (not pivot
  choice). Use a deterministic hand-rolled LCG for randomness
  (`x = x*6364136223846793005 + 1442695040888963407`) — **zero external crates**.
- **Cross-check against an independent oracle** — `std::collections::BTreeSet`/
  `HashSet`, a brute-force enumerator, or a second in-test implementation of the
  invariant. This is what makes a false-pass hard.
- **Pin exact counts** where well-defined (permutation histograms, `n−1` merge
  comparisons, `F(n)` for Ford–Johnson, oops/mems). Leave efficiency ceilings
  *loose* where the constant is implementation-dependent (e.g. `< 3 n ln n`) —
  correctness is asserted separately, so a loose ceiling won't false-fail a
  valid variant.
- **`#[should_panic]` must use `expected = "<substring>"`** where the substring
  is NOT contained in a bare `todo!()` panic (else raw stubs would pass). AND —
  **document that exact substring in the stub doc-comment**, e.g.
  `/// the grader checks the panic message contains "positive"`. A student
  reading only the stub must know the required wording, or correct code with
  different words false-fails. (Module 01 is the model; this was a real audit
  finding.)
- Stages ordered easy → hard, one idea each, 4–6 per module.

**Verify the suite bites** with mutation testing (not in fast CI):
```bash
cargo mutants --package taocp-reference --file '*mNN*' -- --workspace --features solutions
```
A surviving mutant = a seeded bug slipped every test; strengthen until it dies.

---

## 5. The hints.md grammar (parsed by the grader — get it exact)

`./grade N --stage K --hint J` reads `course/module-NN-<slug>/hints.md`. Parser:
a `## Stage <k>` header, then lines matching `^<n>.` are the hints in order.
Exactly **3** hints per stage, gentlest first, **never the full solution**:

```markdown
## Stage 1: Euclid's algorithm

1. Which theorem guarantees the remainders strictly decrease? Reach for it.
2. A `loop` that updates `(m, n)` each turn; the exit test is on the remainder.
3. `let r = m % n; if r == 0 { return n } m = n; n = r;` — now add the domain assert.
```

Every stage in the manifest must have a matching hint block (`verify`/manifest
structure checks flag a missing one).

---

## 6. Style invariants (labs & reference)

- Rust 2021, **std only, no `unsafe`, no external crates, never touches the
  network.** The whole workspace has zero dependencies by design.
- **Step-faithful first:** mirror Knuth's control flow (a `loop` with `E1/E2/…`
  step comments) even where iterators would be prettier; idiomatic variants may
  follow as *extra* functions.
- Cite precisely in doc-comments: `TAOCP Vol. 3, §5.2.3, Algorithm H`.
- Prefer index-based arenas (`Vec<Node>` + `usize` links) over
  `Rc<RefCell<…>>` — more faithful to Knuth's memory model and more idiomatic.
- **Reference/labs are exempt from clippy/rustfmt** (their hand-formatted,
  step-faithful style IS the pedagogy) — do NOT run `cargo fmt`/`clippy --fix`
  across them; match the surrounding style. The **grader** is the opposite:
  it's tooling, held to `cargo clippy -p grader -- -D warnings` with unit tests.
- Stubs must compile as-is with **no warnings** — bind unused args with
  `let _ = (a, b);` inside the `todo!()` body.

---

## 7. Definition of done (run before committing / PR)

```bash
./grade doctor                                   # toolchain + workspace sanity
./grade verify                                   # ALL stages pass vs reference + no broken doc links
cargo check --workspace                          # clean, stubs included, no warnings
./grade N                                         # the module fails cleanly on raw stubs, staged
./grade manifest > website/src/data/manifest.json # regenerate mirror if stages changed
cd website && npm run build && npm run check:math # if the lesson changed (KaTeX gate)
```

Checklist:
- [ ] All 7 artifacts present; lesson has the three mandatory sections and cites TAOCP precisely.
- [ ] Reference exports exactly the lab's public names/signatures; `reference/src/lib.rs` has the `pub mod` line.
- [ ] Every `Stage.test_target` maps to a real test file; 4–6 stages, easy→hard.
- [ ] Every `#[should_panic(expected=…)]` substring is documented in the stub.
- [ ] `hints.md` has a `## Stage K` block with 3 graduated hints for every stage.
- [ ] Manifest mirror regenerated (`manifest-check` will fail otherwise).
- [ ] Math/prose follow the **taocp-brand** skill.
