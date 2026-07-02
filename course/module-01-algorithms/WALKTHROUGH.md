# Walkthrough — Module 01: The Notion of an Algorithm

Read this AFTER a stage is green — it explains how the reference solution is built and why.

## Stage 1: Euclid's algorithm, step by step

The reference is a deliberately literal transcription of Algorithm 1.1E: an unconditional `loop` whose body is the three labelled steps E1 (remainder), E2 (zero test), E3 (reduce), with Knuth's step names left in as comments. The correctness invariant is `gcd(m, n)` is unchanged across the loop, because `gcd(m, n) = gcd(n, m mod n)`; the termination argument is that the remainder `r` satisfies `0 <= r < n`, so `n` strictly decreases and cannot do so forever. The idiom worth stealing is returning `n` (not `r`) at the moment `r == 0`: the answer is the *last nonzero* remainder, which is precisely the divisor of the final exact division. The upfront `assert!(m > 0 && n > 0)` is not defensive noise — it enforces Knuth's definiteness requirement, since the recurrence's guarantees only hold for positive inputs.

## Stage 2: Avoiding trivial replacements

`euclid_f` shows that faithfulness to an algorithm's *result* does not require faithfulness to a wasteful *encoding*. By unrolling two E-iterations, the reference alternates which variable plays the divisor: `m %= n` then `n %= m`, each guarded by its own zero-test. This eliminates the `m <- n; n <- r` copies of E3 entirely — the values never move between variables, only shrink in place. The subtle point the structure makes explicit is that the answer is whichever variable is nonzero when its partner reaches zero (hence the two distinct `return` sites). The reference test cross-checks `euclid_f` against `euclid_e` over a 60×60 grid, which is the honest way to validate a "same answer, different shape" rewrite: assert the contract (equal gcd), not any internal trace.

## Stage 3: Extended Euclid: certifying the gcd

The design carries the classic two-row invariant: `a1*m + b1*n = c` and `a*m + b*n = d`, initialized to the trivial identities `1*m + 0*n = m` and `0*m + 1*n = n`. Each step performs the same recycle on the values (`c <- d, d <- r`) and on both coefficient rows using the quotient `q`, so the invariant is preserved by construction; when `r` hits zero, `d` is the gcd and `(a, b)` are its Bézout coefficients — the certificate. The important type choice is `i128` for the coefficients: they legitimately go negative and can exceed the magnitude of the inputs, so unsigned or `i64` would be a lurking bug. The reference returns `(d, a, b)` with `d: u64` but coefficients `i128`, and its test verifies `a*m + b*n == d` rather than pinning specific coefficient values — because extended Euclid's certificate is not unique, only the identity is. This certifying idea is exactly what later yields modular inverses in §4.5.2.

## Stage 4: Counting divisions; Lamé's worst case

`division_steps` is a surgical clone of the Stage-1 loop with the return value swapped from the gcd to a step counter `t` incremented on every remainder. Keeping it a separate function (rather than threading a counter through `euclid_e`) keeps the hot gcd path clean while giving the analysis code exactly the quantity Knuth calls `T(m, n)`. The payoff is empirical: the test rolls a Fibonacci pair `(fk, fk1)` forward and asserts `division_steps(fk1, fk) == k - 1`, turning Lamé's theorem into a checkable claim. Consecutive Fibonacci numbers are the worst case because each division yields quotient 1 and remainder `F_{k-1}` — the descent removes the least possible at every step, so no input below `F_{k+1}` needs more divisions. This is the module's first taste of "the analysis of an algorithm is itself a theorem you can test."
