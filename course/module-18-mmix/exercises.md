# Exercises — Module 18 (MMIX: Knuth's Machine)

Self-contained problems on this module's material — hand-assembly and
disassembly of the `OP X Y Z` format, MMIX's floor division, signed overflow,
the two-pass assembler, the scaled-add opcode family, the cost of FindMax, and
long division. You can work every one **without the fascicle**: each states the
problem in full, gives a **hint** to peek at when stuck, and a worked **answer
sketch** to check against. Numeric encodings and counts here are reproduced by
the machine and assembler you write in the lab (or a few lines at a REPL), and
every opcode is taken from the lesson's chart in §2.2.

Ratings follow Knuth's scale (00–50; `M` = needs mathematics, `▶` = especially
instructive). Exercise numbers follow Vol. 1, Fascicle 1 (§1.3.1´ / §1.3.2´);
match by content if your printing differs. Unnumbered rows are course exercises.

## Tracking

| # | Topic | Rating | Status |
|---|---|---|---|
| 1 | Hand-assemble and disassemble on paper | 10 | ⬜ |
| 2 | ▶ DIV sign table, all four cases and $z = 0$ | 10 | ⬜ |
| 3 | When do full MMIX's signed ops trip? | 15 | ⬜ |
| 4 | ▶ Add a pseudo-op (`LOC` or `OCTA`) to the assembler | 20 | ⬜ |
| 5 | The `2ADDU`–`16ADDU` family; shorten FindMax | 22 | ⬜ |
| 6 | FindMax oops vs. $A$; $\mathbb{E}[A] = H_n - 1$ | M25 | ⬜ |
| 7 | Long division: 128 $\div$ 64 without rD | 30 | ⬜ |

## Problems

### 1. Hand-assemble and disassemble on paper (rating 10 · §1.3.1´-6)

**Problem.** Every MMIX instruction is one tetra encoded as
$\text{OP} \ll 24 \mid \text{X} \ll 16 \mid \text{Y} \ll 8 \mid \text{Z}$.
(a) **Assemble** these three by hand to 32-bit hex, using the §2.2 opcode chart
(ADD register form `0x20`, immediate form `0x21`; SUB register `0x24`; GET
`0xFE` with rR $= 6$): `ADD $1,$2,$3`, `SUB $2,$2,1`, `GET $3,rR`.
(b) **Disassemble** these three words back to mnemonics: `0x1C020001`,
`0x42030004`, `0xF1FFFFFB`. (Recall: `0x1C` = DIV register form; `0x42` = BZ
forward, its `YZ` an offset in *tetras*; `0xF1` = JMP backward, offset
$= \text{XYZ} - 2^{24}$ tetras.)

**Hint.** For immediate operate opcodes, set the low bit (`SUB` register `0x24`
$\to$ immediate `0x25`). Branch/JMP offsets count tetras, not bytes, and the
odd opcode means backward: a backward field $f$ denotes $f - 2^{16}$ (branch) or
$f - 2^{24}$ (JMP).

**Answer sketch.** (a) `ADD $1,$2,$3` uses the *register* opcode `0x20`, so
$\text{OP}{=}0x20, \text{X}{=}1, \text{Y}{=}2, \text{Z}{=}3 \Rightarrow$
**`0x20010203`**. `SUB $2,$2,1` has an *immediate* Z, so `0x24 | 1 = 0x25`,
$\text{X}{=}2, \text{Y}{=}2, \text{Z}{=}1 \Rightarrow$ **`0x25020201`**.
`GET $3,rR` is `0xFE`, $\text{X}{=}3, \text{Y}{=}0, \text{Z}{=}6 \Rightarrow$
**`0xFE030006`**. (b) `0x1C020001`: $\text{OP}{=}0x1C$ = DIV register,
$\text{X}{=}2, \text{Y}{=}0, \text{Z}{=}1 \Rightarrow$ **`DIV $2,$0,$1`**.
`0x42030004`: $\text{OP}{=}0x42$ = BZ forward, $\text{X}{=}3$, $\text{YZ}{=}4$
$\Rightarrow$ **`BZ $3, +4`** (branch to this instruction's address $+ 4 \cdot 4
= +16$ bytes). `0xF1FFFFFB`: $\text{OP}{=}0xF1$ = JMP backward,
$\text{XYZ}{=}0\text{xFFFFFB}{=}2^{24}-5$, offset $= -5$ tetras $\Rightarrow$
**`JMP -5`** (to address $- 20$ bytes). This is the Euclid loop's DIV, its
zero-test, and its loop-closing jump. Stage 3 pins the round trip:
assemble $\leftrightarrow$ these words $\leftrightarrow$ execute, bit for bit.

