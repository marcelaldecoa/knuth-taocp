# Walkthrough — Module 18: MMIX — Knuth's Machine

Read this AFTER a stage is green — it explains how the reference solution is
built and why.

## Stage 1: machine state, memory, loads and stores

The memory layer is built bottom-up from a single primitive: `ld_byte` /
`st_byte` over a sparse `HashMap<u64, u8>` where a missing key reads as zero.
Every wider access is composed from it, and each rounds its address down
(`addr & !1`, `& !3`, `& !7`) *before* touching bytes — that is MMIX's
"ignore the low bits" alignment realized in one masking step, so misalignment
is defined rather than faulted (the test that `ld_octa(0x105)` reads the octa
at 0x100 is exactly this). The big-endian order lives in the shift direction:
loads accumulate `v = (v << 8) | byte`, stores emit `(v >> 8*(size-1-i))` —
most significant byte first, matching a hex dump.

`step` is the fetch–decode–execute skeleton every later stage hangs off. Two
details are easy to get backwards and the reference is deliberate about both.
First, `let at = self.pc & !3;` captures the instruction's own address, then
`self.pc = at.wrapping_add(4)` advances *before* execution — so when a branch
later computes its target it is relative to `at`, not the already-incremented
pc. Second, the immediate/register decode is centralized: `zv = if opc & 1 ==
1 { z as u64 } else { regs[z] }`, computed once, so every operate/load/store
arm reads a single `zv` and the odd-opcode immediate rule is stated in exactly
one place. The sign-extension chains (`as i8 as i64 as u64`) are the whole
content of the signed loads; the unsigned twins just cast `as u64`.

## Stage 2: arithmetic

The arithmetic arms are mostly `wrapping_*` one-liners — and that uniformity
is the point: because MMIX-LITE wraps instead of trapping, each signed op
agrees bit-for-bit with its unsigned twin, so ADD and ADDU share an arm. The
instruction that earns its comment is DIV. Rust's `/` truncates toward zero;
the reference computes the truncated quotient in **i128** (so the single
overflow case `i64::MIN / −1` cannot panic) and then applies the floor
correction `if yq % zq != 0 && (yq < 0) != (zq < 0) { q -= 1 }` — subtract one
exactly when the true quotient is negative and non-integral. The remainder is
recovered as `r = yq - zq*q`, which automatically carries the divisor's sign,
reproducing all four rows of §3.2's table. Division by zero is handled as
Knuth defines it (§1.2.4's `y mod 0 = y`): `$X = 0, rR = $Y`, no trap.

MULU is the other arm with state beyond \$X: it widens both operands to u128,
writes the low half to \$X and the high half to rH — and signed MUL pointedly
does *not* touch rH, matching the fascicle. The shift arms encode "no mod-64
reduction" explicitly (`if zv >= 64 { 0 }`, with SR filling the sign), because
Rust's `<<`/`>>` on a count ≥ 64 is undefined-ish and x86 would silently mask
the count — the guard is what makes the ≥ 64 tests pass.

## Stage 3: branches, loops, and the assembler

The branch and JMP arms reuse the `btarget`/immediate machinery `step` already
computed at the top: `yz = (y<<8)|z`, and `btarget = at + 4·yz` (even) or
`at + 4·(yz − 65536)` (odd). Because the offset is added to `at` (the
instruction's own address) and scaled by 4 (tetras, not bytes), the assembled
code is position-independent — which is why stage 3's tests load the same
words at three addresses and demand identical behavior. `run` is the
finiteness guard made testable: it counts steps up to a budget and stops,
returning the count while leaving `halted()` for the caller to inspect — you
can `assert_eq!` "terminates within N" even though "terminates" is undecidable.

`assemble` is the classic two-pass compiler. Pass 1 strips comments, decides
whether the first token is a mnemonic or a label (via `is_mnemonic`, which
consults the *same* opcode tables the machine uses — one source of truth), and
records `label → tetra index`. Pass 2 encodes, and the two encoding decisions
both hinge on one bit: an operate/load/store with a non-`$` third operand
takes `base | 1` (immediate Z); a branch/JMP with a negative resolved offset
takes `base | 1` (backward) with YZ = d + 65536. The idiom worth stealing is
`target_offset` returning a signed tetra delta `idx − from`, so forward and
backward are the same arithmetic and only the sign selects the opcode form —
which is exactly why hand-encoding `0x4B02FFFE` for a −2 offset falls straight
out.

## Stage 4: programs on the metal

There is no new code here; the stage is the payoff of getting stages 1–3
honest. The EUCLID source maps Algorithm E's three steps onto seven tetras —
DIV's two outputs (\$X quotient, rR remainder) absorb step E1's two results,
`GET $3,rR` lifts the remainder, `BZ` is step E2, and `ADD $x,$y,0` is MMIX's
register-copy idiom (the immediate form, opcode 0x21, because \$0 is not a
hardwired zero). Running it on (544, 119) gives 17 in \$1 at exactly 22 oops
and 0 mems, because each of the T = 4 divisions costs a fixed six-instruction
pass except the last (three) plus the halting TRAP: 6T − 2. That closed form
is Module 01's Lamé bound turned into an assertable machine cost — the
counters make cost a pure function of (program, input), so the test can pin it.
FindMax's contract is the memory-side twin: exactly n mems always (it reads
each key once), with the oops count tracking how often step M4 fires — the
quantity whose average H_n − 1 you computed in Module 02. The analysis and the
machine agree, which is the entire thesis of the module.
