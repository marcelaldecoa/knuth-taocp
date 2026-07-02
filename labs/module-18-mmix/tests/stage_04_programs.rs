//! Stage 4 — Programs: Euclid and FindMax on the metal (Fascicle 1
//! programs). The course's full-circle moment: Algorithm 1.1E (module 01)
//! and Algorithm 1.2.10M (module 02), as MMIX-LITE assembly, running on the
//! machine YOU built, measured with Knuth's own cost model.
//!
//! Nothing new to implement if stages 1–3 are honest; these tests exercise
//! the whole pipeline: assembler -> loader -> machine -> counters.

use lab_18_mmix::{assemble, Mmix};

/// Algorithm 1.1E (Euclid), straight from module 01, as MMIX-LITE assembly.
/// Input: $0 = m, $1 = n (positive). Output: the gcd in $1.
/// Six instructions of loop body — compare the lesson's side-by-side trace.
const EUCLID: &str = "
E1      DIV  $2,$0,$1     ; E1. [Find remainder.] rR <- m mod n
        GET  $3,rR        ;     r into a general register
E2      BZ   $3,DONE      ; E2. [Is it zero?] if r = 0, n is the answer
E3      ADD  $0,$1,0      ; E3. [Reduce.] m <- n
        ADD  $1,$3,0      ;     n <- r
        JMP  E1           ;     back to E1
DONE    TRAP 0,0,0        ; halt: gcd is in $1
";

/// Algorithm 1.2.10M (FindMax), from module 02, as MMIX-LITE assembly.
/// Input: $0 = address of X[1] (octas X[1..n] contiguous), $1 = n >= 1.
/// Output: $2 = m = max, $3 = j = largest index attaining it (1-based).
/// Scans k = n-1 down to 1 with the strict test X[k] > m, so ties keep the
/// LATER element — exactly Knuth's M1-M4.
const FINDMAX: &str = "
M1      SUB  $4,$1,1      ; M1. [Initialize.] k <- n-1
        ADD  $3,$1,0      ;     j <- n
        SL   $5,$4,3      ;     byte offset of X[n] = 8(n-1)
        LDO  $2,$0,$5     ;     m <- X[n]
M2      BZ   $4,DONE      ; M2. [All tested?] if k = 0, done
        SUB  $5,$4,1      ;     byte offset of X[k] = 8(k-1)
        SL   $5,$5,3
        LDO  $6,$0,$5     ;     X[k]
M3      CMP  $7,$6,$2     ; M3. [Compare.] X[k] <=> m
        BN   $7,M5        ;     X[k] < m: skip
        BZ   $7,M5        ;     X[k] = m: skip (ties keep later j)
M4      ADD  $3,$4,0      ; M4. [Change m.] j <- k
        ADD  $2,$6,0      ;     m <- X[k]
M5      SUB  $4,$4,1      ; M5. [Decrease k.] k <- k-1
        JMP  M2
DONE    TRAP 0,0,0
";

/// Assemble + load + run EUCLID on (m, n); returns the machine afterwards.
fn run_euclid(m: u64, n: u64) -> Mmix {
    let words = assemble(EUCLID).expect("EUCLID must assemble");
    let mut mx = Mmix::new();
    mx.set_reg(0, m);
    mx.set_reg(1, n);
    mx.load_program(0x100, &words);
    mx.run(100_000).expect("no faults");
    assert!(mx.halted(), "EUCLID must terminate on ({m}, {n})");
    mx
}

/// Write xs as octas at 0x2000, then assemble + load + run FINDMAX.
fn run_findmax(xs: &[u64]) -> Mmix {
    let words = assemble(FINDMAX).expect("FINDMAX must assemble");
    let mut mx = Mmix::new();
    for (i, &v) in xs.iter().enumerate() {
        mx.st_octa(0x2000 + 8 * i as u64, v);
    }
    mx.set_reg(0, 0x2000);
    mx.set_reg(1, xs.len() as u64);
    mx.load_program(0x100, &words);
    mx.run(100_000).expect("no faults");
    assert!(mx.halted(), "FINDMAX must terminate");
    mx
}

#[test]
fn euclid_gcd_544_119_is_17() {
    // Module 01's very first worked example (TAOCP §1.1): gcd(544, 119)
    // runs remainders 68, 51, 17, 0 — the answer is 17.
    assert_eq!(run_euclid(544, 119).reg(1), 17);
    // And with the operands swapped, E3's swap handles it (one extra pass).
    assert_eq!(run_euclid(119, 544).reg(1), 17);
}

