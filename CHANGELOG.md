# Changelog

All notable changes to this course — modules, grader, website, and tooling.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project is a living course, not a versioned library: there are no release
tags yet, so everything lives under **Unreleased** until that changes.

## [Unreleased]

### Added
- **Module 23 — Constraint Satisfaction** (Vol. 4 Fascicle 7, §7.2.2.3): the
  CSP model with queens/coloring instances and basic backtracking, forward
  checking + MRV, arc consistency (AC-3), and the direct CSP→SAT encoding.
  Four stages; the course is now 23 modules / 98 stages. Module 22 retitled
  "Hamiltonian Paths and Cycles" (CSP proper now lives in 23).
- `./grade verify --stubs` — course-CI gate proving every stage FAILS on the
  pristine stubs (no vacuous suites); wired into the ubuntu CI leg.
- GitHub Codespaces / Dev Containers setup (`.devcontainer/`): Rust
  pre-installed, grader pre-built, safety hook wired — first command is
  `./grade`.
- Website progress bridge: paste your `.taocp/progress` on the course map and
  your grader-recorded stages light up (client-side only, stored in
  localStorage, never leaves the browser).
- CI runs the knuth-navigator map self-test (`navigator_selftest.py --quiet`).
- `./grade watch <module>` — re-grades a module on every save to its `lab.rs`
  (std-only mtime poll; Ctrl-C to stop).
- `rust-toolchain.toml` pinning the stable channel (with `clippy` + `rustfmt`),
  so learners and CI build identically.
- Skills for working in this repo: `taocp-brand` (identity + math-notation
  standard), `taocp-module` (author/extend a module end-to-end), and
  `taocp-grader-review` (validate the grader for accuracy/precision/rigor).
- Regression test: XCC user colour `0` is a real colour, not the "uncoloured"
  sentinel (module 17).

### Changed
- Stub doc-comments now name the exact `#[should_panic]` substring the grader
  checks (modules 04, 08, 10, 19, 21), so a correct panic with different wording
  no longer false-fails a student who reads only the stub.
- Grader failure hint calls the lesson README a "lesson", not a "walkthrough"
  (the reference/`WALKTHROUGH.md` pointers stay gated behind passing a stage).
- Course website: replaced the light/dark toggle with six parchment-based TAOCP
  cover-accent themes (per-volume inks), and fixed navbar/hamburger/mobile-
  sidebar/footer contrast and theme-following of the progress meter.

### Milestones (already in `main`)
- **22 modules · 94 stages** spanning TAOCP Vols. 1–4B and pre-fascicles toward
  4C, each with a self-contained lesson, step-faithful reference, staged
  `todo!()` lab, rigorous stage tests, graduated hints, and a walkthrough.
- **Grader** (`./grade`): staged grading, `verify`, `hint`, `bench`, `doctor`,
  `manifest`; progress tracked locally; CI self-check and manifest-drift guard.
- **Website** (Docusaurus): course map, handbook, KaTeX math, and a Museum of
  Algorithms of interactive single-file exhibits.
- Graduate-level math review and a full 22/22 student-emulation audit of the
  grader (all stages solvable from the lessons alone; zero false pass/fail).
