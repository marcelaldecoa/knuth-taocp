//! Stage 2 — Arithmetic: signed, unsigned, overflow, DIV (MMIX operations).
//!
//! Implement ADD/ADDU/SUB/SUBU/MUL/MULU/DIV/DIVU (+ immediate forms),
//! NEG/NEGU, CMP/CMPU, SL/SLU/SR/SRU, and GET (rR/rH). The heart of the
//! stage is MMIX's FLOOR division. Lesson: §3–§4.

use lab_18_mmix::{op, Mmix};

fn enc(opc: u8, x: u8, y: u8, z: u8) -> u32 {
    ((opc as u32) << 24) | ((x as u32) << 16) | ((y as u32) << 8) | z as u32
}

/// Run one instruction with $1 = y, $2 = z; the result register is $3.
fn unop(word: u32, y: u64, z: u64) -> Mmix {
    let mut m = Mmix::new();
    m.set_reg(1, y);
    m.set_reg(2, z);
    m.load_program(0, &[word]);
    assert_eq!(m.step(), Ok(true));
    m
}

#[test]
fn add_sub_register_and_immediate_forms() {
    assert_eq!(unop(enc(op::ADD, 3, 1, 2), 40, 2).reg(3), 42);
    assert_eq!(unop(enc(op::ADDI, 3, 1, 200), 42, 999).reg(3), 242, "odd opcode: Z is the byte 200");
    assert_eq!(unop(enc(op::SUB, 3, 1, 2), 40, 2).reg(3), 38);
    assert_eq!(unop(enc(op::SUBI, 3, 1, 1), 0, 0).reg(3), u64::MAX, "0 - 1 wraps to all ones (-1)");
    assert_eq!(unop(enc(op::ADDU, 3, 1, 2), u64::MAX, 1).reg(3), 0, "unsigned wraparound");
    assert_eq!(unop(enc(op::SUBU, 3, 1, 2), 5, 7).reg(3), (-2i64) as u64);
    // MMIX-LITE never trips: signed ADD wraps exactly like ADDU.
    assert_eq!(unop(enc(op::ADD, 3, 1, 2), i64::MAX as u64, 1).reg(3), i64::MIN as u64);
}

#[test]
fn neg_is_y_minus_z_with_immediate_y() {
    // NEG $X,Y,$Z computes Y − $Z where Y is an unsigned immediate byte —
    // MMIX has no dedicated unary minus.
    assert_eq!(unop(enc(op::NEG, 3, 0, 2), 0, 7).reg(3) as i64, -7);
    assert_eq!(unop(enc(op::NEG, 3, 5, 2), 0, 7).reg(3) as i64, -2, "5 - 7");
    assert_eq!(unop(enc(op::NEGI, 3, 0, 7), 0, 0).reg(3) as i64, -7, "both operands immediate");
    assert_eq!(unop(enc(op::NEGU, 3, 0, 2), 0, 1).reg(3), u64::MAX);
    assert_eq!(unop(enc(op::NEG, 3, 0, 2), 0, (-9i64) as u64).reg(3) as i64, 9);
}

#[test]
fn mul_wraps_and_mulu_fills_himult() {
    assert_eq!(unop(enc(op::MUL, 3, 1, 2), 6, 7).reg(3), 42);
    assert_eq!(unop(enc(op::MULI, 3, 1, 10), 25, 0).reg(3), 250);
    assert_eq!(
        unop(enc(op::MUL, 3, 1, 2), (-6i64) as u64, 7).reg(3) as i64,
        -42,
        "signed multiply"
    );
    // MULU computes the full 128-bit product: low half to $X, high to rH.
    // (2^64 - 1)^2 = 0xFFFFFFFFFFFFFFFE_0000000000000001.
    let m = unop(enc(op::MULU, 3, 1, 2), u64::MAX, u64::MAX);
    assert_eq!(m.reg(3), 1);
    assert_eq!(m.himult(), 0xFFFF_FFFF_FFFF_FFFE);
    // Small products leave rH = 0.
    let m = unop(enc(op::MULU, 3, 1, 2), 1 << 32, 1 << 31);
    assert_eq!(m.reg(3), 1 << 63);
    assert_eq!(m.himult(), 0);
    // 2^32 * 2^32 = 2^64: low half 0, high half 1.
    let m = unop(enc(op::MULU, 3, 1, 2), 1 << 32, 1 << 32);
    assert_eq!(m.reg(3), 0);
    assert_eq!(m.himult(), 1);
}

