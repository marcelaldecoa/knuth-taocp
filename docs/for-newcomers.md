# New to Knuth? Start here.

*A gentle, book-optional on-ramp to* The Art of Computer Programming *— what it
is, why it has a reputation, and how to walk into it without bouncing off.*

If you have never opened one of Knuth's volumes — or you opened one, saw a page
of summation signs and a step labelled **E3**, and quietly closed it again —
this page is for you. Read it once before Module 01. You do **not** need to own
the books, and you do **not** need to be a mathematician. You need curiosity, a
notebook, and a willingness to trace an algorithm by hand before you trust it.

> Already comfortable with who Knuth is and just want to run the thing? Skip to
> **[Getting started](getting-started.md)** for setup, repo layout, and every
> command.

---

## 1. Who is Knuth, and what is TAOCP?

Donald E. Knuth (born 1938) is a computer scientist at Stanford who, in 1962,
set out to write a book about compilers and instead started writing *The Art of
Computer Programming* (TAOCP) — a survey of the fundamental algorithms of
computing, done properly. "Done properly" turned out to mean: state each method
precisely, prove it correct, and analyze *exactly* how long it takes, with real
mathematics. He is still writing it. Volume 1 appeared in 1968; Volume 4B in
2022; Volume 4C is in progress.

Along the way he invented the typesetting system TeX (because the second
edition looked ugly), coined the term "analysis of algorithms" as a discipline,
won the Turing Award (1974), and became famous for paying a small reward for
every error found in his books. TAOCP is the reference the field measures
itself against — the book Bill Gates said to send him a résumé if you'd read
cover to cover.

**Why it is hard, honestly.** Three reasons, none of them "you're not smart
enough":

1. **It is a reference, not a tutorial.** It's dense on purpose — every
   sentence carries weight, so you read a paragraph in ten minutes, not ten
   seconds. That is normal and expected.
2. **The mathematics is not optional.** Knuth doesn't just tell you quicksort
   is fast; he derives *how* fast, with sums and recurrences. This is the
   point, not a detour — but it means a page can hide an hour of work.
3. **The examples are in MMIX, a fictional assembly language.** Knuth wants
   costs you can count exactly, so he uses his own machine. Most readers today
   don't want to learn assembly first.

**This course removes reason 3 and softens reasons 1 and 2.** You implement
every algorithm in **Rust**, a modern language. The lessons carry the
mathematics but derive it at a walking pace, and the tests turn each theorem
into something you can *watch* come true on your own machine.

Want proof this is worth the climb before you start? **[Why Knuth still runs the
world](why-knuth-matters.md)** tours the landmark algorithms — from the string
search inside every genome aligner to the SAT solver behind your package
manager — and points to the module where you build each one.

## 2. What this course actually is

