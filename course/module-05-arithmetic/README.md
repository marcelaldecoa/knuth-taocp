# Module 05 — Arithmetic

> **Source:** *The Art of Computer Programming*, Vol. 2, 3rd ed., Chapter 4 —
> §4.3.1 (classical algorithms), §4.3.3 (faster multiplication), §4.5.2
> (analysis of Euclid / binary gcd), §4.5.4 (primality testing).
> **Lab:** `labs/module-05-arithmetic` · **Grade it:** `./grade 5`
> **Concrete Mathematics companion:** Chapter 4 (Number Theory) — gcd, modular
> arithmetic, and primality, in depth — see [../../docs/concrete-mathematics.md](../../docs/concrete-mathematics.md).
>
> Self-contained: complete it without the book.

Your machine adds 64-bit numbers in one instruction. But cryptography, computer
algebra, and number theory routinely need integers with *thousands* of digits.
This module builds arithmetic from the digit up — the same algorithms you learned
in grade school, made precise and analyzed — then the divide-and-conquer trick
that beats them, and finally the probabilistic primality test that showed the
world randomness belongs in rigorous algorithms.

> **Companion exhibit — _Fermat's Clock_.** The asymmetry this whole module
> serves — easy to multiply, murderous to factor — is made tactile in the
> Museum's [Cryptography Gear](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-2.3-fermats-clock.html):
> attack a public key $N = p \cdot q$ by trial division and watch a 32-bit key crack
> in a second, a 52-digit key report three billion years, and RSA-2048 report a
> time beyond counting — all on exact BigInt, bounded so the tab never freezes.
> It also closes the loop to Module 01: the private key is a modular inverse,
> computed by the extended Euclidean algorithm (§4.5.2).

Throughout, a big number is a nonnegative integer in **radix $b = 2^{32}$**, stored
as a little-endian `Vec<u32>` of *limbs* (Knuth's "digits"), with no trailing
zero limbs. The empty vector is 0. Working in base $2^{32}$ instead of base 10 is the
only concession to the machine; every algorithm is exactly Knuth's.

---

## 1. Positional notation and the classical algorithms (§4.3.1)

A number $u = (u_{n-1} \ldots u_1 u_0)_b$ means $\sum u_j b^j$ with $0 \le u_j < b$. All of
grade-school arithmetic is a set of algorithms on these digit strings, and
Knuth states them with the care they deserve.

### Algorithm A — addition

```text
A1. [Initialize.]  j <- 0, carry k <- 0.
A2. [Add digits.]  Set t <- u_j + v_j + k;  w_j <- t mod b;  k <- t div b.
A3. [Loop.]        j <- j + 1; if j < n repeat A2; else w_n <- k.
```

**The key invariant:** the carry $k$ is always 0 or 1. Proof: the largest digit
sum is $(b-1) + (b-1) + 1 = 2b - 1 < 2b$, so $\lfloor t/b \rfloor \le 1$. That one-bit carry is why
addition is $O(n)$ and why hardware adders chain a single carry line.

### Algorithm S — subtraction

Identical shape with a *borrow* instead of a carry. We compute $u - v$ for $u \ge v$;
if $v > u$ the result would be negative, so the lab's `big_sub` panics (message
containing "nonnegative") — definiteness again.

### Algorithm M — multiplication

```text
M1. [Initialize.]  w_0 ... w_{m-1} <- 0.
M2. [For each v_i:] carry k <- 0.
M3. [For each u_j:] t <- u_j * v_i + w_{i+j} + k;  w_{i+j} <- t mod b;  k <- t div b.
M4. [Store carry.] w_{i+m} <- k.
```

Why does the inner carry fit in one digit? The worst case is
$(b-1) \cdot (b-1) + (b-1) + (b-1) = b^2 - 1 < b^2$, so $t < b^2$ and $\lfloor t/b \rfloor < b$. In Rust we
compute `t` in `u64` (since $(2^{32}-1)^2$ fits in u64) and the carry stays below $2^{32}$.
This "schoolbook" multiply is **$O(n \cdot m)$**: every digit of $u$ meets every digit of $v$.

