# Getting started — setup, layout, and every command

*The practical companion to [For newcomers](for-newcomers.md). Everything you
need to go from a fresh clone to a green Module 01, on macOS, Linux, or
Windows.*

---

## 1. Install the toolchain (once)

The course is **zero-dependency Rust** — it never touches the network and needs
nothing but a stable compiler. Any recent stable toolchain works.

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# then restart your shell, or: source "$HOME/.cargo/env"
```

**Windows:** download and run **rustup-init.exe** from
<https://rustup.rs>, accept the defaults (it will install the MSVC build tools
if you don't have them). Then open a fresh terminal.

Verify it worked:
```bash
cargo --version      # should print cargo 1.7x or newer
rustc --version
```

Then, from the repository root, let the course check itself against your setup:
```bash
./grade doctor
```
This diagnoses your toolchain and workspace and tells you if anything is off
before you write a line of code.

## 2. Running `./grade` on each platform

`./grade` is a small **bash** wrapper around `cargo run -p grader`. How you call
it depends on your shell:

| Platform / shell | How to run it |
|---|---|
| macOS / Linux (bash, zsh) | `./grade 1` |
| Windows — **PowerShell** | `.\grade.ps1 1` |
| Windows — **cmd.exe** / double-click | `grade 1` |
| Windows — Git Bash / WSL | `./grade 1` |
| Anywhere, no wrapper | `cargo run -q -p grader -- 1` |

> **Windows note.** The repo ships native wrappers so you don't need Git Bash:
> `grade.ps1` for PowerShell and `grade.cmd` for cmd.exe. Both just forward
> their arguments to the grader, so `.\grade.ps1 6 -s 3 --hint` works exactly
> like `./grade 6 -s 3 --hint` elsewhere. (Git Bash and WSL still run the plain
> `./grade` too.) If PowerShell blocks the script with an execution-policy
> error, either run `cargo run -q -p grader -- 1` directly, or allow local
> scripts for your user once with
> `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`.

The rest of this guide writes `./grade …`; substitute whichever form fits your
shell.

## 3. The daily loop

Four steps, repeated until a module is green:

```bash
./grade                        # 1. see the course map and where you are
./grade 1                      # 2. start Module 01 — read the first failure
$EDITOR labs/module-01-algorithms/src/lab.rs   # 3. replace a todo!()
./grade 1                      # 4. re-grade; repeat from step 3
```

Read the lesson **first**: [`course/module-01-algorithms/README.md`](../course/module-01-algorithms/README.md).
The grader runs a module's stages **in order and stops at the first failure**,
just like a CodeCrafters track — so you always know exactly which one thing to
work on next. When a stage passes it points you at that module's
`WALKTHROUGH.md` for the "compare against the reference" step.

## 4. Every command

```bash
./grade                  # progress overview: all modules and your completed stages
./grade 3                # grade module 03 stage by stage; stops at the first failure
./grade next             # jump straight to the module with your next unsolved stage
./grade 3 --stage 2      # re-run just one stage (short form: -s 2)
./grade 3 -s 2 --hint    # a graduated hint for that stage; add a number for the next
./grade 3 -v             # verbose: full cargo test output for the module
./grade bench 6          # run a module's growth-curve benchmark (flagship modules only)
./grade doctor           # diagnose your toolchain and workspace
./grade all              # grade every module in order
./grade reset            # forget recorded progress and start fresh
./grade verify           # course self-check: run every lab test against the reference
```

A few worth knowing early:

- **`--hint`** surfaces three graduated hints per stage, gentlest first.
  `./grade 3 -s 2 --hint` gives hint 1; `--hint 2`, then `--hint 3`, go deeper.
  Hint 3 shows the key line but never the whole solution.
- **`-v`** shows you the raw test output when a one-line failure isn't enough.
- **`bench N`** exists for the modules whose analysis predicts a growth curve
  (sorting, searching, arithmetic, external sorting). It prints an
  `n | time | ratio` table so you can *see* the asymptotics the lesson derives.
- **`reset`** only clears *your progress record*; it never touches your code.

## 5. How the repository is laid out

```text
README.md                        the overview and command cheat-sheet
SYLLABUS.md                      all 22 modules and 94 stages, in order
CONVENTIONS.md                   the contract every module satisfies (for authors)
docs/for-newcomers.md            ← start here if Knuth is new to you
docs/getting-started.md          ← this file
docs/toolkit.md                  the proof techniques the course builds, module by module
docs/dashboard.html              a visual progress map (open in a browser)