### 2. ▶ DIV sign table, all four cases and $z = 0$ (rating 10 · §1.3.1´-12)

**Problem.** MMIX's `DIV $X,$Y,$Z` uses *floor* division: $X \gets \lfloor y/z
\rfloor$ and rR $\gets y - z\lfloor y/z\rfloor$, with $y, z$ signed. Compute the
quotient and remainder for all four sign combinations of $(y, z) = (\pm 7, \pm 2)$,
and for the divide-by-zero case $z = 0$. State the invariant the remainder
always satisfies.

**Hint.** The floor-division theorem of §3.2 says the remainder lands on the
*divisor's* side: $0 \le r < z$ when $z > 0$, and $z < r \le 0$ when $z < 0$. So
$r$ always takes the sign of $z$. Truncating division (round toward $0$) would
give different answers for negative dividends — do not use it.

**Answer sketch.** With $q = \lfloor y/z\rfloor$ and $r = y - qz$:

| $y$ | $z$ | $q = \lfloor y/z\rfloor$ | rR $= r$ | check $y = qz + r$ |
|---|---|---|---|---|
| $7$ | $2$ | $3$ | $1$ | $6 + 1 = 7$ |
| $-7$ | $2$ | $-4$ | $1$ | $-8 + 1 = -7$ |
| $7$ | $-2$ | $-4$ | $-1$ | $8 - 1 = 7$ |
| $-7$ | $-2$ | $3$ | $-1$ | $-6 - 1 = -7$ |

The remainder always takes the divisor's sign, and $|r| < |z|$ — this is the
unique $(q, r)$ of the theorem, and it is exactly §1.2.4's $y \bmod z = y -
z\lfloor y/z\rfloor$. For $z = 0$, MMIX-LITE follows §1.2.4's convention $y \bmod
0 = y$: `DIV` sets $X = 0$ and rR $= Y = y$ (so $7/0$ gives $q = 0$,
$r = 7$). Full MMIX additionally raises an integer-divide-check trip there;
MMIX-LITE keeps the values and skips the exception — a documented cut. (Contrast
truncating hardware, where $-7/2 = -3$ remainder $-1$: the remainder flips sign
with the dividend, and the clean $\bmod$ laws the whole series relies on break.)

### 3. When do full MMIX's signed ops trip? (rating 15 · §1.3.1´-14)

