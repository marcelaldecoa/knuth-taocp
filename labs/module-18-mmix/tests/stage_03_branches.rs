//! Stage 3 — Comparisons, branches, and loops (MMIX control flow).
//!
//! Implement BZ/BNZ/BN/BNN (relative, in tetras, forward and backward),
//! JMP, TRAP-as-halt, SETL, `run(max_steps)` — and the two-pass assembler
//! `assemble`. Lesson: §4–§5.

use lab_18_mmix::{assemble, op, Fault, Mmix};

fn enc(opc: u8, x: u8, y: u8, z: u8) -> u32 {
    ((opc as u32) << 24) | ((x as u32) << 16) | ((y as u32) << 8) | z as u32
}

const HALT: u32 = 0; // TRAP 0,0,0

#[test]
fn trap_halts_the_machine() {
    let mut m = Mmix::new();
    m.load_program(0x100, &[HALT]);
    assert_eq!(m.step(), Ok(false), "TRAP reports 'not running'");
    assert!(m.halted());
    assert_eq!(m.oops(), 1, "the TRAP itself costs one oop");
    assert_eq!(m.step(), Ok(false), "stepping a halted machine is a no-op");
    assert_eq!(m.oops(), 1);
}

#[test]
fn bz_forward_taken_and_not_taken() {
    // BZ $1,+2 skips one instruction when $1 = 0.
    let prog = [
        enc(op::BZ, 1, 0, 2),    // if $1 == 0 goto +2 tetras
        enc(op::ADDI, 2, 2, 1),  // $2 += 1   (skipped when taken)
        HALT,
    ];
    let mut taken = Mmix::new();
    taken.set_reg(1, 0);
    taken.load_program(0, &prog);
    taken.run(10).unwrap();
    assert!(taken.halted());
    assert_eq!(taken.reg(2), 0, "ADD was skipped");
    assert_eq!(taken.oops(), 2, "BZ + TRAP");

    let mut not_taken = Mmix::new();
    not_taken.set_reg(1, 7);
    not_taken.load_program(0, &prog);
    not_taken.run(10).unwrap();
    assert!(not_taken.halted());
    assert_eq!(not_taken.reg(2), 1, "ADD executed");
    assert_eq!(not_taken.oops(), 3);
}

#[test]
fn branch_conditions_read_x_as_signed() {
    // BN: negative; BNN: not negative; BNZ: nonzero; BZ: zero.
    for &(cond_op, x_val, expect_taken) in &[
        (op::BN, (-5i64) as u64, true),
        (op::BN, 0, false),
        (op::BN, 5, false),
        (op::BNN, (-5i64) as u64, false),
        (op::BNN, 0, true),
        (op::BNN, 5, true),
        (op::BNZ, 0, false),
        (op::BNZ, (-1i64) as u64, true),
        (op::BZ, 0, true),
        (op::BZ, u64::MAX, false),
    ] {
        let mut m = Mmix::new();
        m.set_reg(1, x_val);
        m.load_program(0, &[enc(cond_op, 1, 0, 2), enc(op::ADDI, 2, 2, 1), HALT]);
        m.run(10).unwrap();
        assert!(m.halted());
        let taken = m.reg(2) == 0;
        assert_eq!(
            taken, expect_taken,
            "opcode {cond_op:#x} with $1 = {x_val:#x}"
        );
    }
}

#[test]
fn backward_branch_makes_a_loop_sum_1_to_10() {
    // The counted loop, hand-encoded: sum 10 + 9 + ... + 1 = 55.
    // Backward BNZ uses the ODD opcode with YZ = 65536 - 2 (two tetras
    // back, relative to the branch instruction itself).
    let words = [
        enc(op::SETL, 1, 0, 0),      // $1 <- 0   (sum)
        enc(op::SETL, 2, 0, 10),     // $2 <- 10  (i)
        enc(op::ADD, 1, 1, 2),       // LOOP: $1 += $2
        enc(op::SUBI, 2, 2, 1),      // $2 -= 1
        enc(op::BNZB, 2, 0xFF, 0xFE),// if $2 != 0 goto LOOP
        HALT,
    ];
    let mut m = Mmix::new();
    m.load_program(0x400, &words);
    let steps = m.run(1_000).unwrap();
    assert!(m.halted());
    assert_eq!(m.reg(1), 55);
    // 2 setup + 10 iterations x 3 + the TRAP = 33 instructions.
    assert_eq!(steps, 33);
    assert_eq!(m.oops(), 33);
}

#[test]
fn jmp_is_relative_in_tetras_both_directions() {
    // Forward JMP skips; backward JMPB (odd opcode) returns.
    let words = [
        enc(op::SETL, 1, 0, 1),                     // 0: $1 <- 1
        (op::JMP as u32) << 24 | 3,                 // 1: goto 4
        enc(op::SETL, 1, 0, 99),                    // 2: (dead code)
        HALT,                                       // 3: end
        enc(op::SETL, 2, 0, 2),                     // 4: $2 <- 2
        (op::JMPB as u32) << 24 | ((1 << 24) - 2),  // 5: goto 3 (back 2)
    ];
    let mut m = Mmix::new();
    m.load_program(0, &words);
    m.run(10).unwrap();
    assert!(m.halted());
    assert_eq!(m.reg(1), 1, "dead code not executed");
    assert_eq!(m.reg(2), 2, "landing pad executed");
    assert_eq!(m.oops(), 5);
}

#[test]
fn run_guards_against_infinite_loops() {
    // JMP-to-self: `run` must stop at the budget and report the count,
    // with the machine NOT halted — finiteness as an observable.
    let mut m = Mmix::new();
    m.load_program(0x800, &[(op::JMP as u32) << 24]); // JMP +0 = self
    assert_eq!(m.run(100), Ok(100));
    assert!(!m.halted());
    assert_eq!(m.pc(), 0x800, "still spinning at the jump");
    assert_eq!(m.oops(), 100);
    // A second budget keeps going from where we stopped.
    assert_eq!(m.run(50), Ok(50));
    assert_eq!(m.oops(), 150);
}