#[test]
fn div_is_floor_division_remainder_takes_divisor_sign() {
    // Fascicle 1's definition: $X = floor($Y / $Z), rR = $Y − $Z·$X.
    // Floor rounds toward −infinity, so the remainder has the sign of the
    // DIVISOR — Knuth's mathematically clean `mod` from §1.2.4.
    let table: &[(i64, i64, i64, i64)] = &[
        (7, 2, 3, 1),
        (-7, 2, -4, 1),  // floor(-3.5) = -4, r = -7 - 2(-4) = 1
        (7, -2, -4, -1), // floor(-3.5) = -4, r = 7 - (-2)(-4) = -1
        (-7, -2, 3, -1), // floor(3.5) = 3, r = -7 - (-2)(3) = -1
        (8, 2, 4, 0),
        (-8, 2, -4, 0),
        (1, 3, 0, 1),
        (-1, 3, -1, 2), // floor(-1/3) = -1, r = -1 - 3(-1) = 2
    ];
    for &(y, z, q, r) in table {
        let m = unop(enc(op::DIV, 3, 1, 2), y as u64, z as u64);
        assert_eq!(m.reg(3) as i64, q, "quotient of {y} / {z}");
        assert_eq!(m.remainder() as i64, r, "remainder of {y} mod {z}");
        // The defining identity: y = z*q + r, with r on the divisor's side.
        assert_eq!(z.wrapping_mul(q).wrapping_add(r), y);
        assert!(r == 0 || (r < 0) == (z < 0), "remainder sign for {y}/{z}");
    }
    // Immediate form (Z = byte 2):
    let m = unop(enc(op::DIVI, 3, 1, 2), (-7i64) as u64, 0);
    assert_eq!(m.reg(3) as i64, -4);
    assert_eq!(m.remainder() as i64, 1);
}

#[test]
fn division_by_zero_yields_zero_quotient_and_dividend_remainder() {
    // MMIX's convention: quotient 0, remainder = the dividend. (Full MMIX
    // additionally raises an "integer divide check"; MMIX-LITE does not.)
    let m = unop(enc(op::DIV, 3, 1, 2), 12345, 0);
    assert_eq!(m.reg(3), 0);
    assert_eq!(m.remainder(), 12345);
    let m = unop(enc(op::DIV, 3, 1, 2), (-5i64) as u64, 0);
    assert_eq!(m.reg(3), 0);
    assert_eq!(m.remainder() as i64, -5);
    let m = unop(enc(op::DIVU, 3, 1, 2), u64::MAX, 0);
    assert_eq!(m.reg(3), 0);
    assert_eq!(m.remainder(), u64::MAX);
}

#[test]
fn divu_is_plain_unsigned_division() {
    // No rD in MMIX-LITE: DIVU divides the 64-bit $Y (as if rD = 0).
    let m = unop(enc(op::DIVU, 3, 1, 2), u64::MAX, 10);
    assert_eq!(m.reg(3), u64::MAX / 10);
    assert_eq!(m.remainder(), u64::MAX % 10);
    // The bit pattern of -7 is a huge unsigned number, NOT negative:
    let m = unop(enc(op::DIVU, 3, 1, 2), (-7i64) as u64, 2);
    assert_eq!(m.reg(3), ((-7i64) as u64) / 2);
    assert_eq!(m.remainder(), 1);
}

#[test]
fn div_floor_property_sweep() {
    // Deterministic LCG sweep: floor quotient, remainder identity, and
    // remainder bound |r| < |z| with r on the divisor's side.
    let mut seed: u64 = 20260702;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed
    };
    for _ in 0..300 {
        let y = next() as i64 % 100_000;
        let mut z = next() as i64 % 1_000;
        if z == 0 {
            z = 13;
        }
        let m = unop(enc(op::DIV, 3, 1, 2), y as u64, z as u64);
        let q = m.reg(3) as i64;
        let r = m.remainder() as i64;
        assert_eq!(y, z * q + r, "identity for {y}/{z}");
        assert!(r.abs() < z.abs(), "|r| < |z| for {y}/{z}");
        assert!(r == 0 || (r < 0) == (z < 0), "sign of r for {y}/{z}");
        // q is the floor of y/z: compute it independently and compare.
        let floor_q = {
            let mut t = y / z; // Rust truncates toward zero...
            if y % z != 0 && (y < 0) != (z < 0) {
                t -= 1; // ...so round down when the signs differ.
            }
            t
        };
        assert_eq!(q, floor_q, "floor quotient for {y}/{z}");
    }
}

