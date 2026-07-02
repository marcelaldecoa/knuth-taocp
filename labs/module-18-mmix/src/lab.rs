//! Module 18 — MMIX: Knuth's Machine (TAOCP Vol. 1, Fascicle 1). The course
//! closer: you build a working subset of MMIX ("MMIX-LITE") and then run
//! Euclid's algorithm and FindMax — the very first programs of this course —
//! on your own metal.
//!
//! **Scaffolding tier — Module 05 and up:** the stub states the algorithm and
//! the contract and trusts you to translate it to Rust; the guided-tour aids of
//! Modules 01–04 are gone by design. The nets remain for every stage — the
//! lesson, three graduated hints (`--hint`), the reference, and the walkthrough.
//! (The taper is described in docs/for-newcomers.md §5.)
//!
//! YOUR WORKSPACE. Replace each `todo!()` with an implementation, then run
//! `./grade 18` from the repository root. The lesson in
//! `course/module-18-mmix/README.md` develops the machine model, the
//! instruction semantics (floor division!), the υ/μ cost model, and the
//! two-pass assembler these stubs refer to.
//!
//! The subset, honestly stated:
//! - 256 general registers `$0..$255` (u64), big-endian byte-addressable
//!   memory defaulting to zero (a sparse `HashMap<u64, u8>` works well),
//!   a program counter.
//! - Instructions are 4 bytes `OP X Y Z` with MMIX's true opcode values
//!   (given below in [`op`] — do not invent your own). MMIX rule: for
//!   operate/load/store instructions the ODD opcode means Z is an immediate
//!   byte; for branches/JMP the odd opcode means "backward".
//! - Special registers: stand-ins only — rR (remainder) via
//!   [`Mmix::remainder`], rH (himult) via [`Mmix::himult`], both readable in
//!   programs with `GET $X,rR` / `GET $X,rH`.
//! - No floating point, no overflow trips (signed arithmetic wraps), no rD
//!   (DIVU behaves as if rD = 0), no TRIP/TRAP system beyond "TRAP halts",
//!   no MMIXAL pseudo-ops beyond labels.
//!
//! Cost model (Knuth's): `oops()` counts instructions executed (υ),
//! `mems()` counts memory references by executed load/store instructions (μ).

/// MMIX opcode values (from Fascicle 1's opcode chart) for the MMIX-LITE
/// subset, plus the special-register codes used by `GET`.
///
/// These constants are GIVEN — they are the specification your machine and
/// assembler must implement, and the stage tests encode instructions with
/// them. Defined once here; verify against Fascicle 1's opcode table if you
/// own the book.
pub mod op {
    /// TRAP — MMIX-LITE halts on every TRAP (think `TRAP 0,Halt,0`).
    pub const TRAP: u8 = 0x00;

    // Multiplication and division.
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

    // Conditional branches: even = forward @ + 4·YZ, odd = backward
    // @ + 4·(YZ − 65536), relative to the branch instruction itself.
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

    // Stores (STB/STW/STT store the low bytes of $X; MMIX-LITE never trips).
    pub const STB: u8 = 0xA0;
    pub const STBI: u8 = 0xA1;
    pub const STW: u8 = 0xA4;
    pub const STWI: u8 = 0xA5;
    pub const STT: u8 = 0xA8;
    pub const STTI: u8 = 0xA9;
    pub const STO: u8 = 0xAC;
    pub const STOI: u8 = 0xAD;

    /// SETL $X,YZ — set $X to the 16-bit immediate YZ.
    pub const SETL: u8 = 0xE3;

    /// JMP — relative jump, 24-bit tetra offset XYZ (odd form = backward).
    pub const JMP: u8 = 0xF0;
    pub const JMPB: u8 = 0xF1;

    /// GET $X,Z — read special register Z into $X (only rH and rR exist).
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

/// The MMIX-LITE machine.
///
/// Suggested state (replace `_todo` with real fields; only the public
/// methods are graded):
///
/// ```text
/// regs:   [u64; 256]                             the registers $0..$255
/// mem:    std::collections::HashMap<u64, u8>     sparse memory, default 0
/// pc:     u64                                    the program counter (`@`)
/// halted: bool                                   set by TRAP
/// rr:     u64                  rR stand-in (remainder of last DIV/DIVU)
/// rh:     u64                  rH stand-in (high half of last MULU)
/// mems:   u64                  μ counter
/// oops:   u64                  υ counter
/// ```
pub struct Mmix {
    _todo: (),
}

impl Mmix {
    /// A fresh machine: every register and every byte of memory zero, the
    /// program counter 0, not halted, counters zero.
    pub fn new() -> Self {
        todo!("construct the initial machine state")
    }

