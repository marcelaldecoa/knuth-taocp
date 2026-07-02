# Hints — Module 04: Random Numbers

Graduated hints, three per stage. Reach for hint 1 first; drop to 3 only when stuck.

## Stage 1: The linear congruential method

1. A linear congruential generator iterates `X_{n+1} = (a*X_n + c) mod m` (§3.2.1). Theorem 3.2.1.2A tells you when it achieves full period m for every seed: `c` coprime to `m`, `a ≡ 1` modulo every prime dividing `m`, and `a ≡ 1 (mod 4)` when `4 | m`. The period for a small modulus can be found by direct cycle detection.
2. Store `(x, a, c, m)` and encode the word-size modulus `2^64` as `m == 0`, handled with wrapping arithmetic; for `m > 0`, reduce the seed and parameters mod m and carry the product in `u128` to avoid overflow. For `period`, record the first step at which each state value appears and return the gap when a state repeats — that gap is the cycle length (the tail is not counted).
3. `next`: if `m == 0`, `a.wrapping_mul(x).wrapping_add(c)`; else `((a as u128 * x as u128 + c as u128) % m as u128) as u64`. `period`: a `vec![u64::MAX; m]` of first-seen steps; loop `if first_seen[x] != MAX { return step - first_seen[x]; }`, else record and advance.

## Stage 2: The chi-square test

1. The chi-square statistic (§3.3.1) measures how far observed category counts stray from what a probability model predicts: `V = sum_s (Y_s - n*p_s)^2 / (n*p_s)`. Compare V against the table with `k-1` degrees of freedom. The expected counts must all be reasonably large (Knuth: merge sparse categories first).
2. Zip observed counts against expected counts, accumulate `(obs - exp)^2 / exp`. Guard the preconditions: equal lengths, non-empty, and every expected count strictly positive (a zero expected count would divide by zero). The uniform variant just sets every expected count to `n/k`.
3. `observed.iter().zip(expected).map(|(&o, &e)| { let d = o as f64 - e; d*d/e }).sum()`. For `chi_square_uniform`: `let n: u64 = counts.iter().sum(); let e = n as f64 / counts.len() as f64;` then call `chi_square` with a vector of `e`s.

## Stage 3: Shuffling

1. Algorithm 3.4.2P (Fisher–Yates) walks position `j` from the top down and swaps `items[j]` with a random position in `0..=j` — a *shrinking* range. That shrinking range is exactly what makes the map from draw-sequences to permutations a bijection onto all `t!` outcomes, hence uniform. The naive version that draws from the *full* range each time is biased, because `n^n` tapes cannot divide evenly among `n!` permutations for `n > 2`.
2. Take `rng: &mut impl FnMut(u64) -> u64` returning a value in `0..bound`. Loop `j` from `t-1` down to `1`, draw `k = rng(j+1)` (uniform in `0..=j`), swap, decrement. Slices of length < 2 need no work and must consume no randomness.
3. `let mut j = t-1; loop { let k = rng(j as u64 + 1) as usize; items.swap(k, j); if j == 1 { return; } j -= 1; }`. `naive_shuffle` is the deliberately-wrong contrast: `for i in 0..n { let k = rng(n as u64) as usize; items.swap(i, k); }` — keep it to demonstrate the bias table.

## Stage 4: Reservoir sampling

1. Algorithm 3.4.2R samples `k` items uniformly from a stream of *unknown length* in one pass and O(k) memory. The invariant (proved by induction): after `t` records have been seen, each is in the reservoir with probability exactly `k/t`. Newly read record number `t` enters with probability `k/t`, displacing a uniformly chosen current occupant.
2. Fill the reservoir with the first `k` records. Thereafter, for each further record increment `t`, draw `M = rng(t)` uniform in `0..t`, and if `M < k` overwrite `reservoir[M]` with the new record. Handle the short-stream case (fewer than `k` items) by returning everything read, and `k == 0` by returning empty without consulting `rng`.
3. Prime with `while reservoir.len() < k { match it.next() { Some(x) => reservoir.push(x), None => return reservoir } }`, set `t = k`; then `for item in it { t += 1; let m = rng(t); if (m as usize) < k { reservoir[m as usize] = item; } }`. Return the reservoir.