#[test]
fn cmp_returns_minus_one_zero_one() {
    // CMP is signed; −1 is stored as all ones.
    assert_eq!(unop(enc(op::CMP, 3, 1, 2), 3, 5).reg(3), u64::MAX);
    assert_eq!(unop(enc(op::CMP, 3, 1, 2), 5, 5).reg(3), 0);
    assert_eq!(unop(enc(op::CMP, 3, 1, 2), 7, 5).reg(3), 1);
    // Signed vs unsigned: −1 < 1 signed, but 0xFFFF… > 1 unsigned.
    assert_eq!(unop(enc(op::CMP, 3, 1, 2), (-1i64) as u64, 1).reg(3), u64::MAX);
    assert_eq!(unop(enc(op::CMPU, 3, 1, 2), (-1i64) as u64, 1).reg(3), 1);
    assert_eq!(unop(enc(op::CMPI, 3, 1, 9), 9, 0).reg(3), 0);
    assert_eq!(unop(enc(op::CMPUI, 3, 1, 255), 1, 0).reg(3), u64::MAX);
}

#[test]
fn shifts_arithmetic_vs_logical() {
    assert_eq!(unop(enc(op::SLI, 3, 1, 4), 0b1011, 0).reg(3), 0b1011_0000);
    assert_eq!(unop(enc(op::SLUI, 3, 1, 63), 1, 0).reg(3), 1 << 63);
    // SR sign-fills (arithmetic), SRU zero-fills (logical):
    assert_eq!(unop(enc(op::SRI, 3, 1, 2), (-16i64) as u64, 0).reg(3) as i64, -4);
    assert_eq!(unop(enc(op::SRUI, 3, 1, 2), (-16i64) as u64, 0).reg(3), ((-16i64) as u64) >> 2);
    assert_eq!(unop(enc(op::SRI, 3, 1, 4), 0xF0, 0).reg(3), 0x0F);
    // Register-form count:
    assert_eq!(unop(enc(op::SL, 3, 1, 2), 3, 2).reg(3), 12);
}

#[test]
fn shift_counts_of_64_or_more_do_not_wrap() {
    // MMIX does NOT reduce shift counts mod 64 (unlike x86!): shifting an
    // octabyte by >= 64 pushes every bit out.
    assert_eq!(unop(enc(op::SLUI, 3, 1, 64), u64::MAX, 0).reg(3), 0);
    assert_eq!(unop(enc(op::SLI, 3, 1, 100), 1, 0).reg(3), 0);
    assert_eq!(unop(enc(op::SRUI, 3, 1, 64), u64::MAX, 0).reg(3), 0);
    // Arithmetic right shift by >= 64 leaves only sign bits:
    assert_eq!(unop(enc(op::SRI, 3, 1, 64), (-1i64) as u64, 0).reg(3), u64::MAX);
    assert_eq!(unop(enc(op::SRI, 3, 1, 200), (-12345i64) as u64, 0).reg(3), u64::MAX);
    assert_eq!(unop(enc(op::SRI, 3, 1, 64), 12345, 0).reg(3), 0);
    // Huge register-form count:
    assert_eq!(unop(enc(op::SLU, 3, 1, 2), 1, u64::MAX).reg(3), 0);
}

#[test]
fn get_reads_rr_and_rh_from_programs() {
    // GET $X,rR / GET $X,rH — how assembly programs reach the stand-in
    // special registers (Euclid needs this in stage 4!).
    let mut m = Mmix::new();
    m.set_reg(1, 544);
    m.set_reg(2, 119);
    m.load_program(
        0,
        &[
            enc(op::DIV, 3, 1, 2),           // 544 = 4*119 + 68
            enc(op::GET, 4, 0, op::SPEC_RR), // $4 <- rR
            enc(op::MULU, 5, 1, 1),          // 544^2, fits: rH = 0
            enc(op::GET, 6, 0, op::SPEC_RH), // $6 <- rH
        ],
    );
    for _ in 0..4 {
        m.step().unwrap();
    }
    assert_eq!(m.reg(3), 4);
    assert_eq!(m.reg(4), 68, "GET rR sees the DIV remainder");
    assert_eq!(m.reg(6), 0, "GET rH sees the MULU high half");
    assert_eq!(m.remainder(), 68);
}

#[test]
fn arithmetic_never_touches_memory() {
    // Knuth's cost model: register arithmetic is pure υ, no μ.
    let m = unop(enc(op::ADD, 3, 1, 2), 1, 2);
    assert_eq!(m.mems(), 0);
    assert_eq!(m.oops(), 1);
}