#[test]
fn euclid_gcd_2166_6099_is_57() {
    // Exercise 1.1-1's numbers, closing the loop with module 01.
    assert_eq!(run_euclid(2166, 6099).reg(1), 57);
    assert_eq!(run_euclid(6099, 2166).reg(1), 57);
    assert_eq!(run_euclid(7, 7).reg(1), 7);
    assert_eq!(run_euclid(1, 999_999_937).reg(1), 1);
}

#[test]
fn euclid_cost_in_knuths_model() {
    // T(544, 119) = 4 divisions (module 01, stage 4). Our listing costs
    // 6 oops per non-final pass (DIV GET BZ ADD ADD JMP), 3 for the final
    // pass (DIV GET BZ-taken), plus the TRAP:
    //     oops = 6(T-1) + 3 + 1 = 6T - 2 = 22.
    // Euclid never touches memory, so mems = 0. THIS is Knuth's point:
    // the cost of an algorithm is a theorem about a machine, not a
    // stopwatch reading.
    let m = run_euclid(544, 119);
    assert_eq!(m.oops(), 22, "6*T - 2 with T = 4 divisions");
    assert_eq!(m.mems(), 0, "Euclid is register-only");
    // T(2166, 6099) = 6 (the first pass just swaps): 6*6 - 2 = 34.
    let m = run_euclid(2166, 6099);
    assert_eq!(m.oops(), 34);
}

#[test]
fn euclid_agrees_with_host_gcd_on_random_pairs() {
    // The pipeline vs. a plain Rust gcd, on LCG-generated pairs.
    fn gcd(mut m: u64, mut n: u64) -> u64 {
        while n != 0 {
            let r = m % n;
            m = n;
            n = r;
        }
        m
    }
    let mut seed: u64 = 42;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed
    };
    for _ in 0..40 {
        let m = next() % 1_000_000 + 1;
        let n = next() % 1_000_000 + 1;
        assert_eq!(run_euclid(m, n).reg(1), gcd(m, n), "gcd({m}, {n})");
    }
}

#[test]
fn findmax_on_module_02_style_data() {
    // Knuth's sixteen keys (they anchor the sorting chapters): the maximum
    // is 908, at (1-based) position 5.
    let xs = [
        503, 87, 512, 61, 908, 170, 897, 275, 653, 426, 154, 509, 612, 677, 765, 703,
    ];
    let m = run_findmax(&xs);
    assert_eq!(m.reg(2), 908, "the maximum");
    assert_eq!(m.reg(3), 5, "its position");
    assert_eq!(m.mems(), 16, "exactly n = 16 loads: X[n] once, then X[k] for k = n-1..1");
}

#[test]
fn findmax_duplicates_report_the_last_maximum() {
    // Algorithm M's strict test X[k] > m while scanning k downward keeps
    // the LARGEST index among equal maxima.
    let m = run_findmax(&[5, 9, 3, 9, 2, 9, 1]);
    assert_eq!(m.reg(2), 9);
    assert_eq!(m.reg(3), 6, "the last 9 is at position 6");
    assert_eq!(m.mems(), 7, "n = 7 loads");
    // Max at the very end:
    let m = run_findmax(&[1, 2, 3, 99]);
    assert_eq!(m.reg(2), 99);
    assert_eq!(m.reg(3), 4);
    // All equal: position n.
    let m = run_findmax(&[8, 8, 8]);
    assert_eq!(m.reg(2), 8);
    assert_eq!(m.reg(3), 3);
}

#[test]
fn findmax_single_element() {
    let m = run_findmax(&[42]);
    assert_eq!(m.reg(2), 42);
    assert_eq!(m.reg(3), 1);
    // M1 (4 instructions), M2's BZ taken, TRAP: 6 oops, 1 mem.
    assert_eq!(m.oops(), 6);
    assert_eq!(m.mems(), 1);
}

#[test]
fn findmax_agrees_with_host_max_on_random_arrays() {
    let mut seed: u64 = 20260702;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed
    };
    for len in 1..=25usize {
        let xs: Vec<u64> = (0..len).map(|_| next() % 1000).collect();
        let m = run_findmax(&xs);
        let best = *xs.iter().max().unwrap();
        let best_j = xs.iter().rposition(|&v| v == best).unwrap() + 1;
        assert_eq!(m.reg(2), best, "max of {xs:?}");
        assert_eq!(m.reg(3), best_j as u64, "position in {xs:?}");
        assert_eq!(m.mems(), len as u64, "n mems for n = {len}");
    }
}