Trace M in base 10 on $12 \times 34$: $12 \cdot 4 = 48$, $12 \cdot 3 = 36$ shifted $\to 048 + 360 = 408$.
The lab checks your `big_mul` by computing $50!$ exactly (that
`30414093201713378043612608166064768844377641568960512000000000000` is not a
float — it is your limbs, decoded by `big_to_decimal`).

---

## 2. Faster multiplication: Karatsuba (§4.3.3)

Is $O(n^2)$ the best we can do? For centuries everyone assumed so. In 1960 the
23-year-old Karatsuba found otherwise. Split each $n$-digit number in half about
the middle digit position $m$:

$$u = u_1 \cdot b^m + u_0, \qquad v = v_1 \cdot b^m + v_0.$$

The schoolbook expansion needs four half-size products:

$$uv = u_1 v_1 \cdot b^{2m} + (u_1 v_0 + u_0 v_1) \cdot b^m + u_0 v_0.$$

Karatsuba's observation: the middle term is obtainable from the two you already
need plus **one** more product, via the identity

$$u_1 v_0 + u_0 v_1 = (u_1 + u_0)(v_1 + v_0) - u_1 v_1 - u_0 v_0.$$

So **three** half-size multiplications suffice, not four (the additions are
cheap, $O(n)$). The running time obeys

$$T(n) = 3 \cdot T(n/2) + O(n).$$

By the recursion-tree / master argument, the leaves dominate and
$T(n) = \Theta(n^{\log_2 3}) = \Theta(n^{1.585})$. For thousand-limb numbers that is a large
win; below a cutoff (a few dozen limbs) the classical method's smaller constant
wins, so real implementations — and your `big_mul_karatsuba` — switch over.

> Knuth continues to Toom–Cook ($n^{1+\varepsilon}$) and the FFT-based methods
> ($O(n \log n \log \log n)$, and since 2019 $O(n \log n)$). Karatsuba is the gateway.

---

## 3. The binary gcd (§4.5.2)

You proved Euclid correct in Module 01. But division is the most expensive
integer operation on real hardware. The **binary gcd** (published by Josef
Stein in 1967; Knuth notes the idea may go back to first-century China) avoids
division entirely, using only subtraction, parity tests, and shifts. Here is
Knuth's Algorithm B, for positive integers $u$ and $v$ ($t$ is a signed helper
variable holding $+u$ or $-v$):

```text
B1. [Find power of 2.]  Set k <- 0; then repeatedly set u <- u/2, v <- v/2,
                        k <- k+1, until u and v are not both even.
B2. [Initialize.]       If u is odd, set t <- -v and go to B4;
                        otherwise set t <- u.
B3. [Halve t.]          (t is even and nonzero.) Set t <- t/2.
B4. [Is t even?]        If t is even, go back to B3.
B5. [Reset max(u,v).]   If t > 0, set u <- t; otherwise set v <- -t.
                        (The larger of u, v is replaced by |t|; both odd now.)
B6. [Subtract.]         Set t <- u - v. If t != 0, go back to B3.
                        Otherwise the answer is u * 2^k.
```

Correctness rests on three gcd identities, one per case:

1. $\gcd(2u, 2v) = 2 \cdot \gcd(u, v)$ — step B1 pulls out common 2s;
2. $\gcd(2u, v) = \gcd(u, v)$ when $v$ is odd — steps B3/B4 discard useless 2s;
3. $\gcd(u, v) = \gcd(u - v, v)$ — step B6, and $u - v$ is *even*
   when $u, v$ are both odd, feeding B3 again.

Each trip through B3–B6 halves $|t|$ at least once and never increases
$\max(u, v)$, so the algorithm terminates after $O(\log uv)$ shift/subtract steps.

