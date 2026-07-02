//! Stage 1 — Machine state, memory, loads and stores (MMIX basics).
//!
//! Implement `Mmix::new`, the register file, the big-endian memory
//! accessors, `load_program`, and the LDx/STx instructions in `step`.
//! Lesson: course/module-18-mmix/README.md, §2–§3.

use lab_18_mmix::{op, Mmix};

/// OP<<24 | X<<16 | Y<<8 | Z — the MMIX instruction layout.
fn enc(opc: u8, x: u8, y: u8, z: u8) -> u32 {
    ((opc as u32) << 24) | ((x as u32) << 16) | ((y as u32) << 8) | z as u32
}

/// Load `words` at 0x1000 and execute exactly `words.len()` instructions.
fn exec(m: &mut Mmix, words: &[u32]) {
    m.load_program(0x1000, words);
    for _ in 0..words.len() {
        assert_eq!(m.step(), Ok(true), "program should keep running");
    }
}

#[test]
fn fresh_machine_is_all_zero() {
    let m = Mmix::new();
    for x in 0..=255u8 {
        assert_eq!(m.reg(x), 0, "register ${x} must start at 0");
    }
    assert_eq!(m.pc(), 0);
    assert!(!m.halted());
    // Unwritten memory reads as zero, at whatever address.
    assert_eq!(m.ld_byte(0), 0);
    assert_eq!(m.ld_octa(0xDEAD_BEE8), 0);
    assert_eq!(m.mems(), 0);
    assert_eq!(m.oops(), 0);
}

#[test]
fn registers_hold_octabytes() {
    let mut m = Mmix::new();
    m.set_reg(0, u64::MAX);
    m.set_reg(7, 0x0123_4567_89AB_CDEF);
    m.set_reg(255, 42);
    assert_eq!(m.reg(0), u64::MAX);
    assert_eq!(m.reg(7), 0x0123_4567_89AB_CDEF);
    assert_eq!(m.reg(255), 42);
    assert_eq!(m.reg(8), 0, "writing $7 must not touch $8");
}

#[test]
fn memory_is_big_endian() {
    // MMIX stores the MOST significant byte at the smallest address.
    let mut m = Mmix::new();
    m.st_octa(0x100, 0x0123_4567_89AB_CDEF);
    let expect: [u8; 8] = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
    for (i, &b) in expect.iter().enumerate() {
        assert_eq!(m.ld_byte(0x100 + i as u64), b, "byte at 0x100+{i}");
    }
    assert_eq!(m.ld_wyde(0x100), 0x0123);
    assert_eq!(m.ld_wyde(0x106), 0xCDEF);
    assert_eq!(m.ld_tetra(0x100), 0x0123_4567);
    assert_eq!(m.ld_tetra(0x104), 0x89AB_CDEF);
    assert_eq!(m.ld_octa(0x100), 0x0123_4567_89AB_CDEF);
    // And the small stores agree with the big picture:
    let mut m2 = Mmix::new();
    m2.st_wyde(0x200, 0xBEEF);
    assert_eq!(m2.ld_byte(0x200), 0xBE);
    assert_eq!(m2.ld_byte(0x201), 0xEF);
    m2.st_tetra(0x204, 0x1122_3344);
    assert_eq!(m2.ld_byte(0x204), 0x11);
    assert_eq!(m2.ld_byte(0x207), 0x44);
}

#[test]
fn accessor_addresses_round_down_to_alignment() {
    // MMIX ignores the low bits of an address: wyde & !1, tetra & !3,
    // octa & !7 (Fascicle 1, §1.3.1´).
    let mut m = Mmix::new();
    m.st_octa(0x100, 0x0123_4567_89AB_CDEF);
    assert_eq!(m.ld_octa(0x105), 0x0123_4567_89AB_CDEF, "octa at 0x105 reads 0x100");
    assert_eq!(m.ld_octa(0x107), m.ld_octa(0x100));
    assert_eq!(m.ld_tetra(0x106), 0x89AB_CDEF, "tetra at 0x106 reads 0x104");
    assert_eq!(m.ld_wyde(0x103), 0x4567, "wyde at 0x103 reads 0x102");
    // Stores round down too: an octa store at 0x109 lands at 0x108.
    m.st_octa(0x109, 0xAAAA_BBBB_CCCC_DDDD);
    assert_eq!(m.ld_octa(0x108), 0xAAAA_BBBB_CCCC_DDDD);
    assert_eq!(m.ld_octa(0x100), 0x0123_4567_89AB_CDEF, "0x100 untouched");
}