A hands-on re-telling of TAOCP where **you write the code**. It is structured
like [CodeCrafters](https://codecrafters.io): 23 modules, each a track of small
**stages**, each stage a test suite. You replace a `todo!()` with an
implementation, run one command, and it tells you pass/fail and what to fix.

```
read the lesson  →  replace a todo!()  →  ./grade  →  green?  →  next stage
```

Every module ships a self-contained **lesson** (the theory a book would give
you), a **lab** (your workspace, full of stubs to fill in), a **reference
solution** (to compare against *after* you pass — that comparison is where a lot
of the learning is), graduated **hints**, and a **walkthrough**. You can do the
entire course without ever touching a physical book; if you own the volumes,
each lesson tells you exactly which section it teaches.

You will start at Euclid's algorithm (300 BC, four lines) and finish at a
working SAT solver and Hamiltonian-path search (the frontier of what's
efficiently computable). That arc is the whole point.

## 3. What you need to know first (prerequisites)

**Mathematics.** Comfortable undergraduate discrete math is plenty:
- You can follow a proof by induction ("true for 1; if true for k then true for
  k+1; therefore true for all").
- You've seen summation notation (Σ) and aren't scared of it.
- You know what "log n" means and roughly why binary search is log-ish.

Everything past that — asymptotics (big-O), recurrences, probability of
average-case behaviour, the combinatorics — **the lessons build from scratch,
in order.** Module 02 exists precisely to construct the mathematical toolkit the
later modules lean on. If a lesson uses a tool it hasn't taught, that's a bug;
tell us.

**Programming / Rust.** You should be able to program in *some* language. You do
**not** need to know Rust already. The algorithms here live in the simple core
of the language — integers, arrays (`Vec`), loops, functions, `struct`s. We
deliberately avoid Rust's scarier corners (no `unsafe`, no lifetimes gymnastics,
no async, zero external libraries). Module 01's lab is written as a **guided
tour**: each stub comes with step-by-step instructions and links to the exact
Rust documentation you need. If you've written a `for` loop in Python or Java,
you can write these — and you'll pick up real Rust as you go.

The one non-negotiable tool: **a notebook and pen.** You cannot absorb TAOCP by
reading alone. Trace the algorithm by hand on a small input *before* you code
it. Knuth's whole style is built for pencil-and-paper simulation; fighting that
is fighting the current.

## 4. How to study a module (the loop that works)

For each module, in order:

1. **Read the lesson** (`course/module-NN-*/README.md`). Derive the math
   yourself in the notebook — don't just nod along. When it states an algorithm
   in step form (E1, E2, E3…), trace it on the small example it gives you.
2. **Implement stage by stage.** Open the lab file, find the first `todo!()`,
   and fill it in. Keep Knuth's step labels as comments — "step-faithful first,
   pretty later." Run `./grade N` and let the failing test tell you what's
   wrong. The tests are written to *teach*; read them.
3. **Let the tests argue with you.** A red test is not a scolding, it's a
   worked example of an input you got wrong. Fix, re-run, repeat.
4. **Compare with the reference** once a stage is green — read the module's
   `WALKTHROUGH.md`. This is the "check your answer against Knuth's" step, and
   it's where you learn the idioms.
5. **Do a couple of exercises** from the lesson's curated table and log them in
   `exercises.md`. Knuth rates every exercise 00–50 (00 = trivial, 20 = about an
   hour, 30 = several hours, 50 = open research problem); a ▶ marks the
   especially instructive ones. Start with the low numbers.

Pacing: roughly **a module every one to two weeks** alongside a job. This is not
a race. Depth beats speed here — a single well-understood proof is worth more
than three stages you rushed.

## 5. The training wheels come off — on purpose

This course is designed as a **guided tour that gradually becomes an open
road.** The amount of hand-holding *tapers* as you gain footing:

- **Module 01** is a fully guided tour. The lab file spells out each step, names
  the Rust methods to call, and links to their documentation. You are never more
  than a paragraph away from what to type next.
- **Modules 02–04** keep the structure and the pseudocode but stop naming every
  Rust method — you'll reach for the standard-library docs yourself (a skill
  worth having).
- **Modules 05 onward** give you the algorithm, the invariant, and the contract,
  and trust you to translate. By the advanced tier (11–23) the lesson states the
  theorem and the reference is a genuine "compare notes," not a crutch.

If you ever feel dropped, that's the design working — but the safety nets never
disappear. **Every stage, in every module, always has:** the self-contained
lesson, three graduated hints (`./grade N --stage K --hint`), the reference
solution, and the walkthrough. Removing aids means asking a little more of you
each time, not abandoning you.

## 6. A word on the mindset

