//! Module 18 — MMIX: Knuth's Machine ("MMIX-LITE").
//! Source: TAOCP Vol. 1, Fascicle 1 (MMIX: A RISC Computer for the New
//! Millennium), §1.3.1´–1.3.2´; programs adapted from §1.1 (Algorithm E)
//! and §1.2.10 (Algorithm M).
//!
//! An honest **subset** of MMIX:
//! - 256 general registers `$0..$255` (all general; no rG/rL local-register
//!   machinery), byte-addressable big-endian memory (sparse
//!   `HashMap<u64, u8>`, defaulting to zero), a program counter.
//! - Instructions are 4 bytes `OP X Y Z`, encoded exactly as in Fascicle 1,
//!   with MMIX's true numeric opcodes (see [`op`]); the MMIX convention
//!   "odd opcode = Z is an immediate byte" is kept for every operate/load/
//!   store instruction, and "odd opcode = backward" for branches and JMP.
//! - Special registers: only stand-ins for rR (remainder, [`Mmix::remainder`])
//!   and rH (high product, [`Mmix::himult`]), readable in programs via
//!   `GET $X,rR` / `GET $X,rH`.
//! - No floating point, no overflow trips (signed ops wrap), no rD (DIVU
//!   divides a 64-bit dividend, i.e. behaves as if rD = 0), no TRIP/TRAP
//!   system beyond "TRAP halts", no MMIXAL pseudo-ops beyond labels.
//!
//! Cost model (Knuth's, simplified): [`Mmix::oops`] counts instructions
//! executed (υ, "oops"), [`Mmix::mems`] counts memory references made by
//! load/store instructions (μ, "mems"). Fascicle 1 additionally weights slow
//! instructions (MUL 10υ, DIV 60υ, mispredicted branches +2υ); we count each
//! instruction once and discuss the full table in the lesson.

use std::collections::HashMap;

/// MMIX opcode values (from Fascicle 1's opcode chart) for the MMIX-LITE
/// subset, plus the special-register codes used by `GET`.
///
/// These are the *true* MMIX numeric opcodes to the best of our
/// transcription; each is defined once here and everything else — machine,
/// assembler, tests — goes through these names, so this table is the single
/// place to verify against Fascicle 1.
pub mod op {
    /// TRAP — in full MMIX, a trap to the operating system; in MMIX-LITE
    /// every TRAP halts the machine (we only ever write `TRAP 0,0,0`,
    /// Fascicle 1's `TRAP 0,Halt,0`).
    pub const TRAP: u8 = 0x00;

    // Multiplication and division (Fascicle 1 prices these 10υ and 60υ).
    pub const MUL: u8 = 0x18;
    pub const MULI: u8 = 0x19;
    pub const MULU: u8 = 0x1A;
    pub const MULUI: u8 = 0x1B;
    pub const DIV: u8 = 0x1C;
    pub const DIVI: u8 = 0x1D;
    pub const DIVU: u8 = 0x1E;
    pub const DIVUI: u8 = 0x1F;

    // Addition and subtraction.
    pub const ADD: u8 = 0x20;
    pub const ADDI: u8 = 0x21;
    pub const ADDU: u8 = 0x22;
    pub const ADDUI: u8 = 0x23;
    pub const SUB: u8 = 0x24;
    pub const SUBI: u8 = 0x25;
    pub const SUBU: u8 = 0x26;
    pub const SUBUI: u8 = 0x27;

    // Comparison, negation, shifts.
    pub const CMP: u8 = 0x30;
    pub const CMPI: u8 = 0x31;
    pub const CMPU: u8 = 0x32;
    pub const CMPUI: u8 = 0x33;
    pub const NEG: u8 = 0x34;
    pub const NEGI: u8 = 0x35;
    pub const NEGU: u8 = 0x36;
    pub const NEGUI: u8 = 0x37;
    pub const SL: u8 = 0x38;
    pub const SLI: u8 = 0x39;
    pub const SLU: u8 = 0x3A;
    pub const SLUI: u8 = 0x3B;
    pub const SR: u8 = 0x3C;
    pub const SRI: u8 = 0x3D;
    pub const SRU: u8 = 0x3E;
    pub const SRUI: u8 = 0x3F;

    // Conditional branches. Even opcode = forward (@ + 4·YZ), odd opcode =
    // backward (@ + 4·(YZ − 65536)); the assembler picks the form.
    pub const BN: u8 = 0x40;
    pub const BNB: u8 = 0x41;
    pub const BZ: u8 = 0x42;
    pub const BZB: u8 = 0x43;
    pub const BNN: u8 = 0x48;
    pub const BNNB: u8 = 0x49;
    pub const BNZ: u8 = 0x4A;
    pub const BNZB: u8 = 0x4B;