**Problem.** MMIX-LITE *wraps* signed arithmetic (two's complement), but full
MMIX raises an overflow **trip** for signed `ADD`, `SUB`, and `NEG` when the true
integer result does not fit a signed octabyte, i.e. outside $[-2^{63},
2^{63}-1]$. Characterize exactly which operand pairs trip each of the three, and
say what MMIX-LITE returns instead.

**Hint.** Signed overflow of a two's-complement add happens only when the two
operands share a sign and the result's sign differs from theirs. `SUB
$X,$Y,$Z` is `ADD` of $y$ and $-z$. `NEG $X,Y,$Z` computes $Y - Z$ with $Y$ an
*immediate byte* ($0 \le Y \le 255$); the bare `NEG $X,$Z` is $Y = 0$, i.e.
$-Z$.

**Answer sketch.** Let $\text{IMIN} = -2^{63}$, $\text{IMAX} = 2^{63}-1$.
- **`ADD $X,$Y,$Z`** trips iff $y$ and $z$ have the *same* sign and the true sum
  $y + z$ leaves $[\text{IMIN}, \text{IMAX}]$ — equivalently $\text{sign}(y) =
  \text{sign}(z) \ne \text{sign}(y \oplus_{\text{wrap}} z)$. Smallest example:
  $\text{IMAX} + 1$ trips (both positive, sum $= 2^{63}$); MMIX-LITE wraps it to
  $\text{IMIN}$.
- **`SUB $X,$Y,$Z`** $= \text{ADD}(y, -z)$ trips iff $y$ and $z$ have *opposite*
  signs and $y - z$ leaves range. Example: $\text{IMIN} - 1$ trips; wraps to
  $\text{IMAX}$. (A subtle case: $z = \text{IMIN}$ has no positive negation, so
  subtracting it can trip on its own.)
- **`NEG $X,Y,$Z`** $= Y - Z$ trips iff $Y - z \notin [\text{IMIN},
  \text{IMAX}]$. For the common $Y = 0$ form ($-z$) the *only* trip is
  $z = \text{IMIN}$, because $-\text{IMIN} = 2^{63} > \text{IMAX}$; wraps back to
  $\text{IMIN}$ (it is its own two's-complement negative).

In every case MMIX-LITE returns the low 64 bits of the two's-complement result —
which is bit-for-bit what full MMIX *would* have stored had the trip handler
returned — so the tests treat wrapping as the contract, and the "trip" is purely
the exception full MMIX raises on top.

### 4. ▶ Add a pseudo-op (`LOC` or `OCTA`) to the assembler (rating 20 · §1.3.2´-x)

**Problem.** The MMIX-LITE assembler (§5) is two-pass — pass 1 builds
`label → instruction index`, pass 2 encodes — and it emits *only* instructions,
so all code is position-independent (stage 3 loads it at three addresses with
identical behavior). Extend it with one real MMIXAL pseudo-op, either `LOC`
(set the assembly origin/current location) or `OCTA` (emit an 8-byte data
constant). Describe the change to each pass and explain precisely what it does
to position-independence.

**Hint.** A pseudo-op is not an instruction — it produces no opcode (LOC) or
produces raw data, not an `OP X Y Z` word (OCTA). Both force you to track a
*location counter in bytes*, not just an instruction index, because data and
`LOC` gaps make address $\ne 4 \times \text{index}$. Ask: once a label can name
an absolute address or a data octabyte, how does code *reach* it — and does the
subset have any absolute-addressing instruction?

**Answer sketch.** *`OCTA`*: pass 1 must advance the location counter by 8 for
each `OCTA` (and by 4 per instruction), so `label → byte address` replaces
`label → index`; pass 2 emits the literal 8 bytes big-endian instead of an
encoded tetra. *`LOC k`*: pass 1 sets the location counter to $k$ (leaving a gap
or fixing an origin); it emits nothing in pass 2. **What breaks:** the *branches*
stay position-independent — they are pc-relative and only need offsets, which
labels-as-byte-addresses still yield by subtraction. But data reached by label
now needs an *address*, and MMIX-LITE has **no absolute-addressing load**: the
only way to point at an `OCTA` is to compute its address pc-relatively
(`GETA`-style, which the subset lacks) or to have hard-wired it via `LOC` at a
fixed origin — either of which pins the program to a load address. So `LOC`
introduces genuine absolute placement (the assembler now presumes a fixed
origin), and `OCTA` data is only reachable if you give up relocatability or add
a pc-relative address-of primitive. This is exactly why §5 notes the subset
"needs no origin": dropping absolute addresses is what buys position-independence,
and adding these pseudo-ops is where you feel the trade.

### 5. The `2ADDU`–`16ADDU` family; shorten FindMax (rating 22 · course exercise)

**Problem.** Add the scaled-add opcode family occupying `0x28`–`0x2F` (right
after `SUBU` at `0x26`/`0x27`) to the machine: `2ADDU`, `4ADDU`, `8ADDU`,
`16ADDU`, computing $X \gets (2^k \cdot Y + Z) \bmod 2^{64}$ for
$k = 1, 2, 3, 4$, following §2.2's even/odd immediate convention. Then use
`8ADDU` to shorten FindMax's array addressing by one instruction per pass.

**Hint.** FindMax scans octabytes $X[k]$ at addresses $\text{base} + 8k$.
Computing that address today takes two instructions (a shift or multiply by 8,
then an add). `8ADDU $addr,$k,$base` is precisely $8\cdot k + base$ in one
op — the exact idiom these opcodes exist for. (These are unsigned adds, so they
never trip.)

**Answer sketch.** Each `nADDU $X,$Y,$Z` sets $X \gets (n\cdot Y + Z) \bmod
2^{64}$ with $n \in \{2,4,8,16\}$; as an even/odd pair its odd opcode makes $Z$
an immediate byte, per §2.2's rule. (The exact opcode-to-scale assignment inside
`0x28`–`0x2F` follows the even/odd pairing — confirm it against the machine's
`op` table / Fascicle 1 rather than guessing.) With `8ADDU`, an address
computation that was, say, `SLU $t,$k,3` then `ADDU $addr,$t,$base` collapses to
the single `8ADDU $addr,$k,$base` — one fewer instruction (and one fewer live
register) on *every* iteration of FindMax's loop, so the oops count of Exercise 6
drops by exactly the loop-trip count. This is why scaled-add opcodes earn a slot
in a RISC that is otherwise ruthless about instruction economy: indexed access
into an array of $2^k$-byte elements is the single most common address pattern in
real code. (Sanity: `8ADDU` of $Y = \text{0x1000}$, $Z = 5$ gives
$8\cdot\text{0x1000} + 5 = \text{0x8005}$.)

### 6. FindMax oops vs. $A$; $\mathbb{E}[A] = H_n - 1$ (rating M25 · course exercise)

**Problem.** FindMax (Algorithm 1.2.10M) scans $X[n-1], X[n-2], \ldots, X[1]$
after initializing its running maximum to $X[n]$, updating whenever it sees a
larger key; let $A$ be the number of updates (the number of times step M4
fires). (a) Argue that its instrumented cost has the form $\text{oops} = a +
b\,n + c\,A$ for constants fixed by the listing, and that $\text{mems} = n$
always. (b) Show that over a uniformly random permutation, $\mathbb{E}[A] =
H_n - 1$, where $H_n = \sum_{i=1}^{n} 1/i$, and verify it empirically with your
`oops()`/`mems()` counters.

**Hint.** The loop body runs $n - 1$ times (once per scanned key); most of it is
fixed work, and only step M4 (the update) is conditional — so oops is linear in
$n$ with an extra $c$ per update. For $\mathbb{E}[A]$: $X[k]$ triggers an update
iff it exceeds every key at positions $k, k+1, \ldots, n$, i.e. iff it is the
maximum of that suffix — a *record*. Records in a random permutation are the
classic $H_n$ computation from Module 02.

**Answer sketch.** (a) Every pass through the scan executes the fixed M2/M3/M5
instructions; step M4 adds a constant number of instructions only when it fires.
So $\text{oops} = a + b\,n + c\,A$: $b\,n$ (really $b(n{-}1)$ plus setup, folded
into $a$) for the mandatory scan, $c\,A$ for the updates. The exact constants
$a, b, c$ come from counting the 15-instruction listing embedded in the lab's
stage 4 (read them off there — do not trust memory). Memory cost is exactly
$n$ mems: $X[n]$ read once at init, then $X[k]$ read once for each $k = n{-}1,
\ldots, 1$ — no stores. (b) Let $R_k$ indicate that $X[k]$ is a *suffix record*
(exceeds all of $X[k], \ldots, X[n]$). Position $k$ is the maximum of its
length-$(n-k+1)$ suffix with probability $1/(n-k+1)$, and $A = \sum_{k=1}^{n-1}
R_k$ (position $n$ is the initial value, not an update). By linearity,

$$
\mathbb{E}[A] = \sum_{k=1}^{n-1} \frac{1}{\,n-k+1\,} = \sum_{j=2}^{n}
\frac{1}{j} = H_n - 1.
$$

Empirically, averaging $A$ over random permutations gives $\approx 1.083$ for
$n = 4$, $\approx 1.718$ for $n = 8$, $\approx 2.381$ for $n = 16$ — matching
$H_n - 1 = 1.0833, 1.7179, 2.3807$. This is the *same* $H_n - 1$ you met in
Module 02's analysis and Module 04's reservoir-sampling replacement count; the
machine and the mathematics agree, as they must.

### 7. Long division: 128 $\div$ 64 without rD (rating 30 · course exercise)

**Problem.** Full MMIX's `DIVU` divides a 128-bit dividend (`rD:$Y`) by a
64-bit divisor; MMIX-LITE has *no* rD, so its `DIVU` is plain 64-by-64. Write an
MMIX-LITE routine that divides a 128-bit dividend (held in two registers,
high:low) by a 64-bit divisor, producing a 64-bit quotient and remainder, using
only the subset's opcodes. Explain why the shift discipline of §3.3 matters.

**Hint.** Do binary long division: run the standard shift-subtract loop 64 times.
Keep a running remainder $R$ (initially $0$); each iteration shift the 128-bit
dividend left by one *into* $R$'s low bit, then if $R \ge \text{divisor}$ subtract
it and set the corresponding quotient bit. You need `SLU`/`SRU` to move bits
across the register boundary, `CMPU` for the unsigned compare, `SUBU` for the
conditional subtract, and a `BNN`/`BNZ` branch to choose. Recall §3.3: MMIX
shifts do **not** reduce the count mod 64, so a shift by 64 zeroes the register
cleanly — no accidental wraparound like x86.

**Answer sketch.** Non-restoring/restoring binary division, 64 iterations, one
quotient bit per step (MSB first):
```text
R <- 0 ;  Q <- 0                    ; remainder, quotient
repeat 64 times:
    R  <- (R << 1) | topbit(hi)     ; SLU R,R,1 ; SRU t,hi,63 ; OR-via-ADDU
    (hi:lo) <- (hi:lo) << 1         ; shift the 128-bit dividend left one
    if CMPU(R, divisor) >= 0:       ; R >= divisor ?
        R <- R - divisor            ; SUBU
        set this step's quotient bit ; SLU Q,Q,1 ; ADDU Q,Q,1
    else:
        Q <- Q << 1                 ; SLU Q,Q,1  (bit 0)
after 64 steps:  quotient = Q, remainder = R
```
Correctness is the invariant "after step $i$, $Q$ holds the top $i$ quotient bits
and $R$ the partial remainder of the top $i$ dividend bits," proved by induction
exactly as pencil-and-paper long division. The §3.3 shift semantics are what make
it safe: because a shift count of $64$ produces $0$ (not a mod-64 no-op), the bit
you extract with `SRU t,hi,63` and the boundary shifts behave definitely, with no
architecture-specific surprises. This routine is *what rD buys you in one
instruction* — full MMIX's `DIVU` with a nonzero rD does this 128-bit division in
hardware; feeling its absence here is the point of the exercise. (Cost: ~64 loop
iterations of a handful of oops each — orders of magnitude more than a single
hardware `DIVU`, which is why the wide-divide register exists.)

---

## Your solutions

Use this space to log your own work — a restated problem, your approach, and how
it compared with the sketch above.

### Exercise N (rating RR)
**Approach.**
**Answer / proof.**
**Compared with the sketch:** what you did differently, what you missed.