**Hand trace on $u = 48$, $v = 18$.** B1: both even $\to (24, 9)$, $k = 1$; stop (9 is
odd). B2: $u = 24$ even $\to t = 24$. B3/B4: $24 \to 12 \to 6 \to 3$ (odd). B5: $t > 0 \to$
$u = 3$. B6: $t = 3 - 9 = -6 \ne 0$. B3/B4: $-6 \to -3$ (odd). B5: $t < 0 \to v = 3$.
B6: $t = 3 - 3 = 0 \to$ answer $3 \cdot 2^1 =$ **6**. ✓ ($\gcd(48,18) = 6$.)

Knuth's own worked example is $\gcd(40902, 24140) = 34 = 2 \cdot 17$ — the lab pins it,
and `binary_gcd` must also agree with Euclid on a whole grid. Convention
(consistent with §4.5.2): $\gcd(0, n) = \gcd(n, 0) = n$.

---

## 4. Primality testing (§4.5.4) — and why randomness is respectable

How do you know a 300-digit number is prime? Trial division is hopeless. The
breakthrough is **Fermat's little theorem**: if $p$ is prime, then for any $a$ not
divisible by $p$,

$$a^{p-1} \equiv 1 \pmod{p}.$$

So if you find an $a$ with $a^{n-1} \not\equiv 1 \pmod{n}$, then $n$ is *certainly* composite —
$a$ is a "witness." (You'll need fast modular exponentiation `pow_mod` by
repeated squaring, and `mul_mod` via u128 to avoid overflow.) The trouble: some
composites pass Fermat's test for *every* $a$ coprime to $n$ — the **Carmichael
numbers**, the smallest being $561 = 3 \cdot 11 \cdot 17$. Fermat alone can be fooled.

**Miller–Rabin** repairs this with one extra idea. Write $n - 1 = 2^s \cdot d$ with $d$
odd. If $n$ is prime, then in the field $\mathbb{Z}/n\mathbb{Z}$ the only square roots of 1 are $\pm 1$. So
the sequence $a^d, a^{2d}, a^{4d}, \ldots, a^{2^{s-1}d} \pmod{n}$ must either start at 1,
or hit $n-1$ somewhere before it can reach 1. If it does neither, we've found a
nontrivial square root of 1 — impossible modulo a prime — so $n$ is composite, and
$a$ is a *strong* witness. This is `is_strong_probable_prime(n, a)`.

The beautiful theorem: **at least $\tfrac{3}{4}$ of the bases in $[2, n-1)$ are strong
witnesses for any odd composite $n$.** So $k$ random bases fail to expose a composite
with probability $< 4^{-k}$: at $k = 30$ that's less than one in $10^{18}$. This was
Knuth's banner example (§4.5.4) that a *probabilistic* algorithm can be more
practical — and every bit as rigorous — than a deterministic one. It helped
launch the whole field of randomized algorithms.

For 64-bit inputs we can even be **deterministic**: testing the fixed base set
$\{2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37\}$ is proven to correctly classify
every $n < 2^{64}$ (Jaeschke, refined by Sinclair — a post-Knuth result). That's
`is_prime_u64`. The lab pins down the subtle cases: $2047 = 23 \cdot 89$ fools base 2
but not the full test; 561 is caught; $2^{61}-1$ is prime; $2^{61}+1 = 3 \cdot 715827883 \cdot \ldots$ is
not.

---

## 5. Stage-by-stage lab guide

Open `labs/module-05-arithmetic/src/lab.rs`. Keep results **canonical** (strip
trailing zero limbs) — many tests check it.

- **Stage 1 — `big_add`, `big_sub`, `big_cmp`, `big_from_u128`, `big_to_u128`.**
  Algorithms A and S. Watch the carry/borrow; panic on negative subtraction.
- **Stage 2 — `big_mul`, `big_to_decimal`.** Algorithm M with a u64 accumulator.
  `big_to_decimal` repeatedly divides by 10 (or $10^9$ per limb for speed).
- **Stage 3 — `big_mul_karatsuba`.** Split-in-half, three recursive products,
  classical below a cutoff. Must agree with `big_mul` on every size.
