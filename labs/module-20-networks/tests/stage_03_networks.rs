//! Stage 3 — Sorting networks: Batcher's odd-even merge (Algorithm 5.3.4M).
//!
//! Implement `odd_even_merge_network`, `apply_network`, and `network_depth`
//! in src/lab.rs. Lesson: course/module-20-networks/README.md.

use lab_20_networks::{apply_network, network_depth, odd_even_merge_network};

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn is_sorted(a: &[i64]) -> bool {
    a.windows(2).all(|w| w[0] <= w[1])
}

fn next_permutation(a: &mut [i64]) -> bool {
    if a.len() < 2 {
        return false;
    }
    let mut i = a.len() - 1;
    while i > 0 && a[i - 1] >= a[i] {
        i -= 1;
    }
    if i == 0 {
        return false;
    }
    let mut j = a.len() - 1;
    while a[j] <= a[i - 1] {
        j -= 1;
    }
    a.swap(i - 1, j);
    a[i..].reverse();
    true
}

#[test]
fn comparator_counts_match_batcher() {
    // §5.3.4: Batcher's odd-even merge sort uses these comparator counts.
    assert_eq!(odd_even_merge_network(2).len(), 1);
    assert_eq!(odd_even_merge_network(4).len(), 5);
    assert_eq!(odd_even_merge_network(8).len(), 19);
    assert_eq!(odd_even_merge_network(16).len(), 63);
}

#[test]
fn depths_match_the_closed_form() {
    // Depth = t(t+1)/2 with t = lg n.
    for &(n, t) in &[(2usize, 1usize), (4, 2), (8, 3), (16, 4)] {
        let net = odd_even_merge_network(n);
        assert_eq!(network_depth(&net, n), t * (t + 1) / 2, "depth for n={n}");
    }
}

#[test]
fn comparators_are_within_range_and_ordered() {
    for n in [2usize, 4, 8, 16] {
        let net = odd_even_merge_network(n);
        for &(i, j) in &net {
            assert!(i < n && j < n, "comparator ({i},{j}) out of range for n={n}");
            assert!(i < j, "odd-even comparators list the low wire first");
        }
    }
}

#[test]
fn sorts_all_permutations_exhaustively_up_to_eight() {
    for n in [2usize, 4, 8] {
        let net = odd_even_merge_network(n);
        let mut perm: Vec<i64> = (1..=n as i64).collect();
        loop {
            let mut a = perm.clone();
            apply_network(&net, &mut a);
            assert!(is_sorted(&a), "n={n}: failed to sort {perm:?}");
            if !next_permutation(&mut perm) {
                break;
            }
        }
    }
}

#[test]
fn sorts_random_inputs_at_sixteen() {
    let net = odd_even_merge_network(16);
    let mut state = 0x5151_5151_2323_2323u64;
    for _ in 0..2000 {
        let mut a: Vec<i64> = (0..16).map(|_| (lcg(&mut state) >> 40) as i64 % 50).collect();
        apply_network(&net, &mut a);
        assert!(is_sorted(&a));
    }
}

#[test]
fn the_network_is_oblivious() {
    // The comparator list is fixed once and sorts every input — no branch on
    // data ever changes which comparisons happen. Build it once, sort many.
    let net = odd_even_merge_network(8);
    let net_again = odd_even_merge_network(8);
    assert_eq!(net, net_again, "the network must be data-independent");
    let mut state = 0x9999_8888_7777_6666u64;
    for _ in 0..1000 {
        let mut a: Vec<i64> = (0..8).map(|_| (lcg(&mut state) >> 40) as i64 % 100).collect();
        apply_network(&net, &mut a);
        assert!(is_sorted(&a));
    }
}

#[test]
fn apply_network_is_a_compare_exchange() {
    // A single comparator (0,1) puts the smaller value on wire 0.
    let net = vec![(0usize, 1usize)];
    let mut a = vec![9i64, 2];
    apply_network(&net, &mut a);
    assert_eq!(a, vec![2, 9]);
    let mut b = vec![2i64, 9];
    apply_network(&net, &mut b);
    assert_eq!(b, vec![2, 9]);
}
