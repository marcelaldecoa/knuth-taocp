//! Stage 2 — The chi-square test (§3.3.1).
//!
//! Implement `chi_square` and `chi_square_uniform` in src/lab.rs.
//! Lesson: course/module-04-random/README.md (§3.3.1, the statistic V).

use lab_04_random::{chi_square, chi_square_uniform, Lcg};

const MMIX_A: u64 = 6364136223846793005;
const MMIX_C: u64 = 1442695040888963407;

// rng(bound) uniform in 0..bound, taking the HIGH-order bits of the LCG via
// the multiply-shift map (Lemire): x/2^64 · bound, truncated. This never
// touches the weak low-order bits of a power-of-two-modulus generator.
fn scaled(x: u64, bound: u64) -> u64 {
    ((x as u128 * bound as u128) >> 64) as u64
}

#[test]
fn worked_dice_example() {
    // §3.3.1's style of worked example: 144 throws of two dice, categories
    // s = 2..=12 with expected counts 144·p_s = 4,8,12,16,20,24,20,16,12,8,4.
    // This particular table gives V = 343/48 = 7 7/48.
    let observed = [2u64, 4, 10, 12, 22, 29, 21, 15, 14, 9, 6];
    let expected = [4.0, 8.0, 12.0, 16.0, 20.0, 24.0, 20.0, 16.0, 12.0, 8.0, 4.0];
    let v = chi_square(&observed, &expected);
    assert!((v - 343.0 / 48.0).abs() < 1e-9, "V = {v}, expected 343/48");
}

#[test]
fn uniform_form_matches_general_form() {
    // n = 10 observations, k = 2 categories, expected 5 each:
    // V = (3-5)^2/5 + (7-5)^2/5 = 4/5 + 4/5 = 1.6.
    let counts = [3u64, 7];
    assert!((chi_square_uniform(&counts) - 1.6).abs() < 1e-12);
    assert!((chi_square(&counts, &[5.0, 5.0]) - 1.6).abs() < 1e-12);
}

#[test]
fn perfect_fit_gives_zero() {
    // If every category matches its expectation exactly, V = 0.
    let counts = [25u64, 25, 25, 25];
    assert!(chi_square_uniform(&counts).abs() < 1e-12);
    assert_eq!(chi_square(&[3, 4, 5], &[3.0, 4.0, 5.0]), 0.0);
}

#[test]
fn a_good_generator_gives_a_moderate_v() {
    // Draw many values from Knuth's MMIX generator, bucket their high bits
    // into 10 equally likely categories, and apply the uniformity test.
    // With k = 10 categories there are k - 1 = 9 degrees of freedom; the
    // 99% point of the chi-square table for 9 d.o.f. is ≈ 21.67. A good
    // generator sits comfortably below it (here V ≈ 5).
    let mut g = Lcg::new(12345, MMIX_A, MMIX_C, 0);
    let mut counts = [0u64; 10];
    for _ in 0..100_000 {
        let b = scaled(g.next(), 10) as usize;
        counts[b] += 1;
    }
    let v = chi_square_uniform(&counts);
    assert!(v < 21.67, "good generator should pass at 99%: V = {v}");
    // ... and it is not suspiciously perfect either (V should not be ~0).
    assert!(v > 0.1, "V = {v} is implausibly small for 100k random draws");
}

#[test]
fn a_rigged_sequence_is_caught() {
    // A "generator" that always returns 0 lands every observation in bucket 0.
    // The uniformity test must scream: V = (n - n/k)^2/(n/k) summed is huge.
    let n = 10_000u64;
    let mut counts = [0u64; 10];
    counts[0] = n; // everything piled into one category
    let v = chi_square_uniform(&counts);
    // V = (n - n/10)^2/(n/10) + 9·(n/10)^2/(n/10) = 9n. Enormously past 21.67.
    assert!((v - 9.0 * n as f64).abs() < 1e-6, "V = {v}");
    assert!(v > 21.67);
}

#[test]
fn a_mild_bias_is_measurable() {
    // A sequence tilted toward small buckets: expected 100 each over 4 cats
    // but observed 130, 90, 90, 90. V = (30^2 + 3·10^2)/100 = 12.
    let counts = [130u64, 90, 90, 90];
    let v = chi_square_uniform(&counts);
    assert!((v - 12.0).abs() < 1e-9, "V = {v}");
}

#[test]
#[should_panic(expected = "same number")]
fn mismatched_lengths_panic() {
    chi_square(&[1, 2, 3], &[1.0, 2.0]);
}

#[test]
#[should_panic(expected = "at least one category")]
fn empty_input_panics() {
    chi_square(&[], &[]);
}

#[test]
#[should_panic(expected = "positive")]
fn nonpositive_expected_panics() {
    chi_square(&[1, 2], &[1.0, 0.0]);
}

#[test]
#[should_panic(expected = "at least one observation")]
fn all_zero_counts_panic() {
    chi_square_uniform(&[0, 0, 0]);
}