    // Loads: signed loads sign-extend, unsigned loads zero-extend.
    pub const LDB: u8 = 0x80;
    pub const LDBI: u8 = 0x81;
    pub const LDBU: u8 = 0x82;
    pub const LDBUI: u8 = 0x83;
    pub const LDW: u8 = 0x84;
    pub const LDWI: u8 = 0x85;
    pub const LDWU: u8 = 0x86;
    pub const LDWUI: u8 = 0x87;
    pub const LDT: u8 = 0x88;
    pub const LDTI: u8 = 0x89;
    pub const LDTU: u8 = 0x8A;
    pub const LDTUI: u8 = 0x8B;
    pub const LDO: u8 = 0x8C;
    pub const LDOI: u8 = 0x8D;
    pub const LDOU: u8 = 0x8E;
    pub const LDOUI: u8 = 0x8F;

    // Stores (STB/STW/STT store the low bytes of $X; full MMIX would also
    // check for overflow on the signed forms — MMIX-LITE never trips).
    pub const STB: u8 = 0xA0;
    pub const STBI: u8 = 0xA1;
    pub const STW: u8 = 0xA4;
    pub const STWI: u8 = 0xA5;
    pub const STT: u8 = 0xA8;
    pub const STTI: u8 = 0xA9;
    pub const STO: u8 = 0xAC;
    pub const STOI: u8 = 0xAD;

    /// SETL $X,YZ — set $X to the 16-bit immediate YZ (low wyde; the
    /// SETH/SETMH/SETML siblings at 0xE0–0xE2 are not in the subset).
    pub const SETL: u8 = 0xE3;

    /// JMP — relative jump, 24-bit tetra offset XYZ (odd form = backward).
    pub const JMP: u8 = 0xF0;
    pub const JMPB: u8 = 0xF1;

    /// GET $X,Z — read special register number Z into $X. MMIX-LITE
    /// implements only Z = `SPEC_RH` (3, rH) and Z = `SPEC_RR` (6, rR).
    pub const GET: u8 = 0xFE;

    /// Special-register code of rH (himult) — verify against Fascicle 1.
    pub const SPEC_RH: u8 = 3;
    /// Special-register code of rR (remainder) — verify against Fascicle 1.
    pub const SPEC_RR: u8 = 6;
}

/// A runtime fault: the machine met an instruction MMIX-LITE cannot execute
/// (an opcode outside the subset, or `GET` of an unimplemented special
/// register). `at` is the address of the offending instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    IllegalOpcode { opcode: u8, at: u64 },
}

/// The MMIX-LITE machine: 256 octabyte registers, sparse big-endian memory,
/// a program counter, rR/rH stand-ins, and Knuth-style cost counters.
pub struct Mmix {
    regs: [u64; 256],
    mem: HashMap<u64, u8>,
    pc: u64,
    halted: bool,
    /// Stand-in for MMIX's remainder register rR (written by DIV/DIVU).
    rr: u64,
    /// Stand-in for MMIX's himult register rH (written by MULU).
    rh: u64,
    mems: u64,
    oops: u64,
}

impl Mmix {
    /// A fresh machine: every register and every byte of memory is zero,
    /// the program counter is 0, the machine is not halted.
    pub fn new() -> Self {
        Mmix {
            regs: [0; 256],
            mem: HashMap::new(),
            pc: 0,
            halted: false,
            rr: 0,
            rh: 0,
            mems: 0,
            oops: 0,
        }
    }

    // -- registers and special registers -------------------------------------

    /// Read general register `$x`.
    pub fn reg(&self, x: u8) -> u64 {
        self.regs[x as usize]
    }

    /// Write general register `$x`.
    pub fn set_reg(&mut self, x: u8, v: u64) {
        self.regs[x as usize] = v;
    }

    /// The program counter (MMIX writes it `@`, "the place where we're at").
    pub fn pc(&self) -> u64 {
        self.pc
    }

    /// Set the program counter.
    pub fn set_pc(&mut self, pc: u64) {
        self.pc = pc;
    }

    /// True once a TRAP has halted the machine.
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// Stand-in for special register rR: the remainder left by the most
    /// recent DIV/DIVU. (Programs read it with `GET $X,rR`.)
    pub fn remainder(&self) -> u64 {
        self.rr
    }

    /// Stand-in for special register rH: the high 64 bits of the most
    /// recent MULU product. (Programs read it with `GET $X,rH`.)
    pub fn himult(&self) -> u64 {
        self.rh
    }

    /// μ — memory references performed by executed load/store instructions.
    /// (Debug accessors like [`ld_octa`](Mmix::ld_octa) are free.)
    pub fn mems(&self) -> u64 {
        self.mems
    }

    /// υ — instructions executed, including the halting TRAP.
    pub fn oops(&self) -> u64 {
        self.oops
    }

    // -- memory ---------------------------------------------------------------
    //
    // Big-endian throughout, as in MMIX: the most significant byte of a
    // wyde/tetra/octa lives at the smallest address. Addresses are rounded
    // DOWN to a multiple of the access size (MMIX simply ignores the low
    // bits — Fascicle 1, §1.3.1´). Unwritten memory reads as zero.

