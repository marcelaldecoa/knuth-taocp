<!--
Thanks for contributing! Small PRs merge fastest.
Full contract: CONTRIBUTING.md and CONVENTIONS.md.
-->

## What this changes

<!-- One or two sentences. If it fixes an issue, add "Closes #123". -->

## Checklist

- [ ] `./grade verify` is green (labs vs. reference, structure, **and** no broken doc links).
- [ ] `cargo check --workspace` is clean — no warnings, stubs included.
- [ ] If I touched the grader: `cargo clippy -p grader --all-targets -- -D warnings` and `cargo test -p grader` pass.
- [ ] New/changed lessons stay self-contained and keep the three mandatory sections; citations are precise (e.g. `Vol. 3, §5.2.3, Algorithm H`).
- [ ] If I added a stage: updated `grader/src/manifest.rs`, then regenerated the website map with `./grade manifest > website/src/data/manifest.json`.
- [ ] I matched the module's scaffolding tier (01 full tour / 02–04 structure / 05+ contract) and the surrounding hand-formatted style (the reference is intentionally not rustfmt/clippy-driven — see CONVENTIONS.md).
