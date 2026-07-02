//! Module 01 — The Notion of an Algorithm (TAOCP Vol. 1, §1.1).
//!
//! # YOUR WORKSPACE — and a fully guided tour
//!
//! This is the file you edit. Each function below is a stub whose body is
//! `todo!()`; your job is to replace each one with a real implementation, then
//! run `./grade 1` from the repository root until every stage is green.
//!
//! **Module 01 is a guided tour.** Every stub carries, right here in the
//! source: the algorithm in Knuth's step-by-step form, a plain-English recipe
//! for turning those steps into Rust, and links to the exact standard-library
//! and Rust-book pages you need. You should never be more than a paragraph away
//! from knowing what to type. New to Rust or to Knuth? Read
//! [`docs/for-newcomers.md`](../../../docs/for-newcomers.md) first.
//!
//! **The training wheels come off gradually.** This much hand-holding is unique
//! to Module 01. Module 02 keeps the pseudocode but stops naming every Rust
//! method; by the later modules you get the algorithm and the contract and
//! translate it yourself. The safety nets never disappear, though — every stage
//! everywhere has a lesson, three graduated hints (`./grade 1 -s K --hint`), a
//! reference solution, and a walkthrough.
//!
//! ## How to work a stage
//!
//! 1. Read the matching section of the lesson,
//!    `course/module-01-algorithms/README.md`. Trace the algorithm by hand on
//!    the small example it gives you — pencil and paper is not optional here.
//! 2. Come back to the stub below, delete the `let _ = ...;` line and the
//!    `todo!(...)`, and write the code following the recipe in the doc comment.
//! 3. Run `./grade 1`. It runs the stages in order and stops at the first
//!    failure, telling you exactly what to fix. Stuck? `./grade 1 -s K --hint`.
//!
//! ## The one convention to keep
//!
//! Leave Knuth's step labels (E1, E2, E3, …) in your code as comments, the way
//! the recipes below show. "Step-faithful first; make it idiomatic later if you
//! like, as long as the tests stay green." It keeps your code auditable against
//! the book and against the reference solution.
//!
//! ## A 90-second Rust primer for this file
//!
//! - Function arguments are immutable by default. To *reassign* a parameter
//!   (e.g. `m = n;`), declare it mutable: `pub fn f(mut m: u64, ...)`. Adding
//!   `mut` to the parameters is allowed and expected — do it when a recipe
//!   tells you to update a variable in place.
//! - `%` is the remainder operator; `m % n` is exactly Knuth's "remainder of
//!   m divided by n". For unsigned integers like `u64` it is always in
//!   `0..n` — which is precisely the `0 <= r < n` guarantee the proofs rely on.
//! - `loop { ... }` repeats forever until a `return` (or `break`) leaves it —
//!   the natural Rust spelling of Knuth's "go back to step E1".
//! - The last expression in a function is its return value (no semicolon), or
//!   use `return x;` to leave early. Most stages here use `return` from inside
//!   a `loop`.
//! - Rust book, control flow: <https://doc.rust-lang.org/book/ch03-05-control-flow.html>
//! - `u64` and its methods: <https://doc.rust-lang.org/std/primitive.u64.html>

/// # Stage 1 — `euclid_e`: Algorithm 1.1E (Euclid's algorithm)
///
/// Given two positive integers `m` and `n`, return their greatest common
/// divisor — the largest positive integer dividing both. Trace it once by hand
/// on `(544, 119)` (answer: 17) before you code.
///
/// ## The algorithm (keep these labels as comments)
///
/// ```text
/// E1. [Find remainder.]  Divide m by n and let r be the remainder.
///                        (We will have 0 <= r < n.)
/// E2. [Is it zero?]      If r = 0, the algorithm terminates; n is the answer.
/// E3. [Reduce.]          Set m <- n, n <- r, and go back to E1.
/// ```
///
/// ## How to write it in Rust
///
/// 1. Change the signature to take `mut m: u64, mut n: u64` so you can reassign
///    them in step E3. ([Why `mut`?](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html))
/// 2. **Definiteness first.** Algorithm E is defined only for positive
///    integers, so reject zero *loudly*:
///    `assert!(m > 0 && n > 0, "Algorithm E requires positive integers");`
///    The grader checks that the panic message contains the word "positive" —
///    an algorithm's input domain is part of the algorithm.
///    ([`assert!`](https://doc.rust-lang.org/std/macro.assert.html))
/// 3. Open a `loop { ... }`. Inside it:
///    - **E1:** `let r = m % n;` — the `%` operator is your remainder.
///      ([remainder for `u64`](https://doc.rust-lang.org/std/ops/trait.Rem.html))
///    - **E2:** `if r == 0 { return n; }`. Return `n`, **not** `r`: the answer
///      is the *last nonzero* remainder, which is the current `n` when `r`
///      first becomes 0.
///    - **E3:** `m = n; n = r;` and let the `loop` carry you back to E1.
///
/// ## How it's tested
///
/// `tests/stage_01_euclid.rs` checks Knuth's worked value `gcd(544,119)=17`,
/// the exercise-1.1-1 value `gcd(2166,6099)=57` (both orders), and edge cases
/// like `gcd(1, 999)=1`. There's also a `#[should_panic]` test feeding a zero.
///
/// Stuck? `./grade 1 -s 1 --hint`.
pub fn euclid_e(m: u64, n: u64) -> u64 {
    let _ = (m, n);
    todo!("implement Algorithm 1.1E — see the recipe in this doc comment")
}

