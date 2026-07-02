# Hints — Module 02: Mathematical Preliminaries

Graduated hints, three per stage. Reach for hint 1 first; drop to 3 only when stuck.

## Stage 1: Sums in closed form

1. Every sum here has a closed form derived in §1.2.3 (Gauss pairing for the linear sum, the perturbation method for squares and the geometric series). Compute the formula directly in O(1) — never loop `k` from 1 to `n`.
2. Use `n(n+1)/2`, `n(n+1)(2n+1)/6`, and Nicomachus `(n(n+1)/2)^2` for the three power sums; for the geometric sum use `(x^(n+1) - 1)/(x - 1)` with a special case for `x == 1`. Widen to `u128`/`i128` before multiplying so the products don't overflow.
3. `sum_first_n`: `n*(n+1)/2` in `u128`. `sum_squares`: form `t = n*(n+1)/2` first, then `t*(2n+1)/3` (each division is exact). `sum_cubes`: `let t = sum_first_n(n); t*t`. `geometric_sum`: `if x == 1 { n+1 } else { (x.pow(n+1) - 1)/(x - 1) }`.

## Stage 2: Binomial coefficients

1. Never compute `C(n,k)` as `n!/(k!(n-k)!)` — the factorials overflow long before the answer does. Use the multiplicative recurrence of §1.2.6 Eq. (3), which keeps every partial result an honest integer.
2. Build the coefficient as a running product `c <- c*(n-k+i)/i` for `i = 1..=k`. Because after step `i` the value equals `C(n-k+i, i)`, each division lands exactly. Exploit the symmetry `C(n,k) = C(n,n-k)` to replace `k` with `min(k, n-k)`, and return 0 when `k > n`.
3. `if k > n { return 0; } let k = k.min(n-k);` then `let mut c: u128 = 1; for i in 1..=k { c = c*(n-k+i)/i; } c`. Do the multiply *before* the divide so the intermediate is divisible.

## Stage 3: Fibonacci numbers, fast

1. `F_n` is defined by `F_0 = 0`, `F_1 = 1`, `F_{n+1} = F_n + F_{n-1}` (§1.2.8). A single left-to-right pass keeping the last two values computes it in O(n) additions — no recursion, no memo table.
2. Carry a rolling pair `(a, b)` equal to `(F_{i-1}, F_i)` and advance it. Guard the domain: `F_186` is the largest that fits in `u128`, so assert `n <= 186` (the test expects a panic containing "overflows" for `n = 187`).
3. `if n == 0 { return 0; } let (mut a, mut b) = (0u128, 1u128); for _ in 1..n { let t = a+b; a = b; b = t; } b`. The assert message must contain the substring `overflows`.

## Stage 4: Harmonic numbers, exactly and asymptotically

1. `H_n = 1 + 1/2 + ... + 1/n` is rational; to keep it *exact* you must do fraction arithmetic and reduce by the gcd each step, or the denominator explodes. The floating version is a separate, approximate computation (§1.2.7).
2. For the exact form keep a reduced `(num, den)`; adding `1/k` gives `(num*k + den)/(den*k)`, then divide both by their gcd. For `harmonic_f64`, sum the reciprocals from the *smallest* term upward so rounding error stays a few ulps.
3. Exact: `num = num*k + den; den *= k; let g = gcd(num, den); num /= g; den /= g;` looping `k` in `1..=n`, starting `(0, 1)`. Float: `for k in (1..=n).rev() { h += 1.0/k as f64; }`. You'll need a small `u128` gcd helper (Euclid again).

## Stage 5: Analysis of an algorithm: finding the maximum

1. Algorithm 1.2.10M scans the array and tracks both the running maximum and the count `A` of times it was *updated*. That count is the quantity §1.2.10 analyzes; its expectation on a random permutation of distinct values is `H_n - 1`, tying this stage back to Stage 4.
2. Follow Knuth's steps: initialize with the last element as the max at index `n`, then scan `k` downward; replace the max only on a *strict* increase (M4), which makes ties keep the rightmost index. Return `(j, m, a)` with `j` converted to 0-based.
3. `let mut j = n; let mut k = n-1; let mut m = xs[n-1]; let mut a = 0;` then `loop`: `if k == 0 { return (j-1, m, a); }`; `if xs[k-1] > m { j = k; m = xs[k-1]; a += 1; }`; `k -= 1;`. The plain `find_max` just drops `a` from the triple.