course/module-NN-*/README.md     the LESSON: theory + a stage-by-stage lab guide
course/module-NN-*/hints.md      graduated hints (surfaced by --hint)
course/module-NN-*/WALKTHROUGH.md design commentary on the reference (read AFTER passing)
course/module-NN-*/exercises.md  your log for Knuth's exercises, with his ratings

labs/module-NN-*/src/lab.rs      YOUR file — the stubs with todo!() you replace
labs/module-NN-*/tests/          one test file per stage (read them — they teach)

reference/                       complete reference solutions (spoilers — see §7)
grader/                          the ./grade tool itself
```

The only file you edit to complete a module is its `labs/module-NN-*/src/lab.rs`.
Everything else is there to guide, test, or grade you. (`src/lib.rs` and
`Cargo.toml` inside a lab are generated plumbing — leave them alone.)

**Where to look when you're stuck on a stage**, in order:
1. The **lesson** section for that stage (`course/module-NN-*/README.md`).
2. The **test file** for that stage (`labs/module-NN-*/tests/stage_NN_*.rs`) —
   it shows you the exact inputs and expected outputs.
3. A **hint**: `./grade N -s K --hint`.
4. Only after passing: the **walkthrough** and the **reference solution**.

## 6. The visual dashboard (optional)

Prefer a map to a list? Open [`docs/dashboard.html`](dashboard.html) in any
browser: every module and stage grouped by TAOCP volume, with a click-to-track
progress meter saved in your browser's local storage.

It's a convenience, not the source of truth — **`./grade` remains the
authoritative record** of what you've actually passed. Your real progress lives
in `.taocp/progress` (git-ignored, on your machine only).

## 7. About the reference solutions (mild spoiler warning)

`reference/` holds complete, documented, step-faithful solutions to every stage.
There are two honest ways to use them:

1. **After** a stage is green, read the reference and the module's
   `WALKTHROUGH.md` and compare with what you wrote. Knuth says the same about
   his exercise answers: the comparison is where much of the learning happens.
2. `./grade verify` runs every lab's tests *against* the reference, proving that
   every stage is passable exactly as specified. It's the CI for the course
   itself — handy if you ever suspect a stage is broken rather than just hard.

Reaching for the reference *before* you've struggled is the one way to waste
this course. The struggle is the curriculum. Use the hints first — that's what
they're for.

## 8. Troubleshooting

- **`./grade: command not found` or `permission denied` (macOS/Linux).**
  Run it as `bash grade 1`, or `chmod +x grade` once.
- **`./grade` does nothing useful on Windows PowerShell.** Use the native
  wrapper `.\grade.ps1 1` (or `grade 1` from cmd.exe), or the raw
  `cargo run -q -p grader -- 1`. See §2.
- **`cargo: command not found`.** The toolchain isn't on your PATH — restart
  your terminal after installing, or run `source "$HOME/.cargo/env"`
  (macOS/Linux). On Windows, open a fresh terminal after rustup finishes.
- **First run is slow.** The very first `./grade` compiles the grader and your
  lab crate; later runs are incremental and fast. This is normal.
- **A stage won't pass and you're sure your logic is right.** Run `./grade N -v`
  to see the full test output, then read the stage's test file directly — the
  assertion message names the failing input.
- **Something deeper looks wrong.** `./grade doctor` checks your setup;
  `./grade verify` checks the course. If `verify` fails on a clean clone, that's
  a course bug worth reporting.
- **You can always bypass the wrapper.** The grader is `cargo` underneath,
  nothing magic. To run a single stage's tests directly:
  ```bash
  cargo test -p lab-01-algorithms --test stage_01_euclid
  ```

Now go read [the Module 01 lesson](../course/module-01-algorithms/README.md) and
run `./grade 1`.
