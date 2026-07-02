//! Stage 3 — Polyphase merge: Fibonacci on tape (§5.4.2, Algorithm 5.4.2D).
//!
//! Implement `polyphase_distribution` and `polyphase_merge` in src/lab.rs.
//! The lesson: course/module-15-external/README.md.

use lab_15_external::{polyphase_distribution, polyphase_merge, Run};

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

/// Chop deterministic data into `num_runs` sorted runs of varying lengths.
fn make_runs(num_runs: usize, seed: u64) -> (Vec<Run>, Vec<i64>) {
    let mut runs = Vec::with_capacity(num_runs);
    let mut all = Vec::new();
    for i in 0..num_runs {
        let len = 5 + (i * 11) % 40;
        let mut r = lcg_vec(len, seed.wrapping_add(i as u64));
        r.sort();
        all.extend_from_slice(&r);
        runs.push(r);
    }
    all.sort();
    (runs, all)
}

/// Independent reference for the perfect-distribution *levels*: level 0 is
/// (1, 0, ..., 0); each next level applies the generalized-Fibonacci step.
/// Returns (level, perfect_total) for the smallest perfect total >= num_runs.
fn perfect_level(num_runs: usize, tapes: usize) -> (usize, usize) {
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
    (level, a.iter().sum())
}

#[test]
fn three_tape_distribution_is_fibonacci() {
    // §5.4.2 Table 1 for T = 3: consecutive Fibonacci pairs.
    assert_eq!(polyphase_distribution(1, 3), vec![1, 0]);
    assert_eq!(polyphase_distribution(2, 3), vec![1, 1]);
    assert_eq!(polyphase_distribution(3, 3), vec![2, 1]);
    assert_eq!(polyphase_distribution(5, 3), vec![3, 2]);
    assert_eq!(polyphase_distribution(8, 3), vec![5, 3]);
    assert_eq!(polyphase_distribution(13, 3), vec![8, 5]);
    assert_eq!(polyphase_distribution(21, 3), vec![13, 8]);
}

#[test]
fn non_fibonacci_counts_round_up_with_dummies() {
    // 6 real runs can't be a perfect distribution; the next perfect level
    // is (5, 3) = 8, so 2 dummy runs make up the difference.
    assert_eq!(polyphase_distribution(6, 3), vec![5, 3]);
    assert_eq!(polyphase_distribution(4, 3), vec![3, 2]);
    assert_eq!(polyphase_distribution(14, 3), vec![13, 8]);
    // General law, cross-checked against an independent level table:
    // the distribution total is the SMALLEST perfect total >= num_runs.
    for tapes in [3usize, 4, 5] {
        for num_runs in 1..=120usize {
            let d = polyphase_distribution(num_runs, tapes);
            assert_eq!(d.len(), tapes - 1, "{num_runs} runs, {tapes} tapes");
            assert!(
                d.windows(2).all(|w| w[0] >= w[1]),
                "distribution must be non-increasing"
            );
            let (_, total) = perfect_level(num_runs, tapes);
            assert_eq!(
                d.iter().sum::<usize>(),
                total,
                "{num_runs} runs on {tapes} tapes: wrong perfect level"
            );
        }
    }
}

#[test]
fn more_tapes_use_higher_order_fibonacci() {
    // T = 4: totals 1, 3, 5, 9, 17, 31 (each ~ the sum of the previous 3).
    assert_eq!(polyphase_distribution(3, 4), vec![1, 1, 1]);
    assert_eq!(polyphase_distribution(5, 4), vec![2, 2, 1]);
    assert_eq!(polyphase_distribution(9, 4), vec![4, 3, 2]);
    assert_eq!(polyphase_distribution(17, 4), vec![7, 6, 4]);
    assert_eq!(polyphase_distribution(31, 4), vec![13, 11, 7]);
    // T = 5: totals 1, 4, 7, 13, 25.
    assert_eq!(polyphase_distribution(13, 5), vec![4, 4, 3, 2]);
    assert_eq!(polyphase_distribution(25, 5), vec![8, 7, 6, 4]);
}

#[test]
fn zero_runs_distribution_is_all_zeros() {
    assert_eq!(polyphase_distribution(0, 3), vec![0, 0]);
    assert_eq!(polyphase_distribution(0, 5), vec![0, 0, 0, 0]);
}

#[test]
fn single_run_needs_zero_phases() {
    let mut r = lcg_vec(64, 5);
    r.sort();
    let (out, phases) = polyphase_merge(vec![r.clone()], 3);
    assert_eq!(out, r);
    assert_eq!(phases, 0);
}

#[test]
fn fibonacci_run_counts_hand_verified_phase_counts() {
    // Hand-traced in the lesson: with a perfect level-n distribution the
    // merge takes exactly n phases. 2 runs -> 1, 3 -> 2, 5 -> 3, 8 -> 4,
    // 13 -> 5, 21 -> 6.
    for (num_runs, want) in [(2usize, 1usize), (3, 2), (5, 3), (8, 4), (13, 5), (21, 6)] {
        let (runs, expect) = make_runs(num_runs, 1000 + num_runs as u64);
        let (out, phases) = polyphase_merge(runs, 3);
        assert_eq!(out, expect, "{num_runs} runs: wrong contents");
        assert_eq!(phases, want, "{num_runs} runs: wrong phase count");
    }
}

#[test]
fn non_fibonacci_run_counts_merge_correctly() {
    // Dummy runs fill the gap to the next perfect distribution; the phase
    // count is that level's (dummies merge for free but phases still run).
    for num_runs in [4usize, 6, 7, 9, 10, 11, 12, 14, 20, 33] {
        let (runs, expect) = make_runs(num_runs, 2000 + num_runs as u64);
        let (out, phases) = polyphase_merge(runs, 3);
        assert_eq!(out, expect, "{num_runs} runs");
        let (level, _) = perfect_level(num_runs, 3);
        assert_eq!(phases, level, "{num_runs} runs: phases != perfect level");
    }
}

#[test]
fn four_and_five_tape_merges() {
    for (num_runs, tapes) in [(9usize, 4usize), (17, 4), (10, 4), (13, 5), (18, 5)] {
        let (runs, expect) = make_runs(num_runs, 3000 + (num_runs * tapes) as u64);
        let (out, phases) = polyphase_merge(runs, tapes);
        assert_eq!(out, expect, "{num_runs} runs on {tapes} tapes");
        let (level, _) = perfect_level(num_runs, tapes);
        assert_eq!(phases, level, "{num_runs} runs on {tapes} tapes");
    }
}

#[test]
fn duplicates_survive_polyphase() {
    let runs: Vec<Run> = (0..8).map(|_| vec![5i64; 20]).collect();
    let (out, _) = polyphase_merge(runs, 3);
    assert_eq!(out, vec![5i64; 160]);
}

#[test]
#[should_panic(expected = "at least 3")]
fn two_tapes_are_rejected() {
    // Polyphase needs T >= 3: with two tapes there is no "partially
    // consumed" tape to keep — every merge would be a full copy pass.
    polyphase_distribution(10, 2);
}
