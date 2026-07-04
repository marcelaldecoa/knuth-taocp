# Module 08 — Combinatorial Generation

> **Source:** *The Art of Computer Programming*, Vol. 4A, §7.2.1 — specifically
> §7.2.1.1 (Gray binary code), §7.2.1.2 (permutations: lexicographic and plain
> changes), §7.2.1.3 (combinations), and §7.2.1.4 (integer partitions).
> **Lab:** `labs/module-08-generation` · **Grade it:** `./grade 8`
> **Concrete Mathematics companion:** Chapters 5 (Binomial Coefficients) and 6
> (Special Numbers) — the combinatorics this module generates — see [../../docs/concrete-mathematics.md](../../docs/concrete-mathematics.md).
>
> This lesson is self-contained: you can complete the module without the book.
> If you own Vol. 4A, read §7.2.1 alongside — it is the heart of the "Generating
> All Combinatorial Objects" material and this lesson tells you where each idea
> lives.

Earlier modules *searched* structures (sorting, trees, hashing). This module
*generates* them: given a combinatorial family — all subsets, all permutations,
all combinations, all partitions of an integer — visit every member exactly
once. The recurring craft is to pass from one object to the "next" with as
little work as possible, ideally O(1) amortized, and often changing only *one
small feature* between consecutive objects. By the end you will have generated
five classical families the way Knuth does, each with a step-labeled algorithm
you can trace on paper.

