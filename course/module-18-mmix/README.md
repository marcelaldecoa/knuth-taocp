# Module 18 — MMIX: Knuth's Machine

> **Source:** *The Art of Computer Programming*, Vol. 1, Fascicle 1 —
> *MMIX: A RISC Computer for the New Millennium* (§1.3.1´ describes the
> machine, §1.3.2´ the MMIXAL assembly language). Programs from §1.1
> (Algorithm E) and §1.2.10 (Algorithm M).
> **Lab:** `labs/module-18-mmix` · **Grade it:** `./grade 18`
>
> This lesson is self-contained: you can complete the module without the
> fascicle. If you own it, read §1.3.1´ first.

This is the course closer, and it closes a circle. In module 01 you proved
Euclid's algorithm correct and counted its divisions; in module 02 you
analyzed Algorithm M, the maximum-finder. In this module you build a
computer — a faithful subset of Knuth's MMIX — plus an assembler for it, and
then you run Euclid and FindMax *on your own machine*, measuring their cost
with Knuth's own instruments. The programs you started the course proving,
you end the course executing, four bytes at a time.

---

## 1. Why does Knuth make you learn a machine?

Every algorithm in TAOCP is analyzed down to the instruction. That is not
nostalgia; it is the whole point of the books. High-level pseudocode hides
constant factors, and constant factors are where real programs live: an
"O(n)" pass that touches memory n times and an "O(n)" pass that touches it
3n times differ by exactly the factor your users notice. Knuth's rule is
**assembly-level honesty**: state the algorithm abstractly, prove it
abstractly, then compile it by hand onto a concrete machine and count.

The first editions used **MIX**, a 1960s-style machine (decimal-or-binary
words, one accumulator, magnetic tapes). By the 1990s MIX's architecture
was actively misleading — real machines had become load/store RISC designs
with big register files — so Knuth replaced it with **MMIX** ("*em-mix*",
arithmetic mean of the model numbers of 14 real computers = 2009, also
MMIX in Roman numerals): a clean 64-bit RISC with 256 general registers,
big-endian byte-addressable memory, and fixed-width 4-byte instructions.
It was designed with advice from the architects of real RISC machines, and
it looks like what it is: an idealized MIPS/Alpha/RISC-V sibling with the
warts filed off.

**What we build — MMIX-LITE, an honest subset.** A full MMIX has 256
opcodes, floating point, a register stack, and a TRIP/TRAP interrupt
system. Our machine keeps the parts the course needs and *documents* every
cut:

- 256 general registers `$0..$255`, each one octabyte (u64). (Full MMIX
  splits them into local/global via rG and rL; we don't.)
- Byte-addressable **big-endian** memory, default zero — a sparse
  `HashMap<u64, u8>` in the reference, because a $2^{64}$-byte `Vec` is not on
  the menu; any map with "unwritten = 0" semantics works.
- The true MMIX instruction encoding and the true numeric opcodes.
- Integer arithmetic (with MMIX's floor DIV!), loads/stores of all four
  sizes, four branches, JMP, SETL, GET, and TRAP-as-halt.
- Special registers: only stand-ins for **rR** (remainder) and **rH**
  (himult), exposed as `remainder()` / `himult()` and readable in programs
  via `GET`. No rD (so DIVU divides a 64-bit dividend), no rA overflow
  flags — signed arithmetic **wraps** where full MMIX would trip.
- No floating point, no MMIXAL pseudo-ops (`LOC`, `GREG`, `BYTE`…) beyond
  labels, no operating system: `TRAP 0,0,0` means halt.

Every one of these simplifications is disclosed where it bites, and the
opcode table lives in **one** place (`op` in the lab) so you can audit it
against Fascicle 1's chart.

---

## 2. The machine model

### 2.1 State

A machine state is: 256 registers of 64 bits; a memory function
$M : 2^{64} \to 2^8$; the program counter `@` (Knuth's symbol: "the place where
we're at"); plus our rR/rH stand-ins, the halted flag, and two cost
counters. That's it — this is a computational method in the §1.1 sense,
$(Q, I, \Omega, f)$, with `step()` playing $f$. You met this definition in module
01; now you are implementing f.

Memory names things by size: a **byte** (8 bits), **wyde** (16), **tetra**
(32), **octa** (64). MMIX is big-endian: the most significant byte of a
multi-byte value sits at the smallest address. And MMIX never faults on a
misaligned address — it **ignores the low bits**: a wyde access uses
`addr & !1`, a tetra `addr & !3`, an octa `addr & !7`. (So `LDO` of address
`0x105` quietly reads the octa at `0x100`; stage 1 tests exactly that.)

### 2.2 Instructions: OP X Y Z

Every instruction is one tetra, four bytes:

```text
  31      24 23      16 15       8 7        0
 +----------+----------+----------+----------+
 |    OP    |    X     |    Y     |    Z     |
 +----------+----------+----------+----------+
```

X is (almost always) the destination register; Y and Z are sources. The
stroke of genius is the **immediate rule**: opcodes come in even/odd pairs,
and the odd opcode means *Z is the byte itself, not a register number*.
`ADD $1,$2,$3` is opcode 0x20 (\$3 as register); `ADD $1,$2,3` is opcode
0x21 (the constant 3). The decoder tests one bit; the instruction set
doubles. For branches the same low bit means *backward* instead (§2.3).

The MMIX-LITE opcode chart (verify against Fascicle 1 — it is defined once,
in `op`, and every test goes through those names):

| opcodes | mnemonics | meaning |
|---|---|---|
| 0x00 | TRAP | halt (full MMIX: OS trap) |
| 0x18–0x1F | MUL, MULU, DIV, DIVU | multiply and divide, $\times 2$ for immediates |
| 0x20–0x27 | ADD, ADDU, SUB, SUBU | add and subtract |
| 0x30–0x37 | CMP, CMPU, NEG, NEGU | compare ($-1/0/1$), negate ($Y - Z$, $Y$ immediate) |
| 0x38–0x3F | SL, SLU, SR, SRU | shifts: arithmetic vs logical |
| 0x40–0x4B | BN, BZ, BNN, BNZ | branches (odd = backward) |
| 0x80–0x8F | LDB(U), LDW(U), LDT(U), LDO(U) | loads, signed/unsigned |
| 0xA0–0xAD | STB, STW, STT, STO | stores (low bytes of \$X) |
| 0xE3 | SETL | \$X ← 16-bit immediate YZ |
| 0xF0, 0xF1 | JMP | 24-bit relative jump (odd = backward) |
| 0xFE | GET | read special register (rH = 3, rR = 6) |

### 2.3 Control flow is relative

A taken branch sets `@ ← branch_address + 4·YZ` (even opcode, forward) or
`@ ← branch_address + 4·(YZ − 2¹⁶)` (odd opcode, backward); JMP is the same
with the 24-bit field XYZ and $2^{24}$. Offsets are counted in **tetras**, not
bytes — instructions are tetra-aligned, so the two free low bits buy $4\times$
the range. Because *everything* is pc-relative, assembled code is
position-independent: stage 3 loads the same words at three different
addresses and demands identical behavior.

Branch conditions read \$X as a *signed* octabyte: BN (negative), BZ
(zero), BNN (nonnegative), BNZ (nonzero). Together with CMP's $-1/0/1$
result these four are a complete comparison kit.

---

## 3. Semantics that deserve care

### 3.1 Loads: sign extension

`LDB $2,$1,4` fetches the byte at `$1 + 4` and **sign-extends** it: byte
0x89 becomes 0xFFFFFFFFFFFFFF89 (that is $-119$, as a signed octabyte).
`LDBU` zero-extends: 0x89 becomes 0x0000…0089. Same for wydes and tetras;
for octas LDO and LDOU coincide (there is nothing left to extend). Stores
go the other way: `STB` writes the low byte of \$X. Work one example by
hand before coding — store 0x0123456789ABCDEF at 0x100 and predict all
eight LDB results; stage 1 pins exactly these.

### 3.2 Floor division — the mathematically right one

Most hardware truncates quotients toward zero. Knuth's MMIX does not:

**Definition (Fascicle 1).** `DIV $X,$Y,$Z` sets \$X ← $\lfloor y/z \rfloor$ and
rR ← $y - z\cdot\lfloor y/z \rfloor$, operands signed.

**Theorem (floor division).** For integers $y, z$ with $z \ne 0$ there is
exactly one pair $(q, r)$ with $y = qz + r$ and $r$ lying between $0$ and $z$ ("on
$z$'s side": $0 \le r < z$ if $z > 0$, and $z < r \le 0$ if $z < 0$). That $q$ is $\lfloor y/z \rfloor$.

*Proof.* Existence: take $q = \lfloor y/z \rfloor$, $r = y - qz$. By definition of floor,
$q \le y/z < q + 1$. If $z > 0$, multiplying by $z$ preserves order: $qz \le y < qz + z$,
i.e. $0 \le r < z$. If $z < 0$ the inequalities flip: $qz \ge y > qz + z$,
i.e. $z < r \le 0$. Uniqueness: if $y = qz + r = q'z + r'$ with both remainders
on $z$'s side, then $(q - q')z = r' - r$ and $\lvert r' - r\rvert < \lvert z\rvert$, forcing $q = q'$,
$r = r'$. ∎

So the remainder always takes the **divisor's sign**. The four cases to
tattoo somewhere (stage 2 pins them):

| $y$ | $z$ | $\lfloor y/z \rfloor$ | rR |
|---|---|---|---|
| 7 | 2 | 3 | 1 |
| $-7$ | 2 | $-4$ | 1 |
| 7 | $-2$ | $-4$ | $-1$ |
| $-7$ | $-2$ | 3 | $-1$ |

Why does Knuth insist? Because this remainder **is** the `mod` of §1.2.4:
$y \bmod z = y - z\lfloor y/z \rfloor$. That operation satisfies the clean laws the whole
series relies on — $y \bmod z$ is periodic in $y$, $(y \bmod z) \bmod z = y \bmod z$,
and $\gcd$ arguments never need sign case-splits. Truncating division's
remainder changes sign with the dividend and breaks all three. Even the
edge case is Knuth's: §1.2.4 *defines* $y \bmod 0 = y$, and MMIX's DIV by zero
duly sets \$X = 0, rR = \$Y. (Full MMIX also raises an "integer divide
check" exception there; MMIX-LITE keeps the values, skips the exception —
a documented cut.)

Implementation note: Rust's `/` truncates. Compute the truncated quotient,
then subtract 1 when the remainder is nonzero and the operand signs
differ; do it in i128 so i64::MIN / $-1$ cannot overflow (the wrapped answer
is i64::MIN, which is also what full MMIX produces bit-wise).

### 3.3 The rest of the arithmetic

- **Wrapping.** Full MMIX raises overflow *trips* for signed ADD/SUB/MUL/
  NEG/SL; MMIX-LITE wraps (two's complement), which makes each signed op
  agree bit-for-bit with its unsigned twin. Documented simplification;
  the tests treat wrapping as the contract.
- **MULU** computes the full 128-bit product: low half to \$X, high half to
  rH (read with `GET $X,rH`). Pin: $(2^{64}-1)^2$ has high half $2^{64}-2$ and low
  half 1. Signed MUL does *not* touch rH.
- **DIVU** with no rD register is plain 64-bit unsigned division — exactly
  full MMIX's behavior when rD = 0, including divide-by-zero: \$X = 0,
  rR = \$Y.
- **NEG** is spelled subtraction: `NEG $X,Y,$Z` $= Y -$ \$Z where $Y$ is
  *always* an immediate byte; `NEG $X,$Z` abbreviates $Y = 0$. MMIX has no
  unary minus.
- **Shifts** do **not** reduce the count mod 64 (x86 does; MMIX doesn't):
  a count $\ge 64$ pushes every bit out — SL/SLU/SRU give 0, SR gives the sign
  fill (0 or all ones). SR is arithmetic (sign-propagating), SRU logical.
- **CMP/CMPU** put $-1$, 0, or 1 in \$X ($-1$ = all ones), signed vs unsigned.
  CMP then BN/BZ/BNZ/BNN is how MMIX says `if`.

---

## 4. The cost model: oops and mems

Knuth prices every MMIX instruction in two units:

- **$\upsilon$** ("oops") — one unit of machine work; and
- **$\mu$** ("mems") — one reference to memory.

Fascicle 1's price list (a selection): most register operations cost $1\upsilon$;
LDx and STx cost $\mu + \upsilon$; MUL costs $10\upsilon$; DIV costs $60\upsilon$; a branch costs $1\upsilon$
plus $2\upsilon$ more when mispredicted. The running time of a program is a
polynomial $a\mu + b\upsilon$ — a *theorem about the program*, independent of the
building's air conditioning, the compiler's mood, or this year's cache
hierarchy. That is why counting beats the stopwatch: it is reproducible,
comparable across decades, and provable. When Knuth reports in Vol. 4 that
one SAT solver variant costs "3.9 gigamems" and another "12.1 gigamems",
those numbers mean the same thing on your machine as on his.

MMIX-LITE instruments the model with two counters:

- `oops()` — instructions executed (each costs 1; we don't weight MUL/DIV,
  and we say so);
- `mems()` — memory references made by executed loads and stores
  (instruction fetch is free, as in Knuth's accounting; the Rust-side
  debug accessors `ld_octa` & co. are also free — they are your
  oscilloscope, not the program's work).

Stage 4 turns the counters on Euclid and FindMax and pins exact values.

---

## 5. The assembler: a tiny compiler

MMIXAL is Knuth's assembly language; ours is MMIXAL-in-miniature. Line
format:

```text
LABEL   OP   X,Y,Z    ; comment
```

A label is any first token that isn't a mnemonic; literals are decimal,
`0x…`, or MMIX-style `#…` hex; registers are `$0`..`$255`; branch targets
are labels. `assemble` makes **two passes** — the classic architecture,
unchanged since the 1950s, and your first compiler:

1. **Pass 1 (symbol table).** Strip comments, split off labels, record
   `label → instruction index`. You cannot encode a *forward* branch until
   you know where its target lands — that is the whole reason two passes
   exist.
2. **Pass 2 (encode).** Each line becomes `OP<<24 | X<<16 | Y<<8 | Z`,
   choosing the odd opcode for immediate Z (operate/load/store) or for
   negative offsets (branches, JMP).

**Hand-assembly, once, on paper** — this is the skill the module is named
for. Take `SUB $2,$2,1` : SUB's register form is 0x24, but Z here is the
*immediate* 1, so the opcode is 0x24|1 = 0x25; X = 2, Y = 2, Z = 1; the
tetra is `0x25020201`. Now a backward branch, `BNZ $2,LOOP` where LOOP is
two instructions earlier: the offset is $-2$ tetras, so we need the backward
opcode 0x4A|1 = 0x4B and YZ = $2^{16} - 2$ = 0xFFFE: the tetra is
`0x4B02FFFE`. Stage 3 pins both, plus the round trip: the assembled loop
must equal the hand-encoded words *bit for bit*, then run.

Because every branch is relative and there are no absolute addresses in
the subset, the assembler needs no origin — `assemble` returns words you
may `load_program` anywhere.

---

## 6. Full circle: Euclid on MMIX

Module 01, Algorithm 1.1E:

```text
E1. [Find remainder.]  Divide m by n; let r be the remainder.
E2. [Is it zero?]      If r = 0, terminate; n is the answer.
E3. [Reduce.]          Set m <- n, n <- r; go back to E1.
```

The same algorithm, compiled by hand for MMIX-LITE (\$0 = m, \$1 = n; the
answer lands in \$1):

```text
E1      DIV  $2,$0,$1     ; E1. q <- floor(m/n), rR <- m mod n
        GET  $3,rR        ;     bring the remainder into $3
E2      BZ   $3,DONE      ; E2. r = 0?  then n is the answer
E3      ADD  $0,$1,0      ; E3. m <- n        (ADD with immediate 0 = move)
        ADD  $1,$3,0      ;     n <- r
        JMP  E1           ;     back to E1
DONE    TRAP 0,0,0        ; halt
```

Seven tetras. Note the compilation choices: step E1's *two* outputs
(quotient and remainder) map onto DIV's two destinations (\$X and rR);
"go back" becomes a relative JMP; the move `m ← n` is an ADD with
immediate zero, MMIX's idiom for register copy. Trace it on Knuth's
(544, 119), one line per pass, next to the module-01 table:

| pass | \$0 (m) | \$1 (n) | DIV: \$2 (q) | rR → \$3 (r) | BZ taken? |
|---|---|---|---|---|---|
| 1 | 544 | 119 | 4 | 68 | no |
| 2 | 119 | 68 | 1 | 51 | no |
| 3 | 68 | 51 | 1 | 17 | no |
| 4 | 51 | 17 | 3 | **0** | yes → halt, \$1 = 17 |

Four divisions, answer 17 — the same table you wrote by hand in module
01, now produced by a machine you built. The cost is a closed form: each
non-final pass executes DIV, GET, BZ, ADD, ADD, JMP = 6 instructions, the
final pass 3, plus 1 for the TRAP:

$$\text{oops} = 6(T - 1) + 3 + 1 = 6T - 2, \qquad \text{mems} = 0$$

where $T = T(m, n)$ is module 01's division count. $T(544, 119) = 4$ gives
**22 oops**; stage 4 pins it, and Lamé's theorem from module 01 is now
literally a bound on your CPU's running time. (Under Fascicle 1's real
prices the DIV at $60\upsilon$ dominates everything — one more reason Knuth counts
divisions and not lines.)

And FindMax, module 02's Algorithm 1.2.10M (find the maximum of
$X[1..n]$ and the largest index attaining it), maps M1–M5 onto fifteen
instructions — the lab's stage 4 embeds the listing with a comment per
step. Its memory cost is exactly **$n$ mems** ($X[n]$ once, then $X[k]$ for
$k = n-1..1$), and its oops count depends on how many times step M4 fires —
the very quantity whose average, $H_n - 1$, you computed in module 02. The
analysis and the machine agree; they'd better.

---

## 7. Stage-by-stage lab guide

Open `labs/module-18-mmix/src/lab.rs`. The opcode table and `Fault` are
given; everything else is yours. Suggested build order inside each stub is
in its doc comment.

### Stage 1 — machine state, memory, loads and stores

`Mmix::new`, register file, big-endian `ld_/st_` accessors with
round-down alignment, `load_program`, and the fetch–decode–execute
skeleton of `step` with the LDx/STx families. Address = \$Y + Z-operand,
wrapping. Get the sign-extension example of §3.1 right and this stage
falls. Watch the two easy-to-swap details: big-endian byte order, and
`pc` advancing by 4 *before* the instruction executes (branches are
relative to the instruction's own address, not the incremented pc).

### Stage 2 — arithmetic

ADD/SUB/MUL (wrapping), MULU→rH, DIV (floor! §3.2, computed via i128),
DIVU, CMP/CMPU, NEG ($Y$ immediate!), the four shifts with $\ge 64$ counts, and
GET for rR/rH. The tests include a 300-case LCG sweep that checks the
floor-quotient identity independently — if your DIV truncates, it fails
on the first negative dividend.

### Stage 3 — branches, loops, and the assembler

BZ/BNZ/BN/BNN both directions, JMP/JMPB, TRAP-as-halt, SETL,
`run(max_steps)` (the finiteness guard: report steps executed, let the
caller check `halted()`), `Fault::IllegalOpcode` — then `assemble`, the
two-pass design of §5. The encoding pins in
`assembler_emits_the_documented_encodings` are your ground truth; write
them on paper first.

### Stage 4 — programs on the metal

No new machine code — the tests feed the embedded EUCLID and FINDMAX
sources through `assemble → load_program → run` and check answers *and
costs*: gcd(544,119) = 17 at exactly 22 oops and 0 mems; gcd(2166,6099) =
57 (module 01's numbers); FindMax on Knuth's sixteen keys (max 908 at
position 5), duplicate maxima reporting the *last* index, a single
element, and $n$ mems always. If stages 1–3 are honest this stage is a
victory lap; if anything was fudged, this is where it surfaces.

---

## Why it's done this way

- **Why a fixed 4-byte OP X Y Z format?** Decoding is a shift and three
  masks — no length decoding, no microcode. This is the RISC bet: make
  every instruction cheap and uniform, win it back in frequency. It also
  makes your `step()` a twenty-line match instead of a parser.
- **Why the odd-opcode immediate rule?** Constants are the most common
  operand in real code. One opcode bit gives every operation an immediate
  form without a second decoder path — compare x86's dozen addressing
  forms. (Branches reuse the same bit for direction because a branch's
  "Z operand" is its offset — the bit was free.)
- **Why floor division?** Because `mod` should satisfy theorems. §3.2:
  the remainder with the divisor's sign is the unique choice making
  $y \bmod z$ periodic in $y$ — Knuth aligned his machine with his mathematics,
  not the other way around. C99 chose truncation for hardware
  compatibility, and every C programmer has paid for it in `((a % b) + b)
  % b` ever since.
- **Why big-endian, round-down alignment?** Big-endian reads like the
  hex dump (most significant first); ignoring low address bits means no
  alignment-fault machinery in a teaching machine — misalignment is
  *defined*, not punished.
- **Why counters instead of a profiler?** $\upsilon$ and $\mu$ make cost a pure
  function of (program, input) — assertable in a unit test. You cannot
  `assert_eq!` a wall-clock time.
- **Why make students write the assembler too?** Because "label →
  address, mnemonic → bits" is the minimal complete compiler, and because
  hand-encoding branch offsets once immunizes you forever against
  off-by-one-instruction bugs in anything relative.

## 8. Check your understanding

1. Store 0x0123456789ABCDEF at 0x200. What do `LDW $1,$2,6`,
   `LDWU $1,$2,6`, and `LDT $1,$2,5` load (\$2 = 0x200)? *(Hint: 0xCDEF
   sign-extends; the tetra address rounds down to 0x204.)*
2. What are $-1 \bmod 5$ and $1 \bmod -5$ under MMIX's DIV, and why would a
   truncating DIV break the invariant `gcd(m, n) = gcd(n, m mod n)` proof
   from module 01 for negative inputs? *(Hint: which lemma step needs
   $0 \le r < \lvert n\rvert$?)*
3. A branch's YZ field is 16 bits and offsets count tetras. How far, in
   bytes, can a forward branch reach? A JMP? *(Hint: $4\cdot(2^{16}-1)$ and
   $4\cdot(2^{24}-1)$.)*
4. Why must `run(max_steps)` exist at all — what property of MMIX-LITE
   programs is *not* decidable by the grader otherwise? *(Module 01,
   finiteness; you cannot test "terminates", only "terminates within
   budget".)*
5. Euclid's listing uses `ADD $0,$1,0` as a move. Why is the immediate
   form (opcode 0x21) essential — what would `ADD $0,$1,$0` (0x20) do
   instead? *(Hint: \$0 is not zero; MMIX has no hardwired zero register,
   unlike RISC-V.)*

## 9. Exercises from the text

Ratings on Knuth's 00–50 scale; ▶ = especially instructive. Exercise
numbers follow Vol. 1, Fascicle 1 (§1.3.1´/§1.3.2´); match by content if
your printing differs. Log attempts in `exercises.md`.

| Ex. | Rating | Statement (paraphrased) |
|---|---|---|
| 1.3.1´-6 | 10 | Hand-assemble a handful of instructions and disassemble a hex dump back to mnemonics — both directions, on paper. |
| ▶1.3.1´-12 | 10 | Work out DIV's quotient and remainder for all four sign combinations and for $z = 0$; check against §3.2's theorem. |
| 1.3.1´-14 | 15 | Exactly which $(y, z)$ make signed ADD/SUB/NEG overflow in full MMIX? (MMIX-LITE wraps — say what full MMIX would trip on.) |
| ▶1.3.2´-x | 20 | Extend the assembler with one MMIXAL pseudo-op: `LOC` (set origin) or `OCTA` (emit data). What breaks in position-independence? |
| — | 22 | Add 2ADDU/4ADDU/8ADDU/16ADDU (0x28–0x2F) to the machine and use 8ADDU to shorten FindMax's addressing by one instruction per pass. |
| — | 25 | Derive FindMax's exact oops count as a function of $n$ and $A$ = number of times M4 fires; verify $E[A] = H_n - 1$ (module 02!) empirically with your `oops()` counter over random permutations. |
| — | 30 | Write long division: 128-bit $\div$ 64-bit using only MMIX-LITE ops (this is what rD is for in full MMIX — feel its absence). |

## In the real world

**Mems are how Knuth still measures.** From *The Stanford GraphBase*
onward, Knuth instruments his programs to report **mems** — memory
references — instead of seconds, and Vol. 4's algorithm shoot-outs
(BDDs, SAT) are all scored in mems and gigamems. The reasoning is the
module's: a mem count is machine-independent, reproducible, and provable,
while a benchmark time expires with the hardware that produced it. Your
`mems()`/`oops()` pair is precisely this instrument, and the discipline —
*state cost as a countable, assertable quantity* — is what you see today
in "allocations per op" benchmarks, DB query planners costing plans in
page reads, and big-O-plus-constants performance reviews.

**RISC-V is MMIX's spiritual production sibling.** MMIX (1999) distilled
the classic RISC designs — MIPS, Alpha, SPARC — into a clean, open,
teaching-grade ISA. Eleven years later Berkeley did the same distillation
for production: RISC-V, an open, minimal, fixed-width load/store ISA with
register+immediate instruction forms, now shipping in real silicon from
microcontrollers to server parts. Set the two side by side and the family
resemblance is uncanny (32/64-bit clean design, no condition-code flags,
simple relative branches); the differences are instructive too — RISC-V
is little-endian, has a hardwired zero register x0 where MMIX uses
immediate-0 idioms, and traps on nothing it can define away. If you can
implement MMIX-LITE, a RV64I interpreter is a weekend.

**You just wrote a VM, and every language runtime contains one.** Your
`step()` — fetch, decode via a match on an opcode byte, execute, advance
pc — is the same loop at the heart of CPython's bytecode evaluator, the
JVM's interpreter, WebAssembly engines, the BEAM, SQLite's VDBE, and
regex engines; it is also the core of QEMU and of every game-console
emulator, where the "subset, faithfully, with documented deviations"
discipline you practiced is the entire job. The follow-on tricks are all
incremental from here: threaded dispatch replaces the match, inline
caching specializes hot opcodes, and a JIT is "pass 2 of your assembler,
run at runtime, in reverse." Emulator/VM engineering is not exotic — it
is this module, iterated.

## Proof techniques you practiced

- **Existence-and-uniqueness arguments** — the floor-division theorem
  (§3.2): construct (q, r), then pin uniqueness with an inequality
  squeeze. The same two-part shape as module 02's closed forms.
- **Definition-driven case analysis** — sign extension, shift counts
  $\ge 64$, branch direction: each semantic rule became a total function with
  every case pinned by a test; "definiteness" from module 01, executed.
- **Invariant transport across refinement** — Algorithm E's correctness
  proof (invariant + decreasing n) survives compilation: each E-step maps
  to fixed instructions, so partial correctness and termination transfer
  from pseudocode to machine code. That is the germ of compiler
  correctness proofs.
- **Cost as a closed form** — oops $= 6T - 2$ turns module 01's $T(m, n)$
  into an exact machine cost, then a test asserts it: derive, then
  measure, Knuth's two-step, now end-to-end.
- **Consistency-based verification** — the opcode table is defined once
  and every test round-trips through it (assemble ↔ hand-encode ↔
  execute), so a transcription doubt can't silently split the truth into
  two versions.

## 10. Where this leads

- **Fascicle 1 itself** — the full register stack (PUSHJ/POP), floating
  point, and the TRIP/TRAP system; then MMIXware, Knuth's own simulator
  with the complete pipeline-accurate `MMIX-PIPE`.
- **Backwards, everywhere** — every "cost" claimed in modules 01–17 was
  implicitly a claim about a machine like this one; you now own the
  machine. Re-read module 06's "comparisons" or module 15's "I/O
  transfers" as $\mu$ and $\upsilon$ and the whole course snaps into one picture.
- **Forwards, to real systems** — a RV64I emulator, a bytecode VM for a
  toy language, or a JIT: all three are this module plus persistence.
  And when you meet Knuth's MMIX programs in later fascicles, you can
  run them.

*This is the last module. $\gcd(544, 119) = 17$ — but you knew that; now a
machine you built from nothing knows it too, in 22 oops. Onward.*