    // -- registers and special registers -------------------------------------

    /// Read general register `$x`.
    pub fn reg(&self, x: u8) -> u64 {
        let _ = x;
        todo!("read a general register")
    }

    /// Write general register `$x`.
    pub fn set_reg(&mut self, x: u8, v: u64) {
        let _ = (x, v);
        todo!("write a general register")
    }

    /// The program counter (MMIX writes it `@`).
    pub fn pc(&self) -> u64 {
        todo!("return the program counter")
    }

    /// Set the program counter.
    pub fn set_pc(&mut self, pc: u64) {
        let _ = pc;
        todo!("set the program counter")
    }

    /// True once a TRAP has halted the machine.
    pub fn halted(&self) -> bool {
        todo!("return the halted flag")
    }

    /// Stand-in for special register rR: the remainder left by the most
    /// recent DIV/DIVU. Programs read it with `GET $X,rR`.
    pub fn remainder(&self) -> u64 {
        todo!("return the rR stand-in")
    }

    /// Stand-in for special register rH: the high 64 bits of the most
    /// recent MULU product. Programs read it with `GET $X,rH`.
    pub fn himult(&self) -> u64 {
        todo!("return the rH stand-in")
    }

    /// μ — memory references performed by executed load/store instructions
    /// (instruction fetch and the Rust-side `ld_*`/`st_*` accessors do NOT
    /// count).
    pub fn mems(&self) -> u64 {
        todo!("return the mems (mu) counter")
    }

    /// υ — instructions executed, including the halting TRAP.
    pub fn oops(&self) -> u64 {
        todo!("return the oops (upsilon) counter")
    }

    // -- memory ---------------------------------------------------------------
    //
    // Big-endian, as in MMIX: the most significant byte of a wyde/tetra/octa
    // lives at the smallest address. Addresses are rounded DOWN to a
    // multiple of the access size (MMIX ignores the low bits: a wyde access
    // uses addr & !1, a tetra addr & !3, an octa addr & !7). Unwritten
    // memory reads as zero.

    /// Read one byte.
    pub fn ld_byte(&self, addr: u64) -> u8 {
        let _ = addr;
        todo!("read a byte (default 0)")
    }

    /// Write one byte.
    pub fn st_byte(&mut self, addr: u64, v: u8) {
        let _ = (addr, v);
        todo!("write a byte")
    }

    /// Read a wyde (2 bytes, big-endian) at `addr & !1`.
    pub fn ld_wyde(&self, addr: u64) -> u16 {
        let _ = addr;
        todo!("read a big-endian wyde")
    }

    /// Write a wyde at `addr & !1`.
    pub fn st_wyde(&mut self, addr: u64, v: u16) {
        let _ = (addr, v);
        todo!("write a big-endian wyde")
    }

    /// Read a tetra (4 bytes, big-endian) at `addr & !3`.
    pub fn ld_tetra(&self, addr: u64) -> u32 {
        let _ = addr;
        todo!("read a big-endian tetra")
    }

    /// Write a tetra at `addr & !3`.
    pub fn st_tetra(&mut self, addr: u64, v: u32) {
        let _ = (addr, v);
        todo!("write a big-endian tetra")
    }

    /// Read an octa (8 bytes, big-endian) at `addr & !7`.
    pub fn ld_octa(&self, addr: u64) -> u64 {
        let _ = addr;
        todo!("read a big-endian octa")
    }

    /// Write an octa at `addr & !7`.
    pub fn st_octa(&mut self, addr: u64, v: u64) {
        let _ = (addr, v);
        todo!("write a big-endian octa")
    }

    // -- program loading and execution ------------------------------------------

    /// Store the tetras of `words` at `addr, addr+4, …` (big-endian), set
    /// the program counter to `addr`, and clear the halted flag. Do NOT
    /// reset the cost counters (use a fresh machine to measure a program).
    pub fn load_program(&mut self, addr: u64, words: &[u32]) {
        let _ = (addr, words);
        todo!("copy the program into memory and point pc at it")
    }