    /// Read one byte.
    pub fn ld_byte(&self, addr: u64) -> u8 {
        *self.mem.get(&addr).unwrap_or(&0)
    }

    /// Write one byte.
    pub fn st_byte(&mut self, addr: u64, v: u8) {
        self.mem.insert(addr, v);
    }

    /// Read a wyde (2 bytes, big-endian) at `addr & !1`.
    pub fn ld_wyde(&self, addr: u64) -> u16 {
        let a = addr & !1;
        ((self.ld_byte(a) as u16) << 8) | self.ld_byte(a.wrapping_add(1)) as u16
    }

    /// Write a wyde at `addr & !1`.
    pub fn st_wyde(&mut self, addr: u64, v: u16) {
        let a = addr & !1;
        self.st_byte(a, (v >> 8) as u8);
        self.st_byte(a.wrapping_add(1), v as u8);
    }

    /// Read a tetra (4 bytes, big-endian) at `addr & !3`.
    pub fn ld_tetra(&self, addr: u64) -> u32 {
        let a = addr & !3;
        let mut v = 0u32;
        for i in 0..4 {
            v = (v << 8) | self.ld_byte(a.wrapping_add(i)) as u32;
        }
        v
    }

    /// Write a tetra at `addr & !3`.
    pub fn st_tetra(&mut self, addr: u64, v: u32) {
        let a = addr & !3;
        for i in 0..4 {
            self.st_byte(a.wrapping_add(i), (v >> (8 * (3 - i))) as u8);
        }
    }

    /// Read an octa (8 bytes, big-endian) at `addr & !7`.
    pub fn ld_octa(&self, addr: u64) -> u64 {
        let a = addr & !7;
        let mut v = 0u64;
        for i in 0..8 {
            v = (v << 8) | self.ld_byte(a.wrapping_add(i)) as u64;
        }
        v
    }

    /// Write an octa at `addr & !7`.
    pub fn st_octa(&mut self, addr: u64, v: u64) {
        let a = addr & !7;
        for i in 0..8 {
            self.st_byte(a.wrapping_add(i), (v >> (8 * (7 - i))) as u8);
        }
    }

    // -- program loading and execution ------------------------------------------

    /// Store the tetras of `words` at `addr, addr+4, …` (big-endian), set
    /// the program counter to `addr`, and clear the halted flag. Cost
    /// counters are *not* reset — use a fresh machine to measure a program.
    pub fn load_program(&mut self, addr: u64, words: &[u32]) {
        for (i, &w) in words.iter().enumerate() {
            self.st_tetra(addr.wrapping_add(4 * i as u64), w);
        }
        self.pc = addr;
        self.halted = false;
    }

