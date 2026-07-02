# Hints — Module 05: Arithmetic

## Stage 1: Multiple-precision addition and subtraction

1. A big number is a little-endian `Vec<u32>` of base-`b = 2³²` digits in
   canonical form (no trailing zero limbs; the empty vector is 0). Reach for
   Algorithm 4.3.1A's central invariant: the carry out of any digit position is
   always 0 or 1, because `(b−1) + (b−1) + 1 = 2b − 1 < 2b`. Subtraction is the
   same shape with a borrow of 0 or −1.
2. Walk both operands digit by digit up to the longer length, treating missing
   digits as 0; accumulate each column plus the incoming carry in a wider type
   so nothing overflows. `big_cmp` needs no arithmetic: because both inputs are
   canonical, the longer limb vector is strictly larger, and equal lengths
   compare most-significant-limb first. Subtraction must be *definite* — panic
   (message containing "nonnegative") when `u < v`.
3. In `big_add`, hold the running column in a `u64`: `let t = uj + vj + k;`
   then `w.push(t as u32); k = t >> 32;`. Push the final `k` only if nonzero
   (keeps canonical form). In `big_sub`, compute `t = u[j] as i64 - vj + k`; if
   `t < 0`, push `(t + (1<<32)) as u32` and set the borrow `k = -1`, else push
   `t as u32` with `k = 0`; finally pop trailing zero limbs.

## Stage 2: Classical multiplication

1. This is Algorithm 4.3.1M — the O(n·m) schoolbook multiply. The invariant that
   makes it safe is the single-word bound: with `t = uᵢ·vⱼ + w_{i+j} + k`, the
   worst case is `(b−1)² + (b−1) + (b−1) = b² − 1 < b²`, so `t` fits in a `u64`
   and the carry `k = ⌊t/b⌋` stays below `b`.
2. Zero the whole product area (`m + n` limbs) up front, then for each digit
   `vⱼ` of `v` run an inner loop over the digits `uᵢ` of `u`, accumulating into
   `w[i+j]` with a carry that you store into `w[j+m]` when the inner loop ends.
   Trim trailing zero limbs at the very end. `big_to_decimal` is repeated short
   division of the limb vector by `10⁹` (the largest power of ten below 2³²),
   emitting one 9-digit chunk per division.
3. Inner step: `let t = u[i] as u64 * v[j] as u64 + w[i+j] as u64 + k;
   w[i+j] = t as u32; k = t >> 32;`. For the decimal conversion, divide most
   significant limb first: `let cur = (rem << 32) | *d as u64; *d = (cur/CHUNK)
   as u32; rem = cur % CHUNK;`, collect the remainders, print the top chunk
   bare and every later chunk zero-padded with `{c:09}`.

## Stage 3: Faster multiplication by divide and conquer

1. Karatsuba (§4.3.3) beats O(n²) by computing the middle cross-term from the
   two products you already need plus *one* extra, via
   `u₁v₀ + u₀v₁ = (u₁+u₀)(v₁+v₀) − u₁v₁ − u₀v₀`. Three half-size products, not
   four, giving `T(n) = 3T(n/2) + O(n) = Θ(n^{lg 3}) ≈ Θ(n^{1.585})`.
2. Split each operand at limb position `p = max(len)/2` into low and high
   halves. Recurse to get `z0 = u₀v₀`, `z2 = u₁v₁`, and
   `mid = (u₀+u₁)(v₀+v₁)`; then `z1 = mid − z2 − z0`. Recombine with limb
   shifts: `u·v = z2·b^{2p} + z1·b^p + z0`. Below a cutoff of a few dozen limbs
   the classical multiply's smaller constant wins, so fall back to `big_mul`.
3. Reuse your Stage 1/2 primitives: `big_add` for the sums, `big_sub` for
   `z1 = big_sub(&big_sub(&mid, &z2), &z0)`, and a "prepend p zero limbs" helper
   for the shifts. Guard the recursion: `if u.len().min(v.len()) < CUTOFF {
   return big_mul(u, v); }`. When splitting the low half, strip its trailing
   zero limbs so recursion stays canonical.

## Stage 4: The binary gcd algorithm

1. Algorithm 4.5.2B avoids division entirely, using only parity tests, shifts,
   and subtraction. It rests on three identities: `gcd(2u,2v) = 2·gcd(u,v)`,
   `gcd(2u,v) = gcd(u,v)` when `v` is odd, and `gcd(u,v) = gcd(u−v,v)` — and
   `u−v` is even when both are odd, feeding the halving step again.
2. First pull out the common power of two `k` (step B1), then keep a *signed*
   helper `t` holding `+u` or `−v`; repeatedly halve `t` while it is even (B3/B4),
   use its sign to reset the larger of `u,v` (B5), and set `t = u − v` (B6). When
   `t` reaches 0 the answer is `u << k`. Handle `gcd(0,n) = gcd(n,0) = n` first.
3. Use `i128` for `t` so `+u`/`−v` never overflow. Enter at B4 (skip the first
   halving) exactly when the post-B1 `u` is odd, since then `t = -(v as i128)`.
   The loop body: if not entering-at-B4, `t /= 2`; `if t & 1 == 0 { continue; }`;
   then `if t > 0 { u = t as u64 } else { v = (-t) as u64 }`; then
   `t = u as i128 - v as i128; if t == 0 { return u << k; }`.

## Stage 5: Probabilistic primality testing

1. Fermat's test (`a^{n−1} ≡ 1`) can be fooled by Carmichael numbers like 561.
   Miller–Rabin repairs it with the √1 argument: modulo a prime the only square
   roots of 1 are ±1, so a nontrivial square root of 1 exposes a composite. You
   need fast modular exponentiation by repeated squaring, and `mul_mod` via
   `u128` to dodge overflow.
2. Write `n − 1 = 2^s · d` with `d` odd. Compute `x = a^d mod n`; pass
   immediately if `x = 1` or `x = n−1`. Otherwise square up to `s−1` times,
   passing the moment you see `n−1`; if you never do, `n` is composite. For a
   fully deterministic `is_prime_u64`, the fixed witness set
   `{2,3,5,7,11,13,17,19,23,29,31,37}` correctly classifies every `n < 2⁶⁴`.
3. `pow_mod`: `let mut y = 1 % m; a %= m; while e > 0 { if e & 1 == 1 { y =
   mul_mod(y,a,m); } a = mul_mod(a,a,m); e >>= 1; }`. Get `s` from
   `(n-1).trailing_zeros()` and `d = (n-1) >> s`. In `is_prime_u64`, first return
   true/false for the small witness primes themselves (and reject multiples of
   them), then require `is_strong_probable_prime(n, a)` for all twelve bases.