#[test]
fn load_program_places_tetras_and_sets_pc() {
    let mut m = Mmix::new();
    m.load_program(0x2000, &[0x2001_0203, 0xE307_1234]);
    assert_eq!(m.pc(), 0x2000);
    assert_eq!(m.ld_tetra(0x2000), 0x2001_0203);
    assert_eq!(m.ld_tetra(0x2004), 0xE307_1234);
    assert_eq!(m.ld_byte(0x2000), 0x20, "programs are stored big-endian");
    assert_eq!(m.ld_byte(0x2007), 0x34);
}

#[test]
fn ldb_sign_extends_every_byte() {
    // Store 0x0123456789ABCDEF at 0x100, then LDB each offset. Bytes with
    // the high bit set (0x89, 0xAB, 0xCD, 0xEF) sign-extend to all-ones
    // high bytes; the others zero-extend. (Signed loads, Fascicle 1.)
    let mut m = Mmix::new();
    m.st_octa(0x100, 0x0123_4567_89AB_CDEF);
    m.set_reg(1, 0x100);
    let words: Vec<u32> = (0..8).map(|k| enc(op::LDBI, 2 + k, 1, k)).collect();
    exec(&mut m, &words);
    let expect: [u64; 8] = [
        0x01,
        0x23,
        0x45,
        0x67,
        0xFFFF_FFFF_FFFF_FF89,
        0xFFFF_FFFF_FFFF_FFAB,
        0xFFFF_FFFF_FFFF_FFCD,
        0xFFFF_FFFF_FFFF_FFEF,
    ];
    for (k, &e) in expect.iter().enumerate() {
        assert_eq!(m.reg(2 + k as u8), e, "LDB of byte at offset {k}");
    }
}

#[test]
fn ldbu_zero_extends_every_byte() {
    let mut m = Mmix::new();
    m.st_octa(0x100, 0x0123_4567_89AB_CDEF);
    m.set_reg(1, 0x100);
    let words: Vec<u32> = (0..8).map(|k| enc(op::LDBUI, 2 + k, 1, k)).collect();
    exec(&mut m, &words);
    let expect: [u64; 8] = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
    for (k, &e) in expect.iter().enumerate() {
        assert_eq!(m.reg(2 + k as u8), e, "LDBU of byte at offset {k}");
    }
}

#[test]
fn wyde_tetra_octa_loads_signed_and_unsigned() {
    let mut m = Mmix::new();
    m.st_octa(0x100, 0x0123_4567_89AB_CDEF);
    m.set_reg(1, 0x100);
    exec(
        &mut m,
        &[
            enc(op::LDWI, 2, 1, 0),  // wyde 0x0123, positive
            enc(op::LDWI, 3, 1, 4),  // wyde 0x89AB, negative
            enc(op::LDWUI, 4, 1, 4), // zero-extended
            enc(op::LDTI, 5, 1, 4),  // tetra 0x89ABCDEF, negative
            enc(op::LDTUI, 6, 1, 4), // zero-extended
            enc(op::LDOI, 7, 1, 0),  // the whole octa
            enc(op::LDTI, 8, 1, 0),  // tetra 0x01234567, positive
        ],
    );
    assert_eq!(m.reg(2), 0x0123);
    assert_eq!(m.reg(3), 0xFFFF_FFFF_FFFF_89AB);
    assert_eq!(m.reg(4), 0x89AB);
    assert_eq!(m.reg(5), 0xFFFF_FFFF_89AB_CDEF);
    assert_eq!(m.reg(6), 0x89AB_CDEF);
    assert_eq!(m.reg(7), 0x0123_4567_89AB_CDEF);
    assert_eq!(m.reg(8), 0x0123_4567);
}