- **Stage 4 — `binary_gcd`.** Stein's algorithm, shifts and subtracts only.
- **Stage 5 — `mul_mod`, `pow_mod`, `is_strong_probable_prime`, `is_prime_u64`.**
  Modular exponentiation, then Miller–Rabin with the deterministic witness set.

Run `./grade 5`; stages unlock in order.

---

## 6. Check your understanding

1. In Algorithm A, why is one extra limb ($w_n$) always enough for the result?
2. Karatsuba does three multiplications and several additions per level. Why do
   the additions not change the $\Theta(n^{1.585})$ exponent?
3. Give the three gcd identities that justify the binary gcd's three cases.
4. Why does a nontrivial square root of 1 modulo $n$ prove $n$ is composite?
5. 561 passes the Fermat test for all coprime bases but fails Miller–Rabin.
   What does the strong test see that the Fermat test misses?

## 7. Exercises from the text

Ratings: 00 immediate · 20 an hour · 30 hours · 40 term project · 50 open.
▶ = instructive. Log attempts in `exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 4.3.1–? | 15 | Prove the carry in Algorithm A never exceeds 1. |
| ▶4.3.1–? | 25 | Analyze Algorithm M's exact number of digit multiplications. |
| 4.3.3–? | 30 | Work out the constant in Karatsuba's recurrence; find the crossover with classical multiply empirically. |
| ▶4.5.2–? | 28 | Compare the number of steps of binary gcd vs Euclid on Fibonacci pairs. |
| 4.5.4–? | 35 | Implement Pollard's rho factoring and factor a 60-bit semiprime. |

## Why it's done this way

Base $2^{32}$ limbs in a `Vec<u32>` are Knuth's "base-b digits" with $b$ chosen so
that a limb product fits in a hardware word pair — the exact consideration
(§4.3.1) that made him analyze carry bounds so carefully. The canonical-form
rule (no leading zero limbs) is definiteness applied to data: one value, one
representation, testable equality. And the classical-before-clever ordering
(Algorithm M before Karatsuba) is the chapter's thesis in miniature: you
cannot appreciate — or correctly cut over to — the $O(n^{1.585})$ algorithm
until you know exactly what the $O(n^2)$ one costs.

## In the real world

Your stage 1–3 functions are a miniature GMP: real bignum libraries are
Algorithms A/S/M plus Karatsuba (then Toom–Cook and FFT beyond some size),
with the same cutover tuning your Karatsuba threshold models. Binary gcd
variants run inside cryptographic libraries where division timing leaks
secrets. Miller–Rabin with a deterministic witness set is not an
approximation of practice — it *is* practice: mainstream crypto libraries
run exactly this test on candidate primes for your TLS keys, and the
u64-witness result you verified is the reason competitive programmers treat
64-bit primality as a solved $O(12 \cdot \log n)$ subroutine.

## Proof techniques you practiced

- **Invariant bounds on intermediate values** — the carry fits because
  $(b-1)^2 + (b-1) + (b-1) = b^2 - 1 < b^2$: arithmetic correctness as an
  inequality you prove once and assert everywhere.
- **Divide-and-conquer recurrences** — $T(n) = 3T(n/2) + O(n)$ solved by the
  recursion tree; the exponent $\lg 3$ falls out, not in.
- **Exhaustive case analysis** — binary gcd's three identities, one per
  parity case, welded into a terminating whole.
- **The $\sqrt{1}$ argument** — a nontrivial square root of 1 mod $n$ factors $n$:
  group theory converted directly into a composite-detector.
- **Quantified probabilistic guarantees** — error $\le 4^{-k}$ per $k$ rounds, and
  the honest asymmetry: "probably prime" vs "certainly composite".

## 8. Where this leads

- **Extended Euclid** from Module 01 gives modular inverses; combined with this
  module's `pow_mod` you have the arithmetic core of RSA.
- **Karatsuba** opens onto Toom–Cook and FFT multiplication (§4.3.3 continues),
  the reason big-integer libraries are fast.
- **Miller–Rabin** and Pollard's rho lead into the factoring material and the
  randomized-algorithm viewpoint that pervades modern cryptography.