> **Companion exhibit — _The Traveling Salesperson and the Ant_.** Lexicographic
> permutation generation (§7.2.1.2) is not an abstraction in the Museum's
> [Lost Time Machine](https://marcelaldecoa.github.io/knuth-taocp/museum/exhibit-4.1-traveling-salesperson.html):
> it enumerates every tour of a set of cities by exactly the next-permutation
> step you build here, bounded per animation frame so it never freezes. Watch
> the count `(n−1)!/2` detonate — 43 billion tours at 15 cities, past the age of
> the universe beyond ~25 — while a nearest-neighbour "ant" answers instantly.
> It is combinatorial *generation* meeting combinatorial *explosion*, and the
> visceral face of P vs NP.

---

## 1. Three things that are not the same

Knuth opens §7.2 by separating three tasks people casually lump together:

1. **Generating** all objects of a family, each exactly once. (This module.)
2. **Counting** them — often far cheaper than listing them. `p(100)` has
   9 digits; you would die before printing all `190{,}569{,}292` partitions,
   yet the *count* falls out of a recurrence in microseconds (Stage 5).
3. **Random selection** — producing one uniformly-random member without
   enumerating the rest (Module 04's territory).

A generation algorithm is judged on two axes: the **order** it visits objects
in (lexicographic? Gray-code-like minimal-change?) and the **cost per object**.
The gold standard is a *loopless* or *combinatorial Gray code* algorithm: each
successive object differs from the last by a single, local change, produced in
O(1) worst-case time. Stage 1 (Gray binary) and Stage 3 (plain changes) are
minimal-change; Stages 2, 4, 5 are lexicographic.

A note on totals, so the sizes never surprise you:

| family | count | example |
|---|---|---|
| subsets of an n-set | 2ⁿ | Stage 1 |
| permutations of n | n! | Stages 2, 3 |
| k-subsets of an n-set | C(n,k) | Stage 4 |
| partitions of n | p(n) | Stage 5 |

All grow fast, so the lab caps sizes (permutations at n ≤ 7 = 5040).

---

## 2. Gray binary code (§7.2.1.1)

### The problem with counting in binary

To visit all 2ⁿ binary strings you could just count 0, 1, 2, 3, … in binary.
But 0111 → 1000 flips **four** bits at once. In a physical system — a
rotary encoder, a bank of relays, a Karnaugh map — you want consecutive
codewords to differ in exactly **one** bit, so no transient garbage states
appear. Frank Gray's 1947 patent (for pulse-code communication) gives such a
code; it had been used even earlier by Émile Baudot (1878).

**Definition.** The *reflected binary Gray code* of order n lists all 2ⁿ
binary n-tuples so that consecutive tuples (and the last and first, cyclically)
differ in exactly one coordinate.

### Reflection construction

Build Γₙ recursively:

- Γ₁ = (0, 1).
- Γₙ₊₁ = take Γₙ, prefix every word with 0; then take Γₙ **reversed**, prefix
  every word with 1; concatenate.

For n = 2: Γ₁ = 0, 1 gives `0`0, `0`1 then reflected `1`1, `1`0, i.e.

```
00, 01, 11, 10   =   0, 1, 3, 2
```

For n = 3, prefix Γ₂ with 0, then reflected Γ₂ with 1:

```
000, 001, 011, 010,  110, 111, 101, 100
```

**Why one bit changes.** Within the first half only the low n bits move (they
are a Gray code by induction). Within the second half likewise. At the seam,
the reflection guarantees the two words straddling it are identical except for
the freshly-prefixed top bit — that is the whole point of reversing the second
copy. Induction closes the argument. ∎

### The closed form (and its proof)

The reflection construction has a beautiful one-line description:

> **Theorem.** The k-th word of Γₙ is `g(k) = k XOR ⌊k/2⌋ = k ^ (k >> 1)`.

*Proof.* We show consecutive `g(k)` differ in one bit, `g(0) = 0`, and `g` is a
bijection on {0,…,2ⁿ−1}; those three facts pin down the reflected code.
Consider `g(k) XOR g(k−1)`. Writing `g(k) = k ^ (k>>1)`,

```
g(k) XOR g(k-1) = (k XOR (k-1)) XOR ((k>>1) XOR ((k-1)>>1)).
```

Let ρ(k) be the number of trailing zero bits of k (the "ruler function":
ρ = 0,1,0,2,0,1,0,3,… for k = 1,2,3,…). Incrementing k−1 to k flips exactly the
lowest ρ(k)+1 bits (a run of 1s becomes 0s and the next 0 becomes 1). Working
the XORs out, all but one of those cancel between the two terms, leaving a
single set bit at position ρ(k). So consecutive words differ in exactly bit
ρ(k) — one bit. And `g` is invertible (next section), hence a bijection. ∎

The flipped-bit-is-ρ(k) fact is worth remembering: it says the bit that changes
at step k is governed by the *ruler sequence*, exactly the pattern of tick
heights on an imperial ruler.

### Inverting g: the rank

Given a codeword, which k produced it? Solve `g(k) = k ^ (k>>1)` from the top
bit down. Bit n−1 of k equals bit n−1 of the word; each lower bit is the word's
bit XOR the already-known next-higher bit of k. Unrolling gives the elegant
prefix-XOR

```
k = word ^ (word>>1) ^ (word>>2) ^ (word>>3) ^ ...
```

which is exactly what `gray_rank` computes (shift-and-XOR until zero).

### Algorithm 7.2.1.1G — counterless generation

Knuth's Algorithm G produces the sequence *in place* with no loop counter at
all, using only a parity bit `a_inf` (the parity of the current word's
popcount):

```text
G1. [Initialize.]     a_j <- 0 for 0 <= j < n; a_inf <- 0.
G2. [Visit.]          Visit (a_{n-1}, ..., a_1, a_0).
G3. [Change parity.]  a_inf <- 1 - a_inf.
G4. [Choose j.]       If a_inf = 1, j <- 0;
                      else j <- 1 + (position of rightmost 1-bit of a).
G5. [Complement.]     Terminate if j = n; else a_j <- 1 - a_j, go to G2.
```

The trick: half the steps (odd k) just flip bit 0; the other half flip the bit
just left of the lowest 1. That is exactly bit ρ(k), reproduced without ever
computing k. In Rust the "position of rightmost 1" is `a.trailing_zeros()`.

**Hand trace, n = 3** (writing the word in binary, low bit right):

| k | a (before visit) | a_inf after G3 | j chosen | flip |
|---|---|---|---|---|
| 0 | 000 | 1 | 0 | bit 0 |
| 1 | 001 | 0 | 1+0 = 1 | bit 1 |
| 2 | 011 | 1 | 0 | bit 0 |
| 3 | 010 | 0 | 1+1 = 2 | bit 2 |
| 4 | 110 | 1 | 0 | bit 0 |
| 5 | 111 | 0 | 1+0 = 1 | bit 1 |
| 6 | 101 | 1 | 0 | bit 0 |
| 7 | 100 | 0 | 1+2 = 3 = n | **stop** |

Read the "a before visit" column: 000, 001, 011, 010, 110, 111, 101, 100 — the
reflected code, as promised.

### Applications

Karnaugh maps (adjacent cells differ in one variable), mechanical/optical
position encoders, tower-of-Hanoi move sequences (move k flips disk ρ(k)),
genetic-algorithm encodings that avoid "Hamming cliffs", and error-tolerant
analog-to-digital conversion.

---

## 3. Permutations in lexicographic order (§7.2.1.2)

### Algorithm L

To list all arrangements of a sequence in dictionary order, repeatedly compute
the *next* one. The insight (known to 14th-century Indian prosodists, and to
Narayana Pandita): the last block that is already descending, `a_{j+1} ≥ … ≥
a_n`, is "maxed out" — no rearrangement of it alone can increase the string.
So find the pivot `a_j` just left of it, bump it up minimally, and reset the
tail to its smallest arrangement.

```text
L2. [Find j.]    j <- n-1; while a_j >= a_{j+1}, j <- j-1. Terminate if j = 0.
L3. [Increase.]  l <- n; while a_j >= a_l, l <- l-1. Swap a_j <-> a_l.
L4. [Reverse.]   Reverse a_{j+1} ... a_n.
```

(1-based indices, as in Knuth.) Step L2 finds the **longest decreasing
suffix**; `j` is the position just before it. L3 finds the rightmost element of
that suffix still larger than `a_j` — since the suffix is decreasing, this is
the smallest element exceeding `a_j`, the correct minimal increase. After the
swap the suffix is still decreasing, so L4 reversing it makes it *increasing* =
the smallest possible tail. Result: the immediate lexicographic successor.

**Hand trace on 1 3 4 2** (n = 4):
- L2: compare from the right. a₃a₄ = 4,2 descending; a₂a₃ = 3,4 **ascending** —
  stop, j = 2 (the value 3). Suffix is `4 2`.
- L3: scan the suffix for the rightmost value > 3: that is 4 at position 3.
  Swap → `1 4 3 2`.
- L4: reverse the tail after position 2: `3 2` → `2 3`. Result **1 4 2 3**. ✓

### Why `≥` makes it correct for multisets

Notice L2 and L3 compare with `≥`, not `>`. On a set this is irrelevant (no
ties). On a **multiset** — say `1 2 2 3` — it is exactly what makes each
*distinct* arrangement appear once. With `≥`, equal elements are treated as an
already-maximal (non-strictly-descending) run, so the algorithm never produces
two orderings that differ only by swapping equal elements.

> **Theorem.** Started from the sorted multiset and iterated until L2
> terminates, Algorithm L visits every distinct arrangement of the multiset
> exactly once, in increasing lexicographic order.

*Proof sketch.* Each step produces a string strictly greater than the current
one (L3 raises position j and L4 minimizes the suffix, so the result is the
least string exceeding the current one). Strictly increasing ⇒ no repeats.
And it is the *least* greater string ⇒ nothing is skipped. Termination at the
non-increasing string (the lexicographically greatest) means all are seen. ∎

For `1 1 2` this yields `1 1 2`, `1 2 1`, `2 1 1` — three, which is
3!/2! = 3. For `1 2 2 3`: twelve arrangements, 4!/2! = 12.

---

## 4. Plain changes — minimal-change permutations (§7.2.1.2)

Lexicographic order can move many elements between neighbours (`1 2 3 4` →
`1 2 4 3` is fine, but `1 4 3 2` → `2 1 3 4` moves everything). **Plain
changes** — the Steinhaus–Johnson–Trotter order — visits all n! permutations so
that each differs from the last by one **adjacent transposition** (swap of two
neighbours). This is a *combinatorial Gray code* for permutations.

### The weaving picture

Think of it recursively. Suppose you have all (n−1)! permutations of
1…(n−1), each one adjacent-swap from the last. To get permutations of 1…n,
take element n and *weave* it: in the first permutation slide n from the far
right to the far left (n−1 adjacent swaps, visiting n positions); advance the
smaller elements by one plain-change step; slide n back left-to-right; and so
on, boustrophedon (as an ox plows). Because n only ever moves one step, and the
"advance the rest" step is itself a single adjacent swap by induction, the
whole listing is single-adjacent-swap throughout.

For n = 3, weaving 3 through the two permutations `12`, `21` of {1,2}:

```
123, 132, 312,   (3 slides left through 12)
321, 231, 213    (advance to 21, then 3 slides right)
```

Reading down: **123, 132, 312, 321, 231, 213**. Each neighbour is one adjacent
swap. And the last, `213`, is one adjacent swap from the first `123` — so the
sequence is a Hamiltonian *cycle* on the permutations.

### Algorithm P — the control tables

Doing the weave without recursion uses two odometer-like tables: `c_j` = how far
element j has moved in its current sweep (0 ≤ c_j < j), `o_j = ±1` = its current
direction, and an offset `s` counting bigger elements parked at the left.

```text
P1. [Initialize.]       a_j <- j, c_j <- 0, o_j <- 1  (1 <= j <= n).
P2. [Visit.]            Visit a_1 ... a_n.
P3. [Prepare.]          j <- n, s <- 0.
P4. [Ready to change?]  q <- c_j + o_j.  If q < 0 go to P7; if q = j go to P6.
P5. [Change.]           swap a_{j-c_j+s} <-> a_{j-q+s}; c_j <- q; return to P2.
P6. [Increase s.]       Terminate if j = 1; else s <- s + 1.
P7. [Switch direction.] o_j <- -o_j, j <- j - 1; go to P4.
```

The largest element that still has "room to move" (P4 finds it by walking j
down from n) takes one step in its current direction (P5). When an element hits
the end of its track (q = j or q < 0) it reverses (P7) and control passes to a
smaller element — precisely the boustrophedon weave, made iterative. The final
permutation is always `2 1 3 4 … n`.

Plain changes is how English **change-ringing** works: teams of bell-ringers
have rung "plain changes" and "Grandsire" on church bells since the 1600s,
sounding every permutation of the bells with only adjacent bells ever swapping
(you cannot physically reorder a swinging bell by more than one position per
round). Fabian Stedman's *Tintinnalogia* (1668) is essentially a treatise on
permutation generation, three centuries before computers.

---

## 5. Combinations (§7.2.1.3)

A **combination** is a k-subset of {0, 1, …, n−1}. There are C(n,k) of them.
Represent each as its sorted list c₁ < c₂ < … < c_k, or equivalently as a
bitstring of n bits with exactly k ones (bit i set ⇔ i is chosen). A third
picture: a lattice path from corner to corner of a k×(n−k) grid — each
combination is a monotone staircase. These three views (subset / bitstring /
path) are constantly interchanged in practice.

### Algorithm T — colex order

Knuth's Algorithm T visits combinations in **colexicographic** order: compare
the reversed strings c_k…c₁, i.e. sort primarily by the *largest* element.

```text
T1. [Initialize.]   c_j <- j-1 (1<=j<=k); c_{k+1} <- n; c_{k+2} <- 0; j <- k.
T2. [Visit.]        Visit c_k ... c_1. If j > 0, x <- j and go to T6.
T3. [Easy case?]    If c_1 + 1 < c_2, c_1 <- c_1 + 1, return to T2. Else j <- 2.
T4. [Find j.]       c_{j-1} <- j-2; x <- c_j + 1. If x = c_{j+1}, j <- j+1, repeat.
T5. [Done?]         Terminate if j > k.
T6. [Increase c_j.] c_j <- x, j <- j-1, return to T2.
```

The common case (T3) just bumps the smallest element c₁ up by one — O(1). Only
when c₁ bumps into c₂ do we carry, resetting the run of small elements and
advancing the first one with room. Sentinels `c_{k+1}=n`, `c_{k+2}=0` remove
boundary tests. The lab handles k = 0 (one empty set) and k = n (one full set)
directly, since T assumes 0 < k < n.

**Hand trace, (n,k) = (4,2).** Read the visited pair as the set {c₁, c₂}:

```
{0,1}, {0,2}, {1,2}, {0,3}, {1,3}, {2,3}
```

Notice the largest element is non-decreasing (0,0 then all ≤2, then all with 3):
that is colex order — combinations are grouped by their maximum. A pleasant
consequence used in the lab: the first C(m,k) combinations of an n-set use only
{0,…,m−1}, so the (n,k) list is a **prefix** of the (n′,k) list for n′ > n.
This "prefix stability" is why colex is the natural order for *ranking*
combinations with the combinatorial number system: the set {c₁<…<c_k} has rank
C(c₁,1) + C(c₂,2) + … + C(c_k,k).

The count itself obeys **Pascal's rule** C(n,k) = C(n−1,k−1) + C(n−1,k)
(choose the largest element or not), the identity the lab checks the output
sizes against.

---

## 6. Partitions of an integer (§7.2.1.4)

A **partition** of n is a multiset of positive integers (the *parts*) summing
to n; by convention we write the parts in non-increasing order. p(n) counts
them: p(1)=1, p(2)=2, p(3)=3, p(4)=5, p(5)=7, p(6)=11, …, p(100)=190569292.
Partitions are the arithmetic of "how many ways to break n into a sum,
order-blind" — the additive analogue of factoring.

### Algorithm P — reverse-lexicographic generation

Knuth generates partitions from [n] down to [1,1,…,1] in reverse lexicographic
order. The state is a₁ ≥ a₂ ≥ … ≥ a_m; `q` tracks the rightmost part > 1.

```text
P1. [Initialize.]           a_0 <- 0, m <- 1.
P2. [Store final part.]     a_m <- n, q <- m - [n = 1].
P3. [Visit.]                Visit a_1 ... a_m. If a_q != 2, go to P5.
P4. [Change 2 to 1+1.]      a_q <- 1, q <- q-1, m <- m+1, a_m <- 1; go to P3.
P5. [Decrease a_q.]         Terminate if q = 0. Else x <- a_q-1, a_q <- x,
                            n <- m-q+1, m <- q+1.
P6. [Copy x if necessary.]  If n <= x go to P2; else a_m <- x, m <- m+1,
                            n <- n-x, repeat P6.
```

The special-cased P3/P4 (splitting a trailing 2 into 1+1) is an optimization for
the very common case where the smallest part exceeding 1 is exactly 2. P5/P6
handle the general "decrease the rightmost part > 1 by one, then redistribute
the freed amount into as-equal-as-possible parts ≤ x."

**The p(5) = 7 partitions in the order produced:**

```
5,  4+1,  3+2,  3+1+1,  2+2+1,  2+1+1+1,  1+1+1+1+1
```

Each is lexicographically less than the one before — reverse-lex.

### Ferrers diagrams and conjugation

Draw a partition as left-justified rows of dots, one row per part. For 4+2+1:

```
* * * *      row 1 (part 4)
* *          row 2 (part 2)
*            row 3 (part 1)
```

The **conjugate** (or transpose) partition is what you read by *columns*:
column j has as many dots as there are parts ≥ j. Reflecting the diagram about
its main diagonal:

```
* * * *          * * *
* *        -->   * *
*                *
                 *
4 + 2 + 1        3 + 2 + 1 + 1
```

so conjugate(4+2+1) = 3+2+1+1. Formally, part j of the conjugate is
`#{ i : a_i ≥ j }`. Two immediate facts, both checked in the lab:

- **Involution.** Transposing twice returns the original: conjugate is its own
  inverse. (Reflecting a diagram across the diagonal twice is the identity.)
- **It swaps two statistics:** the *number of parts* of p becomes the *largest
  part* of the conjugate, and vice versa. This proves, e.g., that the number of
  partitions of n into at most k parts equals the number into parts each ≤ k.
  Partitions fixed by conjugation are *self-conjugate* (like 3+2+1, whose
  diagram is symmetric); a classic theorem says these are equinumerous with
  partitions of n into distinct odd parts.

### Counting p(n) without listing: Euler

Listing partitions is hopeless for large n, but p(n) satisfies a fast
recurrence. The **generating function** is Euler's product

```
sum_{n>=0} p(n) x^n = prod_{k>=1} 1 / (1 - x^k)
                    = (1 + x + x^2 + ...)(1 + x^2 + x^4 + ...)(1 + x^3 + ...)...
```

— the k-th factor `1/(1-x^k) = 1 + x^k + x^{2k} + …` chooses how many parts of
size k to use, and the coefficient of xⁿ in the product counts all partitions.

Euler's **pentagonal number theorem** expands the reciprocal product:

```
prod_{k>=1} (1 - x^k) = sum_{j=-inf}^{inf} (-1)^j x^{j(3j-1)/2}
                      = 1 - x - x^2 + x^5 + x^7 - x^12 - x^15 + ...
```

The exponents g_j = j(3j∓1)/2 = 1, 2, 5, 7, 12, 15, … are the *generalized
pentagonal numbers*. Multiplying the two products = 1 and matching coefficients
gives the recurrence the lab uses:

```
p(n) = sum_{k>=1} (-1)^{k+1} [ p(n - k(3k-1)/2) + p(n - k(3k+1)/2) ]
```

with p(0)=1 and p(m)=0 for m<0. Only about √n terms are nonzero (the pentagonal
numbers up to n), so each p(m) costs O(√m) and the whole table O(n√n) — the
`partition_count` function computes p(100) instantly where enumeration never
could. (In fact 2^{... } bits are needed for large p(n); we stay in u64, good
past p(400).)

---

## 7. Stage-by-stage lab guide

Open `labs/module-08-generation/src/lab.rs`. Each stage below has a test file
`tests/stage_NN_*.rs`; `./grade 8` runs them in order, stopping at the first
failure.

### Stage 1 — `gray_code`, `gray_rank`

Implement Algorithm G with the parity-bit trick and no counter. Model "go back
to G2" with a `loop`. Use `a.trailing_zeros()` for "position of the rightmost
1-bit" (careful: define j = trailing_zeros + 1). For `gray_rank`, the
shift-and-XOR loop `while g != 0 { k ^= g; g >>= 1; }` inverts g. The tests
check the exact n ≤ 4 tables, the one-bit property up to n = 16, completeness,
the closed form g(k) = k^(k>>1), the ruler-function flipped bit, and rank
round-trips both directions.

### Stage 2 — `next_permutation`, `all_permutations`

Implement one step of Algorithm L (find j, increase, reverse), returning
`false` and leaving the array unchanged at the last permutation. Keep the `≥`
comparisons so multisets work. `all_permutations(n)` seeds with `1..=n` and
iterates. Tests: n=3 exact list, n! counts through n=7, strictly-increasing lex
output, and the multiset cases `[1,1,2]` → 3 and `{1,2,2,3}` → 12.

### Stage 3 — `plain_changes`

Implement Algorithm P with the c/o control tables and offset s. Watch the
1-based ↔ 0-based index conversion in P5 (`a.swap(u-1, v-1)`). Return the single
empty permutation for n = 0. Tests: the exact n=3 sequence, n! counts, the
one-adjacent-transposition property for every neighbour (n ≤ 6), completeness,
and the `2 1 3 4 … n` terminal.

### Stage 4 — `combinations`

Implement Algorithm T with the two sentinels; handle k=0 and k=n directly and
`assert!(k <= n, "...k <= n...")`. Return each combination ascending. Tests: the
(4,2) exact colex list, sizes against Pascal's triangle on a 10×10 grid,
sorted/distinct/complete outputs, the colex prefix property, and the k>n panic.

### Stage 5 — `partitions`, `partition_count`, `conjugate`

Implement Algorithm P for `partitions` (reverse-lex), the pentagonal recurrence
for `partition_count`, and the "count parts ≥ j" formula for `conjugate` (assert
non-increasing positive parts; the message must contain "non-increasing"). Tests
cover exact lists for n ≤ 6, the sum/non-increasing invariants, reverse-lex
order, count-vs-enumeration agreement, the headline counts (42, 627, 204226,
190569292), conjugation worked examples, the statistic swap, and the involution
over all partitions of 8.

---

## 8. Check your understanding

1. In Γ₄, which bit changes going from word #7 to word #8? (ρ(8) = 3, the top
   bit — the seam of the reflection.)
2. Why does Algorithm L's L4 *reverse* the suffix rather than sort it? (The
   suffix is already non-increasing after the swap, so reversing = sorting, in
   O(length) not O(length log length).)