    /// Execute one instruction. Returns `Ok(true)` if the machine is still
    /// running, `Ok(false)` if it halted (executed a TRAP, or was already
    /// halted), `Err(fault)` on an instruction outside the subset.
    ///
    /// Fetch–decode–execute: fetch the tetra at `@` (low two bits ignored),
    /// split it into OP X Y Z, advance `@` by 4, then do what OP says.
    pub fn step(&mut self) -> Result<bool, Fault> {
        if self.halted {
            return Ok(false);
        }
        let at = self.pc & !3;
        let w = self.ld_tetra(at);
        let opc = (w >> 24) as u8;
        let x = (w >> 16) as u8;
        let y = (w >> 8) as u8;
        let z = w as u8;
        self.pc = at.wrapping_add(4);
        self.oops += 1;

        // The MMIX immediate convention: for operate/load/store opcodes the
        // odd variant takes Z as an unsigned byte instead of a register.
        let zv = if opc & 1 == 1 {
            z as u64
        } else {
            self.regs[z as usize]
        };
        // Branch target: even = forward @ + 4·YZ, odd = backward
        // @ + 4·(YZ − 65536), both relative to the branch's own address.
        let yz = ((y as u32) << 8) | z as u32;
        let btarget = if opc & 1 == 1 {
            at.wrapping_add((4 * (yz as i64 - 65536)) as u64)
        } else {
            at.wrapping_add(4 * yz as u64)
        };
        // Load/store address: A = ($Y + Z) mod 2^64.
        let addr = self.regs[y as usize].wrapping_add(zv);

        match opc {
            // TRAP — halt (MMIX-LITE treats every TRAP as Fascicle 1's
            // `TRAP 0,Halt,0`).
            op::TRAP => {
                self.halted = true;
                return Ok(false);
            }

            // -- arithmetic -----------------------------------------------
            op::ADD | op::ADDI | op::ADDU | op::ADDUI => {
                // Full MMIX: ADD trips on signed overflow, ADDU never does.
                // MMIX-LITE: both wrap (two's complement makes them agree).
                self.regs[x as usize] = self.regs[y as usize].wrapping_add(zv);
            }
            op::SUB | op::SUBI | op::SUBU | op::SUBUI => {
                self.regs[x as usize] = self.regs[y as usize].wrapping_sub(zv);
            }
            op::MUL | op::MULI => {
                // Signed multiply; full MMIX trips on overflow, we wrap.
                // MUL does NOT touch rH — only MULU does.
                self.regs[x as usize] = self.regs[y as usize].wrapping_mul(zv);
            }
            op::MULU | op::MULUI => {
                // Unsigned 64×64 → 128: low half to $X, high half to rH.
                let p = self.regs[y as usize] as u128 * zv as u128;
                self.regs[x as usize] = p as u64;
                self.rh = (p >> 64) as u64;
            }
            op::DIV | op::DIVI => {
                // Signed FLOOR division (Fascicle 1): $X = ⌊$Y / Z⌋ and
                // rR = $Y − Z·⌊$Y / Z⌋, so the remainder has the divisor's
                // sign. Division by zero: $X = 0 and rR = $Y (full MMIX also
                // raises an "integer divide check"; MMIX-LITE never trips).
                let yv = self.regs[y as usize] as i64;
                let zi = zv as i64;
                if zi == 0 {
                    self.regs[x as usize] = 0;
                    self.rr = yv as u64;
                } else {
                    // Work in i128 so i64::MIN / −1 cannot overflow, then
                    // wrap the quotient back to 64 bits.
                    let (yq, zq) = (yv as i128, zi as i128);
                    let mut q = yq / zq; // truncated…
                    if yq % zq != 0 && (yq < 0) != (zq < 0) {
                        q -= 1; // …corrected to floor
                    }
                    let r = yq - zq * q;
                    self.regs[x as usize] = q as u64;
                    self.rr = r as i64 as u64;
                }
            }
            op::DIVU | op::DIVUI => {
                // Unsigned division. Full MMIX divides the 128-bit value
                // rD‖$Y; MMIX-LITE has no rD (behaves as rD = 0), so this is
                // plain 64-bit division. Division by zero: $X = 0, rR = $Y —
                // exactly full MMIX's answer when rD = 0.
                if zv == 0 {
                    self.regs[x as usize] = 0;
                    self.rr = self.regs[y as usize];
                } else {
                    self.rr = self.regs[y as usize] % zv;
                    self.regs[x as usize] = self.regs[y as usize] / zv;
                }
            }
            op::CMP | op::CMPI => {
                // $X ← −1, 0, or 1 as $Y <, =, > Z (signed); −1 is all ones.
                let yv = self.regs[y as usize] as i64;
                let zi = zv as i64;
                self.regs[x as usize] = match yv.cmp(&zi) {
                    std::cmp::Ordering::Less => u64::MAX,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
            }
            op::CMPU | op::CMPUI => {
                self.regs[x as usize] = match self.regs[y as usize].cmp(&zv) {
                    std::cmp::Ordering::Less => u64::MAX,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
            }
            op::NEG | op::NEGI | op::NEGU | op::NEGUI => {
                // $X ← Y − Z where Y is ALWAYS an immediate byte (this is
                // how MMIX spells negation: NEG $X,0,$Z). Signed NEG would
                // trip on overflow in full MMIX; MMIX-LITE wraps, which
                // makes NEG and NEGU compute the same bits.
                self.regs[x as usize] = (y as u64).wrapping_sub(zv);
            }
            op::SL | op::SLI | op::SLU | op::SLUI => {
                // Left shift; counts ≥ 64 yield 0 (MMIX does not reduce the
                // count mod 64). Full MMIX: SL trips on signed overflow,
                // SLU doesn't; MMIX-LITE wraps both.
                let yv = self.regs[y as usize];
                self.regs[x as usize] = if zv >= 64 { 0 } else { yv << zv };
            }
            op::SR | op::SRI => {
                // Arithmetic right shift: sign-fills; counts ≥ 64 give all
                // sign bits (0 or −1).
                let yv = self.regs[y as usize] as i64;
                self.regs[x as usize] = if zv >= 64 {
                    if yv < 0 {
                        u64::MAX
                    } else {
                        0
                    }
                } else {
                    (yv >> zv) as u64
                };
            }
            op::SRU | op::SRUI => {
                // Logical right shift: zero-fills; counts ≥ 64 give 0.
                let yv = self.regs[y as usize];
                self.regs[x as usize] = if zv >= 64 { 0 } else { yv >> zv };
            }

            // -- loads (address = ($Y + Z), low bits ignored per size;
            //    signed loads sign-extend, unsigned loads zero-extend) -----
            op::LDB | op::LDBI => {
                self.mems += 1;
                self.regs[x as usize] = self.ld_byte(addr) as i8 as i64 as u64;
            }
            op::LDBU | op::LDBUI => {
                self.mems += 1;
                self.regs[x as usize] = self.ld_byte(addr) as u64;
            }
            op::LDW | op::LDWI => {
                self.mems += 1;
                self.regs[x as usize] = self.ld_wyde(addr) as i16 as i64 as u64;
            }
            op::LDWU | op::LDWUI => {
                self.mems += 1;
                self.regs[x as usize] = self.ld_wyde(addr) as u64;
            }
            op::LDT | op::LDTI => {
                self.mems += 1;
                self.regs[x as usize] = self.ld_tetra(addr) as i32 as i64 as u64;
            }
            op::LDTU | op::LDTUI => {
                self.mems += 1;
                self.regs[x as usize] = self.ld_tetra(addr) as u64;
            }
            op::LDO | op::LDOI | op::LDOU | op::LDOUI => {
                // A whole octa is a whole octa: LDO and LDOU agree.
                self.mems += 1;
                self.regs[x as usize] = self.ld_octa(addr);
            }

            // -- stores (low bytes of $X) -----------------------------------
            op::STB | op::STBI => {
                self.mems += 1;
                let v = self.regs[x as usize] as u8;
                self.st_byte(addr, v);
            }
            op::STW | op::STWI => {
                self.mems += 1;
                let v = self.regs[x as usize] as u16;
                self.st_wyde(addr, v);
            }
            op::STT | op::STTI => {
                self.mems += 1;
                let v = self.regs[x as usize] as u32;
                self.st_tetra(addr, v);
            }
            op::STO | op::STOI => {
                self.mems += 1;
                let v = self.regs[x as usize];
                self.st_octa(addr, v);
            }

            // -- constants, branches, jumps ---------------------------------
            op::SETL => {
                // $X ← YZ (16-bit immediate, zero-extended).
                self.regs[x as usize] = yz as u64;
            }
            op::BN | op::BNB => {
                if (self.regs[x as usize] as i64) < 0 {
                    self.pc = btarget;
                }
            }
            op::BZ | op::BZB => {
                if self.regs[x as usize] == 0 {
                    self.pc = btarget;
                }
            }
            op::BNN | op::BNNB => {
                if (self.regs[x as usize] as i64) >= 0 {
                    self.pc = btarget;
                }
            }
            op::BNZ | op::BNZB => {
                if self.regs[x as usize] != 0 {
                    self.pc = btarget;
                }
            }
            op::JMP | op::JMPB => {
                // 24-bit relative jump, offset XYZ in tetras.
                let xyz = (w & 0x00FF_FFFF) as i64;
                let d = if opc & 1 == 1 { xyz - (1 << 24) } else { xyz };
                self.pc = at.wrapping_add((4 * d) as u64);
            }
            op::GET => {
                // GET $X,Z — special register Z. Only rH (3) and rR (6).
                match z {
                    op::SPEC_RH => self.regs[x as usize] = self.rh,
                    op::SPEC_RR => self.regs[x as usize] = self.rr,
                    _ => return Err(Fault::IllegalOpcode { opcode: opc, at }),
                }
            }

            _ => return Err(Fault::IllegalOpcode { opcode: opc, at }),
        }
        Ok(true)
    }

    /// Run for at most `max_steps` instructions. Returns the number of
    /// instructions actually executed (the halting TRAP counts). Check
    /// [`halted`](Mmix::halted) to tell "program finished" from "budget
    /// exhausted" — the guard that makes finiteness a *testable* property.
    pub fn run(&mut self, max_steps: u64) -> Result<u64, Fault> {
        let mut n = 0;
        while n < max_steps && !self.halted {
            let running = self.step()?;
            n += 1;
            if !running {
                break;
            }
        }
        Ok(n)
    }
}

impl Default for Mmix {
    fn default() -> Self {
        Mmix::new()
    }
}

// ---------------------------------------------------------------------------
// The assembler — a two-pass MMIXAL-in-miniature.
// ---------------------------------------------------------------------------

/// Mnemonics of the "$X,$Y,Z-or-immediate" shape and their (even) opcodes.
const RRZ_OPS: &[(&str, u8)] = &[
    ("ADD", op::ADD),
    ("ADDU", op::ADDU),
    ("SUB", op::SUB),
    ("SUBU", op::SUBU),
    ("MUL", op::MUL),
    ("MULU", op::MULU),
    ("DIV", op::DIV),
    ("DIVU", op::DIVU),
    ("CMP", op::CMP),
    ("CMPU", op::CMPU),
    ("SL", op::SL),
    ("SLU", op::SLU),
    ("SR", op::SR),
    ("SRU", op::SRU),
    ("LDB", op::LDB),
    ("LDBU", op::LDBU),
    ("LDW", op::LDW),
    ("LDWU", op::LDWU),
    ("LDT", op::LDT),
    ("LDTU", op::LDTU),
    ("LDO", op::LDO),
    ("LDOU", op::LDOU),
    ("STB", op::STB),
    ("STW", op::STW),
    ("STT", op::STT),
    ("STO", op::STO),
];

/// Branch mnemonics and their (forward) opcodes.
const BRANCH_OPS: &[(&str, u8)] = &[
    ("BN", op::BN),
    ("BZ", op::BZ),
    ("BNN", op::BNN),
    ("BNZ", op::BNZ),
];

fn is_mnemonic(t: &str) -> bool {
    let u = t.to_ascii_uppercase();
    RRZ_OPS.iter().any(|&(m, _)| m == u)
        || BRANCH_OPS.iter().any(|&(m, _)| m == u)
        || matches!(u.as_str(), "NEG" | "NEGU" | "SETL" | "JMP" | "TRAP" | "GET")
}

fn enc(opc: u8, x: u8, y: u8, z: u8) -> u32 {
    ((opc as u32) << 24) | ((x as u32) << 16) | ((y as u32) << 8) | z as u32
}

fn parse_reg(t: &str, line: usize) -> Result<u8, String> {
    let body = t
        .strip_prefix('$')
        .ok_or_else(|| format!("line {line}: expected a register like $3, got `{t}`"))?;
    let n: u32 = body
        .parse()
        .map_err(|_| format!("line {line}: bad register `{t}`"))?;
    if n > 255 {
        return Err(format!("line {line}: register `{t}` out of range $0..$255"));
    }
    Ok(n as u8)
}

/// Parse a literal: decimal (optionally negative), `0x…`/`#…` hexadecimal.
fn parse_imm(t: &str, line: usize) -> Result<i64, String> {
    let (neg, body) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t),
    };
    let mag: i64 = if let Some(h) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
        u64::from_str_radix(h, 16)
            .map(|v| v as i64)
            .map_err(|_| format!("line {line}: bad hex literal `{t}`"))?
    } else if let Some(h) = body.strip_prefix('#') {
        u64::from_str_radix(h, 16)
            .map(|v| v as i64)
            .map_err(|_| format!("line {line}: bad hex literal `{t}`"))?
    } else {
        body.parse::<i64>()
            .map_err(|_| format!("line {line}: bad literal `{t}`"))?
    };
    Ok(if neg { -mag } else { mag })
}

