//! Stage 5 — integer partitions (Algorithm 7.2.1.4P), their count, and
//! conjugation.
//!
//! Implement `partitions`, `partition_count`, `conjugate` in src/lab.rs.
//! Lesson: course/module-08-generation/README.md.

use lab_08_generation::{conjugate, partition_count, partitions};

#[test]
fn small_partitions_are_exact_in_reverse_lex() {
    assert_eq!(partitions(1), vec![vec![1]]);
    assert_eq!(partitions(2), vec![vec![2], vec![1, 1]]);
    assert_eq!(partitions(3), vec![vec![3], vec![2, 1], vec![1, 1, 1]]);
    assert_eq!(
        partitions(4),
        vec![vec![4], vec![3, 1], vec![2, 2], vec![2, 1, 1], vec![1, 1, 1, 1]]
    );
    // §7.2.1.4: the p(5) = 7 partitions in reverse lexicographic order.
    assert_eq!(
        partitions(5),
        vec![
            vec![5],
            vec![4, 1],
            vec![3, 2],
            vec![3, 1, 1],
            vec![2, 2, 1],
            vec![2, 1, 1, 1],
            vec![1, 1, 1, 1, 1],
        ]
    );
    // p(6) = 11.
    assert_eq!(
        partitions(6),
        vec![
            vec![6],
            vec![5, 1],
            vec![4, 2],
            vec![4, 1, 1],
            vec![3, 3],
            vec![3, 2, 1],
            vec![3, 1, 1, 1],
            vec![2, 2, 2],
            vec![2, 2, 1, 1],
            vec![2, 1, 1, 1, 1],
            vec![1, 1, 1, 1, 1, 1],
        ]
    );
}

#[test]
fn zero_has_one_empty_partition() {
    assert_eq!(partitions(0), vec![Vec::<u32>::new()]);
    assert_eq!(partition_count(0), 1);
}

#[test]
fn each_partition_sums_to_n_and_is_non_increasing() {
    for n in 1..=30u32 {
        for p in partitions(n) {
            assert_eq!(p.iter().sum::<u32>(), n, "n={n}: {p:?} sums to n");
            assert!(p.iter().all(|&x| x >= 1), "n={n}: {p:?} positive parts");
            assert!(
                p.windows(2).all(|w| w[0] >= w[1]),
                "n={n}: {p:?} non-increasing"
            );
        }
    }
}

#[test]
fn output_is_strictly_decreasing_in_reverse_lex() {
    // Reverse lexicographic order: each partition is lexicographically
    // greater than the next.
    for n in 1..=30u32 {
        let ps = partitions(n);
        assert!(ps.windows(2).all(|w| w[0] > w[1]), "n={n}: reverse-lex order");
    }
}

#[test]
fn count_agrees_with_enumeration() {
    for n in 0..=30u32 {
        assert_eq!(partitions(n).len() as u64, partition_count(n), "p({n})");
    }
}

#[test]
fn known_partition_counts() {
    // Classic values of p(n).
    assert_eq!(partition_count(10), 42);
    assert_eq!(partition_count(20), 627);
    assert_eq!(partition_count(50), 204_226);
    assert_eq!(partition_count(100), 190_569_292);
}

#[test]
fn conjugate_worked_examples() {
    // Ferrers-diagram transpose: conjugate of 4+1 is 2+1+1+1.
    assert_eq!(conjugate(&[4, 1]), vec![2, 1, 1, 1]);
    // Self-conjugate staircase.
    assert_eq!(conjugate(&[3, 2, 1]), vec![3, 2, 1]);
    // Single row <-> single column.
    assert_eq!(conjugate(&[5]), vec![1, 1, 1, 1, 1]);
    assert_eq!(conjugate(&[1, 1, 1]), vec![3]);
    // Empty partition.
    assert_eq!(conjugate(&[]), Vec::<u32>::new());
}

#[test]
fn conjugate_swaps_largest_part_and_number_of_parts() {
    // Largest part of conjugate = number of parts of p, and vice versa.
    for n in 1..=15u32 {
        for p in partitions(n) {
            let c = conjugate(&p);
            assert_eq!(c.len() as u32, p[0], "n={n}: |c| = largest part of p");
            assert_eq!(c[0] as usize, p.len(), "n={n}: largest of c = #parts of p");
            assert_eq!(c.iter().sum::<u32>(), n, "n={n}: conjugate still sums to n");
            assert!(c.windows(2).all(|w| w[0] >= w[1]), "n={n}: c non-increasing");
        }
    }
}

#[test]
fn conjugation_is_an_involution() {
    // conjugate(conjugate(p)) == p over all partitions of 8.
    for p in partitions(8) {
        assert_eq!(conjugate(&conjugate(&p)), p, "involution on {p:?}");
    }
}

#[test]
#[should_panic(expected = "non-increasing")]
fn conjugate_rejects_non_partition() {
    conjugate(&[1, 3, 2]);
}