3. Plain changes and lexicographic order both list all n! permutations. Give one
   task where each is the right choice. (Minimal-change: incremental
   recomputation, bell-ringing, Gray-like hardware. Lex: ranking/unranking,
   dictionary output, resumability.)
4. Combinations come out in colex order. What is rank of {2,3} in C(4,2)? (Its
   position is last, #5 counting from 0 — C(2,1)+C(3,2) = 2+3 = 5.)
5. Using conjugation, argue that the number of partitions of 6 into exactly 3
   parts equals the number into parts whose largest is exactly 3. (Transpose
   swaps "number of parts" with "largest part".)

## 9. Exercises from the text

Ratings use Knuth's scale: 00 immediate · 10 a minute · 20 fifteen minutes to
an hour · 30 hours · 40 term project · 50 open research problem. An arrow ▶
marks especially instructive exercises. Log attempts in
`course/module-08-generation/exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 7.2.1.1-1 | 10 | Show the reflected Gray code is *cyclic*: the last word differs from the first in one bit too. |
| ▶7.2.1.1-7 | 22 | Prove `g^{-1}(word) = word ^ (word>>1) ^ (word>>2) ^ …` inverts the Gray map (you coded it as `gray_rank`). |
| 7.2.1.2-1 | 15 | Hand-trace Algorithm L on the multiset {1,2,2,3}; list all 12 arrangements. |
| ▶7.2.1.2-11 | 26 | Prove Algorithm L visits multiset arrangements in lexicographic order, each once (the `≥` argument). |
| 7.2.1.2-? | 20 | Show plain changes ends at `2 1 3 4 … n`, one adjacent swap from the start (Hamiltonian cycle). |
| 7.2.1.3-1 | 15 | Hand-trace Algorithm T for (n,k) = (5,3); confirm the colex order. |
| ▶7.2.1.3-4 | 24 | Prove the combinatorial number system: {c₁<…<c_k} has colex rank Σ C(c_i, i). |
| 7.2.1.4-1 | 10 | List the 11 partitions of 6 by hand in reverse-lex; check against `partitions(6)`. |
| ▶7.2.1.4-? | 30 | Derive the pentagonal-number recurrence for p(n) from Euler's product and verify p(50)=204226. |

## In the real world

C++'s `std::next_permutation` is Algorithm L, verbatim — multiset handling
included. Gray codes live in rotary encoders and ADCs (one bit flips per
step, so a mid-transition read is off by at most one position), in
error-masking for asynchronous counters, and in algorithmics wherever a
set changes by one element per step and you want O(1) incremental updates
(subset-sum sweeps, Hamiltonian walks on hypercubes). Combination and
partition generation drive property-based testing, experimental design,
and the enumeration loops of computational chemistry. When you next see an
API offering "iterate all k-subsets", you now know the three questions
that matter: what order, what cost per step, and is it loopless.

## Proof techniques you practiced

- **Induction on recursive structure** — Gray code correctness from its
  reflection construction; the closed form g(k) = k ⊕ ⌊k/2⌋ verified
  against it.
- **Invariant control state** — plain changes' direction/offset tables:
  a nontrivial invariant maintained across n! steps and provable by
  induction on n.
- **Bijections as involutions** — partition conjugation proved by
  transposing Ferrers diagrams; conjugate∘conjugate = identity is the test.
- **Order as specification** — lexicographic, reflected-Gray, revolving-
  door: the *sequence* is part of the contract, and your tests pin it.
- **Generating-function reasoning** — Euler's pentagonal recurrence for
  p(n): stated, used, and empirically confirmed to p(100) = 190 569 292.

## 10. Where this leads

- **Backtracking (Module 09)** generalizes generation: when you cannot afford to
  visit *all* objects, prune the search tree instead — n-queens, exact cover,
  dancing links. Gray-code and minimal-change ideas reappear as *update/undo*
  tricks that make backtracking cheap.
- **Ranking/unranking** (touched on in Stage 4) turns any of these orders into a
  bijection with 0…N−1 — the basis for random selection (Module 04) and for
  compressing combinatorial data.
- The **generating-function** viewpoint from Stage 5 (products encode choices) is
  the gateway to analytic combinatorics and to Vol. 4's deeper counting.
