---
name: taocp-grader-review
description: How to validate the knuth-taocp ./grade system for accuracy, precision, and rigor — the reusable QA harness. Covers the ground-truth self-check, injected-bug precision/rigor probes (does the grader reject wrong and overfit solutions?), and fanning out parallel "student" subagents in isolated git worktrees to solve modules from the lessons alone, then resetting. Use when asked to audit/validate the grader, prove a stage is passable/unpassable, check whether a test suite bites, or confirm the course is solvable without the reference.
---

# Validating the TAOCP grader

The `./grade` system is a CodeCrafters-style staged grader. "Validating" it means
proving three properties, in order of increasing effort. Do them yourself for
Phases 1–2; fan out subagents for Phase 3.

- **Accuracy** — every stage is passable; the grader greens correct work.
- **Precision** — it fails incorrect/incomplete work, staged, with a non-zero exit.
- **Rigor** — it rejects *plausible-but-wrong* work: contract violations and
  overfit "teach-to-the-test" cheats.

Background on how grading works: `run_stage` in `grader/src/main.rs` runs
`cargo test -p <lab_crate> --test <target>` (plus `--features solutions` to swap
in the reference); modules grade stages **in order, stopping at the first
failure**; progress lives in gitignored `.taocp/progress`.

---

## Phase 1 — Ground truth & precision (do directly)

```bash
rm -rf .taocp                       # reset stale progress first
./grade verify                      # ACCURACY: all 94 stages pass vs reference (~40s)
./grade N >/dev/null 2>&1; echo $?  # PRECISION: on raw stubs → non-zero exit (CI-usable)
./grade N                           # PRECISION: fails at stage 1, staged, "later stages not run"
```

`verify` uses the `solutions` feature (reference), doesn't touch student progress,
and reporting "every stage passes" is the accuracy gate. On raw `todo!()` stubs
the grader must fail and exit non-zero (mind shell pipes: `$?` after `| tail` is
tail's exit — test the exit code without a pipe).

---

## Phase 2 — Rigor via injected bugs (do directly, then restore)

Prove the tests *bite* by planting plausible-but-wrong solutions in a stub and
confirming the grader catches them. Always `git checkout -- <lab.rs>` after.

Two archetypes that must both be caught (Module 01 `euclid_e` example):

1. **Contract violation** — correct math, missing Knuth's domain contract
   (e.g. a correct gcd with no positive-integer `assert!`). Must fail the
   definiteness / `#[should_panic]` tests.
2. **Overfit cheat** — hardcodes the named asserted examples, returns a wrong
   default otherwise. Must fail the exhaustive property sweep (e.g.
   `gcd(2,4)=1 but 2 also divides both`).

```bash
# inject a wrong body into the stub (python/edit), then:
./grade N 2>&1 | grep -iE 'FAILED|✗|panic|but .* also'
git checkout -- labs/module-NN-<slug>/src/lab.rs   # ALWAYS restore
```

If either archetype slips through green, the suite is too weak — that's a real
finding: strengthen the stage test (see the taocp-module skill, "make it BITE").
Complement with mutation testing:
`cargo mutants --package taocp-reference --file '*mNN*' -- --workspace --features solutions`.

---

## Phase 3 — Student emulation (fan out subagents in worktrees)

The end-to-end test: can a diligent student solve each module **from the lessons
alone** (no reference/walkthrough), and does the grader behave well? Spawn one
subagent per module with `isolation: "worktree"` so they can't collide and the
**main tree stays pristine — that isolation IS the "reset."**

Each agent's prompt must enforce a fair test:
- **Only edit** `labs/module-NN-<slug>/src/lab.rs`.
- **Never read** anything under `reference/` or `course/module-NN-<slug>/WALKTHROUGH.md`
  (both are answer keys). MAY read the lesson `README.md`, the stub, the stage
  test files, and `./grade N --stage K --hint`.
- Implement all stages, run `./grade N` to green (timebox hard modules: genuine
  effort, then report friction with exact grader output rather than grinding).
- **Report structured:** stages passed/attempts; grader-quality assessment
  (were failure messages accurate? hints useful/non-spoiling? lesson sufficient?
  any FALSE PASS / FALSE FAIL / flakiness?); and an **integrity proof** —
  `git diff --stat` (only that one `lab.rs` changed) + an attestation of no
  peeking.

Sampling: the foundational/mid modules give the cleanest signal on *grader*
quality; the hardest (CDCL, ZDD/XCC, MMIX, Hamiltonian) mostly measure problem
difficulty — cover them too for completeness, but read "stuck" there as
difficulty, not necessarily a grader gap. `verify` already proves all are
passable against the reference. If you sample rather than cover all 22, **say so**
— no silent caps.

### Reset (mandatory cleanup)

Worktree agents leave locked worktrees behind (they changed files). Remove them:

```bash
for wt in $(git worktree list --porcelain | grep '^worktree' | awk '{print $2}' | grep 'worktrees/agent-'); do
  git worktree remove --force "$wt"; done
for br in $(git branch --list 'worktree-agent-*' | tr -d ' *'); do git branch -D "$br"; done
git worktree prune
rm -rf .taocp                 # clear progress written by your own Phase-1/2 runs
git status --short            # must be empty — main tree pristine
```

---

## Reporting

Give a verdict on **accuracy / precision / rigor** with the evidence (verify
result; injected-bug outcomes; the per-module pass table + integrity-clean
count). Separate genuine defects from by-design choices and cosmetic nits, and
recommend fixes for real findings (e.g. undocumented `should_panic` substrings,
a suite that didn't bite, a test blind spot). Fixes to the labs/tests follow the
**taocp-module** skill.
