# Walkthrough — Module 02: Mathematical Preliminaries

Read this AFTER a stage is green — it explains how the reference solution is built and why.

## Stage 1: Sums in closed form

Each function is a one-line closed form, and the reference's discipline is *widen before you multiply*: it casts `n` to `u128` (or works in `i128`) so that `n*(n+1)` cannot overflow for any `u64` input. `sum_squares` is the instructive one — instead of `n*(n+1)*(2n+1)/6` it forms the triangular number `t = n*(n+1)/2` first and then `t*(2n+1)/3`, splitting the `/6` into two exact divisions whose divisibility is guaranteed at each step (one of `n`, `n+1` is even; one of `n`, `n+1`, `2n+1` is divisible by 3). `sum_cubes` reuses `sum_first_n` and squares it (Nicomachus), which is both faster and a nice piece of self-documentation. `geometric_sum` guards `x == 1` before dividing by `x - 1`, the one input where the closed form is undefined. The naive alternative — looping to accumulate — is O(n) and, worse, invites overflow on the partial sums; the closed forms are O(1) and provably exact within `u128`.

## Stage 2: Binomial coefficients

The reference computes `C(n,k)` by the multiplicative method, and the reason it never overflows for `n <= 100` is the loop invariant: after iteration `i`, `c == C(n-k+i, i)`, always an integer, so `c = c*(n-k+i)/i` divides evenly every time. The multiply happens *before* the divide precisely so the numerator is divisible by `i`. Two guards do a lot of work: `k > n` returns 0 (no k-subsets exist), and `k = min(k, n-k)` uses the symmetry `C(n,k) = C(n,n-k)` to run the fewest iterations and keep intermediates smallest — the largest value seen for the stage anchor `C(100,50)` stays under 2^104. The bug this design avoids is the factorial approach, where `100!` overflows any fixed-width integer even though the final answer fits comfortably.

## Stage 3: Fibonacci numbers, fast

This is the iterative two-variable recurrence with the invariant stated in the comment: entering iteration `i` (1-based), `(a, b) = (F_{i-1}, F_i)`. Each turn computes `t = a+b` and rotates, so after `n-1` turns `b == F_n`. The `assert!(n <= 186, "... overflows ...")` is a real part of the contract, not decoration: `F_186` is the last Fibonacci number below 2^128, and the panic message is required to contain "overflows" so the stub's bare `todo!()` cannot masquerade as a passing implementation. The idiom worth stealing is refusing to recurse: naive recursive Fibonacci is exponential, and even the memoized version costs O(n) space for what an O(1)-space scan does cleanly.

## Stage 4: Harmonic numbers, exactly and asymptotically

The exact `harmonic` keeps a reduced fraction `(num, den)` and, crucially, reduces by the gcd *inside* the loop rather than at the end. Without the per-step reduction the denominator would be `lcm(1..n)` built as a raw product and would blow past `u128` quickly; reducing each step keeps `H_30`'s denominator at 2329089562800 and everything in range. `harmonic_f64` is a separate function on purpose — it answers a different question (a fast float approximation) and sums the reciprocals from the *smallest* term upward, the standard trick to limit floating-point error to a few ulps because you add small numbers before they are swamped by the large partial sum. The private `gcd_u128` helper is just Algorithm 1.1E from Module 01, reused "one flight down". The test asserts the fraction is reduced (`gcd == 1`) and agrees with the float version, checking the contract rather than a specific representation.

## Stage 5: Analysis of an algorithm: finding the maximum

`find_max_counting` is a step-faithful transcription of Algorithm M, and the design decision that matters is scanning from the right with a *strict* comparison (`xs[k-1] > m`). That combination makes ties resolve to the largest index `j`, matching Knuth's specification exactly, and it is why the increasing input gives `A = 0` while the decreasing input gives `A = n-1`. The counter `a` records executions of step M4 — the "register update" whose expected value is `H_n - 1` on a random permutation, which is the whole reason the harmonic numbers of Stage 4 appear in the analysis of algorithms at all. `find_max` is layered on top by discarding the count, so there is a single source of truth for the scan. The reference keeps `j` and `k` 1-based through the loop to mirror the book and only converts to 0-based (`j - 1`) at the return, a small honesty that keeps the step comments readable.