Knuth's five properties of an *algorithm* — finiteness, definiteness, input,
output, effectiveness (you'll meet them on page one of Module 01) — are also a
good description of how to study this material: be **finite** (one stage at a
time, done), **definite** (know exactly what your code does, no hand-waving),
and **effective** (every step something you could do with pencil and paper).
When a test fails, resist pattern-matching random changes until it goes green.
Trace the input by hand, find where your mental model and the machine disagree,
*then* fix. That habit — insisting you understand *why* — is the actual
curriculum. The algorithms are just the vehicle.

## 7. Reading Knuth's notation (a small cheat-sheet)

TAOCP (and its sister book *Concrete Mathematics*) uses a few compact notations
that trip up first-time readers. You don't need these to pass the course — the
lessons spell everything out — but they make the *books* far less intimidating,
so here they are up front. Knuth deliberately chose or popularized each one to
remove an ambiguity.

- **Iverson brackets** `[P]` — a proposition in square brackets *is* a number:
  `1` if `P` is true, `0` if false. It turns "sum only the terms where…" into
  plain multiplication. The Kronecker delta `δ_ij` is just `[i = j]`, and
  `Σ_{1≤i≤10} i²` can be written `Σ_i i²·[1 ≤ i ≤ 10]` — the condition rides
  *inside* the sum instead of hanging off it. (You'll feel why this is convenient
  in Module 02.)

- **Falling and rising factorials** `x^{underline n}` and `x^{overline n}` —
  Knuth fixed the old ambiguous "(x)_n" notation with an underline/overline that
  *points* in the direction the factors move:
  - falling: `x^{underline n} = x(x−1)(x−2)···(x−n+1)` — the count of ways to
    arrange `n` of `x` items (permutations); `k^{underline n} = k!/(k−n)!`.
  - rising: `x^{overline n} = x(x+1)(x+2)···(x+n−1)`.

  The falling factorial is the discrete world's analogue of a power: the finite
  difference `Δ(x^{underline n}) = n·x^{underline(n−1)}` mirrors the derivative
  `d/dx (xⁿ) = n·xⁿ⁻¹` exactly. That parallel is a recurring theme in Module 02.

- **Stirling numbers** — Knuth standardized the readable, binomial-style
  notation: `{n atop k}` (curly braces) counts ways to *partition* `n` items into
  `k` non-empty groups; `[n atop k]` (square brackets) counts permutations of `n`
  items with `k` cycles. Braces for subsets, brackets for cycles — a mnemonic
  that replaced a zoo of clashing symbols.

If you ever want to go deeper on this machinery, *Concrete Mathematics*
(Graham–Knuth–Patashnik) is the companion volume that builds it patiently — see
**[Concrete Mathematics: the companion volume](concrete-mathematics.md)** for how
it maps onto this course. Module 02 gives you the working subset you actually
need.

## 8. Reading TAOCP's section numbers

Throughout the lessons — and throughout Knuth's books — a reference like
**§1.2.3**, **§5.2.1**, or **Algorithm 4.5.2B** points to a numbered section of
TAOCP. The number tells you exactly where to look, *across* the volumes, because
TAOCP is numbered by **chapter**, and the chapters are split into volumes:

| A section starting with… | is in chapter… | which is in… |
|---|---|---|
| §1, §2 | 1 (Basic Concepts), 2 (Information Structures) | **Vol. 1** |
| §3, §4 | 3 (Random Numbers), 4 (Arithmetic) | **Vol. 2** |
| §5, §6 | 5 (Sorting), 6 (Searching) | **Vol. 3** |
| §7 | 7 (Combinatorial Searching) | **Vol. 4** (4A, 4B, 4C…) |

So **§1.2.3** is Vol. 1, and **§4.5.3** — Euclid's average-case analysis, cited
back in Module 01 — is Vol. 2, Chapter 4, *even though Module 01 itself teaches
§1.1*. Section numbers nest as deep as they need to: §7.2.2.2 reads as chapter
7 → 7.2 → 7.2.2 → 7.2.2.2 (satisfiability, Vol. 4B).

An **algorithm** is named by its section plus a letter: *Algorithm 1.1E* is the
fifth algorithm displayed in §1.1 (Euclid's); *Algorithm 5.2.2Q* is quicksort in
§5.2.2. Its steps inherit that letter — E1, E2, E3 — which is why this course
asks you to keep those labels as comments in your code.

Every lesson also states its exact home in a **Source** header at the top, so
you can open the book to the right page before you begin. You never need the
books here — but this is how to find any citation if you own them, or on
[Knuth's TAOCP page](https://cs.stanford.edu/~knuth/taocp.html).

## 9. Ready?

- Set up your machine and learn the commands: **[Getting started](getting-started.md)**.
- Then open **[the Module 01 lesson](../course/module-01-algorithms/README.md)**
  and run `./grade 1`.
- Curious where it all goes? The **[Syllabus](../SYLLABUS.md)** maps all 22
  modules; **[docs/toolkit.md](toolkit.md)** maps the proof techniques and
  engineering judgment the journey builds in you.
- Want to go deeper on the mathematics? **[Concrete Mathematics](concrete-mathematics.md)**
  is the one companion book worth reading — Module 02's §1.2, grown into a full
  course.
- Keep the **[Glossary](glossary.md)** handy — every recurring convention
  (ratings, §-references, step labels, notation, `LAMBDA`, "mems") in one place.

Welcome. Euclid is waiting.