#[test]
fn load_address_is_y_plus_z_register_form() {
    // Even opcode: A = ($Y + $Z) mod 2^64.
    let mut m = Mmix::new();
    m.st_octa(0x100, 777);
    m.set_reg(1, 0x0F0);
    m.set_reg(2, 0x010);
    exec(&mut m, &[enc(op::LDO, 3, 1, 2)]);
    assert_eq!(m.reg(3), 777);
    // Odd opcode: A = ($Y + Z-as-byte).
    let mut m2 = Mmix::new();
    m2.st_octa(0x100, 888);
    m2.set_reg(1, 0x0F8);
    exec(&mut m2, &[enc(op::LDOI, 3, 1, 8)]);
    assert_eq!(m2.reg(3), 888);
}

#[test]
fn unaligned_instruction_loads_round_down() {
    // LDO with address 0x105 must read the octa at 0x100 — MMIX ignores
    // the low three bits rather than faulting (Fascicle 1).
    let mut m = Mmix::new();
    m.st_octa(0x100, 0x0123_4567_89AB_CDEF);
    m.set_reg(1, 0x105);
    exec(&mut m, &[enc(op::LDOI, 2, 1, 0), enc(op::LDWI, 3, 1, 0)]);
    assert_eq!(m.reg(2), 0x0123_4567_89AB_CDEF, "LDO of 0x105 reads 0x100");
    // wyde at 0x105 & !1 = 0x104 → 0x89AB, sign-extended.
    assert_eq!(m.reg(3), 0xFFFF_FFFF_FFFF_89AB, "LDW of 0x105 reads 0x104");
}

#[test]
fn stores_write_the_low_bytes_of_x() {
    let mut m = Mmix::new();
    m.set_reg(1, 0x100); // base
    m.set_reg(2, 0x1122_3344_5566_7788);
    exec(
        &mut m,
        &[
            enc(op::STOI, 2, 1, 0),  // whole octa at 0x100
            enc(op::STBI, 2, 1, 8),  // low byte 0x88 at 0x108
            enc(op::STWI, 2, 1, 16), // low wyde 0x7788 at 0x110
            enc(op::STTI, 2, 1, 24), // low tetra 0x55667788 at 0x118
        ],
    );
    assert_eq!(m.ld_octa(0x100), 0x1122_3344_5566_7788);
    assert_eq!(m.ld_byte(0x108), 0x88);
    assert_eq!(m.ld_wyde(0x110), 0x7788);
    assert_eq!(m.ld_tetra(0x118), 0x5566_7788);
    // Neighbours stay zero:
    assert_eq!(m.ld_byte(0x109), 0);
    assert_eq!(m.ld_octa(0x120), 0);
}

#[test]
fn store_octa_then_load_it_back_through_the_machine() {
    // A store/load round trip executed entirely by instructions.
    let mut m = Mmix::new();
    m.set_reg(1, 0x300);
    m.set_reg(2, 0xCAFE_F00D_DEAD_BEEF);
    exec(&mut m, &[enc(op::STOI, 2, 1, 0), enc(op::LDOI, 3, 1, 0)]);
    assert_eq!(m.reg(3), 0xCAFE_F00D_DEAD_BEEF);
    assert_eq!(m.mems(), 2, "one store + one load = two mems");
    assert_eq!(m.oops(), 2);
}

#[test]
fn pc_advances_by_four_per_instruction() {
    let mut m = Mmix::new();
    m.set_reg(1, 0x100);
    m.load_program(0x1000, &[enc(op::LDOI, 2, 1, 0), enc(op::LDOI, 3, 1, 0)]);
    assert_eq!(m.pc(), 0x1000);
    m.step().unwrap();
    assert_eq!(m.pc(), 0x1004);
    m.step().unwrap();
    assert_eq!(m.pc(), 0x1008);
}
