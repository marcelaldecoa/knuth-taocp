# Hints — Module 01: The Notion of an Algorithm

Graduated hints, three per stage. Reach for hint 1 first; drop to 3 only when stuck.

## Stage 1: Euclid's algorithm, step by step

1. The whole method rests on one fact: if `r = m mod n`, then the common divisors of `(m, n)` are exactly the common divisors of `(n, r)`, so `gcd(m, n) = gcd(n, r)`. The remainder strictly shrinks, so the process must stop — that is the termination (finiteness) argument.
2. Follow Knuth's three steps literally as a `loop`: E1 forms the remainder, E2 tests it against zero, E3 shifts `n` into `m` and `r` into `n`. Don't reach for recursion or a `while r != 0` rewrite yet — mirror the control flow. Remember Algorithm E is stated for positive integers only, so assert `m > 0 && n > 0`.
3. `let r = m % n; if r == 0 { return n; } m = n; n = r;` inside `loop {}`. The answer is the last nonzero remainder, which is the current `n` when `r` first hits zero.

## Stage 2: Avoiding trivial replacements

1. Step E3's `m <- n; n <- r` is pure data shuffling that computes nothing. The observation (exercise 1.1-3) is that you can let the two variables swap *roles* every half-turn instead of copying values between them.
2. Unroll one iteration into two: reduce `m` modulo `n` in place, test for zero; then reduce `n` modulo `m` in place, test for zero. The loop body now has two remainder steps and two zero-tests, and never assigns one variable's value into another.
3. `m %= n; if m == 0 { return n; } n %= m; if n == 0 { return m; }` inside `loop {}`. Note the answer is whichever variable is *nonzero* when the other becomes zero. Cross-check against `euclid_e` on a small grid to convince yourself they agree.

## Stage 3: Extended Euclid: certifying the gcd

1. Alongside the remainders, carry Bézout coefficients so that at every moment you know integers making `a*m + b*n` equal to the current value. Then the final gcd arrives already *certified* by the identity `a*m + b*n = gcd(m, n)`.
2. Maintain two rows of the invariant simultaneously: `(a1, b1, c)` for the larger value and `(a, b, d)` for the smaller, with `a1*m + b1*n = c` and `a*m + b*n = d`. Each division step recycles `c <- d`, `d <- r`, and updates the coefficient rows by exactly the same recurrence with the quotient `q`. Use a signed wide type (`i128`) for the coefficients — they go negative.
3. Start `(a1,b1)=(1,0)`, `(a,b)=(0,1)`, `(c,d)=(m,n)`. Each step: `q = c/d`, `r = c%d`; if `r==0` return `(d, a, b)`; else `c=d; d=r;` and set the new `(a,b)` to `(a1 - q*a, b1 - q*b)` while `(a1,b1)` takes the old `(a,b)`. Return the gcd as `u64` but the coefficients as `i128`.

## Stage 4: Counting divisions; Lamé's worst case

1. The running time of Algorithm E is governed by how many division steps (executions of E1) it performs. Lamé's theorem says the worst case for numbers below a bound comes from *consecutive Fibonacci numbers*, because they force every quotient down to 1 — the slowest possible descent.
2. Write a counter version of Euclid that returns the number of times the remainder step ran instead of the gcd. Then feed it consecutive Fibonacci pairs `(F_{k+1}, F_k)` and observe the step count is exactly `k - 1`.
3. Clone the Stage-1 loop but accumulate `let mut t = 0; ... t += 1;` on each remainder, returning `t` when `r == 0`. For the worst-case check, generate Fibonacci numbers with a rolling pair `(fk, fk1)` and assert `division_steps(fk1, fk) == k - 1`.
