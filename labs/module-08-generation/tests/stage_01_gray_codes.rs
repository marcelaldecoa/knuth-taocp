//! Stage 1 — the reflected Gray binary code (Algorithm 7.2.1.1G).
//!
//! Implement `gray_code` and `gray_rank` in src/lab.rs.
//! Lesson: course/module-08-generation/README.md.

use lab_08_generation::{gray_code, gray_rank};

#[test]
fn tiny_cases_are_exact() {
    // n = 0: a single empty tuple, the word 0.
    assert_eq!(gray_code(0), vec![0]);
    // n = 1: 0, 1.
    assert_eq!(gray_code(1), vec![0, 1]);
    // n = 2: 00, 01, 11, 10  ==  0, 1, 3, 2.
    assert_eq!(gray_code(2), vec![0, 1, 3, 2]);
    // n = 3: §7.2.1.1 lists 000,001,011,010,110,111,101,100.
    assert_eq!(
        gray_code(3),
        vec![0b000, 0b001, 0b011, 0b010, 0b110, 0b111, 0b101, 0b100]
    );
    // n = 4: the standard 16-entry reflected code.
    assert_eq!(
        gray_code(4),
        vec![0, 1, 3, 2, 6, 7, 5, 4, 12, 13, 15, 14, 10, 11, 9, 8]
    );
}

#[test]
fn length_is_two_to_the_n() {
    for n in 0..=16u32 {
        assert_eq!(gray_code(n).len(), 1usize << n, "|gray_code({n})|");
    }
}

#[test]
fn successive_words_differ_in_one_bit() {
    // The defining property of a Gray code.
    for n in 0..=16u32 {
        let g = gray_code(n);
        for w in g.windows(2) {
            assert_eq!((w[0] ^ w[1]).count_ones(), 1, "n={n}: one-bit change");
        }
    }
}

#[test]
fn all_distinct_and_complete() {
    // Every n-bit word appears exactly once: the code is a permutation of
    // 0..2^n. Sorting the output must give the identity range.
    for n in 0..=14u32 {
        let mut g = gray_code(n);
        assert!(g.iter().all(|&w| w < (1u64 << n)), "n={n}: in range");
        g.sort_unstable();
        for (k, &w) in g.iter().enumerate() {
            assert_eq!(w, k as u64, "n={n}: value {k} missing");
        }
    }
}

#[test]
fn matches_the_closed_form() {
    // g(k) = k XOR floor(k/2).
    for n in 0..=14u32 {
        let g = gray_code(n);
        for (k, &w) in g.iter().enumerate() {
            let k = k as u64;
            assert_eq!(w, k ^ (k >> 1), "g({k}) for n={n}");
        }
    }
}

#[test]
fn changed_bit_is_the_ruler_function() {
    // From g(k-1) to g(k) the flipped bit sits at position rho(k) =
    // number of trailing 0s of k.
    let g = gray_code(12);
    for k in 1..g.len() {
        let diff = g[k - 1] ^ g[k];
        assert_eq!(diff.count_ones(), 1);
        assert_eq!(diff.trailing_zeros(), (k as u64).trailing_zeros(), "rho({k})");
    }
}

#[test]
fn rank_inverts_the_code() {
    // gray_rank(g(k)) = k for every k.
    for n in 0..=14u32 {
        let g = gray_code(n);
        for (k, &w) in g.iter().enumerate() {
            assert_eq!(gray_rank(w), k as u64, "gray_rank of g({k}) for n={n}");
        }
    }
}

#[test]
fn rank_roundtrips_arbitrary_words() {
    // For any word, g(gray_rank(word)) == word. Build g on the fly:
    // g(k) = k ^ (k >> 1).
    let mut s = 0x1234_5678_9abc_def0u64;
    for _ in 0..2000 {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let word = s >> 8; // any u64 value
        let k = gray_rank(word);
        assert_eq!(k ^ (k >> 1), word, "g(gray_rank({word})) must be word");
    }
}