fn imm_in(t: &str, line: usize, lo: i64, hi: i64, what: &str) -> Result<i64, String> {
    let v = parse_imm(t, line)?;
    if v < lo || v > hi {
        return Err(format!("line {line}: {what} `{t}` out of range {lo}..={hi}"));
    }
    Ok(v)
}

/// Assemble MMIX-LITE assembly into instruction tetras.
///
/// Line format (MMIXAL in miniature): `[LABEL] OP X,Y,Z ; comment`.
/// A label is any first token that is not a mnemonic; a line may also be
/// just a label. Literals are decimal, `0x…` or `#…` hex. Registers are
/// `$0`..`$255`. Branch/JMP targets are labels (or literal tetra offsets).
/// `NEG $X,$Z` abbreviates `NEG $X,0,$Z`; `TRAP 0` abbreviates `TRAP 0,0,0`;
/// `GET $X,rR` / `GET $X,rH` read the two supported special registers.
///
/// Two passes, like every assembler since the 1950s: pass 1 walks the lines,
/// counting instructions and recording each label's tetra index in the
/// symbol table; pass 2 encodes, resolving label references relative to each
/// instruction's own index (all control flow is pc-relative, so the output
/// can be loaded at any address).
pub fn assemble(src: &str) -> Result<Vec<u32>, String> {
    struct Item {
        line: usize,
        mnem: String,
        args: Vec<String>,
        index: usize, // tetra index of this instruction
    }

    // Pass 1: strip comments, split labels from mnemonics, build symbols.
    let mut symbols: HashMap<String, usize> = HashMap::new();
    let mut items: Vec<Item> = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let line = i + 1;
        let text = raw.split(';').next().unwrap_or("");
        let mut tokens = text.split_whitespace();
        let first = match tokens.next() {
            None => continue, // blank or comment-only line
            Some(t) => t,
        };
        let mnem = if is_mnemonic(first) {
            first
        } else {
            // `first` is a label.
            if first.starts_with('$') || first.chars().next().unwrap().is_ascii_digit() {
                return Err(format!("line {line}: `{first}` cannot be a label"));
            }
            if symbols.insert(first.to_string(), items.len()).is_some() {
                return Err(format!("line {line}: duplicate label `{first}`"));
            }
            match tokens.next() {
                None => continue, // label-only line
                Some(t) if is_mnemonic(t) => t,
                Some(t) => return Err(format!("line {line}: unknown mnemonic `{t}`")),
            }
        };
        let joined = tokens.collect::<Vec<_>>().join(" ");
        let args: Vec<String> = if joined.trim().is_empty() {
            Vec::new()
        } else {
            joined.split(',').map(|s| s.trim().to_string()).collect()
        };
        if args.iter().any(|a| a.is_empty()) {
            return Err(format!("line {line}: empty operand"));
        }
        items.push(Item {
            line,
            mnem: mnem.to_ascii_uppercase(),
            args,
            index: items.len(),
        });
    }

    // Resolve a branch/jump target to a signed tetra offset from `from`.
    let target_offset = |t: &str, from: usize, line: usize| -> Result<i64, String> {
        if let Some(&idx) = symbols.get(t) {
            Ok(idx as i64 - from as i64)
        } else if t.starts_with('-')
            || t.starts_with('#')
            || t.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            parse_imm(t, line)
        } else {
            Err(format!("line {line}: unknown label `{t}`"))
        }
    };

    // Pass 2: encode.
    let mut out = Vec::with_capacity(items.len());
    for it in &items {
        let line = it.line;
        let argc = it.args.len();
        let need = |n: usize| -> Result<(), String> {
            if argc == n {
                Ok(())
            } else {
                Err(format!(
                    "line {line}: {} takes {n} operand(s), got {argc}",
                    it.mnem
                ))
            }
        };
        let word = if let Some(&(_, base)) = RRZ_OPS.iter().find(|&&(m, _)| m == it.mnem) {
            need(3)?;
            let x = parse_reg(&it.args[0], line)?;
            let y = parse_reg(&it.args[1], line)?;
            if it.args[2].starts_with('$') {
                enc(base, x, y, parse_reg(&it.args[2], line)?)
            } else {
                // The MMIX rule: odd opcode = Z is an immediate byte.
                let z = imm_in(&it.args[2], line, 0, 255, "immediate Z")?;
                enc(base | 1, x, y, z as u8)
            }
        } else if let Some(&(_, base)) = BRANCH_OPS.iter().find(|&&(m, _)| m == it.mnem) {
            need(2)?;
            let x = parse_reg(&it.args[0], line)?;
            let d = target_offset(&it.args[1], it.index, line)?;
            if (0..65536).contains(&d) {
                enc(base, x, (d >> 8) as u8, d as u8)
            } else if (-65536..0).contains(&d) {
                let yz = d + 65536;
                enc(base | 1, x, (yz >> 8) as u8, yz as u8)
            } else {
                return Err(format!("line {line}: branch offset {d} out of range"));
            }
        } else {
            match it.mnem.as_str() {
                "NEG" | "NEGU" => {
                    let base = if it.mnem == "NEG" { op::NEG } else { op::NEGU };
                    if argc != 2 && argc != 3 {
                        return Err(format!("line {line}: {} takes 2 or 3 operands", it.mnem));
                    }
                    let x = parse_reg(&it.args[0], line)?;
                    let (ytok, ztok) = if argc == 2 {
                        ("0", it.args[1].as_str())
                    } else {
                        (it.args[1].as_str(), it.args[2].as_str())
                    };
                    let y = imm_in(ytok, line, 0, 255, "immediate Y")? as u8;
                    if ztok.starts_with('$') {
                        enc(base, x, y, parse_reg(ztok, line)?)
                    } else {
                        let z = imm_in(ztok, line, 0, 255, "immediate Z")?;
                        enc(base | 1, x, y, z as u8)
                    }
                }
                "SETL" => {
                    need(2)?;
                    let x = parse_reg(&it.args[0], line)?;
                    let v = imm_in(&it.args[1], line, 0, 0xFFFF, "wyde immediate")?;
                    enc(op::SETL, x, (v >> 8) as u8, v as u8)
                }
                "JMP" => {
                    need(1)?;
                    let d = target_offset(&it.args[0], it.index, line)?;
                    if (0..1 << 24).contains(&d) {
                        ((op::JMP as u32) << 24) | d as u32
                    } else if (-(1 << 24)..0).contains(&d) {
                        ((op::JMPB as u32) << 24) | (d + (1 << 24)) as u32
                    } else {
                        return Err(format!("line {line}: jump offset {d} out of range"));
                    }
                }
                "TRAP" => {
                    if argc != 1 && argc != 3 {
                        return Err(format!("line {line}: TRAP takes 1 or 3 operands"));
                    }
                    if argc == 1 {
                        let z = imm_in(&it.args[0], line, 0, 255, "TRAP operand")?;
                        enc(op::TRAP, 0, 0, z as u8)
                    } else {
                        let xv = imm_in(&it.args[0], line, 0, 255, "TRAP operand")?;
                        let yv = imm_in(&it.args[1], line, 0, 255, "TRAP operand")?;
                        let z = imm_in(&it.args[2], line, 0, 255, "TRAP operand")?;
                        enc(op::TRAP, xv as u8, yv as u8, z as u8)
                    }
                }
                "GET" => {
                    need(2)?;
                    let x = parse_reg(&it.args[0], line)?;
                    let z = match it.args[1].as_str() {
                        "rR" | "rr" => op::SPEC_RR,
                        "rH" | "rh" => op::SPEC_RH,
                        t => imm_in(t, line, 0, 255, "special register")? as u8,
                    };
                    enc(op::GET, x, 0, z)
                }
                m => return Err(format!("line {line}: unknown mnemonic `{m}`")),
            }
        };
        out.push(word);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Worked examples from the text as unit tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_layout_op_x_y_z() {
        // Hand-assembly, Fascicle 1 style: ADD $1,$2,$3 is the four bytes
        // 0x20 01 02 03; the immediate form flips the low opcode bit.
        assert_eq!(assemble("ADD $1,$2,$3").unwrap(), vec![0x2001_0203]);
        assert_eq!(assemble("ADD $1,$2,3").unwrap(), vec![0x2101_0203]);
        assert_eq!(assemble("SETL $7,0x1234").unwrap(), vec![0xE307_1234]);
        assert_eq!(assemble("TRAP 0,0,0").unwrap(), vec![0x0000_0000]);
        assert_eq!(assemble("LDO $1,$2,$3").unwrap(), vec![0x8C01_0203]);
    }

    #[test]
    fn big_endian_sign_extension() {
        // Store 0x0123456789ABCDEF at 0x100: the byte at 0x104 is 0x89, and
        // LDB sign-extends it to …FF89 (§1.3.1´, signed loads).
        let mut m = Mmix::new();
        m.st_octa(0x100, 0x0123_4567_89AB_CDEF);
        assert_eq!(m.ld_byte(0x104), 0x89);
        m.set_reg(1, 0x100);
        m.load_program(0x1000, &[enc(op::LDBI, 2, 1, 4), enc(op::LDBUI, 3, 1, 4)]);
        m.step().unwrap();
        m.step().unwrap();
        assert_eq!(m.reg(2), 0xFFFF_FFFF_FFFF_FF89);
        assert_eq!(m.reg(3), 0x89);
        // MMIX ignores low address bits: an octa access at 0x105 reads 0x100.
        assert_eq!(m.ld_octa(0x105), 0x0123_4567_89AB_CDEF);
    }

    #[test]
    fn floor_division_examples() {
        // Fascicle 1: DIV is floor division; the remainder takes the
        // divisor's sign. The four sign combinations of 7 ÷ 2:
        for &(y, z, q, r) in &[
            (7i64, 2i64, 3i64, 1i64),
            (-7, 2, -4, 1),
            (7, -2, -4, -1),
            (-7, -2, 3, -1),
        ] {
            let mut m = Mmix::new();
            m.set_reg(1, y as u64);
            m.set_reg(2, z as u64);
            m.load_program(0, &[enc(op::DIV, 3, 1, 2)]);
            m.step().unwrap();
            assert_eq!(m.reg(3) as i64, q, "{y} / {z}");
            assert_eq!(m.remainder() as i64, r, "{y} mod {z}");
        }
    }

    #[test]
    fn mulu_high_half() {
        // (2^64 − 1)^2 = 0xFFFFFFFFFFFFFFFE_0000000000000001.
        let mut m = Mmix::new();
        m.set_reg(1, u64::MAX);
        m.load_program(0, &[enc(op::MULU, 2, 1, 1)]);
        m.step().unwrap();
        assert_eq!(m.reg(2), 1);
        assert_eq!(m.himult(), 0xFFFF_FFFF_FFFF_FFFE);
    }

    #[test]
    fn euclid_on_mmix_full_circle() {
        // Algorithm 1.1E compiled by hand; gcd(544, 119) = 17 in exactly
        // 4 divisions, hence 6·4 − 2 = 22 instructions (see module 01).
        let src = "
E1      DIV  $2,$0,$1     ; E1: rR <- m mod n
        GET  $3,rR
E2      BZ   $3,DONE      ; E2: r = 0? then n is the answer
E3      ADD  $0,$1,0      ; E3: m <- n
        ADD  $1,$3,0      ;     n <- r
        JMP  E1
DONE    TRAP 0,0,0
";
        let words = assemble(src).unwrap();
        let mut m = Mmix::new();
        m.set_reg(0, 544);
        m.set_reg(1, 119);
        m.load_program(0, &words);
        let steps = m.run(10_000).unwrap();
        assert!(m.halted());
        assert_eq!(m.reg(1), 17);
        assert_eq!(steps, 22);
        assert_eq!(m.oops(), 22);
        assert_eq!(m.mems(), 0); // Euclid never touches memory
    }

    #[test]
    fn counted_loop_sums_1_to_10() {
        let src = "
        SETL $1,0
        SETL $2,10
LOOP    ADD  $1,$1,$2
        SUB  $2,$2,1
        BNZ  $2,LOOP
        TRAP 0,0,0
";
        let words = assemble(src).unwrap();
        // Backward BNZ must use the odd opcode with YZ = 65536 − 2.
        assert_eq!(words[4], enc(op::BNZB, 2, 0xFF, 0xFE));
        let mut m = Mmix::new();
        m.load_program(0x100, &words);
        assert_eq!(m.run(1_000).unwrap(), 33);
        assert!(m.halted());
        assert_eq!(m.reg(1), 55);
    }

    #[test]
    fn assembler_reports_errors_with_line_numbers() {
        assert!(assemble("FROB $1,$2,$3").unwrap_err().contains("line 1"));
        assert!(assemble("BZ $1,NOWHERE").unwrap_err().contains("NOWHERE"));
        assert!(assemble("ADD $1,$2,999").is_err()); // immediate > 255
        assert!(assemble("ADD $300,$2,$3").is_err()); // no such register
        assert!(assemble("X ADD $1,$2,$3\nX SUB $1,$2,$3").is_err()); // dup label
    }
}