    /// Execute one instruction: the fetch–decode–execute cycle.
    ///
    /// ```text
    /// 1. If halted, return Ok(false).
    /// 2. Fetch the tetra w at `pc & !3`; split it into bytes OP X Y Z.
    /// 3. Advance pc by 4. Count one "oop" (υ).
    /// 4. Execute OP (semantics in the lesson, §3–§4; a summary):
    ///    - TRAP: set halted, return Ok(false). It still cost one υ.
    ///    - operate/load/store: Z operand = register $Z for even OP,
    ///      the byte Z itself for odd OP (the MMIX immediate rule).
    ///    - ADD/ADDU/SUB/SUBU/MUL: wrapping 64-bit arithmetic.
    ///    - MULU: full 128-bit product; low half to $X, high half to rH.
    ///    - DIV: signed FLOOR division; quotient to $X, remainder (with
    ///      the DIVISOR's sign) to rR; divide-by-zero: $X = 0, rR = $Y.
    ///    - DIVU: plain unsigned division (no rD); by zero as DIV.
    ///    - CMP/CMPU: $X = −1 (all ones), 0, or 1.
    ///    - NEG/NEGU: $X = Y − Z where Y is ALWAYS an immediate byte.
    ///    - SL/SLU: left shift, counts ≥ 64 give 0. SR: arithmetic right
    ///      shift, counts ≥ 64 give the sign fill (0 or all ones).
    ///      SRU: logical right shift, counts ≥ 64 give 0.
    ///    - loads/stores: address = ($Y + Zoperand) mod 2^64, low bits
    ///      ignored per size; signed loads sign-extend, unsigned loads
    ///      zero-extend; stores write the low bytes of $X. Each executed
    ///      load/store counts one "mem" (μ).
    ///    - SETL: $X = YZ (16-bit immediate).
    ///    - BN/BZ/BNN/BNZ on $X as a SIGNED octabyte; a taken branch sets
    ///      pc = branch_address + 4·YZ (even OP, forward) or
    ///      pc = branch_address + 4·(YZ − 65536) (odd OP, backward).
    ///    - JMP: pc = address + 4·XYZ (0xF0) or + 4·(XYZ − 2^24) (0xF1).
    ///    - GET: $X = rH if Z = SPEC_RH, rR if Z = SPEC_RR; any other Z
    ///      is a Fault::IllegalOpcode.
    ///    - anything else: Err(Fault::IllegalOpcode { opcode, at }).
    /// 5. Return Ok(true).
    /// ```
    pub fn step(&mut self) -> Result<bool, Fault> {
        todo!("fetch, decode, execute one instruction")
    }

    /// Run for at most `max_steps` instructions; return how many were
    /// actually executed (the halting TRAP counts as one). Callers check
    /// [`halted`](Mmix::halted) to tell "program finished" from "budget
    /// exhausted" — this guard makes finiteness a testable property.
    pub fn run(&mut self, max_steps: u64) -> Result<u64, Fault> {
        let _ = max_steps;
        todo!("step until halt, fault, or the step budget runs out")
    }
}

/// Assemble MMIX-LITE assembly into instruction tetras — a two-pass
/// MMIXAL-in-miniature (lesson §5).
///
/// Line format: `[LABEL] OP X,Y,Z ; comment`. A label is any first token
/// that is not a mnemonic (a label-only line is allowed and names the next
/// instruction). Literals are decimal, `0x…` or `#…` hexadecimal. Registers
/// are `$0`..`$255`. Branch and JMP targets are labels (or literal tetra
/// offsets). Conveniences: `NEG $X,$Z` = `NEG $X,0,$Z`; `TRAP 0` =
/// `TRAP 0,0,0`; `GET $X,rR` / `GET $X,rH` name the special registers.
///
/// ```text
/// Pass 1. Strip comments; for each nonblank line, split off the optional
///         label and record  label -> instruction index  in a symbol table
///         (error on duplicates). Collect (mnemonic, operands) per line.
/// Pass 2. Encode each instruction to  OP<<24 | X<<16 | Y<<8 | Z :
///         - "$X,$Y,$Z"-shaped ops: even opcode; "$X,$Y,imm" (imm in
///           0..=255): odd opcode — the MMIX immediate rule.
///         - branches: offset d = target index − this index (in tetras);
///           0 <= d < 65536 uses the even opcode with YZ = d,
///           −65536 <= d < 0 the odd opcode with YZ = d + 65536;
///           JMP likewise with the 24-bit XYZ field.
///         - SETL: YZ = the 16-bit immediate.
///         Report errors as strings mentioning the (1-based) line number.
/// ```
///
/// Everything is pc-relative, so the output can be loaded at any address.
pub fn assemble(src: &str) -> Result<Vec<u32>, String> {
    let _ = src;
    todo!("two-pass assembler: symbol table, then encode")
}