#[test]
fn illegal_opcode_faults_with_location() {
    // 0xFF is TRIP in full MMIX — outside the MMIX-LITE subset.
    let mut m = Mmix::new();
    m.load_program(0x100, &[0xFF00_0000]);
    assert_eq!(
        m.step(),
        Err(Fault::IllegalOpcode { opcode: 0xFF, at: 0x100 })
    );
    // And run() propagates the fault.
    let mut m2 = Mmix::new();
    m2.load_program(0x200, &[enc(op::SETL, 1, 0, 5), 0xF800_0000]); // 0xF8 = POP
    assert!(matches!(
        m2.run(10),
        Err(Fault::IllegalOpcode { opcode: 0xF8, at: 0x204 })
    ));
}

#[test]
fn assembler_emits_the_documented_encodings() {
    // Pin the OP X Y Z byte layout once and for all (hand-assembly, §5):
    assert_eq!(assemble("ADD $1,$2,$3").unwrap(), vec![0x2001_0203]);
    assert_eq!(assemble("ADD $1,$2,3").unwrap(), vec![0x2101_0203], "immediate = odd opcode");
    assert_eq!(assemble("SETL $7,0x1234").unwrap(), vec![0xE307_1234]);
    assert_eq!(assemble("LDO $1,$2,$3").unwrap(), vec![0x8C01_0203]);
    assert_eq!(assemble("STO $1,$2,0").unwrap(), vec![0xAD01_0200]);
    assert_eq!(assemble("TRAP 0,0,0").unwrap(), vec![0x0000_0000]);
    assert_eq!(assemble("DIV $3,$1,$2").unwrap(), vec![0x1C03_0102]);
    assert_eq!(assemble("GET $3,rR").unwrap(), vec![0xFE03_0006]);
    assert_eq!(assemble("CMP $7,$6,$2").unwrap(), vec![0x3007_0602]);
    // Hex via # (MMIX style) and comments:
    assert_eq!(assemble("SETL $1,#FF ; a comment").unwrap(), vec![0xE301_00FF]);
}

#[test]
fn assembler_resolves_labels_forward_and_backward() {
    let src = "
        SETL $1,0        ; sum <- 0
        SETL $2,10       ; i <- 10
LOOP    ADD  $1,$1,$2    ; sum += i
        SUB  $2,$2,1     ; i -= 1
        BNZ  $2,LOOP     ; backward branch
        BZ   $2,END      ; forward branch
        SETL $1,999      ; dead code
END     TRAP 0,0,0
";
    let words = assemble(src).unwrap();
    assert_eq!(words.len(), 8);
    // The backward BNZ must be the odd opcode with YZ = 65536 - 2:
    assert_eq!(words[4], enc(op::BNZB, 2, 0xFF, 0xFE));
    // The forward BZ must be the even opcode with YZ = 2:
    assert_eq!(words[5], enc(op::BZ, 2, 0, 2));
    let mut m = Mmix::new();
    m.load_program(0, &words);
    m.run(1_000).unwrap();
    assert!(m.halted());
    assert_eq!(m.reg(1), 55, "sum 1..10 via the assembler");
}

#[test]
fn assembled_and_hand_encoded_words_agree() {
    // The same loop as `backward_branch_makes_a_loop_sum_1_to_10`,
    // written in assembly: the assembler must reproduce those exact words.
    let src = "
        SETL $1,0
        SETL $2,10
LOOP    ADD  $1,$1,$2
        SUB  $2,$2,1
        BNZ  $2,LOOP
        TRAP 0,0,0
";
    let words = assemble(src).unwrap();
    let hand = vec![
        enc(op::SETL, 1, 0, 0),
        enc(op::SETL, 2, 0, 10),
        enc(op::ADD, 1, 1, 2),
        enc(op::SUBI, 2, 2, 1),
        enc(op::BNZB, 2, 0xFF, 0xFE),
        HALT,
    ];
    assert_eq!(words, hand, "assemble(src) == hand assembly");
}

#[test]
fn assembler_rejects_nonsense_with_line_numbers() {
    assert!(assemble("FROB $1,$2,$3").is_err(), "unknown mnemonic");
    let e = assemble("ADD $1,$2,$3\nWIBBLE $1,$2,$3").unwrap_err();
    assert!(e.contains("2"), "error should name line 2, got: {e}");
    assert!(assemble("BZ $1,NOWHERE").is_err(), "unknown label");
    assert!(assemble("ADD $1,$2,300").is_err(), "immediate byte > 255");
    assert!(assemble("ADD $256,$0,$0").is_err(), "no register $256");
    assert!(assemble("SETL $1,65536").is_err(), "SETL takes a wyde");
    assert!(
        assemble("X ADD $1,$1,1\nX SUB $1,$1,1").is_err(),
        "duplicate label"
    );
}

#[test]
fn assembler_output_is_position_independent() {
    // All control flow is relative: the same words run at any address.
    let words = assemble(
        "
LOOP    ADD  $1,$1,1
        CMP  $2,$1,5
        BNZ  $2,LOOP
        TRAP 0,0,0
",
    )
    .unwrap();
    for &base in &[0u64, 0x100, 0xF000] {
        let mut m = Mmix::new();
        m.load_program(base, &words);
        m.run(100).unwrap();
        assert!(m.halted(), "halted when loaded at {base:#x}");
        assert_eq!(m.reg(1), 5, "counts to 5 when loaded at {base:#x}");
    }
}
