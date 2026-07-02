# Hints — Module 18: MMIX — Knuth's Machine

Graduated hints, three per stage, gentlest first. Read only as far as you
need; after a stage is green, read `WALKTHROUGH.md`.

## Stage 1: machine state, memory, loads and stores

1. A machine state is the §1.1 computational method (Q, I, Ω, f) with
   `step()` playing f. The three rules that decide this stage: memory is
   **big-endian** (most significant byte at the smallest address),
   alignment **rounds down** (a size-s access ignores the low bits —
   `addr & !(s−1)`), and unwritten memory reads as zero.
2. Registers are `[u64; 256]`, memory a sparse `HashMap<u64, u8>`. Build
   `ld_/st_` for byte/wyde/tetra/octa big-endian. `load_program` writes
   the tetras and sets `pc`. In `step`, fetch the tetra at `pc & !3`, split
   OP X Y Z, advance `pc` by 4 *before* executing (branches are relative to
   the instruction's own address), then LDx sign-extends / LDxU
   zero-extends and STx writes the low bytes of $X. The effective address
   is $Y + (Z operand), wrapping.
3. Octa load: `let a = addr & !7; let mut v=0u64; for i in 0..8 { v =
   (v<<8) | ld_byte(a.wrapping_add(i)) as u64 }`. Sign-extend a byte load
   with `ld_byte(addr) as i8 as i64 as u64`. The Z operand is
   `if opc & 1 == 1 { z as u64 } else { regs[z] }`; address is
   `regs[y].wrapping_add(zv)`.

## Stage 2: arithmetic

1. The one instruction that deserves care is DIV: MMIX uses **floor**
   division, so the remainder always takes the *divisor's* sign (§3.2's
   theorem) — this is the `mod` the whole series relies on. Signed ADD/SUB/
   MUL/NEG/SL **wrap** (no overflow trips), and shifts do **not** reduce
   the count mod 64.
2. ADD/SUB/MUL are wrapping ops. MULU forms the full 128-bit product: low
   half to $X, high half to rH. For DIV, compute the truncated quotient
   then subtract 1 when the remainder is nonzero and the operand signs
   differ — do it in i128 so `i64::MIN / −1` cannot overflow. Divide by
   zero: $X = 0, rR = $Y (both DIV and DIVU). CMP gives −1/0/1; NEG is
   Y − Z with Y *always* an immediate byte; a shift count ≥ 64 yields 0
   (SR fills with the sign). GET reads rR (code 6) or rH (code 3).
3. Floor fix-up: `let mut q = yq/zq; if yq%zq != 0 && (yq<0) != (zq<0) {
   q -= 1; } let r = yq - zq*q;` with yq, zq as i128. MULU:
   `let p = regs[y] as u128 * zv as u128; regs[x] = p as u64;
   rh = (p >> 64) as u64;`. Left shift: `if zv >= 64 { 0 } else { yv << zv }`.

## Stage 3: branches, loops, and the assembler

1. All control flow is **pc-relative and counted in tetras**: a taken
   branch goes to @ + 4·YZ (even/forward opcode) or @ + 4·(YZ − 2¹⁶)
   (odd/backward); JMP is the same over the 24-bit XYZ field. The assembler
   is two-pass — you cannot encode a forward branch until pass 1 has
   recorded where every label lands.
2. BN/BZ/BNN/BNZ test $X as a signed octabyte and set `pc = btarget` only
   when the condition holds. `run(max_steps)` loops `step`, returns the
   count executed, and lets the caller check `halted()` — that is the
   finiteness guard. An opcode outside the subset is
   `Fault::IllegalOpcode`. Assembler pass 1 records label → instruction
   index; pass 2 encodes, choosing the odd opcode for an immediate Z or a
   negative offset.
3. Branch encoding: if 0 ≤ d < 65536 use the base opcode with YZ = d; if
   −65536 ≤ d < 0 use `base | 1` with YZ = d + 65536. Hand-check the pins:
   `SUB $2,$2,1` → `0x25020201` (immediate flips the low bit), and a
   backward `BNZ $2,LOOP` two tetras back → `0x4B02FFFE` (opcode 0x4A|1,
   YZ = 2¹⁶ − 2).

## Stage 4: programs on the metal

1. No new machine code — this stage assembles the embedded EUCLID and
   FINDMAX sources and runs them, checking answers *and* costs. The cost is
   a closed form: Euclid runs 6T − 2 oops (T = Module 01's division count),
   0 mems; FindMax always makes exactly n mems.
2. For each program: `assemble` the source, set the input registers,
   `load_program` at any address (the code is position-independent), `run`
   with a generous budget, then read the answer register and the `oops()`/
   `mems()` counters.
3. gcd(544, 119) = 17 lands in $1 at exactly **22 oops and 0 mems**;
   FindMax on Knuth's sixteen keys reports max 908 at position 5 (last
   index on ties) and n mems. Assert `m.halted()` alongside the values.
