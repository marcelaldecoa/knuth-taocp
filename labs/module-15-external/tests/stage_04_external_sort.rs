//! Stage 4 — The full pipeline, with I/O accounted (§5.4 synthesis).
//!
//! Implement `external_sort` in src/lab.rs: replacement selection for run
//! formation, polyphase merging, and an honest I/O bill.
//! The lesson: course/module-15-external/README.md.

use lab_15_external::{external_sort, replacement_selection};

fn lcg_vec(n: usize, seed: u64) -> Vec<i64> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (x >> 16) as i64
        })
        .collect()
}

/// Independent stage-3 logic: how many phases does a polyphase merge of
/// `num_runs` initial runs on `tapes` tapes take? (= the perfect level.)
fn perfect_level(num_runs: usize, tapes: usize) -> usize {
    if num_runs <= 1 {
        return 0;
    }
    let t = tapes - 1;
    let mut a = vec![0usize; t];
    a[0] = 1;
    let mut level = 0;
    while a.iter().sum::<usize>() < num_runs {
        let a1 = a[0];
        let mut b = vec![0usize; t];
        for i in 0..t - 1 {
            b[i] = a1 + a[i + 1];
        }
        b[t - 1] = a1;
        a = b;
        level += 1;
    }
    level
}

/// Sortedness + permutation + the I/O ceiling from polyphase theory:
/// run formation is one pass (n reads + n writes) and each of the
/// `phases` merge phases moves at most n records — so
/// records_written <= n * (1 + phases), and likewise for reads.
fn check(input: &[i64], memory: usize, tapes: usize) {
    let n = input.len() as u64;
    let (out, io) = external_sort(input, memory, tapes);
    let mut expect = input.to_vec();
    expect.sort();
    assert_eq!(out, expect, "output not sorted / not a permutation");

    let num_runs = replacement_selection_len(input, memory);
    let phases = perfect_level(num_runs, tapes) as u64;
    let ceiling = n * (1 + phases);
    assert!(
        io.records_written <= ceiling,
        "written {} > n*(1+phases) = {} ({} runs, {} phases)",
        io.records_written,
        ceiling,
        num_runs,
        phases
    );
    assert!(
        io.records_read <= ceiling,
        "read {} > n*(1+phases) = {}",
        io.records_read,
        ceiling
    );
    if n > 0 {
        // Everything must at least pass through run formation once.
        assert!(io.records_read >= n && io.records_written >= n);
    }
}

fn replacement_selection_len(input: &[i64], memory: usize) -> usize {
    if input.is_empty() {
        0
    } else {
        replacement_selection(input, memory).len()
    }
}

#[test]
fn big_random_input_sorted_with_bounded_io() {
    // n = 200_000, memory = 256: replacement selection yields about
    // n / 2P ~ 390 runs, and 3-tape polyphase handles those in ~13 phases.
    // The bill must respect records_written <= n * (1 + phases) — and in
    // fact polyphase comes in well under, because phases touch only part
    // of the file.
    let data = lcg_vec(200_000, 7);
    check(&data, 256, 3);
}

#[test]
fn another_seed_and_five_tapes() {
    let data = lcg_vec(60_000, 424242);
    check(&data, 128, 3);
    check(&data, 128, 5);
}

#[test]
fn sorted_input_is_one_pass() {
    // Already sorted: replacement selection emits ONE run, so there is
    // nothing to merge. Pinned reference behaviour: exactly n records read
    // (run formation in) and n written (run formation out) — well under
    // the 3n ceiling.
    let n = 50_000usize;
    let sorted: Vec<i64> = (0..n as i64).collect();
    let (out, io) = external_sort(&sorted, 64, 3);
    assert_eq!(out, sorted);
    assert_eq!(io.records_read, n as u64);
    assert_eq!(io.records_written, n as u64);
    assert!(io.records_read + io.records_written <= 3 * n as u64);
}

#[test]
fn memory_at_least_n_behaves_like_internal_sort() {
    // The whole file fits in memory: one run, no merge phases — the I/O
    // bill is identical to the sorted-input case (one pass), even though
    // the data is random.
    let data = lcg_vec(10_000, 31);
    let mut expect = data.clone();
    expect.sort();
    for memory in [10_000usize, 10_001, 1 << 20] {
        let (out, io) = external_sort(&data, memory, 3);
        assert_eq!(out, expect);
        assert_eq!(io.records_read, 10_000, "memory = {memory}");
        assert_eq!(io.records_written, 10_000, "memory = {memory}");
    }
}

#[test]
fn tiny_inputs_and_tiny_memory() {
    // Empty file: nothing read, nothing written.
    let (out, io) = external_sort(&[], 8, 3);
    assert!(out.is_empty());
    assert_eq!(io.records_read, 0);
    assert_eq!(io.records_written, 0);

    // Exhaustive small sweep: every (n, memory) combination must satisfy
    // the same contract as the big cases.
    for n in [1usize, 2, 3, 5, 17, 40] {
        for memory in [1usize, 2, 3, 8] {
            let data = lcg_vec(n, (n * 31 + memory) as u64);
            check(&data, memory, 3);
        }
    }
}

#[test]
fn worst_case_input_still_respects_the_ceiling() {
    // Reverse-sorted input with memory P produces ceil(n/P) runs — the
    // most replacement selection can ever produce — so this exercises the
    // deepest polyphase schedule for the given memory.
    let data: Vec<i64> = (0..30_000i64).rev().collect();
    check(&data, 64, 3);
    check(&data, 64, 4);
}

#[test]
#[should_panic(expected = "at least one")]
fn zero_memory_is_rejected() {
    external_sort(&[1, 2, 3], 0, 3);
}

#[test]
#[should_panic(expected = "at least 3")]
fn two_tapes_are_rejected() {
    external_sort(&[1, 2, 3], 4, 2);
}
