# Module 05 — Arithmetic

> **Source:** *The Art of Computer Programming*, Vol. 2, 3rd ed., Chapter 4 —
> §4.3.1 (classical algorithms), §4.3.3 (faster multiplication), §4.5.2
> (analysis of Euclid / binary gcd), §4.5.4 (primality testing).
> **Lab:** `labs/module-05-arithmetic` · **Grade it:** `./grade 5`
>
> Self-contained: complete it without the book.

Your machine adds 64-bit numbers in one instruction. But cryptography, computer
algebra, and number theory routinely need integers with *thousands* of digits.
This module builds arithmetic from the digit up — the same algorithms you learned
in grade school, made precise and analyzed — then the divide-and-conquer trick
that beats them, and finally the probabilistic primality test that showed the
world randomness belongs in rigorous algorithms.

Throughout, a big number is a nonnegative integer in **radix b = 2³²**, stored
as a little-endian `Vec<u32>` of *limbs* (Knuth's "digits"), with no trailing
zero limbs. The empty vector is 0. Working in base 2³² instead of base 10 is the
only concession to the machine; every algorithm is exactly Knuth's.

---

## 1. Positional notation and the classical algorithms (§4.3.1)

A number u = (u_{n−1} … u₁ u₀)_b means Σ u_j b^j with 0 ≤ u_j < b. All of
grade-school arithmetic is a set of algorithms on these digit strings, and
Knuth states them with the care they deserve.

### Algorithm A — addition

```text
A1. [Initialize.]  j <- 0, carry k <- 0.
A2. [Add digits.]  Set t <- u_j + v_j + k;  w_j <- t mod b;  k <- t div b.
A3. [Loop.]        j <- j + 1; if j < n repeat A2; else w_n <- k.
```

**The key invariant:** the carry k is always 0 or 1. Proof: the largest digit
sum is (b−1) + (b−1) + 1 = 2b − 1 < 2b, so ⌊t/b⌋ ≤ 1. That one-bit carry is why
addition is O(n) and why hardware adders chain a single carry line.

### Algorithm S — subtraction

Identical shape with a *borrow* instead of a carry. We compute u − v for u ≥ v;
if v > u the result would be negative, so the lab's `big_sub` panics (message
containing "nonnegative") — definiteness again.

### Algorithm M — multiplication

```text
M1. [Initialize.]  w_0 ... w_{m-1} <- 0.
M2. [For each v_i:] carry k <- 0.
M3. [For each u_j:] t <- u_j * v_i + w_{i+j} + k;  w_{i+j} <- t mod b;  k <- t div b.
M4. [Store carry.] w_{i+m} <- k.
```

Why does the inner carry fit in one digit? The worst case is
(b−1)·(b−1) + (b−1) + (b−1) = b² − 1 < b², so t < b² and ⌊t/b⌋ < b. In Rust we
compute `t` in `u64` (since (2³²−1)² fits in u64) and the carry stays below 2³².
This "schoolbook" multiply is **O(n·m)**: every digit of u meets every digit of v.

Trace M in base 10 on 12 × 34: 12·4 = 48, 12·3 = 36 shifted → 048 + 360 = 408.
The lab checks your `big_mul` by computing 50! exactly (that
`30414093201713378043612608166064768844377641568960512000000000000` is not a
float — it is your limbs, decoded by `big_to_decimal`).

---

## 2. Faster multiplication: Karatsuba (§4.3.3)

Is O(n²) the best we can do? For centuries everyone assumed so. In 1960 the
23-year-old Karatsuba found otherwise. Split each n-digit number in half about
the middle digit position m:

    u = u₁·bᵐ + u₀,     v = v₁·bᵐ + v₀.

The schoolbook expansion needs four half-size products:

    uv = u₁v₁·b²ᵐ + (u₁v₀ + u₀v₁)·bᵐ + u₀v₀.

Karatsuba's observation: the middle term is obtainable from the two you already
need plus **one** more product, via the identity

    u₁v₀ + u₀v₁ = (u₁ + u₀)(v₁ + v₀) − u₁v₁ − u₀v₀.

So **three** half-size multiplications suffice, not four (the additions are
cheap, O(n)). The running time obeys

    T(n) = 3·T(n/2) + O(n).

By the recursion-tree / master argument, the leaves dominate and
T(n) = Θ(n^{log₂ 3}) = Θ(n^{1.585}). For thousand-limb numbers that is a large
win; below a cutoff (a few dozen limbs) the classical method's smaller constant
wins, so real implementations — and your `big_mul_karatsuba` — switch over.

> Knuth continues to Toom–Cook (n^{1+ε}) and the FFT-based methods
> (O(n log n log log n), and since 2019 O(n log n)). Karatsuba is the gateway.

---

## 3. The binary gcd (§4.5.2)

You proved Euclid correct in Module 01. But division is the most expensive
integer operation on real hardware. Josef Stein's 1967 **binary gcd** avoids it
entirely, using only subtraction, parity tests, and shifts:

```text
B1. [Find power of 2.]  If u and v are both even, gcd = 2·gcd(u/2, v/2);
                        pull out the common factor 2^k once, up front.
B2. [Remove 2s.]        Now at least one is odd. Halve whichever are even
                        (a factor of 2 in one operand can't be common).
B3. [Subtract.]         With both odd, the larger minus the smaller is even
                        and smaller than the larger; reduce and repeat.
B6. [Done.]             When one operand hits 0, the other times 2^k is gcd.
```

Correctness rests on three facts, one per case: gcd(2u,2v) = 2·gcd(u,v);
gcd(2u,v) = gcd(u,v) when v is odd; and gcd(u,v) = gcd(u−v,v). Each step either
halves a value or replaces the pair by a smaller one, so it terminates in
O(log(uv)) steps — and every step is a shift or subtract. Convention:
gcd(0,n) = n. The lab's `binary_gcd` must agree with Euclid on a whole grid.

---

## 4. Primality testing (§4.5.4) — and why randomness is respectable

How do you know a 300-digit number is prime? Trial division is hopeless. The
breakthrough is **Fermat's little theorem**: if p is prime, then for any a not
divisible by p,

    a^{p−1} ≡ 1 (mod p).

So if you find an a with a^{n−1} ≢ 1 (mod n), then n is *certainly* composite —
a is a "witness." (You'll need fast modular exponentiation `pow_mod` by
repeated squaring, and `mul_mod` via u128 to avoid overflow.) The trouble: some
composites pass Fermat's test for *every* a coprime to n — the **Carmichael
numbers**, the smallest being 561 = 3·11·17. Fermat alone can be fooled.

**Miller–Rabin** repairs this with one extra idea. Write n − 1 = 2^s · d with d
odd. If n is prime, then in the field ℤ/nℤ the only square roots of 1 are ±1. So
the sequence a^d, a^{2d}, a^{4d}, …, a^{2^{s−1}d} (mod n) must either start at 1,
or hit n−1 somewhere before it can reach 1. If it does neither, we've found a
nontrivial square root of 1 — impossible modulo a prime — so n is composite, and
a is a *strong* witness. This is `is_strong_probable_prime(n, a)`.

The beautiful theorem: **at least ¾ of the bases in [2, n−1) are strong
witnesses for any odd composite n.** So k random bases fail to expose a composite
with probability < 4^{−k}: at k = 30 that's less than one in 10¹⁸. This was
Knuth's banner example (§4.5.4) that a *probabilistic* algorithm can be more
practical — and every bit as rigorous — than a deterministic one. It helped
launch the whole field of randomized algorithms.

For 64-bit inputs we can even be **deterministic**: testing the fixed base set
{2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37} is proven to correctly classify
every n < 2⁶⁴ (Jaeschke, refined by Sinclair — a post-Knuth result). That's
`is_prime_u64`. The lab pins down the subtle cases: 2047 = 23·89 fools base 2
but not the full test; 561 is caught; 2⁶¹−1 is prime; 2⁶¹+1 = 3·715827883·… is
not.

---

## 5. Stage-by-stage lab guide

Open `labs/module-05-arithmetic/src/lab.rs`. Keep results **canonical** (strip
trailing zero limbs) — many tests check it.

- **Stage 1 — `big_add`, `big_sub`, `big_cmp`, `big_from_u128`, `big_to_u128`.**
  Algorithms A and S. Watch the carry/borrow; panic on negative subtraction.
- **Stage 2 — `big_mul`, `big_to_decimal`.** Algorithm M with a u64 accumulator.
  `big_to_decimal` repeatedly divides by 10 (or 10⁹ per limb for speed).
- **Stage 3 — `big_mul_karatsuba`.** Split-in-half, three recursive products,
  classical below a cutoff. Must agree with `big_mul` on every size.
- **Stage 4 — `binary_gcd`.** Stein's algorithm, shifts and subtracts only.
- **Stage 5 — `mul_mod`, `pow_mod`, `is_strong_probable_prime`, `is_prime_u64`.**
  Modular exponentiation, then Miller–Rabin with the deterministic witness set.

Run `./grade 5`; stages unlock in order.

---

## 6. Check your understanding

1. In Algorithm A, why is one extra limb (w_n) always enough for the result?
2. Karatsuba does three multiplications and several additions per level. Why do
   the additions not change the Θ(n^{1.585}) exponent?
3. Give the three gcd identities that justify the binary gcd's three cases.
4. Why does a nontrivial square root of 1 modulo n prove n is composite?
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

## 8. Where this leads

- **Extended Euclid** from Module 01 gives modular inverses; combined with this
  module's `pow_mod` you have the arithmetic core of RSA.
- **Karatsuba** opens onto Toom–Cook and FFT multiplication (§4.3.3 continues),
  the reason big-integer libraries are fast.
- **Miller–Rabin** and Pollard's rho lead into the factoring material and the
  randomized-algorithm viewpoint that pervades modern cryptography.