/// # Stage 2 — `euclid_f`: Algorithm 1.1F (exercise 1.1-3, rating 16)
///
/// Knuth's exercise: step E3's `m <- n, n <- r` is pure data-shuffling that
/// computes nothing. Can it be removed? Yes — instead of copying values between
/// the variables, let `m` and `n` **swap roles** every half-turn.
///
/// ## The algorithm
///
/// ```text
/// F1. [Remainder m/n.]  Divide m by n; set m to the remainder.
/// F2. [Is it zero?]     If m = 0, terminate with answer n.
/// F3. [Remainder n/m.]  Divide n by m; set n to the remainder.
/// F4. [Is it zero?]     If n = 0, terminate with answer m.  Go back to F1.
/// ```
///
/// ## How to write it in Rust
///
/// 1. Same setup as Stage 1: `mut m, mut n`, and the positive-integers
///    `assert!`.
/// 2. Inside a `loop`, unroll two of Euclid's iterations so the divisor role
///    alternates. Use the **compound assignment** operator `%=`, which does
///    "remainder in place": `m %= n;` means `m = m % n;`.
///    ([compound assignment](https://doc.rust-lang.org/book/appendix-02-operators.html))
///    - **F1/F2:** `m %= n;  if m == 0 { return n; }`
///    - **F3/F4:** `n %= m;  if n == 0 { return m; }`
/// 3. Note the two *different* return sites: the answer is whichever variable is
///    still nonzero when its partner reaches zero. Values only ever shrink in
///    place — no variable is ever assigned another's value, which is the whole
///    point of the exercise.
///
/// ## Why this stage exists
///
/// Two *different algorithms* (E and F) compute the *same function* (gcd).
/// Knuth is careful to separate the function computed from the method computing
/// it, and so should you. The test proves the point empirically:
/// `tests/stage_02_euclid_f.rs` checks `euclid_f` against your `euclid_e` on a
/// whole grid of inputs — validate the *contract* (same gcd), never the trace.
///
/// Stuck? `./grade 1 -s 2 --hint`.
pub fn euclid_f(m: u64, n: u64) -> u64 {
    let _ = (m, n);
    todo!("implement Algorithm 1.1F — alternate the roles of m and n")
}

