//! Stage 4 — The zero-one principle and bitonic sorting (Theorem 5.3.4Z).
//!
//! Implement `sorts_all_zero_one` and `bitonic_sort_network` in src/lab.rs.
//! Lesson: course/module-20-networks/README.md.

use lab_20_networks::{
    apply_network, bitonic_sort_network, odd_even_merge_network, sorts_all_zero_one,
};

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn is_sorted(a: &[i64]) -> bool {
    a.windows(2).all(|w| w[0] <= w[1])
}

#[test]
fn zero_one_certifies_odd_even() {
    // Theorem Z: sorting all 2^n zero-one inputs certifies the network sorts
    // everything. Cheaper than checking all n! permutations.
    for n in [2usize, 4, 8, 16] {
        assert!(
            sorts_all_zero_one(&odd_even_merge_network(n), n),
            "odd-even network should sort all 0-1 inputs (n={n})"
        );
    }
}

#[test]
fn the_certificate_implies_full_sorting() {
    // For n = 8: the 0-1 certificate holds, and (as the principle promises) the
    // same network then sorts arbitrary integer inputs.
    let net = odd_even_merge_network(8);
    assert!(sorts_all_zero_one(&net, 8));
    let mut state = 0xc0ff_ee00_1234_5678u64;
    for _ in 0..500 {
        let mut a: Vec<i64> = (0..8).map(|_| (lcg(&mut state) >> 30) as i64 % 1000 - 500).collect();
        apply_network(&net, &mut a);
        assert!(is_sorted(&a), "0-1 certified network must sort integers too");
    }
}

#[test]
fn a_broken_network_fails_the_certificate() {
    // Drop one comparator: the network is no longer a valid sorter, and the
    // zero-one principle catches it with a 0-1 counterexample.
    let net = odd_even_merge_network(8);
    // There is at least one comparator whose removal breaks the sort.
    let mut any_broke = false;
    for k in 0..net.len() {
        let mut broken = net.clone();
        broken.remove(k);
        if !sorts_all_zero_one(&broken, 8) {
            any_broke = true;
        }
    }
    assert!(any_broke, "removing some comparator must break the network");

    // Pin one concrete broken network: dropping the very first comparator.
    let mut broken = net.clone();
    broken.remove(0);
    assert!(
        !sorts_all_zero_one(&broken, 8),
        "dropping the first comparator must break the sort"
    );
}

#[test]
fn bitonic_network_sorts_via_zero_one() {
    for n in [2usize, 4, 8, 16] {
        assert!(
            sorts_all_zero_one(&bitonic_sort_network(n), n),
            "bitonic network should sort all 0-1 inputs (n={n})"
        );
    }
}

#[test]
fn bitonic_sorts_integers_too() {
    let net = bitonic_sort_network(8);
    let mut state = 0x1111_2222_3333_4444u64;
    for _ in 0..500 {
        let mut a: Vec<i64> = (0..8).map(|_| (lcg(&mut state) >> 40) as i64 % 100).collect();
        apply_network(&net, &mut a);
        assert!(is_sorted(&a));
    }
}

#[test]
fn a_random_broken_bitonic_also_fails() {
    // Removing a comparator from bitonic sort likewise breaks it (some index).
    let net = bitonic_sort_network(8);
    let mut any_broke = false;
    for k in 0..net.len() {
        let mut broken = net.clone();
        broken.remove(k);
        if !sorts_all_zero_one(&broken, 8) {
            any_broke = true;
            break;
        }
    }
    assert!(any_broke);
}