/// # Stage 3 — `extended_euclid`: Algorithm 1.2.1E (Extended Euclid)
///
/// Euclid's algorithm can do more than find `d = gcd(m, n)`: it can hand back
/// integers `a, b` with
///
/// ```text
///     a*m + b*n = d          (Bézout's identity)
/// ```
///
/// These *certify* the answer — anyone can multiply and check, without trusting
/// your code. Return the triple `(d, a, b)`.
///
/// ## The idea: carry two rows of an invariant
///
/// Alongside the running values, carry coefficients so that at every moment you
/// know how to build the current value out of `m` and `n`:
///
/// ```text
///     invariant:   a1*m + b1*n = c      and      a*m + b*n = d
/// ```
///
/// where `(c, d)` runs through exactly the values `(m, n)` take in Algorithm E.
/// Initially `(a1, b1) = (1, 0)`, `(a, b) = (0, 1)`, `(c, d) = (m, n)` — check
/// that both identities hold at the start. Each division `c = q*d + r` recycles
/// `(c, d) <- (d, r)` and updates the coefficient rows by the *same* recurrence
/// with quotient `q`. When `r` hits 0, `(d, a, b)` is your answer.
///
/// ## How to write it in Rust
///
/// 1. Keep the positive-integers `assert!`.
/// 2. **Types matter here.** The coefficients go *negative* and can grow, so
///    they must be signed and wide: declare them `i128`. The gcd `d` you track
///    as `u64`. You will need to convert between them with an `as` cast —
///    `q as i128` — when you multiply a value by a coefficient.
///    ([`i128`](https://doc.rust-lang.org/std/primitive.i128.html) ·
///    [`as` casts](https://doc.rust-lang.org/rust-by-example/types/cast.html))
/// 3. Declare the six running variables with explicit types, e.g.
///    `let (mut a1, mut b1): (i128, i128) = (1, 0);` (this is **tuple
///    destructuring** — binding two variables at once;
///    [tuples](https://doc.rust-lang.org/book/ch03-02-data-types.html#the-tuple-type)).
/// 4. In a `loop`, per step:
///    - `let q = c / d;  let r = c % d;`  (`/` is integer division for `u64`)
///    - `if r == 0 { return (d, a, b); }`
///    - recycle `c = d; d = r;`
///    - update the coefficient rows: the new `(a, b)` becomes
///      `(a1 - q as i128 * a, b1 - q as i128 * b)`, and the old `(a, b)` slides
///      up into `(a1, b1)`. Compute the new values into temporaries *before*
///      overwriting, so you don't clobber a value you still need.
///
/// ## Why it matters / how it's tested
///
/// When `gcd(m, n) = 1`, the identity reads `a*m ≡ 1 (mod n)` — extended Euclid
/// *is* modular inversion, the workhorse behind RSA and every TLS handshake.
/// `tests/stage_03_extended_euclid.rs` checks the §1.2.1 worked example
/// (`5*1769 - 16*551 = 29`) and then verifies the **identity** `a*m + b*n = d`
/// over a grid — it checks the certificate, not your particular coefficients,
/// because Bézout coefficients aren't unique.
///
/// Stuck? `./grade 1 -s 3 --hint`.
pub fn extended_euclid(m: u64, n: u64) -> (u64, i128, i128) {
    let _ = (m, n);
    todo!("implement Algorithm 1.2.1E — carry the two-row Bézout invariant")
}

/// # Stage 4 — `division_steps`: the analysis of Algorithm E (Lamé, 1845)
///
/// Return `T(m, n)`: how many times step **E1** (one division) executes when
/// Algorithm E runs on `(m, n)`. This is your first *cost function*, and you'll
/// use it to reproduce the first real theorem in the analysis of algorithms.
///
/// ## How to write it in Rust
///
/// This is Stage 1's loop with the return value swapped from the gcd to a
/// counter. Keep it a *separate* function (don't thread a counter through
/// `euclid_e`) so the fast gcd path stays clean:
///
/// 1. Positive-integers `assert!`, `mut m, mut n`.
/// 2. `let mut t: u32 = 0;` before the loop.
/// 3. In the `loop`: `let r = m % n; t += 1;` then `if r == 0 { return t; }`,
///    then `m = n; n = r;`. (Count the division *every* time E1 runs, including
///    the final one that produces `r == 0`.)
///
/// ## The theorem you're about to watch come true
///
/// **Lamé's theorem.** The worst case for Euclid below a bound comes from
/// *consecutive Fibonacci numbers* `(F_{k+1}, F_k)` — they force every quotient
/// down to 1, the slowest possible descent, giving exactly `T = k - 1`
/// divisions. `tests/stage_04_lame.rs` rolls a Fibonacci pair forward and
/// asserts `division_steps(F_{k+1}, F_k) == k - 1`, and confirms
/// `division_steps(544, 119) == 4`. Derive-then-measure — analysis of
/// algorithms exactly as Knuth practices it. (The full proof, and the *average*
/// case ≈ 0.843 ln n, are in Vol. 2 §4.5.3.)
///
/// Stuck? `./grade 1 -s 4 --hint`.
pub fn division_steps(m: u64, n: u64) -> u32 {
    let _ = (m, n);
    todo!("count executions of step E1 — Stage 1's loop with a counter")
}
